//! The Action Phase: two Parties fight on the Board, resolved in Beats.
//!
//! See `docs/DESIGN.md`. Given the same Board and the same Rng state, this always
//! produces the same Resolution.
//!
//! **A Beat is a container of interactions, and Beats are the only clock.** There
//! is no round, no turn, no sweep of the Slots. Every Beat is the same four
//! things, each of them a question asked of the Board rather than a step in a
//! schedule:
//!
//! 1. **Settle what the last Beat did.** A Unit that reached 0 health does not
//!    die in the Beat that took it there; it dies at the top of the next one. So
//!    a fatally wounded Unit still acts for the rest of the Beat it ran out in.
//!    A Divine Shield that absorbed something breaks here for the same reason.
//! 2. **Close ranks.** Each Party re-anchors on its left-most Unit, closing the
//!    hole the burial left.
//! 3. **Renew Intents, if nobody holds one.** An Intent is a Unit's claim on a
//!    future Beat. Running out of them across the whole Board is how the clock
//!    knows everyone has had their go -- nothing counts rounds.
//! 4. **Act.** The left-most Unit on each side still holding an Intent spends it,
//!    on both sides in this one Beat. There is no first swing to have, so a
//!    mirror is a mutual kill.
//!
//! Whose Beat it is therefore lives on the Unit, as [`Unit::intents`], and not in
//! a cursor walking the Slots. That is what lets a Party close ranks the instant a
//! hole opens: a Unit carries its Intents with it when it slides, so sliding can
//! neither skip it nor give it a second go.
//!
//! Windfury is not a special case in here. It is a Unit that renews two Intents
//! where others renew one, so after acting it is still the left-most holder and
//! acts again in the very next Beat -- with the dead buried in between, as
//! Battlegrounds does it.
//!
//! Battlegrounds' own rules, unchanged:
//!
//! - Targeting is **random** among the defending Party's Units, unless that Party
//!   holds a Taunt Unit, in which case the attack must land on a Taunt holder.
//!   Every attack draws its own target.
//! - **An attack is answered**: the Unit struck deals its own attack back in the
//!   same motion, which is what makes health, Taunt and Poisonous mean anything
//!   on a Unit that is not the one swinging. Answering is not attacking -- it
//!   draws no target and spends no Intent.
//! - Nothing ever attacks a Player. An attack with nothing left to target does
//!   not land; damage-on-loss is an end-of-fight calculation over
//!   [`Resolution::final_board`], and is not this module's yet.
//! - Beats run until a Party empties, or [`MAX_BEATS`] is reached.
//!
//! Slots are numbered 1 through 8 in the rules and in every Event below. The
//! array index behind a Slot is zero-based, and only this module knows it.

use crate::party::{Board, Party, SLOTS, Side, Unit};
use crate::rng::Rng;
use crate::units::{DefId, Keyword};

/// Beats after which an unresolved Action Phase is declared a stalemate.
///
/// Two Parties that cannot kill each other -- all zero-attack, say -- would
/// otherwise run forever. The cap makes non-termination a reported outcome
/// rather than a hang.
pub const MAX_BEATS: u32 = 512;

/// The Slot number a zero-based index stands for. Slots count from 1.
fn slot_no(index: usize) -> u32 {
    index as u32 + 1
}

/// How an Action Phase ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    PlayerWins,
    OpposingWins,
    /// Both Parties emptied together.
    Draw,
    /// Neither Party could finish the other within [`MAX_BEATS`].
    Stalemate,
}

