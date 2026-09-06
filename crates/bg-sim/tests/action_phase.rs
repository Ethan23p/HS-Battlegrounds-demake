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

use bg_sim::action_phase::{Event, MAX_BEATS, Outcome, board_of, resolve};
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
// The clock: Beats count one way, and readiness rides on the Unit
// ---------------------------------------------------------------------------

/// Every Beat the log recorded, in order.
fn beats(r: &bg_sim::action_phase::Resolution) -> Vec<u32> {
    r.log
        .iter()
        .filter_map(|e| match e {
            Event::BeatBegan { beat } => Some(*beat),
            _ => None,
        })
        .collect()
}

/// The Beat each *before the first Unit acts* moment preceded.
fn ready_moments(r: &bg_sim::action_phase::Resolution) -> Vec<u32> {
    r.log
        .iter()
        .filter_map(|e| match e {
            Event::AllReady { beat } => Some(*beat),
            _ => None,
        })
        .collect()
}

/// Player Slots that struck, in log order.
fn player_strikes(r: &bg_sim::action_phase::Resolution) -> Vec<(u32, i32)> {
    r.log
        .iter()
        .filter_map(|e| match e {
            Event::Struck {
                by: Side::Player,
                attacker_slot,
                damage,
                ..
            } => Some((*attacker_slot, *damage)),
            _ => None,
        })
        .collect()
}

#[test]
fn beats_count_one_way_and_never_reset() {
    // Zero attack all round, so nothing dies and nothing depends on targeting --
    // only the clock is under test.
    let r = resolve(
        board_of(
            vec![plain("a", 0, 9), plain("b", 0, 9), plain("c", 0, 9)],
            vec![plain("x", 0, 9)],
        ),
        &mut rng(),
    );
    assert_eq!(
        beats(&r)[..7],
        [1, 2, 3, 4, 5, 6, 7],
        "one clock for the Board, and it only goes up"
    );
}

#[test]
fn everybody_acts_once_before_anybody_acts_twice() {
    // Three Units against one. The Player's side takes three Beats to enumerate
    // and the opposing side one, so the boundary falls where the *longer* Party
    // runs out -- it belongs to the Board, not to either Party.
    let r = resolve(
        board_of(
            vec![plain("a", 1, 99), plain("b", 2, 99), plain("c", 3, 99)],
            vec![plain("x", 0, 99)],
        ),
        &mut rng(),
    );
    assert_eq!(
        player_strikes(&r)[..6],
        [(1, 1), (2, 2), (3, 3), (1, 1), (2, 2), (3, 3)],
        "each Unit strikes once per enumeration, in Slot order"
    );
    assert_eq!(
        ready_moments(&r)[..3],
        [1, 4, 7],
        "the Action Phase opens Ready, and comes Ready again every third Beat"
    );
}

/// A Party whose Slot 1 Unit ("first", Taunt, so it takes every one of killer's
/// attacks) acts in Beat 1 and dies at the end of it.
fn board_that_loses_its_slot_1_unit_in_beat_1() -> bg_sim::party::Board {
    board_of(
        vec![
            u("first", 0, 1, &[Keyword::Taunt]),
            plain("second", 5, 99),
            plain("third", 0, 99),
        ],
        vec![plain("killer", 1, 99)],
    )
}

#[test]
fn a_death_behind_the_clock_does_not_cost_the_next_unit_its_turn() {
    // "first" acts in Beat 1 and dies in it, so the Party closes up and "second"
    // slides from Slot 2 to Slot 1. A clock that counted Slots would look for
    // Slot 2 in Beat 2 and find "third", skipping "second" entirely -- which is
    // exactly the positional churn this model exists to remove. Readiness is on
    // the Unit, so "second" gets its Beat.
    let r = resolve(board_that_loses_its_slot_1_unit_in_beat_1(), &mut rng());
    assert_eq!(
        player_strikes(&r)[0],
        (1, 5),
        "\"second\" struck, from the Slot it had closed up into"
    );
}

#[test]
fn a_party_closes_up_the_moment_its_dead_are_applied() {
    let r = resolve(board_that_loses_its_slot_1_unit_in_beat_1(), &mut rng());
    let died = r
        .log
        .iter()
        .position(|e| matches!(e, Event::Died { name, .. } if name == "first"))
        .expect("the Taunt Unit died");
    let beat_2 = r
        .log
        .iter()
        .position(|e| matches!(e, Event::BeatBegan { beat: 2 }))
        .expect("a second Beat ran");
    assert!(died < beat_2, "no gap survives the Beat that opened it");
    assert_eq!(
        r.final_board.player.get(0).map(|u| u.name.as_str()),
        Some("second"),
        "the Party is anchored on its left-most Unit, always"
    );
}

#[test]
fn the_clock_runs_on_while_one_side_has_nobody_left_to_act() {
    // The Player has one Unit and the opposing Party three. Once the Player's
    // Unit has acted it simply sits out until the Board comes Ready again -- the
    // Beats are the Board's, and they keep coming.
    let r = resolve(
        board_of(
            vec![plain("lone", 1, 99)],
            vec![plain("x", 0, 99), plain("y", 0, 99), plain("z", 0, 99)],
        ),
        &mut rng(),
    );
    assert_eq!(beats(&r)[..4], [1, 2, 3, 4]);
    assert_eq!(
        player_strikes(&r)[..2],
        [(1, 1), (1, 1)],
        "one strike per enumeration, three Beats apart"
    );
    assert_eq!(ready_moments(&r)[..2], [1, 4]);
}

#[test]
fn health_only_matters_because_the_clock_comes_back_around() {
    let r = resolve(
        board_of(vec![plain("chip", 1, 9)], vec![plain("x", 1, 3)]),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.beats, 3, "three blows to fell a 3-health Unit");
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
    // Both are 1 health and only "spear" ever attacks. Wall dies in Beat 1;
    // Beat 2 is squishy's turn in the enumeration and spear has already had
    // its own, so the killing blow lands in Beat 3, once the Board is Ready
    // again and squishy has nothing shielding it.
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.beats, 3);
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
    assert_eq!(r.beats, 1, "one Beat, two different targets");
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
    assert_eq!(r.beats, 1);
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
    assert_eq!(r.beats, 1, "both blows landed in the same Beat");
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
    assert_eq!(r.beats, 2, "the shield bought exactly one Beat");
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
    assert_eq!(r.beats, 1, "shield broken and the Unit killed in one Beat");
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
    assert_eq!(r.beats, 1);
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
    assert_eq!(r.beats, 2, "survived the first Beat entirely");
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
    assert_eq!(r.beats, 2, "died, returned with 1 health, died again");
}

// ---------------------------------------------------------------------------
// Termination
// ---------------------------------------------------------------------------

#[test]
fn two_empty_parties_are_a_draw() {
    let r = resolve(board_of(vec![], vec![]), &mut rng());
    assert_eq!(r.outcome, Outcome::Draw);
    assert_eq!(r.beats, 0);
}

#[test]
fn an_empty_party_loses_immediately() {
    let r = resolve(board_of(vec![], vec![plain("x", 1, 1)]), &mut rng());
    assert_eq!(r.outcome, Outcome::OpposingWins);
    assert_eq!(r.beats, 0);
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
    assert_eq!(r.beats, MAX_BEATS);
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
        Some(Event::Ended { outcome, beats }) => {
            assert_eq!(*outcome, Outcome::PlayerWins);
            assert_eq!(*beats, r.beats);
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
    assert!(text.contains("beat 1"), "{text}");
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
