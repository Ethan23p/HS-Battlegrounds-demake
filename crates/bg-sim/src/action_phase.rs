//! The Action Phase: a left-to-right sweep of Beats.
//!
//! The whole of ADR 0003 lives here. The Action Phase takes no input; given two
//! Parties it produces one answer, and the same two Parties always produce the
//! same answer. Note what is *absent*: no coin flip, no target selection, no
//! attack order. There is not yet any randomness in this module at all, because
//! the positional sweep removed every decision that needed it.
//!
//! The rules, in full:
//!
//! - The sweep visits Slots 0 through 7 in order. Each visit to a Slot holding at
//!   least one Unit is a **Beat**.
//! - Within a Beat, both Units act **from the state at the Beat's opening**. A
//!   Unit that takes a fatal wound still lands its own blow: it does not die
//!   until the Beat ends.
//! - A Unit with Windfury acts **twice** in its Beat. Damage still lands instance
//!   by instance, so a Divine Shield absorbs the first blow and the second
//!   connects.
//! - A Unit facing an **empty Slot** strikes the opposing Player instead.
//! - **Deaths apply at the end of the Beat** that caused them, leaving a hole. The
//!   hole stays open for the rest of the Sweep, so facings hold still while a
//!   Sweep runs.
//! - At the end of a Sweep, both Parties **compact**. Because that restores the
//!   left-packed invariant, Slot 0 is occupied on both sides whenever both
//!   Parties are alive -- which is what guarantees the Action Phase terminates
//!   rather than sweeping past each other forever.
//! - The sweep repeats until a Party is empty, or [`MAX_SWEEPS`] is reached.

use crate::party::{Board, Party, SLOTS, Side};
use crate::units::{DefId, Keyword};

/// Sweeps after which an unresolved Action Phase is declared a stalemate.
///
/// Two Parties that cannot kill each other -- all zero-attack, say -- would
/// otherwise sweep forever. The cap makes non-termination a reported outcome
/// rather than a hang.
pub const MAX_SWEEPS: u32 = 64;

/// How an Action Phase ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    PlayerWins,
    OpposingWins,
    /// Both Parties emptied together.
    Draw,
    /// Neither Party could finish the other within [`MAX_SWEEPS`].
    Stalemate,
}

/// Everything that happened, in the order it happened.
///
/// The log is the Action Phase's explanation of itself. A frontend animates
/// these in order and needs to know nothing else; a test asserts against them
/// without reaching into engine internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    SweepBegan {
        sweep: u32,
    },
    BeatBegan {
        sweep: u32,
        slot: usize,
    },
    Struck {
        by: Side,
        slot: usize,
        target: Side,
        damage: i32,
        /// Which action of the Beat this was, counting from 0. Non-zero means
        /// Windfury.
        instance: u32,
    },
    ShieldAbsorbed {
        side: Side,
        slot: usize,
    },
    /// A Unit facing an empty Slot struck the opposing Player.
    StruckPlayer {
        by: Side,
        slot: usize,
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
        sweeps: u32,
    },
}

impl std::fmt::Display for Event {
    /// A one-line rendering, so a log reads as prose.
    ///
    /// Formatting, not I/O: this returns a string and never touches a stream, so
    /// the engine keeps its promise not to print.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::SweepBegan { sweep } => write!(f, "-- sweep {sweep} --"),
            Event::BeatBegan { slot, .. } => write!(f, "slot {slot}:"),
            Event::Struck {
                by,
                slot,
                damage,
                instance,
                ..
            } => {
                let again = if *instance > 0 { " again" } else { "" };
                write!(f, "  {} slot {slot} strikes{again} for {damage}", by.name())
            }
            Event::ShieldAbsorbed { side, slot } => {
                write!(f, "  {} slot {slot} absorbs it on its shield", side.name())
            }
            Event::StruckPlayer {
                by,
                slot,
                damage,
                instance,
            } => {
                let again = if *instance > 0 { " again" } else { "" };
                write!(
                    f,
                    "  {} slot {slot} is unopposed and hits {}{again} for {damage}",
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
            Event::Ended { outcome, sweeps } => {
                write!(f, "== {outcome:?} after {sweeps} sweeps ==")
            }
        }
    }
}