/// Everything that happened, in the order it happened.
///
/// The log is the Action Phase's explanation of itself. A frontend animates
/// these in order and needs to know nothing else; a test asserts against them
/// without reaching into engine internals. Every Slot number here counts from 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A Beat, numbered from 1. Every Beat is logged, including ones in which
    /// nothing else happened, so that a Beat number is the time it says it is.
    BeatBegan {
        beat: u32,
    },
    /// This Party re-anchored on its left-most Unit. Logged only when it actually
    /// moved something.
    Compacted {
        side: Side,
    },
    Struck {
        by: Side,
        attacker_slot: u32,
        target_slot: u32,
        damage: i32,
    },
    /// A defender answering the blow it was struck by, in the same motion.
    StruckBack {
        by: Side,
        slot: u32,
        target_slot: u32,
        damage: i32,
    },
    ShieldAbsorbed {
        side: Side,
        slot: u32,
    },
    /// A Unit that ran out in the previous Beat, dying at the top of this one.
    Died {
        side: Side,
        slot: u32,
        def: DefId,
        name: String,
        /// Killed by Poisonous rather than by running out of health.
        poisoned: bool,
    },
    Reborn {
        side: Side,
        slot: u32,
        name: String,
    },
    Ended {
        outcome: Outcome,
        beats: u32,
    },
}

impl std::fmt::Display for Event {
    /// A one-line rendering, so a log reads as prose.
    ///
    /// Formatting, not I/O: this returns a string and never touches a stream, so
    /// the engine keeps its promise not to print.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::BeatBegan { beat } => write!(f, "-- beat {beat} --"),
            Event::Compacted { side } => {
                write!(f, "  the {} party closes ranks", side.name())
            }
            Event::Struck {
                by,
                attacker_slot,
                target_slot,
                damage,
            } => write!(
                f,
                "  {} slot {attacker_slot} strikes {} slot {target_slot} for {damage}",
                by.name(),
                by.other().name()
            ),
            Event::StruckBack {
                by,
                slot,
                target_slot,
                damage,
            } => write!(
                f,
                "  {} slot {slot} strikes back at {} slot {target_slot} for {damage}",
                by.name(),
                by.other().name()
            ),
            Event::ShieldAbsorbed { side, slot } => {
                write!(f, "  {} slot {slot} absorbs it on its shield", side.name())
            }
            Event::Died {
                side,
                slot,
                name,
                poisoned,
                ..
            } => {
                let how = if *poisoned { " (poisoned)" } else { "" };
                write!(f, "  {} {name} in slot {slot} dies{how}", side.name())
            }
            Event::Reborn { side, slot, name } => write!(
                f,
                "  {} {name} returns in slot {slot} with 1 health",
                side.name()
            ),
            Event::Ended { outcome, beats } => {
                write!(f, "== {outcome:?} after {beats} beats ==")
            }
        }
    }
}

/// What an Action Phase produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub outcome: Outcome,
    /// Beats run, counting from 1. A fight decided before anyone acted ran none.
    pub beats: u32,
    pub log: Vec<Event>,
    /// The Board as it stood when the Action Phase ended, left-anchored. The
    /// winner's survivors are what a damage-on-loss calculation will read, once
    /// it exists.
    pub final_board: Board,
}

