//! The Action Phase: two Parties fight, resolved in Beats.
//!
//! See `docs/DESIGN.md`. Given the same two Parties and the same Rng state, this
//! always produces the same Resolution.
//!
//! **A Beat is a container of interactions.** Battlegrounds resolves combat in
//! steps, and a step holds one interaction that has to finish before the next may
//! begin; a Beat holds however many interactions we choose to put in it, resolved
//! together. Most Beats hold one or two. What this module puts in them:
//!
//! - A **Pass** walks Slots 1 through 8. At each Slot, the Unit standing there
//!   attacks -- on both sides, in the same Beat. That concurrency is the point of
//!   a Beat: there is no first swing to have, so a mirror is a mutual kill rather
//!   than a win for whoever was asked first.
//! - A Unit with **Windfury** attacks again, in a Beat of its own directly after.
//!   Its own Beat, not a second swing inside one, because Battlegrounds clears the
//!   dead between a Windfury Unit's two attacks and Windfury is unchanged here.
//! - **Every Beat opens by burying the dead.** A Unit that reaches 0 health does
//!   not die in the Beat that took it there; it dies at the top of the next one.
//!   So a fatally wounded Unit still lands its blow, and everything else in that
//!   Beat still finds it standing.
//! - Between Passes, a Beat in which the dead are buried and both Parties **close
//!   ranks** on their left-most Unit. Compacting only here is what holds Slots
//!   still while a Pass runs: a Unit that has already acted must not slide into a
//!   Slot the clock has yet to reach.
//!
//! A Beat nothing happened in is not a Beat. Empty ones are neither logged nor
//! counted, so Beat numbers are a Pass's own sequence and not a Slot's.
//!
//! The rest is Battlegrounds' own, unchanged, and so stated here once:
//!
//! - Targeting is **random** among the defending Party's Units, unless that Party
//!   holds a Taunt Unit, in which case the attack must land on a Taunt holder.
//!   Every attack draws its own target, Windfury's second included.
//! - **An attack is answered.** The Unit struck deals its own attack back in the
//!   same motion, which is what makes health, Taunt and Poisonous mean anything on
//!   a Unit that is not the one swinging. Answering is not attacking: it draws no
//!   target.
//! - Nothing ever attacks a Player. An attack with no Unit left to target does not
//!   land -- damage-on-loss is an end-of-fight calculation over
//!   [`Resolution::final_board`], and is not this module's yet.
//! - Passes repeat until a Party empties, or [`MAX_PASSES`] is reached.
//!
//! Slots are numbered **1 through 8** in the rules, in this log, and in every Event
//! below. The array index behind a Slot is zero-based, and only this module knows it.

use crate::party::{Board, Party, SLOTS, Side, Unit};
use crate::rng::Rng;
use crate::units::{DefId, Keyword};

/// Passes after which an unresolved Action Phase is declared a stalemate.
///
/// Two Parties that cannot kill each other -- all zero-attack, say -- would
/// otherwise run forever. The cap makes non-termination a reported outcome
/// rather than a hang.
pub const MAX_PASSES: u32 = 64;

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
    /// Neither Party could finish the other within [`MAX_PASSES`].
    Stalemate,
}

/// Everything that happened, in the order it happened.
///
/// The log is the Action Phase's explanation of itself. A frontend animates
/// these in order and needs to know nothing else; a test asserts against them
/// without reaching into engine internals. Every Slot number here counts from 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A Beat in which something happened. `beat` counts from 1 within its Pass,
    /// over the Beats that held anything -- it is not a Slot number.
    BeatBegan {
        pass: u32,
        beat: u32,
    },
    /// This Party closed ranks. Logged only when it actually moved something.
    Compacted {
        side: Side,
    },
    Struck {
        by: Side,
        attacker_slot: u32,
        target_slot: u32,
        damage: i32,
    },
    /// A defender answering the blow it was struck by, in the same motion. Not an
    /// attack: it draws no target of its own.
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
    /// A Unit that ran out in an earlier Beat, dying at the top of this one.
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
        passes: u32,
    },
}

impl std::fmt::Display for Event {
    /// A one-line rendering, so a log reads as prose.
    ///
    /// Formatting, not I/O: this returns a string and never touches a stream, so
    /// the engine keeps its promise not to print.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::BeatBegan { pass, beat } => {
                write!(f, "-- pass {pass}, beat {beat} --")
            }
            Event::Compacted { side } => {
                write!(f, "  the {} party closes ranks", side.name())
            }
            Event::Struck {
                by,
                attacker_slot,
                target_slot,
                damage,
            } => {
                write!(
                    f,
                    "  {} slot {attacker_slot} strikes {} slot {target_slot} for {damage}",
                    by.name(),
                    by.other().name()
                )
            }
            Event::StruckBack {
                by,
                slot,
                target_slot,
                damage,
            } => {
                write!(
                    f,
                    "  {} slot {slot} strikes back at {} slot {target_slot} for {damage}",
                    by.name(),
                    by.other().name()
                )
            }
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
            Event::Reborn { side, slot, name } => {
                write!(
                    f,
                    "  {} {name} returns in slot {slot} with 1 health",
                    side.name()
                )
            }
            Event::Ended { outcome, passes } => {
                write!(f, "== {outcome:?} after {passes} passes ==")
            }
        }
    }
}

