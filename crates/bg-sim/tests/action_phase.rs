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
//! Targeting is random, so most scenarios below keep the relevant side down to a
//! single Unit, or use Taunt, and hold for every seed rather than a lucky one. A
//! handful seed the Rng explicitly to pin down the targeting mechanics themselves.

use bg_sim::action_phase::{Event, MAX_BEATS, Outcome, Resolution, board_of, resolve};
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

fn count_events<'a>(
    log: impl IntoIterator<Item = &'a Event>,
    pred: impl Fn(&Event) -> bool,
) -> usize {
    log.into_iter().filter(|e| pred(e)).count()
}

/// The log split into its Beats, as `(beat, events)`.
///
/// A Beat is the unit of resolution, so most assertions here are about what
/// shared one and what did not.
fn beats(r: &Resolution) -> Vec<(u32, Vec<&Event>)> {
    let mut out: Vec<(u32, Vec<&Event>)> = Vec::new();
    for event in &r.log {
        match event {
            Event::BeatBegan { beat } => out.push((*beat, Vec::new())),
            _ => {
                if let Some(current) = out.last_mut() {
                    current.1.push(event);
                }
            }
        }
    }
    out
}

/// Who attacked in a Beat, as `(side, slot)`. Answers are not attacks and are not
/// counted.
fn attackers(events: &[&Event]) -> Vec<(Side, u32)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Struck {
                by, attacker_slot, ..
            } => Some((*by, *attacker_slot)),
            _ => None,
        })
        .collect()
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
// A Beat holds concurrent interactions
// ---------------------------------------------------------------------------

