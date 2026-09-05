//! The Action Phase's rules, asserted through its public interface.
//!
//! These tests are the specification. The Unit set is original rather than
//! Battlegrounds', so nothing outside this repository can say what a Unit is
//! supposed to do -- which means the rules only exist as precisely as they are
//! pinned down here.
//!
//! Everything goes through `resolve` and the event log. Nothing reaches into
//! engine internals: the interface is the test surface.
//!
//! Most scenarios below keep one side (or the relevant side) down to a single
//! living Unit, or use Taunt, so the outcome is fixed regardless of which seed
//! drives target selection -- see ADR 0008. A handful of tests seed the Rng
//! explicitly to pin down the targeting/cycling mechanics themselves.

use bg_sim::action_phase::{Event, MAX_PASSES, Outcome, board_of, resolve};
use bg_sim::party::{Side, Unit};
use bg_sim::rng::{Domain, Rng, Seed};
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

/// A fresh Rng for scenarios where the outcome doesn't depend on its draws.
fn rng() -> Rng {
    Seed(0xf00d).stream(Domain::Combat, 0)
}

fn count_events(log: &[Event], pred: impl Fn(&Event) -> bool) -> usize {
    log.iter().filter(|e| pred(e)).count()
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn the_same_board_and_rng_state_always_resolve_the_same_way() {
    let build = || {
        board_of(
            vec![plain("a", 3, 4), u("b", 2, 2, &[Keyword::Windfury])],
            vec![u("x", 2, 5, &[Keyword::DivineShield]), plain("y", 4, 1)],
        )
    };
    let seed = Seed(0xabc123);
    let first = resolve(build(), &mut seed.stream(Domain::Combat, 0));
    let second = resolve(build(), &mut seed.stream(Domain::Combat, 0));
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
    let r = resolve(
        board_of(vec![plain("a", 3, 3)], vec![plain("x", 3, 3)]),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::Draw);
    assert!(r.final_board.player.is_empty());
    assert!(r.final_board.opposing.is_empty());
}

#[test]
fn a_dying_unit_still_lands_its_blow() {
    // Deaths apply at the end of the Beat, so being fatally wounded does not
    // prevent a Unit from striking in the Beat that killed it -- even though
    // the other side's blow, which killed it, happens in that same instance.
    let r = resolve(
        board_of(vec![plain("doomed", 1, 1)], vec![plain("brute", 5, 5)]),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::OpposingWins);
    assert_eq!(
        r.final_board.opposing.get(0).unwrap().health,
        4,
        "the 1/1 died, but its blow still landed"
    );
}

#[test]
fn the_tougher_unit_survives_the_exchange() {
    let r = resolve(
        board_of(vec![plain("stout", 3, 4)], vec![plain("x", 3, 3)]),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.final_board.player.get(0).unwrap().health, 1);
}

// ---------------------------------------------------------------------------
// The clock: Beat 0 closes ranks, Beat n resolves Slot n
// ---------------------------------------------------------------------------

/// Every Beat the log recorded, as `(pass, beat)`.
fn beats(r: &bg_sim::action_phase::Resolution) -> Vec<(u32, u32)> {
    r.log
        .iter()
        .filter_map(|e| match e {
            Event::BeatBegan { pass, beat } => Some((*pass, *beat)),
            _ => None,
        })
        .collect()
}

#[test]
fn passes_repeat_until_a_party_is_empty() {
    // A single Pass would leave the 1/3 alive; health only means something
    // because the clock comes back around.
    let r = resolve(
        board_of(vec![plain("chip", 1, 9)], vec![plain("x", 1, 3)]),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.passes, 3, "three blows to fell a 3-health Unit");
}

#[test]
fn beat_n_resolves_slot_n_for_both_sides_at_once() {
    // Zero attack all round, so nothing dies and nothing depends on targeting --
    // only the clock is under test. The Player holds Slots 1-3 and the opposing
    // Party only Slot 1, but the Beats are the Board's, not either Party's:
    // Beats 1-3 run, Beats 4-8 have nobody in them at all and are not Beats
    // anything happened in.
    let r = resolve(
        board_of(
            vec![plain("a", 0, 9), plain("b", 0, 9), plain("c", 0, 9)],
            vec![plain("x", 0, 9)],
        ),
        &mut rng(),
    );
    assert_eq!(
        beats(&r)[..6],
        [(1, 1), (1, 2), (1, 3), (2, 1), (2, 2), (2, 3)]
    );
}

/// A Party whose Slot 2 Unit ("b", Taunt, so it takes every one of killer's
/// attacks) dies in Beat 1. Used by the two Pass-boundary tests below.
fn board_that_loses_its_slot_2_unit_in_beat_1() -> bg_sim::party::Board {
    board_of(
        vec![
            plain("a", 0, 9),
            u("b", 0, 1, &[Keyword::Taunt]),
            plain("c", 0, 9),
        ],
        vec![plain("killer", 1, 99)],
    )
}

#[test]
fn a_death_leaves_its_slot_empty_until_the_pass_ends() {
    // "b" dies in Beat 1, so Beat 2 of that Pass has nobody in Slot 2 and
    // resolves nothing -- the hole is skipped, not closed. Only the next Beat 0
    // closes it, which is why "c" is back in Slot 2 by Pass 2 (ADR 0009).
    let r = resolve(board_that_loses_its_slot_2_unit_in_beat_1(), &mut rng());
    assert_eq!(
        beats(&r)[..4],
        [(1, 1), (1, 3), (2, 1), (2, 2)],
        "Pass 1 skips Beat 2; Pass 2 has closed ranks, so Slot 3 is now Slot 2"
    );
}

#[test]
fn a_party_closes_ranks_only_at_beat_0() {
    let r = resolve(board_that_loses_its_slot_2_unit_in_beat_1(), &mut rng());
    let position = |pred: fn(&Event) -> bool| r.log.iter().position(pred);
    let compacted = position(|e| {
        matches!(
            e,
            Event::Compacted {
                side: Side::Player,
                ..
            }
        )
    })
    .expect("the Party closed ranks");
    let last_of_pass_1 = position(|e| matches!(e, Event::BeatBegan { pass: 1, beat: 3 })).unwrap();
    let first_of_pass_2 = position(|e| matches!(e, Event::BeatBegan { pass: 2, beat: 1 })).unwrap();
    assert!(
        last_of_pass_1 < compacted && compacted < first_of_pass_2,
        "ranks close at Beat 0, not on the death that opened the hole"
    );
}

// ---------------------------------------------------------------------------
// Targeting: random, constrained by Taunt (ADR 0008)
// ---------------------------------------------------------------------------

#[test]
fn taunt_forces_every_attack_against_its_party_to_target_it() {
    // The opposing side deals no damage, so the only way "wall" (Taunt, 5
    // health) can die before "squishy" is if Taunt is doing its job -- this
    // holds for every seed, not just a lucky one.
    let r = resolve(
        board_of(
            vec![plain("spear", 1, 50)],
            vec![u("wall", 0, 5, &[Keyword::Taunt]), plain("squishy", 0, 1)],
        ),
        &mut rng(),
    );
    let first_death = r
        .log
        .iter()
        .find_map(|e| match e {
            Event::Died { name, .. } => Some(name.clone()),
            _ => None,
        })
        .expect("something died");
    assert_eq!(first_death, "wall", "Taunt must absorb every attack first");
}

#[test]
fn taunt_stops_applying_once_its_holder_dies() {
    let r = resolve(
        board_of(
            vec![plain("spear", 1, 50)],
            vec![u("wall", 0, 1, &[Keyword::Taunt]), plain("squishy", 0, 1)],
        ),
        &mut rng(),
    );
    // Both are 1 health and only "spear" ever attacks: wall dies on Beat 0,
    // squishy (no longer protected) dies on Beat 1.
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.passes, 2);
}