/// What an Action Phase produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub outcome: Outcome,
    /// Damage struck through to each Player by unopposed Units.
    pub damage_to_player: i32,
    pub damage_to_opposing: i32,
    pub sweeps: u32,
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
/// The entire module behind one function of one argument. Deterministic: equal
/// Boards give equal Resolutions, and there is no seed to pass because nothing
/// here is random.
pub fn resolve(mut board: Board) -> Resolution {
    let mut log = Vec::new();
    let mut damage_to_player = 0;
    let mut damage_to_opposing = 0;
    let mut sweep = 0;

    let outcome = loop {
        match (board.player.is_empty(), board.opposing.is_empty()) {
            (true, true) => break Outcome::Draw,
            (true, false) => break Outcome::OpposingWins,
            (false, true) => break Outcome::PlayerWins,
            (false, false) => {}
        }
        if sweep >= MAX_SWEEPS {
            break Outcome::Stalemate;
        }

        log.push(Event::SweepBegan { sweep });
        for slot in 0..SLOTS {
            if board.player.get(slot).is_none() && board.opposing.get(slot).is_none() {
                continue;
            }
            log.push(Event::BeatBegan { sweep, slot });
            resolve_beat(
                &mut board,
                slot,
                &mut log,
                &mut damage_to_player,
                &mut damage_to_opposing,
            );
            if board.player.is_empty() || board.opposing.is_empty() {
                break;
            }
        }
        board.player.compact();
        board.opposing.compact();
        sweep += 1;
    };

    log.push(Event::Ended {
        outcome,
        sweeps: sweep,
    });
    Resolution {
        outcome,
        damage_to_player,
        damage_to_opposing,
        sweeps: sweep,
        log,
        final_board: board,
    }
}

/// One Beat: resolve a single Slot, then apply the deaths it caused.
fn resolve_beat(
    board: &mut Board,
    slot: usize,
    log: &mut Vec<Event>,
    damage_to_player: &mut i32,
    damage_to_opposing: &mut i32,
) {
    // Snapshot both Units as the Beat opens. Every blow struck in this Beat is
    // struck by the Unit as it was now -- a Unit that takes a fatal wound
    // mid-Beat still lands its own, because it does not die until the Beat ends.
    let player = board.player.get(slot).cloned();
    let opposing = board.opposing.get(slot).cloned();

    match (player, opposing) {
        (Some(p), Some(o)) => {
            let instances = p.actions_per_beat().max(o.actions_per_beat());
            for instance in 0..instances {
                if instance < p.actions_per_beat() {
                    strike(
                        board,
                        Side::Player,
                        slot,
                        p.attack,
                        p.has(Keyword::Poisonous),
                        instance,
                        log,
                    );
                }
                if instance < o.actions_per_beat() {
                    strike(
                        board,
                        Side::Opposing,
                        slot,
                        o.attack,
                        o.has(Keyword::Poisonous),
                        instance,
                        log,
                    );
                }
            }
        }
        (Some(p), None) => {
            strike_player(Side::Player, slot, &p, damage_to_opposing, log);
        }
        (None, Some(o)) => {
            strike_player(Side::Opposing, slot, &o, damage_to_player, log);
        }
        (None, None) => unreachable!("a Beat only begins on an occupied Slot"),
    }

    apply_deaths(board, log);
}

/// One Unit's blow against the Unit facing it.
fn strike(
    board: &mut Board,
    by: Side,
    slot: usize,
    damage: i32,
    poisonous: bool,
    instance: u32,
    log: &mut Vec<Event>,
) {
    if damage <= 0 {
        return;
    }
    let target_side = by.other();
    let Some(target) = board.side_mut(target_side).get_mut(slot) else {
        return;
    };

    if target.keywords.remove(&Keyword::DivineShield) {
        // The shield eats the blow whole -- including its Poisonous, which needs
        // damage to actually land.
        log.push(Event::Struck {
            by,
            slot,
            target: target_side,
            damage,
            instance,
        });
        log.push(Event::ShieldAbsorbed {
            side: target_side,
            slot,
        });
        return;
    }

    target.health -= damage;
    if poisonous {
        target.doomed = true;
    }
    log.push(Event::Struck {
        by,
        slot,
        target: target_side,
        damage,
        instance,
    });
}

/// An unopposed Unit's blow, which lands on the opposing Player.
fn strike_player(
    by: Side,
    slot: usize,
    unit: &crate::party::Unit,
    damage_sink: &mut i32,
    log: &mut Vec<Event>,
) {
    if unit.attack <= 0 {
        return;
    }
    for instance in 0..unit.actions_per_beat() {
        *damage_sink += unit.attack;
        log.push(Event::StruckPlayer {
            by,
            slot,
            damage: unit.attack,
            instance,
        });
    }
}

/// Remove everything that died this Beat, resurrecting what has Reborn to spend.
fn apply_deaths(board: &mut Board, log: &mut Vec<Event>) {
    for side in [Side::Player, Side::Opposing] {
        for slot in 0..SLOTS {
            let dead = board
                .side(side)
                .get(slot)
                .is_some_and(crate::party::Unit::is_dead);
            if !dead {
                continue;
            }
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
    }
}

/// Build a Board from two lists of Units. Convenience for tests and setup.
pub fn board_of(player: Vec<crate::party::Unit>, opposing: Vec<crate::party::Unit>) -> Board {
    Board::new(
        Party::from_units(player).expect("player Party fits"),
        Party::from_units(opposing).expect("opposing Party fits"),
    )
}
