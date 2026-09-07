//! The Action Phase: Battlegrounds' attack order, resolved simultaneously.
//!
//! See [ADR 0008](../../../docs/adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md)
//! and [ADR 0010](../../../docs/adr/0010-the-clock-is-a-beat-counter.md). Given the same
//! two Parties and the same Rng state, this always produces the same Resolution.
//!
//! **A Beat is a time-step**, not a turn anybody takes, and the count runs one
//! way for the whole Action Phase: Beat 1, 2, 3, ... It never resets, and there
//! is no larger unit of time above it.
//!
//! In each Beat, the left-most **Ready** Unit of each Party acts -- on both sides
//! at once, which is the one delta from Battlegrounds' resolution. Nobody swings
//! first. Acting spends a Unit's readiness. When no Unit on either side is Ready,
//! every Unit becomes Ready again: that is the whole of the clock's structure,
//! and the only moment *before the first Unit acts* that anything can hang a
//! trigger on. It is a condition met, not a step counted -- there is no looping
//! back to a start.
//!
//! Because readiness rides on the Unit rather than on a Slot number, a Party can
//! close up behind its dead the instant they fall without the clock skipping a
//! Unit or visiting one twice. Nothing acts twice before everything standing has
//! had its turn; a Unit that dies before its turn comes never gets one.
//!
//! The rest is Battlegrounds' own:
//!
//! - Targeting is **random** among the defending Party's living Units, unless that
//!   Party holds a Taunt Unit, in which case the attack must land on a Taunt
//!   holder.
//! - **Every attack is its own choosing of a target.** A Unit with Windfury
//!   attacks twice in its Beat and draws again for the second, which may well be
//!   a different Unit.
//! - Nothing ever attacks a Player. An attack with no living Unit left to target
//!   simply does not land -- Hearthstone lets minions go face, Battlegrounds does
//!   not, and damage-on-loss is a single end-of-fight calculation belonging to
//!   v0.2/v0.3, computed from the survivors on [`Resolution::final_board`].
//! - **Deaths apply at the end of the Beat** that caused them, so a fatally
//!   wounded Unit still lands the blow that killed it, and nothing moves under a
//!   Beat while it is being resolved. Among the Beat's dead, a Unit that attacked
//!   is removed *after* every one that didn't, so a kill stays attributable to
//!   its attacker even when the trade was mutual.
//! - Beats run until a Party empties, or [`MAX_BEATS`] is reached.
//!
//! A Beat is therefore the smallest span worth comparing across: read the Board
//! before it and after it, and the difference is everything that Beat did. The
//! order of the strikes inside one is bookkeeping, not rules.
//!
//! Slots are numbered **1 through 8** in the rules, in this log, and in every
//! Event below, counting from the left of a Party that has no gaps. The array
//! index behind a Slot is zero-based, and only this module knows it.

use crate::party::{Board, Party, Side, Unit};
use crate::rng::Rng;
use crate::units::{DefId, Keyword};

