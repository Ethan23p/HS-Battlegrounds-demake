//! The Action Phase's rules, asserted through its public interface.
//!
//! These tests are the specification. The Unit set is original rather than
//! Battlegrounds', so nothing outside this repository can say what a Unit is
//! supposed to do -- which means the rules only exist as precisely as they are
//! pinned down here.
//!
//! Everything goes through `resolve` and the event log. Nothing reaches into
//! engine internals: the interface is the test surface.

use bg_sim::action_phase::{Event, MAX_SWEEPS, Outcome, board_of, resolve};
use bg_sim::party::{Side, Unit};
use bg_sim::units::{DefId, Keyword, UnitDef};

/// A Unit with the given stats and keywords.
fn u(name: &str, attack: i32, health: i32, keywords: &[Keyword]) -> Unit {
    Unit::new(&UnitDef {
        id: DefId::new(name),
        name: name.to_owned(),
        tier: 1,
        attack,
        health,
        tribes: vec![],
        all_tribes: false,
        keywords: keywords.to_vec(),
        abilities: vec![],
        token: false,
        text: String::new(),
    })
}

/// A Unit with no keywords.
fn plain(name: &str, attack: i32, health: i32) -> Unit {
    u(name, attack, health, &[])
}

fn count_events(log: &[Event], pred: impl Fn(&Event) -> bool) -> usize {
    log.iter().filter(|e| pred(e)).count()
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn the_same_board_always_resolves_the_same_way() {
    let build = || {
        board_of(
            vec![plain("a", 3, 4), u("b", 2, 2, &[Keyword::Windfury])],
            vec![u("x", 2, 5, &[Keyword::DivineShield]), plain("y", 4, 1)],
        )
    };
    let first = resolve(build());
    let second = resolve(build());
    assert_eq!(first.outcome, second.outcome);
    assert_eq!(
        first.log, second.log,
        "the log must be reproducible in full"
    );
    assert_eq!(first.final_board, second.final_board);
}

// ---------------------------------------------------------------------------
// Simultaneity within a Beat
// ---------------------------------------------------------------------------

#[test]
fn equal_units_destroy_each_other() {
    // The headline consequence of simultaneous resolution: no first mover, so a
    // mirror match is mutual destruction rather than a win for whoever swung.
    let r = resolve(board_of(vec![plain("a", 3, 3)], vec![plain("x", 3, 3)]));
    assert_eq!(r.outcome, Outcome::Draw);
    assert!(r.final_board.player.is_empty());
    assert!(r.final_board.opposing.is_empty());
}

#[test]
fn a_dying_unit_still_lands_its_blow() {
    // Deaths apply at the end of the Beat, so being fatally wounded does not
    // prevent a Unit from striking in the Beat that killed it.
    let r = resolve(board_of(
        vec![plain("doomed", 1, 1)],
        vec![plain("brute", 5, 5)],
    ));
    assert_eq!(r.outcome, Outcome::OpposingWins);
    assert_eq!(
        r.final_board.opposing.get(0).unwrap().health,
        4,
        "the 1/1 died, but its blow still landed first"
    );
}

#[test]
fn the_tougher_unit_survives_the_exchange() {
    let r = resolve(board_of(vec![plain("stout", 3, 4)], vec![plain("x", 3, 3)]));
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.final_board.player.get(0).unwrap().health, 1);
}

// ---------------------------------------------------------------------------
// The sweep
// ---------------------------------------------------------------------------

#[test]
fn the_sweep_repeats_until_a_party_is_empty() {
    // A single pass would leave the 1/3 alive; health only means something
    // because the sweep loops.
    let r = resolve(board_of(vec![plain("chip", 1, 9)], vec![plain("x", 1, 3)]));
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.sweeps, 3, "three blows to fell a 3-health Unit");
}