/// What an Action Phase produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub outcome: Outcome,
    /// Passes swept, counting from 1. A fight decided before anyone acted swept
    /// none. The log can hold one Beat beyond this: a Pass opens on the Beat that
    /// buries the dead and closes ranks, and what that Beat leaves behind can be
    /// a Party with nothing in it.
    pub passes: u32,
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
    let mut passes = 0u32;

    let outcome = loop {
        let mut beat = 0u32;

        // The Beat that opens a Pass: the dead are buried and both Parties
        // re-anchor. On the first Pass there is nothing to do and no Beat.
        run_beat(&mut board, passes + 1, &mut beat, &mut log, |board, log| {
            close_ranks(&mut board.player, Side::Player, log);
            close_ranks(&mut board.opposing, Side::Opposing, log);
        });

        if let Some(outcome) = decide(&board) {
            break outcome;
        }
        if passes >= MAX_PASSES {
            break Outcome::Stalemate;
        }
        passes += 1;

        for index in 0..SLOTS {
            for swing in [Swing::First, Swing::Windfury] {
                run_beat(&mut board, passes, &mut beat, &mut log, |board, log| {
                    for attack in declare(board, index, swing, rng) {
                        resolve_transaction(board, &attack, log);
                    }
                });
            }
            // A Party can empty mid-Pass; the Slots after it hold nothing worth a
            // Beat.
            if decide(&board).is_some() {
                break;
            }
        }
    };

    log.push(Event::Ended { outcome, passes });
    Resolution {
        outcome,
        passes,
        log,
        final_board: board,
    }
}

/// Run one Beat: the dead are buried, then `act` happens.
///
/// Burying first is the departure on death timing -- by the time a Beat opens,
/// whatever ran out in the previous one has already died, and whatever `act` does
/// happens on the Board that leaves behind.
///
/// A Beat nothing happened in is not logged and does not advance the count, which
/// is what keeps a Pass's Beat numbers to the Beats that exist.
fn run_beat(
    board: &mut Board,
    pass: u32,
    beat: &mut u32,
    log: &mut Vec<Event>,
    act: impl FnOnce(&mut Board, &mut Vec<Event>),
) {
    let mut happened = Vec::new();
    bury_the_dead(board, &mut happened);
    act(board, &mut happened);
    if happened.is_empty() {
        return;
    }
    *beat += 1;
    log.push(Event::BeatBegan { pass, beat: *beat });
    log.append(&mut happened);
}

/// The outcome, if the Action Phase is over.
///
/// Reads Slots, not health: a Party of Units that have all run out is not empty
/// until the Beat that buries them.
fn decide(board: &Board) -> Option<Outcome> {
    match (board.player.is_empty(), board.opposing.is_empty()) {
        (true, true) => Some(Outcome::Draw),
        (true, false) => Some(Outcome::OpposingWins),
        (false, true) => Some(Outcome::PlayerWins),
        (false, false) => None,
    }
}

/// Remove every Unit that ran out in an earlier Beat, returning what has Reborn
/// to spend.
///
/// The Slots left behind stay empty until the Parties next close ranks. Order
/// here is Side then Slot and means nothing: these deaths all belong to the same
/// Beat, and no two of them can affect each other.
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

/// Re-anchor a Party on its left-most Unit, logging it only if anything moved.
fn close_ranks(party: &mut Party, side: Side, log: &mut Vec<Event>) {
    if party.is_packed() {
        return;
    }
    party.compact();
    log.push(Event::Compacted { side });
}

/// Which of a Slot's two Beats is resolving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Swing {
    /// Whoever stands in the Slot attacks.
    First,
    /// Only a Windfury Unit attacks, having had the dead cleared in between.
    Windfury,
}

/// One attack, drawn but not yet resolved. Who swung, and at whom.
///
/// A Beat's attacks are all drawn before any of them resolves, so no attack can
/// take a target away from another -- which is what stops one side's attack from
/// pre-empting the other's.
struct Transaction {
    by: Side,
    attacker: usize,
    defender: usize,
}

/// Every attack of one Beat, drawn against the Board as the Beat found it.
///
/// A Unit that cannot attack -- absent, not the one this Beat is for, or with no
/// attack to deal -- draws nothing; Battlegrounds' zero-attack minions do not
/// swing either. Nothing is logged here: an attack is narrated when it resolves.
///
/// Nothing on the Board is dying at this point. The Beat opened by burying
/// whatever ran out, so every Unit `declare` can see is one that will still be
/// standing when the attacks land.
fn declare(board: &Board, index: usize, swing: Swing, rng: &mut Rng) -> Vec<Transaction> {
    let mut transactions = Vec::with_capacity(2);
    for by in [Side::Player, Side::Opposing] {
        let Some(attacker) = board.side(by).get(index) else {
            continue;
        };
        if swing == Swing::Windfury && !attacker.has(Keyword::Windfury) {
            continue;
        }
        if attacker.attack <= 0 {
            continue;
        }
        // An attack that finds nothing standing does not land: no Unit in
        // Battlegrounds attacks a Player.
        let Some(defender) = select_target(rng, board.side(by.other())) else {
            continue;
        };
        transactions.push(Transaction {
            by,
            attacker: index,
            defender,
        });
    }
    transactions
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
/// **An attack is a transaction, not a trade.** It has a direction: this Unit
/// swings at that one, and the one struck answers with its own attack in the same
/// motion. Two Units that chose each other in the same Beat are two transactions,
/// each with its own attacker -- not one symmetrical meeting. Collapsing them
/// would quietly restore the pre-emption a Beat exists to remove: in Battlegrounds
/// the second attack goes missing only because the first one killed its attacker
/// first.
///
/// Both Units' attack is read before either blow lands, so the exchange within a
/// transaction is genuinely mutual -- a Unit's answer is not weakened by the blow
/// it is answering.
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
/// to actually land. One blow, one shield: a Unit struck by two attacks in the same
/// Beat spends its shield on the first and takes the second, which is Battlegrounds'
/// rule and needs no help from ours.
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
    if unit.keywords.remove(&Keyword::DivineShield) {
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
