//! The Action Phase: Battlegrounds' attack order, resolved simultaneously.
//!
//! See [ADR 0008](../../../docs/adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md).
//! The Action Phase takes a Board and an [`Rng`]; given the same two Parties and the same
//! Rng state, it always produces the same Resolution. Every other rule below is exactly
//! Battlegrounds' own:
//!
//! - Each side cycles left-to-right through its own Party to find its next attacker,
//!   wrapping around -- Battlegrounds' own attack order, per side.
//! - **The only delta**: both sides' current attacker act in the same moment, a
//!   **Beat**, instead of alternating turns. Nobody swings first.
//! - Targeting is **random** among the opposing Party's living Units, unless that Party
//!   holds a Taunt Unit, in which case every attack against it must target a Taunt
//!   holder. Exactly Battlegrounds' rule; nothing about it changed.
//! - A Unit with Windfury acts **twice** in its Beat. Each swing draws its own random
//!   target; a target killed by an earlier swing this Beat cannot be drawn again.
//! - **Deaths apply at the end of the Beat** that caused them, so a fatally wounded Unit
//!   still lands the blow that killed it. Among units that died this Beat, one that
//!   attacked is removed *after* every one that didn't -- so a kill is attributable to
//!   its attacker even when the trade was mutual.
//! - If a side's Party is emptied mid-Beat by the other side's simultaneous swing, its
//!   remaining swings strike the Player directly, exactly as an emptied board does in
//!   Battlegrounds. This is the only source of damage this module produces; the
//!   end-of-fight damage-on-loss calculation is a v0.2/v0.3 concern computed from
//!   [`Resolution::final_board`].
//! - The Action Phase repeats Beats until a Party empties, or [`MAX_BEATS`] is reached.

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
        /// Which action of the Beat this was, counting from 0. Non-zero means
        /// Windfury.
        instance: u32,
    },
    ShieldAbsorbed {
        side: Side,
        slot: usize,
    },
    /// A side's Party was emptied mid-Beat by the other side's simultaneous
    /// swing, so this instance found nothing to target and struck the Player.
    StruckPlayer {
        by: Side,
        attacker_slot: usize,
        damage: i32,
        instance: u32,
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
            Event::StruckPlayer {
                by,
                attacker_slot,
                damage,
                instance,
            } => {
                let again = if *instance > 0 { " again" } else { "" };
                write!(
                    f,
                    "  {} slot {attacker_slot} finds no target and strikes{again} {} for {damage}",
                    by.name(),
                    by.other().player_name()
                )
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
    /// Damage struck through to each Player when a side's Party was emptied
    /// mid-Beat with swings still pending. Not a damage-on-loss total -- see
    /// the module documentation.
    pub damage_to_player: i32,
    pub damage_to_opposing: i32,
    pub beats: u32,
    pub log: Vec<Event>,
    /// The Board as it stood when the Action Phase ended.
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
    let mut damage_to_player = 0;
    let mut damage_to_opposing = 0;
    let mut beat = 0u32;
    let mut player_turns: usize = 0;
    let mut opposing_turns: usize = 0;

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

        let player_slot = player_turns % board.player.len();
        let opposing_slot = opposing_turns % board.opposing.len();
        log.push(Event::BeatBegan {
            beat,
            player_slot,
            opposing_slot,
        });

        resolve_beat(
            &mut board,
            player_slot,
            opposing_slot,
            rng,
            &mut log,
            &mut damage_to_player,
            &mut damage_to_opposing,
        );

        board.player.compact();
        board.opposing.compact();
        player_turns += 1;
        opposing_turns += 1;
        beat += 1;
    };

    log.push(Event::Ended {
        outcome,
        beats: beat,
    });
    Resolution {
        outcome,
        damage_to_player,
        damage_to_opposing,
        beats: beat,
        log,
        final_board: board,
    }
}

/// One Beat: both sides' current attacker act simultaneously, then deaths apply.
fn resolve_beat(
    board: &mut Board,
    player_slot: usize,
    opposing_slot: usize,
    rng: &mut Rng,
    log: &mut Vec<Event>,
    damage_to_player: &mut i32,
    damage_to_opposing: &mut i32,
) {
    let player_actions = board
        .player
        .get(player_slot)
        .map_or(0, Unit::actions_per_beat);
    let opposing_actions = board
        .opposing
        .get(opposing_slot)
        .map_or(0, Unit::actions_per_beat);
    let instances = player_actions.max(opposing_actions);

    let mut attacked: Vec<(Side, usize)> = Vec::with_capacity(2);
    for instance in 0..instances {
        // Eligibility for this instance is decided from state *before* either
        // side swings this instance -- otherwise a unit killed by the other
        // side's simultaneous blow this instance would wrongly lose its own,
        // when a dying Unit is supposed to still land the blow that kills it.
        let player_acts = instance < player_actions
            && board.player.get(player_slot).is_some_and(|u| !u.is_dead());
        let opposing_acts = instance < opposing_actions
            && board
                .opposing
                .get(opposing_slot)
                .is_some_and(|u| !u.is_dead());

        if player_acts {
            attack(
                board,
                Side::Player,
                player_slot,
                instance,
                rng,
                log,
                damage_to_opposing,
            );
            if !attacked.contains(&(Side::Player, player_slot)) {
                attacked.push((Side::Player, player_slot));
            }
        }
        if opposing_acts {
            attack(
                board,
                Side::Opposing,
                opposing_slot,
                instance,
                rng,
                log,
                damage_to_player,
            );
            if !attacked.contains(&(Side::Opposing, opposing_slot)) {
                attacked.push((Side::Opposing, opposing_slot));
            }
        }
    }

    apply_deaths(board, &attacked, log);
}

/// One attacker's single instance of action: a random target in the opposing
/// Party (respecting Taunt), or the opposing Player if none remain.
fn attack(
    board: &mut Board,
    by: Side,
    attacker_slot: usize,
    instance: u32,
    rng: &mut Rng,
    log: &mut Vec<Event>,
    damage_sink: &mut i32,
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
    let opposing_side = by.other();

    match select_target(rng, board.side(opposing_side)) {
        Some(target_slot) => strike(
            board,
            by,
            attacker_slot,
            opposing_side,
            target_slot,
            damage,
            poisonous,
            instance,
            log,
        ),
        None => {
            *damage_sink += damage;
            log.push(Event::StruckPlayer {
                by,
                attacker_slot,
                damage,
                instance,
            });
        }
    }
}

/// A random living Unit in `party`, constrained to Taunt holders if any are alive.
/// `None` means the Party has no living Unit left to target.
fn select_target(rng: &mut Rng, party: &Party) -> Option<usize> {
    let living: Vec<usize> = party
        .iter()
        .filter(|(_, u)| !u.is_dead())
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