/// Beats after which an unresolved Action Phase is declared a stalemate.
///
/// Two Parties that cannot kill each other -- all zero-attack, say -- would
/// otherwise run forever. The cap makes non-termination a reported outcome
/// rather than a hang. Sixty-four full Parties' worth of Beats: far past any
/// fight that is actually going somewhere.
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
/// without reaching into engine internals. Every Slot number here counts from 1,
/// and names where a Unit stood at the moment of the Event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Nobody owed the clock a turn, so everybody owes one again. Logged for the
    /// Beat it precedes -- the *before the first Unit acts* moment, and the only
    /// boundary the Action Phase has.
    AllReady {
        beat: u32,
    },
    /// A Beat began. Beats count from 1 and never reset.
    BeatBegan {
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
    /// `slot` is where the Unit stood as it died, before its Party closed up.
    Died {
        side: Side,
        slot: u32,
        def: DefId,
        name: String,
        /// Killed by Poisonous rather than by running out of health.
        poisoned: bool,
    },
    /// `slot` is where the returning Unit stands now, after its Party closed up
    /// around the Beat's dead.
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
            Event::AllReady { .. } => {
                write!(f, ".. every unit is ready ..")
            }
            Event::BeatBegan { beat } => {
                write!(f, "-- beat {beat} --")
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
    let mut beat = 0u32;
    // The Action Phase opens on the same footing a refresh leaves it in: nobody
    // has acted yet, so the Beat about to run is a first-Unit-acts moment.
    let mut all_ready = true;

    let outcome = loop {
        if let Some(outcome) = decide(&board) {
            break outcome;
        }
        if beat >= MAX_BEATS {
            break Outcome::Stalemate;
        }
        if !board.has_ready() {
            board.ready_all();
            all_ready = true;
        }

        beat += 1;
        if all_ready {
            log.push(Event::AllReady { beat });
            all_ready = false;
        }
        log.push(Event::BeatBegan { beat });
        resolve_beat(&mut board, rng, &mut log);
    };

    log.push(Event::Ended {
        outcome,
        beats: beat,
    });
    Resolution {
        outcome,
        beats: beat,
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

/// One Beat: the left-most Ready Unit of each Party acts, simultaneously, and
/// then the Beat's deaths apply.
///
/// Nothing is removed part-way through, so the Slot each actor occupies holds
/// still for the length of the Beat and the two indices below stay valid.
fn resolve_beat(board: &mut Board, rng: &mut Rng, log: &mut Vec<Event>) {
    let actors = [
        (Side::Player, board.player.first_ready()),
        (Side::Opposing, board.opposing.first_ready()),
    ];
    for (side, index) in actors {
        if let Some(index) = index {
            // Readiness is spent by taking the turn, not by landing a blow: a
            // Unit with no attack still had its Beat.
            if let Some(unit) = board.side_mut(side).get_mut(index) {
                unit.ready = false;
            }
        }
    }

    let attacks = |party: &Party, index: Option<usize>| {
        index
            .and_then(|index| party.get(index))
            .map_or(0, Unit::actions_per_beat)
    };
    let player_attacks = attacks(&board.player, actors[0].1);
    let opposing_attacks = attacks(&board.opposing, actors[1].1);

    let mut attacked: Vec<(Side, usize)> = Vec::with_capacity(2);
    for instance in 0..player_attacks.max(opposing_attacks) {
        // Eligibility for this instance is decided from state *before* either
        // side swings in it -- otherwise a Unit killed by the other side's
        // simultaneous blow would lose its own, when a dying Unit is supposed to
        // still land the blow that kills it.
        let eligible = |party: &Party, index: Option<usize>, budget: u32| {
            instance < budget
                && index.is_some_and(|index| party.get(index).is_some_and(|u| !u.is_dead()))
        };
        let player_acts = eligible(&board.player, actors[0].1, player_attacks);
        let opposing_acts = eligible(&board.opposing, actors[1].1, opposing_attacks);

        for (side, acts) in [(Side::Player, player_acts), (Side::Opposing, opposing_acts)] {
            if !acts {
                continue;
            }
            let index = actors[usize::from(side == Side::Opposing)]
                .1
                .expect("eligibility implies an actor");
            attack(board, side, index, instance, rng, log);
            if !attacked.contains(&(side, index)) {
                attacked.push((side, index));
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

/// Remove everything that died this Beat, bringing back what has Reborn to spend.
///
/// A Unit that attacked this Beat is recorded as dying after every Unit that
/// didn't, so a kill stays attributable to its attacker even in a mutual trade
/// (ADR 0008). The removal itself is one sweep per Party once all the deaths are
/// on record, so the survivors close up exactly once and the Board is left with
/// no gaps to carry into the next Beat.
fn apply_deaths(board: &mut Board, attacked: &[(Side, usize)], log: &mut Vec<Event>) {
    let mut bystanders = Vec::new();
    let mut attackers = Vec::new();

    for side in [Side::Player, Side::Opposing] {
        for (index, unit) in board.side(side).iter() {
            if !unit.is_dead() {
                continue;
            }
            let died = Event::Died {
                side,
                slot: slot_no(index),
                def: unit.def.clone(),
                name: unit.name.clone(),
                poisoned: unit.doomed,
            };
            if attacked.contains(&(side, index)) {
                attackers.push(died);
            } else {
                bystanders.push(died);
            }
        }
    }

    if bystanders.is_empty() && attackers.is_empty() {
        return;
    }
    log.extend(bystanders);
    log.extend(attackers);

    for side in [Side::Player, Side::Opposing] {
        for (index, name) in board.side_mut(side).sweep_dead() {
            log.push(Event::Reborn {
                side,
                slot: slot_no(index),
                name,
            });
        }
    }
}

/// Build a Board from two lists of Units. Convenience for tests and setup.
pub fn board_of(player: Vec<crate::party::Unit>, opposing: Vec<crate::party::Unit>) -> Board {
    Board::new(
        Party::from_units(player).expect("player Party fits"),
        Party::from_units(opposing).expect("opposing Party fits"),
    )
}