#[test]
fn beats_run_left_to_right() {
    let r = resolve(board_of(
        vec![plain("a", 1, 9), plain("b", 1, 9), plain("c", 1, 9)],
        vec![plain("x", 1, 9), plain("y", 1, 9), plain("z", 1, 9)],
    ));
    let first_sweep: Vec<usize> = r
        .log
        .iter()
        .filter_map(|e| match e {
            Event::BeatBegan { sweep: 0, slot } => Some(*slot),
            _ => None,
        })
        .collect();
    assert_eq!(first_sweep, vec![0, 1, 2], "slots resolve in order");
}

#[test]
fn an_empty_slot_facing_an_empty_slot_produces_no_beat() {
    let r = resolve(board_of(vec![plain("a", 1, 1)], vec![plain("x", 1, 1)]));
    let beats = count_events(&r.log, |e| matches!(e, Event::BeatBegan { .. }));
    assert_eq!(beats, 1, "only the one occupied Slot is a Beat");
}

#[test]
fn parties_compact_between_sweeps_so_survivors_face_each_other() {
    // The player's Slot 0 dies in the first sweep. Compaction brings the 3/3
    // across to face the survivor, which is what lets the fight finish.
    let r = resolve(board_of(
        vec![plain("bait", 1, 1), plain("hitter", 3, 3)],
        vec![plain("wall", 1, 5)],
    ));
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(
        r.damage_to_opposing, 3,
        "while unopposed in sweep 0, the hitter struck the opposing player"
    );
}

// ---------------------------------------------------------------------------
// Unopposed Slots
// ---------------------------------------------------------------------------

#[test]
fn an_unopposed_unit_strikes_the_opposing_player() {
    let r = resolve(board_of(
        vec![plain("front", 1, 9), plain("free", 4, 4)],
        vec![plain("wall", 1, 9)],
    ));
    assert!(
        r.damage_to_opposing >= 4,
        "the unopposed Unit in Slot 1 hit the opposing player, got {}",
        r.damage_to_opposing
    );
    assert_eq!(r.damage_to_player, 0, "no player Slot was unopposed");
}

#[test]
fn a_zero_attack_unit_strikes_nobody() {
    let r = resolve(board_of(
        vec![plain("front", 1, 9), plain("pacifist", 0, 4)],
        vec![plain("wall", 1, 9)],
    ));
    assert_eq!(r.damage_to_opposing, 0);
}

// ---------------------------------------------------------------------------
// Keywords
// ---------------------------------------------------------------------------

#[test]
fn windfury_acts_twice_in_its_beat() {
    // 3 attack twice is lethal to a 5-health Unit in a single Beat; once is not.
    let r = resolve(board_of(
        vec![u("gusty", 3, 9, &[Keyword::Windfury])],
        vec![plain("x", 1, 5)],
    ));
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.sweeps, 1, "both blows landed in the same Beat");
}

#[test]
fn a_divine_shield_absorbs_one_blow_entirely() {
    let r = resolve(board_of(
        vec![plain("hammer", 9, 9)],
        vec![u("shielded", 1, 1, &[Keyword::DivineShield])],
    ));
    assert_eq!(
        count_events(&r.log, |e| matches!(e, Event::ShieldAbsorbed { .. })),
        1
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.sweeps, 2, "the shield bought exactly one sweep");
}

#[test]
fn windfury_breaks_a_divine_shield_and_then_connects() {
    // The two blows are separate instances, so the shield eats the first and the
    // second lands. This is why Windfury is modelled as two actions rather than
    // one doubled action.
    let r = resolve(board_of(
        vec![u("gusty", 3, 9, &[Keyword::Windfury])],
        vec![u("shielded", 1, 3, &[Keyword::DivineShield])],
    ));
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.sweeps, 1, "shield broken and the Unit killed in one Beat");
    assert_eq!(
        count_events(&r.log, |e| matches!(e, Event::ShieldAbsorbed { .. })),
        1
    );
}