#[test]
fn a_second_attack_chooses_a_new_target() {
    // Windfury's second attack draws again rather than repeating the first, so
    // both 1-health Units die in the same Beat whichever order they come up in.
    let r = resolve(
        board_of(
            vec![u("gusty", 5, 9, &[Keyword::Windfury])],
            vec![plain("p", 0, 1), plain("q", 0, 1)],
        ),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.passes, 1, "one Beat, two different targets");
}

// ---------------------------------------------------------------------------
// Nothing ever attacks a Player
// ---------------------------------------------------------------------------

#[test]
fn an_attack_with_nothing_left_to_target_does_not_land() {
    // The first attack empties the opposing Party; Battlegrounds has no
    // attacking the Player, so the Windfury second attack simply does not
    // happen.
    let r = resolve(
        board_of(
            vec![u("slayer", 10, 10, &[Keyword::Windfury])],
            vec![plain("lone", 1, 1)],
        ),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.passes, 1);
    assert_eq!(
        count_events(&r.log, |e| matches!(
            e,
            Event::Struck {
                by: Side::Player,
                ..
            }
        )),
        1,
        "the second attack found nothing standing and did not land"
    );
}

#[test]
fn a_zero_attack_unit_strikes_nobody() {
    let r = resolve(
        board_of(vec![plain("pacifist", 0, 4)], vec![plain("wall", 1, 9)]),
        &mut rng(),
    );
    assert_eq!(
        count_events(&r.log, |e| matches!(
            e,
            Event::Struck {
                by: Side::Player,
                ..
            }
        )),
        0
    );
}

// ---------------------------------------------------------------------------
// Keywords
// ---------------------------------------------------------------------------

