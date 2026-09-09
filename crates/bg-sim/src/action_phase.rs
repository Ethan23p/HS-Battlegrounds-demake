//! The Action Phase: Battlegrounds' attack order, resolved simultaneously.
//!
//! See [ADR 0008](../../../docs/adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md)
//! and [ADR 0009](../../../docs/adr/0009-the-party-is-left-anchored.md). Given the same
//! two Parties and the same Rng state, this always produces the same Resolution.
//!
//! **A Beat is a time-step**, not a turn anybody takes. The clock runs in Passes of nine
//! Beats:
//!
//! - **Beat 0** closes ranks: both Parties re-anchor on their left-most Unit.
//! - **Beats 1 through 8** are Slots 1 through 8. In Beat *n*, the Unit standing in Slot
//!   *n* acts -- on both sides at once, which is the one delta from Battlegrounds'
//!   resolution. Nobody swings first. An empty Slot simply has nobody to act.
//!
//! **The delta is one sentence: ordering grants no advantage.** Battlegrounds picks a
//! side to swing first, and in a mirror that side wins; here Slot *n* acts on both sides
//! in the same Beat, so there is no first swing to have. Everything else in this module
//! is Battlegrounds' own rule, and where simultaneity leaves Battlegrounds with nothing
//! to say, the tie-break is chosen to be one no ordering could change.
//!
//! - **An attack is an exchange.** The attacker deals its attack to its target and the
//!   target deals its attack back, in the same instant. That is Battlegrounds, and it is
//!   what makes health, Taunt and Poisonous mean anything on a Unit that is not currently
//!   swinging.
//! - Targeting is **random** among the defending Party's living Units, unless that Party
//!   holds a Taunt Unit, in which case the attack must land on a Taunt holder.
//! - **Every attack is its own choosing of a target.** A Unit with Windfury attacks twice
//!   in its Beat and draws again for the second, which may well be a different Unit.
//! - Nothing ever attacks a Player. An attack with no living Unit left to target simply
//!   does not land -- Hearthstone lets minions go face, Battlegrounds does not, and
//!   damage-on-loss is a single end-of-fight calculation belonging to v0.2/v0.3, computed
//!   from the survivors on [`Resolution::final_board`].
//! - **Deaths resolve immediately after the attack that caused them**, as in
//!   Battlegrounds. A Beat runs in *instances* -- one, or two for Windfury -- and an
//!   instance is the indivisible unit: both sides declare, every blow is answered, all of
//!   that damage lands together, then the instance's dead are removed. So a fatally
//!   wounded Unit still lands the blow that killed it, and a Unit killed in instance 0
//!   does not swing again in instance 1. Corpses never fight on.
//! - Among an instance's dead, a Unit that **attacked** is removed after every one that
//!   didn't, so a kill stays attributable to its attacker even when the trade was mutual.
//!   Answering a blow is not attacking.
//! - A death leaves its Slot empty for the rest of the Pass; the next Beat 0 closes it.
//!   Nothing shifts under the clock while a Pass runs (ADR 0009).
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
    /// Beat 0 of a Pass: this Party closed ranks. Logged only when it actually
    /// moved something.
    Compacted {
        pass: u32,
        side: Side,
    },
    /// A Beat in which somebody stands to act. `beat` is the Slot it resolves,
    /// so Beat 3 is Slot 3; Beat 0 is the compaction step and never appears
    /// here. Beats with both Slots empty are not logged -- nothing happened.
    BeatBegan {
        pass: u32,
        beat: u32,
    },
    Struck {
        by: Side,
        attacker_slot: u32,
        target_slot: u32,
        damage: i32,
        /// Which attack of the Beat this was, counting from 0. Non-zero means
        /// Windfury -- and a target drawn afresh.
        instance: u32,
    },
    /// A defender answering the blow it was struck by, in the same instant. Not
    /// an attack: it draws no target, and it does not make the defender an
    /// attacker for the purpose of who is removed last.
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
            Event::Compacted { side, .. } => {
                write!(f, "  the {} party closes ranks", side.name())
            }
            Event::BeatBegan { pass, beat } => {
                write!(f, "-- pass {pass}, beat {beat} --")
            }
            Event::Struck {
                by,
                attacker_slot,
                target_slot,
                damage,
                instance,
            } => {
                let again = if *instance > 0 { " again" } else { "" };
                write!(
                    f,
                    "  {} slot {attacker_slot} strikes{again} {} slot {target_slot} for {damage}",
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
    /// Passes entered, counting from 1. A fight decided before anyone acted ran
    /// none.
    pub passes: u32,
    pub log: Vec<Event>,
    /// The Board as it stood when the Action Phase ended. The winner's survivors
    /// are what a damage-on-loss calculation will read, once Health exists.
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
    let mut pass = 0u32;

    let outcome = loop {
        if let Some(outcome) = decide(&board) {
            break outcome;
        }
        if pass >= MAX_PASSES {
            break Outcome::Stalemate;
        }
        pass += 1;

        // Beat 0.
        close_ranks(&mut board.player, pass, Side::Player, &mut log);
        close_ranks(&mut board.opposing, pass, Side::Opposing, &mut log);

        // Beats 1 through 8: Beat n resolves Slot n, on both sides at once.
        for index in 0..SLOTS {
            if decide(&board).is_some() {
                break;
            }
            if board.player.get(index).is_none() && board.opposing.get(index).is_none() {
                continue;
            }
            log.push(Event::BeatBegan {
                pass,
                beat: slot_no(index),
            });
            resolve_beat(&mut board, index, rng, &mut log);
        }
    };

    log.push(Event::Ended {
        outcome,
        passes: pass,
    });
    Resolution {
        outcome,
        passes: pass,
        log,
        final_board: board,
    }
}

/// The outcome, if the Action Phase is over.
fn decide(board: &Board) -> Option<Outcome> {
    match (board.player.is_empty(), board.opposing.is_empty()) {
        (true, true) => Some(Outcome::Draw),
        (true, false) => Some(Outcome::OpposingWins),
        (false, true) => Some(Outcome::PlayerWins),
        (false, false) => None,
    }
}

/// Beat 0: re-anchor a Party on its left-most Unit, logging it only if anything
/// moved. This is the only moment an Action Phase compacts, which is what holds
/// Slots still under the clock for the length of a Pass (ADR 0009).
fn close_ranks(party: &mut Party, pass: u32, side: Side, log: &mut Vec<Event>) {
    if party.is_packed() {
        return;
    }
    party.compact();
    log.push(Event::Compacted { pass, side });
}

/// One Beat: whoever stands in this Slot, on either side, acts -- and with
/// Windfury, acts again.
///
/// A Beat runs in **instances**: one, or two for a Windfury Unit. The instance is
/// the indivisible step. Within it both sides' Slot-`index` Unit declares its
/// attack against the Board as it stood when the instance began, every blow is
/// answered, all of that damage lands at once, and only then are the dead removed.
///
/// Nothing in that sequence can be changed by considering one side before the
/// other, which is the whole of the delta: ordering grants no advantage.
fn resolve_beat(board: &mut Board, index: usize, rng: &mut Rng, log: &mut Vec<Event>) {
    let instances = [Side::Player, Side::Opposing]
        .into_iter()
        .filter_map(|side| board.side(side).get(index))
        .map(Unit::actions_per_beat)
        .max()
        .unwrap_or(0);

    for instance in 0..instances {
        let blows = declare(board, index, instance, rng);
        if blows.is_empty() {
            continue;
        }
        for blow in &blows {
            resolve_attack(board, blow, log);
        }
        let attackers: Vec<(Side, usize)> = blows.iter().map(|b| (b.by, b.attacker)).collect();
        apply_deaths(board, &attackers, log);
    }
}

/// One attack, drawn but not yet resolved. Who swung, and at whom.
///
/// Targets for a whole instance are drawn before any of them resolve, so no
/// attack can take a target away from another. That, and nothing else, is what
/// stops one side's attack from pre-empting the other's.
struct Blow {
    by: Side,
    attacker: usize,
    defender: usize,
    instance: u32,
}

/// Every attack of one instance, drawn against the Board as the instance found it.
///
/// A Unit that cannot attack -- absent, dead, out of instances, or with no attack
/// to deal -- draws nothing; Battlegrounds' zero-attack minions do not swing
/// either. Nothing is logged here: an attack is narrated when it resolves.
fn declare(board: &Board, index: usize, instance: u32, rng: &mut Rng) -> Vec<Blow> {
    let mut blows = Vec::with_capacity(2);
    for by in [Side::Player, Side::Opposing] {
        let Some(attacker) = board.side(by).get(index) else {
            continue;
        };
        if instance >= attacker.actions_per_beat() || attacker.is_dead() || attacker.attack <= 0 {
            continue;
        }
        // An attack that finds nothing left standing does not land: no Unit in
        // Battlegrounds attacks a Player.
        let Some(defender) = select_target(rng, board.side(by.other())) else {
            continue;
        };
        blows.push(Blow {
            by,
            attacker: index,
            defender,
            instance,
        });
    }
    blows
}

/// A random living Unit in `party`, constrained to Taunt holders if any are alive.
/// `None` means the Party has no living Unit left to target.
fn select_target(rng: &mut Rng, party: &Party) -> Option<usize> {
    let living: Vec<usize> = party
        .iter()
        .filter(|(_, unit)| !unit.is_dead())
        .map(|(index, _)| index)
        .collect();
    if living.is_empty() {
        return None;
    }
    let taunts: Vec<usize> = living
        .iter()
        .copied()
        .filter(|&index| party.get(index).is_some_and(|u| u.has(Keyword::Taunt)))
        .collect();
    let pool = if taunts.is_empty() { &living } else { &taunts };
    rng.choose(pool).copied()
}

/// One attack, resolved exactly as Battlegrounds resolves one.
///
/// **An attack is a transaction, not a trade.** It has a direction: this Unit
/// swings at that one, and the one struck answers with its own attack in the same
/// motion. Two Units that chose each other in the same Beat are two transactions,
/// each with its own attacker -- not one symmetrical meeting. Collapsing them
/// would be reaching for "one action, one outcome," and it would quietly restore
/// the pre-emption this Action Phase exists to remove: in Battlegrounds the second
/// attack goes missing only because the first one killed its attacker first.
///
/// Both Units' attack is read before either blow lands, so the exchange within a
/// transaction is genuinely mutual -- a Unit's answer is not weakened by the blow
/// it is answering.
fn resolve_attack(board: &mut Board, blow: &Blow, log: &mut Vec<Event>) {
    let defending = blow.by.other();
    let (Some(attacker), Some(defender)) = (
        board.side(blow.by).get(blow.attacker),
        board.side(defending).get(blow.defender),
    ) else {
        return;
    };
    let (attack, attacker_poisons) = (attacker.attack, attacker.has(Keyword::Poisonous));
    let (answer, defender_poisons) = (defender.attack, defender.has(Keyword::Poisonous));

    log.push(Event::Struck {
        by: blow.by,
        attacker_slot: slot_no(blow.attacker),
        target_slot: slot_no(blow.defender),
        damage: attack,
        instance: blow.instance,
    });
    hit(
        board,
        defending,
        blow.defender,
        attack,
        attacker_poisons,
        log,
    );

    if answer > 0 {
        log.push(Event::StruckBack {
            by: defending,
            slot: slot_no(blow.defender),
            target_slot: slot_no(blow.attacker),
            damage: answer,
        });
        hit(board, blow.by, blow.attacker, answer, defender_poisons, log);
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
        unit.doomed = true;
    }
}

/// Remove everything that died in this instance, resurrecting what has Reborn to
/// spend.
///
/// A Unit that attacked in this instance is removed after every Unit that didn't,
/// so a kill stays attributable to its attacker even in a mutual trade (ADR 0008).
/// A Unit that merely answered a blow did not attack. The Slots left behind stay
/// empty until the next Beat 0 (ADR 0009).
fn apply_deaths(board: &mut Board, attacked: &[(Side, usize)], log: &mut Vec<Event>) {
    let mut bystander_deaths = Vec::new();
    let mut attacker_deaths = Vec::new();

    for side in [Side::Player, Side::Opposing] {
        for index in 0..SLOTS {
            if !board.side(side).get(index).is_some_and(Unit::is_dead) {
                continue;
            }
            if attacked.contains(&(side, index)) {
                attacker_deaths.push((side, index));
            } else {
                bystander_deaths.push((side, index));
            }
        }
    }

    for (side, index) in bystander_deaths.into_iter().chain(attacker_deaths) {
        remove_and_log(board, side, index, log);
    }
}

/// Remove one dead Unit and log it, reviving it in place if it has Reborn to spend.
fn remove_and_log(board: &mut Board, side: Side, index: usize, log: &mut Vec<Event>) {
    let unit = board
        .side_mut(side)
        .take(index)
        .expect("just observed as occupied");
    log.push(Event::Died {
        side,
        slot: slot_no(index),
        def: unit.def.clone(),
        name: unit.name.clone(),
        poisoned: unit.doomed,
    });

    if unit.has(Keyword::Reborn) && !unit.reborn_spent {
        let mut returned = unit;
        returned.reborn_spent = true;
        returned.keywords.remove(&Keyword::Reborn);
        returned.health = 1;
        returned.doomed = false;
        let name = returned.name.clone();
        board.side_mut(side).put(index, returned);
        log.push(Event::Reborn {
            side,
            slot: slot_no(index),
            name,
        });
    }
}

/// Build a Board from two lists of Units. Convenience for tests and setup.
pub fn board_of(player: Vec<crate::party::Unit>, opposing: Vec<crate::party::Unit>) -> Board {
    Board::new(
        Party::from_units(player).expect("player Party fits"),
        Party::from_units(opposing).expect("opposing Party fits"),
    )
}
