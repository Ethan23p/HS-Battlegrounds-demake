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
//! The rest is Battlegrounds' own:
//!
//! - Targeting is **random** among the defending Party's living Units, unless that Party
//!   holds a Taunt Unit, in which case the attack must land on a Taunt holder.
//! - **Every attack is its own choosing of a target.** A Unit with Windfury attacks twice
//!   in its Beat and draws again for the second, which may well be a different Unit.
//! - Nothing ever attacks a Player. An attack with no living Unit left to target simply
//!   does not land -- Hearthstone lets minions go face, Battlegrounds does not, and
//!   damage-on-loss is a single end-of-fight calculation belonging to v0.2/v0.3, computed
//!   from the survivors on [`Resolution::final_board`].
//! - **Deaths apply at the end of the Beat** that caused them, so a fatally wounded Unit
//!   still lands the blow that killed it. Among the Beat's dead, a Unit that attacked is
//!   removed *after* every one that didn't, so a kill stays attributable to its attacker
//!   even when the trade was mutual.
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

/// One Beat: whoever stands in this Slot, on either side, acts simultaneously,
/// then the Beat's deaths apply.
fn resolve_beat(board: &mut Board, index: usize, rng: &mut Rng, log: &mut Vec<Event>) {
    let player_attacks = board.player.get(index).map_or(0, Unit::actions_per_beat);
    let opposing_attacks = board.opposing.get(index).map_or(0, Unit::actions_per_beat);

    let mut attacked: Vec<(Side, usize)> = Vec::with_capacity(2);
    for instance in 0..player_attacks.max(opposing_attacks) {
        // Eligibility for this instance is decided from state *before* either
        // side swings in it -- otherwise a Unit killed by the other side's
        // simultaneous blow would lose its own, when a dying Unit is supposed to
        // still land the blow that kills it.
        let player_acts =
            instance < player_attacks && board.player.get(index).is_some_and(|u| !u.is_dead());
        let opposing_acts =
            instance < opposing_attacks && board.opposing.get(index).is_some_and(|u| !u.is_dead());

        if player_acts {
            attack(board, Side::Player, index, instance, rng, log);
            if !attacked.contains(&(Side::Player, index)) {
                attacked.push((Side::Player, index));
            }
        }
        if opposing_acts {
            attack(board, Side::Opposing, index, instance, rng, log);
            if !attacked.contains(&(Side::Opposing, index)) {
                attacked.push((Side::Opposing, index));
            }
        }
    }

    apply_deaths(board, &attacked, log);
}

/// One attack: draw a target from the defending Party and strike it.
///
/// The draw happens per attack, not per Beat, so Windfury's second attack picks
/// its own target. An attack that finds nothing left standing does not land --
/// no Unit in Battlegrounds attacks a Player.
fn attack(
    board: &mut Board,
    by: Side,
    attacker_index: usize,
    instance: u32,
    rng: &mut Rng,
    log: &mut Vec<Event>,
) {
    // Eligibility (including whether the attacker is already dead) was decided
    // by the caller from pre-instance state; this only needs the attacker's
    // stats, which do not change from taking damage.
    let Some(attacker) = board.side(by).get(attacker_index) else {
        return;
    };
    let damage = attacker.attack;
    if damage <= 0 {
        return;
    }
    let poisonous = attacker.has(Keyword::Poisonous);
    let defending = by.other();
    let Some(target_index) = select_target(rng, board.side(defending)) else {
        return;
    };

    strike(
        board,
        by,
        attacker_index,
        defending,
        target_index,
        damage,
        poisonous,
        instance,
        log,
    );
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

/// One attacker's blow against a chosen target.
#[allow(clippy::too_many_arguments)]
fn strike(
    board: &mut Board,
    by: Side,
    attacker_index: usize,
    target_side: Side,
    target_index: usize,
    damage: i32,
    poisonous: bool,
    instance: u32,
    log: &mut Vec<Event>,
) {
    let Some(target) = board.side_mut(target_side).get_mut(target_index) else {
        return;
    };
    let struck = Event::Struck {
        by,
        attacker_slot: slot_no(attacker_index),
        target_slot: slot_no(target_index),
        damage,
        instance,
    };

    if target.keywords.remove(&Keyword::DivineShield) {
        // The shield eats the blow whole -- including its Poisonous, which needs
        // damage to actually land.
        log.push(struck);
        log.push(Event::ShieldAbsorbed {
            side: target_side,
            slot: slot_no(target_index),
        });
        return;
    }

    target.health -= damage;
    if poisonous {
        target.doomed = true;
    }
    log.push(struck);
}

/// Remove everything that died this Beat, resurrecting what has Reborn to spend.
///
/// A Unit that attacked this Beat is removed after every Unit that didn't, so a
/// kill stays attributable to its attacker even in a mutual trade (ADR 0008).
/// The Slots left behind stay empty until the next Beat 0 (ADR 0009).
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