#[test]
fn windfury_acts_twice_in_its_beat() {
    // 3 attack twice is lethal to a 5-health Unit in a single Beat; once is not.
    let r = resolve(
        board_of(
            vec![u("gusty", 3, 9, &[Keyword::Windfury])],
            vec![plain("x", 1, 5)],
        ),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.passes, 1, "both blows landed in the same Beat");
}

#[test]
fn a_divine_shield_absorbs_one_blow_entirely() {
    let r = resolve(
        board_of(
            vec![plain("hammer", 9, 9)],
            vec![u("shielded", 1, 1, &[Keyword::DivineShield])],
        ),
        &mut rng(),
    );
    assert_eq!(
        count_events(&r.log, |e| matches!(e, Event::ShieldAbsorbed { .. })),
        1
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.passes, 2, "the shield bought exactly one Beat");
}

#[test]
fn windfury_breaks_a_divine_shield_and_then_connects() {
    // The two blows are separate instances, so the shield eats the first and the
    // second lands. This is why Windfury is modelled as two actions rather than
    // one doubled action.
    let r = resolve(
        board_of(
            vec![u("gusty", 3, 9, &[Keyword::Windfury])],
            vec![u("shielded", 1, 3, &[Keyword::DivineShield])],
        ),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.passes, 1, "shield broken and the Unit killed in one Beat");
    assert_eq!(
        count_events(&r.log, |e| matches!(e, Event::ShieldAbsorbed { .. })),
        1
    );
}

#[test]
fn poisonous_kills_whatever_it_wounds() {
    let r = resolve(
        board_of(
            vec![u("venom", 1, 9, &[Keyword::Poisonous])],
            vec![plain("colossus", 1, 50)],
        ),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.passes, 1);
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
    let r = resolve(
        board_of(
            vec![u("venom", 1, 9, &[Keyword::Poisonous])],
            vec![u("shielded", 1, 4, &[Keyword::DivineShield])],
        ),
        &mut rng(),
    );
    assert_eq!(r.passes, 2, "survived the first Beat entirely");
    assert_eq!(r.outcome, Outcome::PlayerWins);
}

#[test]
fn reborn_returns_a_unit_once_with_one_health() {
    let r = resolve(
        board_of(
            vec![plain("hammer", 3, 20)],
            vec![u("phoenix", 1, 3, &[Keyword::Reborn])],
        ),
        &mut rng(),
    );
    assert_eq!(
        count_events(&r.log, |e| matches!(e, Event::Reborn { .. })),
        1,
        "Reborn is spent, not repeated"
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.passes, 2, "died, returned with 1 health, died again");
}

// ---------------------------------------------------------------------------
// Termination
// ---------------------------------------------------------------------------

#[test]
fn two_empty_parties_are_a_draw() {
    let r = resolve(board_of(vec![], vec![]), &mut rng());
    assert_eq!(r.outcome, Outcome::Draw);
    assert_eq!(r.passes, 0);
}

#[test]
fn an_empty_party_loses_immediately() {
    let r = resolve(board_of(vec![], vec![plain("x", 1, 1)]), &mut rng());
    assert_eq!(r.outcome, Outcome::OpposingWins);
    assert_eq!(r.passes, 0);
}

#[test]
fn units_that_cannot_hurt_each_other_reach_a_stalemate() {
    // Without the cap this would run forever. Non-termination is reported
    // rather than hung on.
    let r = resolve(
        board_of(vec![plain("rock", 0, 5)], vec![plain("stone", 0, 5)]),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::Stalemate);
    assert_eq!(r.passes, MAX_PASSES);
}

// ---------------------------------------------------------------------------
// The log
// ---------------------------------------------------------------------------

#[test]
fn the_log_ends_by_stating_the_outcome() {
    let r = resolve(
        board_of(vec![plain("a", 3, 3)], vec![plain("x", 1, 1)]),
        &mut rng(),
    );
    match r.log.last() {
        Some(Event::Ended { outcome, passes }) => {
            assert_eq!(*outcome, Outcome::PlayerWins);
            assert_eq!(*passes, r.passes);
        }
        other => panic!("expected the log to end with Ended, found {other:?}"),
    }
}

#[test]
fn the_log_narrates_as_readable_lines() {
    let r = resolve(
        board_of(
            vec![plain("badger", 2, 3)],
            vec![u("toad", 1, 2, &[Keyword::DivineShield])],
        ),
        &mut rng(),
    );
    let text = r.narrate();
    assert!(text.contains("pass 1, beat 1"), "{text}");
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
    let r = resolve(
        board_of(vec![plain("a", 2, 9)], vec![plain("x", 1, 9)]),
        &mut rng(),
    );
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
            target_slot: 1,
            damage: 2,
            instance: 0,
            ..
        }
    ));
}