#[test]
fn equal_units_destroy_each_other() {
    // The headline consequence of putting both attacks in one Beat: no first
    // mover, so a mirror is mutual destruction rather than a win for whoever
    // swung.
    let r = resolve(
        board_of(vec![plain("a", 3, 3)], vec![plain("x", 3, 3)]),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::Draw);
    assert!(r.final_board.player.is_empty());
    assert!(r.final_board.opposing.is_empty());
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

#[test]
fn both_sides_act_in_one_beat_and_the_slots_resolve_in_order() {
    // Nothing here can die, so only the clock is under test. Slot 1 on both sides
    // shares a Beat, then Slot 2 on both sides shares the next.
    let r = resolve(
        board_of(
            vec![plain("a", 1, 99), plain("b", 1, 99)],
            vec![plain("x", 1, 99), plain("y", 1, 99)],
        ),
        &mut rng(),
    );
    let b = beats(&r);
    assert_eq!(b[0].0, 1);
    assert_eq!(attackers(&b[0].1), [(Side::Player, 1), (Side::Opposing, 1)]);
    assert_eq!(b[1].0, 2);
    assert_eq!(attackers(&b[1].1), [(Side::Player, 2), (Side::Opposing, 2)]);
}

// ---------------------------------------------------------------------------
// A Unit dies in the Beat after the one that took it to 0
// ---------------------------------------------------------------------------

#[test]
fn a_unit_that_runs_out_dies_in_the_next_beat() {
    // The departure, in its simplest form. The mark is struck in Beat 1 and dies
    // in Beat 2 -- a Beat kills nobody, it only buries whoever the last one did.
    let r = resolve(
        board_of(vec![plain("chip", 1, 99)], vec![plain("mark", 0, 1)]),
        &mut rng(),
    );
    let b = beats(&r);
    assert!(
        b[0].1.iter().any(|e| matches!(e, Event::Struck { .. }))
            && !b[0].1.iter().any(|e| matches!(e, Event::Died { .. })),
        "the Beat that struck the mark does not bury it"
    );
    assert!(
        b[1].1
            .iter()
            .any(|e| matches!(e, Event::Died { name, .. } if name == "mark")),
        "the death belongs to the Beat after"
    );
}

#[test]
fn a_dying_unit_still_lands_its_blow() {
    // What the delay buys: being fatally wounded does not stop a Unit acting for
    // the rest of the Beat it ran out in.
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
fn a_unit_that_ran_out_answers_in_its_own_beat_but_never_attacks_again() {
    // The other side of the delay. The bait is taken to 0 in Beat 1, where it
    // still answers the blow -- but Beat 2 is its Slot's, and it is buried at the
    // top of it, so its own attack never comes.
    let r = resolve(
        board_of(
            vec![plain("front", 3, 99), u("bait", 5, 1, &[Keyword::Taunt])],
            vec![plain("killer", 3, 99), plain("other", 3, 99)],
        ),
        &mut rng(),
    );
    let b = beats(&r);
    assert_eq!(
        count_events(b[0].1.iter().copied(), |e| matches!(
            e,
            Event::StruckBack {
                by: Side::Player,
                slot: 2,
                ..
            }
        )),
        1,
        "the bait answers the blow that took it to 0"
    );
    assert!(
        b[1].1.iter().any(|e| matches!(
            e,
            Event::Died {
                side: Side::Player,
                slot: 2,
                ..
            }
        )),
        "and is buried when its own Beat opens"
    );
    assert_eq!(
        count_events(b[1].1.iter().copied(), |e| matches!(
            e,
            Event::Struck {
                by: Side::Player,
                attacker_slot: 2,
                ..
            }
        )),
        0,
        "so it never attacks from Slot 2"
    );
}

// ---------------------------------------------------------------------------
// The clock: Intents, and closing ranks the moment a hole opens
// ---------------------------------------------------------------------------

#[test]
fn beats_keep_coming_until_a_party_is_empty() {
    // The x has 3 health and takes 2 a cycle -- the chip's attack and its answer
    // to the x's -- so it survives the first cycle. Health means something only
    // because Intents renew.
    let r = resolve(
        board_of(vec![plain("chip", 1, 9)], vec![plain("x", 1, 3)]),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(
        r.beats, 3,
        "two Beats of fighting, and a third to bury the x"
    );
}

#[test]
fn a_party_closes_ranks_in_the_beat_that_buries_its_dead() {
    // "b" holds Taunt, so it takes the killer's attack in Beat 1 and runs out.
    // Beat 2 buries it and closes the hole in the same breath -- and "c", which
    // has not spent its Intent, acts from the Slot it just slid into.
    let r = resolve(
        board_of(
            vec![
                plain("a", 1, 99),
                u("b", 1, 1, &[Keyword::Taunt]),
                plain("c", 1, 99),
            ],
            vec![plain("killer", 1, 99)],
        ),
        &mut rng(),
    );
    let b = beats(&r);
    assert!(
        b[1].1
            .iter()
            .any(|e| matches!(e, Event::Died { slot: 2, .. })),
        "b is buried at the top of Beat 2"
    );
    assert!(
        b[1].1
            .iter()
            .any(|e| matches!(e, Event::Compacted { side: Side::Player })),
        "and the Party re-anchors in that same Beat"
    );
    assert_eq!(
        attackers(&b[1].1),
        [(Side::Player, 2)],
        "c slid from Slot 3 to Slot 2 and acts from there: sliding neither skips \
         a Unit nor gives it a second go, because the Intent rides on the Unit"
    );
}

#[test]
fn a_unit_that_already_acted_does_not_act_again_after_sliding() {
    // The counterpart hazard. "a" acts in Beat 1 and spends its Intent; when "b"
    // is buried and "c" slides up behind it, the clock must still reach c rather
    // than re-reading a Slot a spent Unit now occupies.
    let r = resolve(
        board_of(
            vec![
                plain("a", 1, 99),
                u("b", 1, 1, &[Keyword::Taunt]),
                plain("c", 1, 99),
            ],
            vec![plain("killer", 1, 99)],
        ),
        &mut rng(),
    );
    let b = beats(&r);
    assert_eq!(
        attackers(&b[0].1),
        [(Side::Player, 1), (Side::Opposing, 1)],
        "a acts in Beat 1"
    );
    assert_eq!(
        attackers(&b[1].1),
        [(Side::Player, 2)],
        "and not again in Beat 2, even though Slot 1 is still where it stands"
    );
}

// ---------------------------------------------------------------------------
// Targeting: random, constrained by Taunt
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
    // Both are 1 health and only "spear" ever attacks. The wall runs out first;
    // squishy -- no longer protected, and moved up by closing ranks -- takes the
    // next attack the spear has an Intent for.
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.beats, 4);
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
    assert_eq!(r.beats, 2, "one Beat to strike, one to bury");
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
// An attack is answered
// ---------------------------------------------------------------------------
//
// A 1v1 hides all of this: both Units are their side's Slot-1 attacker, so the
// answering blow is indistinguishable from the other side's attack. Every scenario
// here puts the answering Unit somewhere its own side's clock has not reached.

#[test]
fn a_defender_answers_the_blow_it_was_struck_by() {
    // The brute draws the Taunt holder in Slot 2, which never attacks: Beat 1 is
    // Slot 1's. Under a one-way attack the brute would take 1 -- from the opposing
    // Slot-1 Unit alone. It takes 2, because the Unit it struck struck back.
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
    // in it: Beat 1 is Slot 1's. Its own attack waits for Beat 2, which is the
    // clock doing its job -- an answer is not a turn.
    let r = resolve(
        board_of(
            vec![plain("brute", 1, 99)],
            vec![plain("front", 1, 99), u("bait", 5, 99, &[Keyword::Taunt])],
        ),
        &mut rng(),
    );
    let b = beats(&r);
    assert_eq!(
        count_events(b[0].1.iter().copied(), |e| matches!(
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
        count_events(b[0].1.iter().copied(), |e| matches!(
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
    assert_eq!(
        attackers(&b[1].1),
        [(Side::Opposing, 2)],
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
fn a_shield_absorbs_everything_its_beat_brings() {
    // The paladin is struck twice in one Beat: once by the Slot-1 attacker, once
    // as the answer from the Taunt holder it chose. Both are absorbed, and the
    // shield breaks at the top of the next Beat.
    //
    // Battlegrounds spends a shield on one blow, but Battlegrounds cannot deliver
    // two at once. Here they arrive in the same slice of time, and picking one of
    // them to absorb would mean picking by the order the engine walks the Board --
    // which would make a Unit's side worth something.
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
        2,
        "both blows of the Beat met the same shield"
    );
    assert_eq!(
        r.outcome,
        Outcome::OpposingWins,
        "the shield broke at the top of the next Beat, and the next blow landed"
    );
}

// ---------------------------------------------------------------------------
// Keywords
// ---------------------------------------------------------------------------

#[test]
fn windfury_attacks_again_in_a_beat_of_its_own() {
    // The x takes 3 from the attack and 3 more answering gusty in Beat 1, and the
    // second Intent's 3 finishes it -- 9 where a Unit without Windfury would
    // manage 6. The two attacks fall in two Beats, because Battlegrounds clears
    // the dead between them.
    let r = resolve(
        board_of(
            vec![u("gusty", 3, 9, &[Keyword::Windfury])],
            vec![plain("x", 1, 9)],
        ),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(r.beats, 3);
    let b = beats(&r);
    assert_eq!(attackers(&b[0].1), [(Side::Player, 1), (Side::Opposing, 1)]);
    assert_eq!(
        attackers(&b[1].1),
        [(Side::Player, 1)],
        "the second attack is a Beat of its own, and only the Windfury Unit is in it"
    );
}

#[test]
fn a_unit_killed_in_its_first_attack_does_not_attack_again() {
    // Windfury swings twice; a corpse swings no times. The Unit runs out to the
    // answer its own attack drew, and is buried when its second Beat opens.
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
        "one attack, not two: it was buried before the second"
    );
}

#[test]
fn a_corpse_is_gone_before_the_second_attack_draws() {
    // Two 1-health Units, so whichever the first attack draws runs out. Giving
    // Windfury's second attack its own Beat is what buries that one first, so the
    // second attack has to draw the other.
    let r = resolve(
        board_of(
            vec![u("gusty", 5, 9, &[Keyword::Windfury])],
            vec![plain("p", 0, 1), plain("q", 0, 1)],
        ),
        &mut rng(),
    );
    let targets: Vec<u32> = r
        .log
        .iter()
        .filter_map(|e| match e {
            Event::Struck {
                by: Side::Player,
                target_slot,
                ..
            } => Some(*target_slot),
            _ => None,
        })
        .collect();
    assert_eq!(targets.len(), 2, "Windfury attacked twice");
    assert_ne!(
        targets[0], targets[1],
        "the second attack cannot draw a Slot whose Unit is already buried"
    );

    let b = beats(&r);
    assert!(
        b[1].1.iter().any(|e| matches!(
            e,
            Event::Died {
                side: Side::Opposing,
                ..
            }
        )) && b[1].1.iter().any(|e| matches!(
            e,
            Event::Struck {
                by: Side::Player,
                ..
            }
        )),
        "the Windfury Beat buries the first target before drawing the second"
    );
}

#[test]
fn a_divine_shield_absorbs_one_blow_entirely() {
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
    assert_eq!(r.beats, 3, "the shield bought exactly one cycle of Intents");
}

#[test]
fn windfury_breaks_a_divine_shield_and_then_connects() {
    // The two attacks are separate Beats, so the first Beat's blows are absorbed,
    // the shield breaks at the top of the second, and the second attack lands.
    // This is why Windfury is modelled as two attacks rather than one doubled
    // attack.
    let r = resolve(
        board_of(
            vec![u("gusty", 3, 9, &[Keyword::Windfury])],
            vec![u("shielded", 1, 3, &[Keyword::DivineShield])],
        ),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::PlayerWins);
    assert_eq!(
        r.beats, 3,
        "one Beat absorbed, one to land the second attack, one to bury"
    );
    assert_eq!(
        count_events(&r.log, |e| matches!(e, Event::ShieldAbsorbed { .. })),
        2,
        "the first Beat's attack and the answer to it both met the shield"
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
    assert_eq!(r.beats, 2, "one Beat to poison, one to bury");
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
    assert_eq!(r.beats, 3, "survived the first blow entirely");
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
    assert_eq!(r.beats, 3, "died, returned with 1 health, died again");
}

#[test]
fn reborn_returns_a_unit_in_the_beat_that_buries_it() {
    let r = resolve(
        board_of(
            vec![plain("hammer", 3, 20)],
            vec![u("phoenix", 1, 3, &[Keyword::Reborn])],
        ),
        &mut rng(),
    );
    let b = beats(&r);
    let burial = b
        .iter()
        .find(|(_, events)| events.iter().any(|e| matches!(e, Event::Died { .. })))
        .expect("the phoenix died");
    assert!(
        burial
            .1
            .iter()
            .any(|e| matches!(e, Event::Reborn { slot: 1, .. })),
        "it returns in the Slot it left, in the same Beat"
    );
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
    // rather than hung on. Both Units spend their Intent every Beat and neither
    // has an attack to make, so the Beats run empty right up to the cap.
    let r = resolve(
        board_of(vec![plain("rock", 0, 5)], vec![plain("stone", 0, 5)]),
        &mut rng(),
    );
    assert_eq!(r.outcome, Outcome::Stalemate);
    assert_eq!(r.beats, MAX_BEATS);
    assert_eq!(
        count_events(&r.log, |e| !matches!(
            e,
            Event::BeatBegan { .. } | Event::Ended { .. }
        )),
        0,
        "nothing happened in any of them"
    );
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
    assert!(text.contains(&format!("{:?}", r.outcome)), "{text}");
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
    let struck = r
        .log
        .iter()
        .find(|e| matches!(e, Event::Struck { .. }))
        .expect("something struck");
    assert!(matches!(
        struck,
        Event::Struck {
            by: Side::Player,
            attacker_slot: 1,
            target_slot: 1,
            damage: 2,
        }
    ));
}

// A Beat is one slice of time, so nothing inside it may depend on the order the
// engine happens to walk the two Parties.
#[test]
fn a_board_and_its_mirror_resolve_the_same_way() {
    let shielded = u("shielded", 9, 200, &[Keyword::DivineShield]);
    let heavy = plain("heavy", 9, 20);
    // Taunt pins targeting, so the only variable left is walk order.
    let taunter = u("taunter", 2, 20, &[Keyword::Taunt]);

    let one = resolve(
        board_of(vec![shielded.clone()], vec![heavy.clone(), taunter.clone()]),
        &mut rng(),
    );
    let other = resolve(board_of(vec![heavy, taunter], vec![shielded]), &mut rng());

    assert_eq!(
        one.final_board.player.get(0).map(|unit| unit.health),
        other.final_board.opposing.get(0).map(|unit| unit.health),
        "the shielded Unit fared differently depending on which side it stood on"
    );
}

// ---------------------------------------------------------------------------
// A Resolution is a replay, and a replay travels
// ---------------------------------------------------------------------------

#[test]
fn a_resolution_survives_a_round_trip_through_serde() {
    // The log is the frontend's whole interface, so it has to be able to leave
    // the process: across a WASM boundary, into a recorded fight, into a golden
    // file.
    let r = resolve(
        board_of(
            vec![
                plain("badger", 2, 3),
                u("gusty", 3, 6, &[Keyword::Windfury]),
            ],
            vec![
                u("shielded", 2, 4, &[Keyword::DivineShield]),
                u("phoenix", 1, 2, &[Keyword::Reborn]),
            ],
        ),
        &mut rng(),
    );
    let text = ron::to_string(&r).expect("a Resolution serializes");
    let back: Resolution = ron::from_str(&text).expect("and comes back");
    assert_eq!(back, r);
}

#[test]
fn the_opening_board_replays_into_the_same_fight() {
    // Every Event states a change rather than a state, so the opening Board is
    // the other half of a replay. Kept honestly: it is the Board as it stood, not
    // the caller's copy.
    let r = resolve(
        board_of(
            vec![plain("badger", 2, 3)],
            vec![u("shielded", 2, 4, &[Keyword::DivineShield])],
        ),
        &mut rng(),
    );
    let again = resolve(r.initial_board.clone(), &mut rng());
    assert_eq!(again, r, "same Board, same Rng state, same fight");
}