impl Resolution {
    /// The log rendered as lines, for a frontend or a failing test to print.
    pub fn narrate(&self) -> String {
        self.log
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Resolve an Action Phase to completion.
///
/// Deterministic given `rng`'s state: equal Boards and equal Rng draws give
/// equal Resolutions.
pub fn resolve(mut board: Board, rng: &mut Rng) -> Resolution {
    let mut log = Vec::new();
    let mut beats = 0u32;

    let outcome = loop {
        if let Some(outcome) = decide(&board) {
            break outcome;
        }
        if beats >= MAX_BEATS {
            break Outcome::Stalemate;
        }
        beats += 1;
        log.push(Event::BeatBegan { beat: beats });

        bury_the_dead(&mut board, &mut log);
        break_spent_shields(&mut board);
        close_ranks(&mut board, &mut log);
        if no_intents_left(&board) {
            renew_intents(&mut board);
        }
        for transaction in declare(&mut board, rng) {
            resolve_transaction(&mut board, &transaction, &mut log);
        }
    };

    log.push(Event::Ended { outcome, beats });
    Resolution {
        outcome,
        beats,
        log,
        final_board: board,
    }
}

/// The outcome, if the Action Phase is over.
///
/// Reads Slots, not health: a Party whose Units have all run out is not empty
/// until the Beat that buries them.
fn decide(board: &Board) -> Option<Outcome> {
    match (board.player.is_empty(), board.opposing.is_empty()) {
        (true, true) => Some(Outcome::Draw),
        (true, false) => Some(Outcome::OpposingWins),
        (false, true) => Some(Outcome::PlayerWins),
        (false, false) => None,
    }
}

/// Remove every Unit that ran out in the previous Beat, returning what has Reborn
/// to spend.
///
/// Order here is Side then Slot and means nothing: these deaths all belong to the
/// same Beat, and none of them can affect another.
fn bury_the_dead(board: &mut Board, log: &mut Vec<Event>) {
    for side in [Side::Player, Side::Opposing] {
        for index in 0..SLOTS {
            if board.side(side).get(index).is_some_and(Unit::is_dying) {
                bury(board, side, index, log);
            }
        }
    }
}

/// Remove one Unit and log it, returning it in place if it has Reborn to spend.
fn bury(board: &mut Board, side: Side, index: usize, log: &mut Vec<Event>) {
    let unit = board
        .side_mut(side)
        .take(index)
        .expect("just observed as occupied");
    log.push(Event::Died {
        side,
        slot: slot_no(index),
        def: unit.def.clone(),
        name: unit.name.clone(),
        poisoned: unit.poisoned,
    });

    if unit.has(Keyword::Reborn) && !unit.reborn_spent {
        let mut returned = unit;
        returned.reborn_spent = true;
        returned.keywords.remove(&Keyword::Reborn);
        returned.health = 1;
        returned.poisoned = false;
        let name = returned.name.clone();
        board.side_mut(side).put(index, returned);
        log.push(Event::Reborn {
            side,
            slot: slot_no(index),
            name,
        });
    }
}

/// Break every Divine Shield that absorbed something in the previous Beat.
///
/// Runs after the burial so a Unit brought back by Reborn does not carry a
/// already-spent shield into this Beat.
fn break_spent_shields(board: &mut Board) {
    for side in [Side::Player, Side::Opposing] {
        for index in 0..SLOTS {
            if let Some(unit) = board.side_mut(side).get_mut(index)
                && unit.shield_spent
            {
                unit.keywords.remove(&Keyword::DivineShield);
                unit.shield_spent = false;
            }
        }
    }
}

/// Re-anchor both Parties on their left-most Unit.
fn close_ranks(board: &mut Board, log: &mut Vec<Event>) {
    for side in [Side::Player, Side::Opposing] {
        let party = board.side_mut(side);
        if party.is_packed() {
            continue;
        }
        party.compact();
        log.push(Event::Compacted { side });
    }
}

/// Whether the Board has run out of Intents, which is what renews them.
fn no_intents_left(board: &Board) -> bool {
    [Side::Player, Side::Opposing]
        .into_iter()
        .flat_map(|side| board.side(side).iter())
        .all(|(_, unit)| !unit.holds_intent())
}

/// Give every Unit on the Board its Intents back.
fn renew_intents(board: &mut Board) {
    for side in [Side::Player, Side::Opposing] {
        for index in 0..SLOTS {
            if let Some(unit) = board.side_mut(side).get_mut(index) {
                unit.renew_intents();
            }
        }
    }
}

/// The Unit acting for this Party: the left-most that still holds an Intent.
fn next_actor(party: &Party) -> Option<usize> {
    party
        .iter()
        .find(|(_, unit)| unit.holds_intent())
        .map(|(index, _)| index)
}

/// One attack, drawn but not yet resolved. Who swung, and at whom.
///
/// A Beat's attacks are all drawn before any of them resolves, so no attack can
/// take a target away from the other.
struct Transaction {
    by: Side,
    attacker: usize,
    defender: usize,
}

/// The attacks of one Beat: at most one per side.
///
/// Acting is spending the Intent, whether or not the act produces an attack. A
/// Unit with nothing to attack with, or nothing left to attack, spends its Intent
/// quietly -- which is what moves the clock on for Parties that cannot fight.
///
/// Nothing on the Board is dying at this point: the Beat opened by burying
/// whatever ran out, so every Unit a target is drawn from will still be standing
/// when the attacks land.
fn declare(board: &mut Board, rng: &mut Rng) -> Vec<Transaction> {
    let actors: Vec<(Side, usize)> = [Side::Player, Side::Opposing]
        .into_iter()
        .filter_map(|side| next_actor(board.side(side)).map(|index| (side, index)))
        .collect();

    for &(side, index) in &actors {
        board
            .side_mut(side)
            .get_mut(index)
            .expect("just found standing there")
            .spend_intent();
    }

    actors
        .into_iter()
        .filter(|&(side, index)| board.side(side).get(index).is_some_and(|u| u.attack > 0))
        .filter_map(|(by, attacker)| {
            let defender = select_target(rng, board.side(by.other()))?;
            Some(Transaction {
                by,
                attacker,
                defender,
            })
        })
        .collect()
}

/// A random Unit in `party`, constrained to Taunt holders if it has any.
/// `None` means the Party has nothing left to target.
fn select_target(rng: &mut Rng, party: &Party) -> Option<usize> {
    let taunts: Vec<usize> = party
        .iter()
        .filter(|(_, unit)| unit.has(Keyword::Taunt))
        .map(|(index, _)| index)
        .collect();
    if !taunts.is_empty() {
        return rng.choose(&taunts).copied();
    }
    let occupied: Vec<usize> = party.iter().map(|(index, _)| index).collect();
    rng.choose(&occupied).copied()
}

/// One attack, resolved exactly as Battlegrounds resolves one.
///
/// An attack has a direction: this Unit swings at that one, and the one struck
/// answers with its own attack in the same motion. Two Units that chose each
/// other in the same Beat are two of these, each with its own attacker, not one
/// symmetrical trade -- collapsing them would restore the pre-emption a Beat
/// exists to remove.
///
/// Both Units' attack is read before either blow lands, so a Unit's answer is not
/// weakened by the blow it is answering.
fn resolve_transaction(board: &mut Board, transaction: &Transaction, log: &mut Vec<Event>) {
    let defending = transaction.by.other();
    let (Some(attacker), Some(defender)) = (
        board.side(transaction.by).get(transaction.attacker),
        board.side(defending).get(transaction.defender),
    ) else {
        return;
    };
    let (attack, attacker_poisons) = (attacker.attack, attacker.has(Keyword::Poisonous));
    let (answer, defender_poisons) = (defender.attack, defender.has(Keyword::Poisonous));

    log.push(Event::Struck {
        by: transaction.by,
        attacker_slot: slot_no(transaction.attacker),
        target_slot: slot_no(transaction.defender),
        damage: attack,
    });
    hit(
        board,
        defending,
        transaction.defender,
        attack,
        attacker_poisons,
        log,
    );

    if answer > 0 {
        log.push(Event::StruckBack {
            by: defending,
            slot: slot_no(transaction.defender),
            target_slot: slot_no(transaction.attacker),
            damage: answer,
        });
        hit(
            board,
            transaction.by,
            transaction.attacker,
            answer,
            defender_poisons,
            log,
        );
    }
}

/// Land one blow on one Unit.
///
/// A Divine Shield absorbs it whole -- including its Poisonous, which needs damage
/// to actually land. The shield does not break here: it absorbs everything this
/// Beat brings and breaks at the top of the next. Otherwise a Unit struck twice in
/// one Beat would spend its shield on whichever blow the engine reached first,
/// which would make the walk order of the Board worth something.
fn hit(
    board: &mut Board,
    side: Side,
    index: usize,
    damage: i32,
    poisonous: bool,
    log: &mut Vec<Event>,
) {
    let Some(unit) = board.side_mut(side).get_mut(index) else {
        return;
    };
    if unit.has(Keyword::DivineShield) {
        unit.shield_spent = true;
        log.push(Event::ShieldAbsorbed {
            side,
            slot: slot_no(index),
        });
        return;
    }
    unit.health -= damage;
    if poisonous {
        unit.poisoned = true;
    }
}

/// Build a Board from two lists of Units. Convenience for tests and setup.
pub fn board_of(player: Vec<Unit>, opposing: Vec<Unit>) -> Board {
    Board::new(
        Party::from_units(player).expect("player Party fits"),
        Party::from_units(opposing).expect("opposing Party fits"),
    )
}
