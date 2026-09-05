//! The Action Phase: Battlegrounds' attack order, resolved simultaneously.
//!
//! See [ADR 0008](../../../docs/adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md)
//! and [ADR 0009](../../../docs/adr/0009-the-party-is-left-anchored.md). Given the same
//! two Parties and the same Rng state, this always produces the same Resolution.
//!
//! - Each side traverses its own Party left to right, one attacker per Beat. A full
//!   traverse is a **Pass**.
//! - **The only delta from Battlegrounds' resolution**: both sides' current attacker act
//!   in the same moment, a **Beat**, instead of alternating turns. Nobody swings first.
//! - Targeting is **random** among the defending Party's living Units, unless that Party
//!   holds a Taunt Unit, in which case the attack must land on a Taunt holder. Exactly
//!   Battlegrounds' rule.
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
//! - A death leaves its Slot empty for the rest of the Pass. Parties close ranks toward
//!   their left-most Unit only at a Pass boundary, so nothing shifts under the cursor
//!   while a Pass runs (ADR 0009).
//! - Beats repeat until a Party empties, or [`MAX_BEATS`] is reached.

use crate::party::{Board, Party, SLOTS, Side, Unit};
use crate::rng::Rng;
use crate::units::{DefId, Keyword};

/// Beats after which an unresolved Action Phase is declared a stalemate.
///
/// Two Parties that cannot kill each other -- all zero-attack, say -- would
/// otherwise run forever. The cap makes non-termination a reported outcome
/// rather than a hang.
pub const MAX_BEATS: u32 = 512;

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
/// without reaching into engine internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A Party closed ranks toward its left-most Unit, at a Pass boundary.
    /// Logged only when it actually moved something.
    Compacted {
        side: Side,
    },
    BeatBegan {
        beat: u32,
        player_slot: usize,
        opposing_slot: usize,
    },
    Struck {
        by: Side,
        attacker_slot: usize,
        target_slot: usize,
        damage: i32,
        /// Which attack of the Beat this was, counting from 0. Non-zero means
        /// Windfury -- and a target drawn afresh.
        instance: u32,
    },
    ShieldAbsorbed {
        side: Side,
        slot: usize,
    },
    Died {
        side: Side,
        slot: usize,
        def: DefId,
        name: String,
        /// Killed by Poisonous rather than by running out of health.
        poisoned: bool,
    },
    Reborn {
        side: Side,
        slot: usize,
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
            Event::Compacted { side } => {
                write!(f, "  the {} party closes ranks", side.name())
            }
            Event::BeatBegan {
                beat,
                player_slot,
                opposing_slot,
            } => write!(
                f,
                "-- beat {beat}: player slot {player_slot} vs opposing slot {opposing_slot} --"
            ),
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
    let mut player_cursor = 0usize;
    let mut opposing_cursor = 0usize;

    // "Before the first Unit attacks": the Action Phase opens left-anchored,
    // whatever shape the Parties arrived in. A Party built in a Prep Phase is
    // already packed; one loaded from stored data need not be.
    close_ranks(&mut board.player, Side::Player, &mut log);
    close_ranks(&mut board.opposing, Side::Opposing, &mut log);

    let outcome = loop {
        match (board.player.is_empty(), board.opposing.is_empty()) {
            (true, true) => break Outcome::Draw,
            (true, false) => break Outcome::OpposingWins,
            (false, true) => break Outcome::PlayerWins,
            (false, false) => {}
        }
        if beat >= MAX_BEATS {
            break Outcome::Stalemate;
        }

        let player_slot = next_attacker(
            &mut board.player,
            &mut player_cursor,
            Side::Player,
            &mut log,
        )
        .expect("a non-empty Party always yields an attacker");
        let opposing_slot = next_attacker(
            &mut board.opposing,
            &mut opposing_cursor,
            Side::Opposing,
            &mut log,
        )
        .expect("a non-empty Party always yields an attacker");

        log.push(Event::BeatBegan {
            beat,
            player_slot,
            opposing_slot,
        });
        resolve_beat(&mut board, player_slot, opposing_slot, rng, &mut log);
        beat += 1;
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

/// The Slot of the next Unit to act for this side, advancing its cursor.
///
/// Scans right from the cursor for an occupied Slot. Running off the end ends
/// the Pass: the Party closes ranks and a new Pass begins at the left. This is
/// the *only* place a Party compacts during an Action Phase, which is what
/// holds Slots still under the cursor for the length of a Pass (ADR 0009).
fn next_attacker(
    party: &mut Party,
    cursor: &mut usize,
    side: Side,
    log: &mut Vec<Event>,
) -> Option<usize> {
    if let Some(slot) = (*cursor..SLOTS).find(|&slot| party.get(slot).is_some()) {
        *cursor = slot + 1;
        return Some(slot);
    }

    close_ranks(party, side, log);
    let slot = (0..SLOTS).find(|&slot| party.get(slot).is_some())?;
    *cursor = slot + 1;
    Some(slot)
}

/// Re-anchor a Party on its left-most Unit, logging it only if anything moved.
fn close_ranks(party: &mut Party, side: Side, log: &mut Vec<Event>) {
    if party.is_packed() {
        return;
    }
    party.compact();
    log.push(Event::Compacted { side });
}

/// One Beat: both sides' current attacker act simultaneously, then deaths apply.
fn resolve_beat(
    board: &mut Board,
    player_slot: usize,
    opposing_slot: usize,
    rng: &mut Rng,
    log: &mut Vec<Event>,
) {
    let player_attacks = board
        .player
        .get(player_slot)
        .map_or(0, Unit::actions_per_beat);
    let opposing_attacks = board
        .opposing
        .get(opposing_slot)
        .map_or(0, Unit::actions_per_beat);

    let mut attacked: Vec<(Side, usize)> = Vec::with_capacity(2);
    for instance in 0..player_attacks.max(opposing_attacks) {
        // Eligibility for this instance is decided from state *before* either
        // side swings in it -- otherwise a Unit killed by the other side's
        // simultaneous blow would lose its own, when a dying Unit is supposed to
        // still land the blow that kills it.
        let player_acts = instance < player_attacks
            && board.player.get(player_slot).is_some_and(|u| !u.is_dead());
        let opposing_acts = instance < opposing_attacks
            && board
                .opposing
                .get(opposing_slot)
                .is_some_and(|u| !u.is_dead());

        if player_acts {
            attack(board, Side::Player, player_slot, instance, rng, log);
            if !attacked.contains(&(Side::Player, player_slot)) {
                attacked.push((Side::Player, player_slot));
            }
        }
        if opposing_acts {
            attack(board, Side::Opposing, opposing_slot, instance, rng, log);
            if !attacked.contains(&(Side::Opposing, opposing_slot)) {
                attacked.push((Side::Opposing, opposing_slot));
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
    attacker_slot: usize,
    instance: u32,
    rng: &mut Rng,
    log: &mut Vec<Event>,
) {
    // Eligibility (including whether the attacker is already dead) was decided
    // by the caller from pre-instance state; this only needs the attacker's
    // stats, which do not change from taking damage.
    let Some(attacker) = board.side(by).get(attacker_slot) else {
        return;
    };
    let damage = attacker.attack;
    if damage <= 0 {
        return;
    }
    let poisonous = attacker.has(Keyword::Poisonous);
    let defending = by.other();
    let Some(target_slot) = select_target(rng, board.side(defending)) else {
        return;
    };

    strike(
        board,
        by,
        attacker_slot,
        defending,
        target_slot,
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
        .map(|(slot, _)| slot)
        .collect();
    if living.is_empty() {
        return None;
    }
    let taunts: Vec<usize> = living
        .iter()
        .copied()
        .filter(|&slot| party.get(slot).is_some_and(|u| u.has(Keyword::Taunt)))
        .collect();
    let pool = if taunts.is_empty() { &living } else { &taunts };
    rng.choose(pool).copied()
}

/// One attacker's blow against a chosen target.
#[allow(clippy::too_many_arguments)]
fn strike(
    board: &mut Board,
    by: Side,
    attacker_slot: usize,
    target_side: Side,
    target_slot: usize,
    damage: i32,
    poisonous: bool,
    instance: u32,
    log: &mut Vec<Event>,
) {
    let Some(target) = board.side_mut(target_side).get_mut(target_slot) else {
        return;
    };

    if target.keywords.remove(&Keyword::DivineShield) {
        // The shield eats the blow whole -- including its Poisonous, which needs
        // damage to actually land.
        log.push(Event::Struck {
            by,
            attacker_slot,
            target_slot,
            damage,
            instance,
        });
        log.push(Event::ShieldAbsorbed {
            side: target_side,
            slot: target_slot,
        });
        return;
    }

    target.health -= damage;
    if poisonous {
        target.doomed = true;
    }
    log.push(Event::Struck {
        by,
        attacker_slot,
        target_slot,
        damage,
        instance,
    });
}

/// Remove everything that died this Beat, resurrecting what has Reborn to spend.
///
/// A Unit that attacked this Beat is removed after every Unit that didn't, so a
/// kill stays attributable to its attacker even in a mutual trade (ADR 0008).
/// The Slots left behind stay empty until the Pass ends (ADR 0009).
fn apply_deaths(board: &mut Board, attacked: &[(Side, usize)], log: &mut Vec<Event>) {
    let mut bystander_deaths = Vec::new();
    let mut attacker_deaths = Vec::new();

    for side in [Side::Player, Side::Opposing] {
        for slot in 0..SLOTS {
            if !board.side(side).get(slot).is_some_and(Unit::is_dead) {
                continue;
            }
            if attacked.contains(&(side, slot)) {
                attacker_deaths.push((side, slot));
            } else {
                bystander_deaths.push((side, slot));
            }
        }
    }

    for (side, slot) in bystander_deaths.into_iter().chain(attacker_deaths) {
        remove_and_log(board, side, slot, log);
    }
}

/// Remove one dead Unit and log it, reviving it in place if it has Reborn to spend.
fn remove_and_log(board: &mut Board, side: Side, slot: usize, log: &mut Vec<Event>) {
    let unit = board
        .side_mut(side)
        .take(slot)
        .expect("just observed as occupied");
    log.push(Event::Died {
        side,
        slot,
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
        board.side_mut(side).put(slot, returned);
        log.push(Event::Reborn { side, slot, name });
    }
}

/// Build a Board from two lists of Units. Convenience for tests and setup.
pub fn board_of(player: Vec<crate::party::Unit>, opposing: Vec<crate::party::Unit>) -> Board {
    Board::new(
        Party::from_units(player).expect("player Party fits"),
        Party::from_units(opposing).expect("opposing Party fits"),
    )
}
