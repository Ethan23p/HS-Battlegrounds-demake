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
        3,
        "the 1/1 died, but both transactions of its Beat still resolved: it \
         answered the brute's attack, and its own attack landed too"
    );
}

#[test]
fn a_beat_holds_two_transactions_not_one_trade() {
    // Battlegrounds leaves a 3/4 that trades with a 3/3 at 1 health -- but only
    // because the 3/3 died before its own attack ran. That is pre-emption, and
    // pre-emption is what putting both attacks in one Beat removes.
    //
    // So both transactions resolve: the stout attacks (and is answered), the 3/3
    // attacks (and is answered). Each takes 6. An attack is a transaction with a
    // direction, not a symmetrical trade, and two of them are not one of them.
    let r = resolve(
        board_of(vec![plain("stout", 3, 4)], vec![plain("x", 3, 3)]),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::Draw);
    assert!(r.final_board.player.is_empty() && r.final_board.opposing.is_empty());
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
    assert_eq!(
        r.passes, 2,
        "two blows a Beat -- the chip's attack and its answer to the x's -- so a \
         3-health Unit falls in two Passes, not three"
    );
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
// An attack is an exchange
// ---------------------------------------------------------------------------
//
// A 1v1 hides all of this: both Units are their side's Slot-1 attacker, so the
// answering blow is indistinguishable from the other side's attack. Every scenario
// here puts the answering Unit somewhere its own side's clock has not reached.

/// A Party of three 1/1s, so whatever the attacker draws, it draws a 1/1 that is
/// not the Slot-1 attacker two times out of three -- and Taunt pins it when it
/// matters.
#[test]
fn a_defender_answers_the_blow_it_was_struck_by() {
    // The brute draws the Taunt holder in Slot 2, which never attacks: Beat 1
    // resolves Slot 1. Under a one-way attack the brute would take 1 -- from the
    // opposing Slot-1 Unit alone. It takes 2, because the Unit it struck struck
    // back.
    let r = resolve(
        board_of(
            vec![plain("brute", 9, 9)],
            vec![plain("front", 1, 9), u("bait", 1, 9, &[Keyword::Taunt])],
        ),
        &mut rng(),
    );
    let brute = r.final_board.player.get(0).expect("brute lives");
    assert_eq!(
        brute.health, 7,
        "1 from the Slot-1 attacker, 1 answered by the Taunt holder it struck"
    );
}

#[test]
fn answering_is_not_attacking_so_it_draws_no_target_of_its_own() {
    // The Taunt holder in Slot 2 answers the brute in Beat 1, but does not attack
    // in it: only Slot 1 acts in Beat 1. Its own attack waits for Beat 2, which is
    // the clock doing its job -- an answer is not a turn.
    let r = resolve(
        board_of(
            vec![plain("brute", 1, 99)],
            vec![plain("front", 1, 99), u("bait", 5, 99, &[Keyword::Taunt])],
        ),
        &mut rng(),
    );
    let beat_1 = &r.log[..r
        .log
        .iter()
        .position(|e| matches!(e, Event::BeatBegan { beat: 2, .. }))
        .expect("Beat 2 happens")];

    assert_eq!(
        count_events(beat_1, |e| matches!(
            e,
            Event::Struck {
                by: Side::Opposing,
                attacker_slot: 2,
                ..
            }
        )),
        0,
        "Slot 2 does not attack in Beat 1"
    );
    assert_eq!(
        count_events(beat_1, |e| matches!(
            e,
            Event::StruckBack {
                by: Side::Opposing,
                slot: 2,
                ..
            }
        )),
        1,
        "but it answers the blow it took in Beat 1"
    );
    assert!(
        r.log.iter().any(|e| matches!(
            e,
            Event::Struck {
                by: Side::Opposing,
                attacker_slot: 2,
                ..
            }
        )),
        "and attacks for itself once the clock reaches Slot 2"
    );
}

#[test]
fn a_zero_attack_defender_answers_with_nothing() {
    let r = resolve(
        board_of(
            vec![plain("brute", 9, 9)],
            vec![plain("front", 1, 9), u("wall", 0, 9, &[Keyword::Taunt])],
        ),
        &mut rng(),
    );
    assert_eq!(
        r.final_board.player.get(0).expect("brute lives").health,
        8,
        "1 from the Slot-1 attacker; the wall it struck has nothing to answer with"
    );
    assert_eq!(
        count_events(&r.log, |e| matches!(
            e,
            Event::StruckBack {
                by: Side::Opposing,
                ..
            }
        )),
        0,
        "the wall answers nothing; the brute still answers the Unit that attacked it"
    );
}

#[test]
fn poisonous_kills_the_unit_that_attacked_into_it() {
    // Poisonous is mostly a defensive keyword in Battlegrounds, and this is why:
    // the venomous Unit never has to swing to take something down with it.
    let r = resolve(
        board_of(
            vec![plain("brute", 1, 99)],
            vec![
                plain("front", 1, 99),
                u("venom", 1, 99, &[Keyword::Taunt, Keyword::Poisonous]),
            ],
        ),
        &mut rng(),
    );
    assert!(
        r.log.iter().any(|e| matches!(
            e,
            Event::Died {
                side: Side::Player,
                poisoned: true,
                ..
            }
        )),
        "the brute struck the poisonous Taunt holder and died of the answer"
    );
}

#[test]
fn a_shield_spends_itself_on_the_first_blow_and_the_second_lands() {
    // The paladin is struck twice in one Beat: once by the Slot-1 attacker, once
    // as the answer from the Taunt holder it chose. One blow, one shield --
    // Battlegrounds' own rule, needing no help from ours. Pooling the Beat's damage
    // and absorbing all of it would be an invention, and one only a pooled
    // implementation would ever need.
    let r = resolve(
        board_of(
            vec![u("paladin", 9, 1, &[Keyword::DivineShield])],
            vec![plain("front", 9, 99), u("bait", 9, 99, &[Keyword::Taunt])],
        ),
        &mut rng(),
    );
    assert_eq!(
        count_events(&r.log, |e| matches!(
            e,
            Event::ShieldAbsorbed {
                side: Side::Player,
                ..
            }
        )),
        1,
        "one shield, spent once"
    );
    assert_eq!(r.outcome, Outcome::OpposingWins, "the second blow landed");
}

// ---------------------------------------------------------------------------
// Deaths resolve after the attack that caused them
// ---------------------------------------------------------------------------
//
// An *instance* is the indivisible step: both sides declare, every blow is
// answered, all of it lands together, and then the dead are removed. A Beat is one
// instance, or two when Windfury is involved -- and the dead do not wait for the
// second.

#[test]
fn a_unit_killed_in_its_first_swing_does_not_swing_again() {
    // Windfury swings twice in a Beat; a corpse swings no times. The Unit dies to
    // the answer its own attack drew, and instance 1 finds nobody standing there.
    let r = resolve(
        board_of(
            vec![u("gusty", 1, 2, &[Keyword::Windfury])],
            vec![plain("wall", 5, 9)],
        ),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::OpposingWins);
    assert_eq!(
        count_events(&r.log, |e| matches!(
            e,
            Event::Struck {
                by: Side::Player,
                ..
            }
        )),
        1,
        "one swing, not two: it was dead before the second"
    );
}

#[test]
fn a_corpse_is_gone_before_the_second_swing_draws() {
    // Two 1-health Units, so whichever the first swing draws, it dies. The second
    // swing must then draw the other: holding the corpse to the end of the Beat
    // would let it be struck twice, which is the invisible bookkeeping the Beat
    // exists to delete.
    let r = resolve(
        board_of(
            vec![u("gusty", 5, 9, &[Keyword::Windfury])],
            vec![plain("p", 0, 1), plain("q", 0, 1)],
        ),
        &mut rng(),
    );
    let swings: Vec<(u32, u32)> = r
        .log
        .iter()
        .filter_map(|e| match e {
            Event::Struck {
                by: Side::Player,
                target_slot,
                instance,
                ..
            } => Some((*instance, *target_slot)),
            _ => None,
        })
        .collect();
    assert_eq!(swings.len(), 2, "Windfury swung twice");
    assert_ne!(
        swings[0].1, swings[1].1,
        "the second swing cannot draw a Slot whose Unit is already gone"
    );

    let first_death = r
        .log
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::Died {
                    side: Side::Opposing,
                    ..
                }
            )
        })
        .expect("something died");
    let second_swing = r
        .log
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::Struck {
                    by: Side::Player,
                    instance: 1,
                    ..
                }
            )
        })
        .expect("Windfury swung twice");
    assert!(
        first_death < second_swing,
        "the dead are removed before the next swing is drawn, not at the Beat's end"
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
    // The shielded Unit has no attack, so it starts no transaction of its own and
    // meets exactly one blow a Beat. The shield eats the first one whole.
    let r = resolve(
        board_of(
            vec![plain("hammer", 9, 9)],
            vec![u("shielded", 0, 1, &[Keyword::DivineShield])],
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
            vec![u("shielded", 0, 4, &[Keyword::DivineShield])],
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