#[test]
fn poisonous_kills_whatever_it_wounds() {
    let r = resolve(board_of(
        vec![u("venom", 1, 9, &[Keyword::Poisonous])],
        vec![plain("colossus", 1, 50)],
    ));
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.sweeps, 1);
    assert!(
        r.log
            .iter()
            .any(|e| matches!(e, Event::Died { poisoned: true, .. })),
        "the death is recorded as a poisoning"
    );
}

#[test]
fn a_divine_shield_stops_poison_because_no_damage_lands() {
    // Poisonous needs damage to actually connect. An absorbed blow deals none,
    // so the shield saves the Unit outright rather than merely delaying it.
    let r = resolve(board_of(
        vec![u("venom", 1, 9, &[Keyword::Poisonous])],
        vec![u("shielded", 1, 4, &[Keyword::DivineShield])],
    ));
    assert_eq!(r.sweeps, 2, "survived the first Beat entirely");
    assert_eq!(r.outcome, Outcome::PlayerWins);
}

#[test]
fn reborn_returns_a_unit_once_with_one_health() {
    let r = resolve(board_of(
        vec![plain("hammer", 3, 20)],
        vec![u("phoenix", 1, 3, &[Keyword::Reborn])],
    ));
    assert_eq!(
        count_events(&r.log, |e| matches!(e, Event::Reborn { .. })),
        1,
        "Reborn is spent, not repeated"
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.sweeps, 2, "died, returned with 1 health, died again");
}

// ---------------------------------------------------------------------------
// Termination
// ---------------------------------------------------------------------------

#[test]
fn two_empty_parties_are_a_draw() {
    let r = resolve(board_of(vec![], vec![]));
    assert_eq!(r.outcome, Outcome::Draw);
    assert_eq!(r.sweeps, 0);
}

#[test]
fn an_empty_party_loses_immediately() {
    let r = resolve(board_of(vec![], vec![plain("x", 1, 1)]));
    assert_eq!(r.outcome, Outcome::OpposingWins);
    assert_eq!(r.sweeps, 0);
}

#[test]
fn units_that_cannot_hurt_each_other_reach_a_stalemate() {
    // Without the cap this would sweep forever. Non-termination is reported
    // rather than hung on.
    let r = resolve(board_of(
        vec![plain("rock", 0, 5)],
        vec![plain("stone", 0, 5)],
    ));
    assert_eq!(r.outcome, Outcome::Stalemate);
    assert_eq!(r.sweeps, MAX_SWEEPS);
}

// ---------------------------------------------------------------------------
// The log
// ---------------------------------------------------------------------------

#[test]
fn the_log_ends_by_stating_the_outcome() {
    let r = resolve(board_of(vec![plain("a", 3, 3)], vec![plain("x", 1, 1)]));
    match r.log.last() {
        Some(Event::Ended { outcome, sweeps }) => {
            assert_eq!(*outcome, Outcome::PlayerWins);
            assert_eq!(*sweeps, r.sweeps);
        }
        other => panic!("expected the log to end with Ended, found {other:?}"),
    }
}

#[test]
fn the_log_narrates_as_readable_lines() {
    let r = resolve(board_of(
        vec![plain("badger", 2, 3)],
        vec![u("toad", 1, 2, &[Keyword::DivineShield])],
    ));
    let text = r.narrate();
    assert!(text.contains("slot 0:"), "{text}");
    assert!(text.contains("shield"), "{text}");
    assert!(text.contains("PlayerWins"), "{text}");
    assert_eq!(
        text.lines().count(),
        r.log.len(),
        "one line per event, so a frontend can animate them one at a time"
    );
}

#[test]
fn a_struck_event_names_who_struck_whom() {
    let r = resolve(board_of(vec![plain("a", 2, 9)], vec![plain("x", 1, 9)]));
    let struck: Vec<&Event> = r
        .log
        .iter()
        .filter(|e| matches!(e, Event::Struck { .. }))
        .take(2)
        .collect();
    assert!(matches!(
        struck[0],
        Event::Struck {
            by: Side::Player,
            target: Side::Opposing,
            damage: 2,
            instance: 0,
            ..
        }
    ));
}
