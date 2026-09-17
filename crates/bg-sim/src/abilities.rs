//! Executes the ability vocabulary declared in `units.rs` -- `Trigger`,
//! `Condition`, `Selector`, `Effect` -- against a live `Board`. `units.rs` is
//! the nouns a data file is written in; this is the only thing that knows
//! what they mean.
//!
//! Every entry point here is a *moment the caller already passes through* --
//! `action_phase::resolve_with_roster` and `prep_phase::RunState` call in at a
//! point their own loop already reaches, never a new phase invented for this.
//!
//! `GainGold` is the one Effect this module cannot apply on its own -- gold
//! lives on `prep_phase::RunState`, not `Board`. Every entry point threads a
//! `gold: &mut u32` through for it; an Action Phase call passes a throwaway
//! scratch value, which is exactly `GainGold`'s own documented "ignored in
//! the Action Phase."

use crate::action_phase::Event;
use crate::party::{Board, Side, Unit};
use crate::rng::Rng;
use crate::units::{Condition, DefId, Effect, Filter, Modifier, Selector, Trigger, UnitDef};

fn def_by_id<'a>(roster: &'a [UnitDef], id: &DefId) -> Option<&'a UnitDef> {
    roster.iter().find(|d| &d.id == id)
}

/// What an ability's `Condition`/`Selector`/`Effect` may reference beyond the
/// Board itself.
struct Ctx {
    side: Side,
    /// The acting Unit's Slot, if it's still standing there. `None` for a
    /// Deathrattle, which fires after the Unit has already been buried --
    /// there is nothing left for `Selector::This` to resolve to, which is
    /// correct: a dead Unit cannot be buffed by its own last word.
    acting_slot: Option<usize>,
    /// The other Unit that caused the trigger -- summoned, played, sold, or
    /// the answering Unit's own attacker. `Selector::Subject` and every
    /// `Condition::Subject*` read this.
    subject: Option<(Side, usize)>,
    /// The Player's Tavern Tier, for `Condition::TierAtLeast`. Meaningless
    /// during the Action Phase (Tavern Tier is a Prep Phase concept), so
    /// Action Phase callers pass 0 -- a Unit gated on Tier can only ever
    /// mean a Prep Phase ability, and this makes that true by construction.
    tier: u8,
}

/// Fire every ability an already-standing Unit owns for `trigger` --
/// Battlecry, OnAttack, OnSurviveDamage, OnBuy, OnSell, StartOfTurn,
/// EndOfTurn. The Unit is still in its Slot when this runs.
#[allow(clippy::too_many_arguments)]
pub fn fire_own(
    board: &mut Board,
    roster: &[UnitDef],
    side: Side,
    slot: usize,
    trigger: Trigger,
    tier: u8,
    log: &mut Vec<Event>,
    rng: &mut Rng,
    gold: &mut u32,
) {
    let Some(def) = board
        .side(side)
        .get(slot)
        .and_then(|u| def_by_id(roster, &u.def))
        .cloned()
    else {
        return;
    };
    let ctx = Ctx {
        side,
        acting_slot: Some(slot),
        subject: None,
        tier,
    };
    run_abilities(board, roster, &def, &ctx, trigger, log, rng, gold);
}

/// Fire Deathrattle for a Unit that has just left the Board. `def_id` is
/// looked up before removal -- there is nothing on the Board to look it up
/// through afterward.
#[allow(clippy::too_many_arguments)]
pub fn fire_deathrattle(
    board: &mut Board,
    roster: &[UnitDef],
    side: Side,
    def_id: &DefId,
    log: &mut Vec<Event>,
    rng: &mut Rng,
    gold: &mut u32,
) {
    let Some(def) = def_by_id(roster, def_id).cloned() else {
        return;
    };
    let ctx = Ctx {
        side,
        acting_slot: None,
        subject: None,
        tier: 0,
    };
    run_abilities(
        board,
        roster,
        &def,
        &ctx,
        Trigger::Deathrattle,
        log,
        rng,
        gold,
    );
}

/// Fire `trigger` for every Unit on `side` other than `subject_slot`, with
/// `subject_slot` as the ability's Subject -- AfterFriendlySummon,
/// AfterFriendlyDeath, AfterFriendlyPlayed.
#[allow(clippy::too_many_arguments)]
pub fn fire_broadcast(
    board: &mut Board,
    roster: &[UnitDef],
    side: Side,
    subject_slot: usize,
    trigger: Trigger,
    tier: u8,
    log: &mut Vec<Event>,
    rng: &mut Rng,
    gold: &mut u32,
) {
    let listeners: Vec<usize> = board
        .side(side)
        .iter()
        .map(|(i, _)| i)
        .filter(|&i| i != subject_slot)
        .collect();
    for slot in listeners {
        let Some(def) = board
            .side(side)
            .get(slot)
            .and_then(|u| def_by_id(roster, &u.def))
            .cloned()
        else {
            continue;
        };
        let ctx = Ctx {
            side,
            acting_slot: Some(slot),
            subject: Some((side, subject_slot)),
            tier,
        };
        run_abilities(board, roster, &def, &ctx, trigger, log, rng, gold);
    }
}

/// Fire `trigger` once for every Unit currently on the Board, each for
/// itself -- StartOfActionPhase.
pub fn fire_all(
    board: &mut Board,
    roster: &[UnitDef],
    trigger: Trigger,
    log: &mut Vec<Event>,
    rng: &mut Rng,
    gold: &mut u32,
) {
    for side in [Side::Player, Side::Opposing] {
        let slots: Vec<usize> = board.side(side).iter().map(|(i, _)| i).collect();
        for slot in slots {
            fire_own(board, roster, side, slot, trigger, 0, log, rng, gold);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_abilities(
    board: &mut Board,
    roster: &[UnitDef],
    def: &UnitDef,
    ctx: &Ctx,
    trigger: Trigger,
    log: &mut Vec<Event>,
    rng: &mut Rng,
    gold: &mut u32,
) {
    for ability in &def.abilities {
        if ability.trigger != trigger {
            continue;
        }
        if let Some(condition) = &ability.condition
            && !eval_condition(condition, board, ctx)
        {
            continue;
        }
        log.push(Event::AbilityFired {
            side: ctx.side,
            name: def.name.clone(),
        });
        for effect in &ability.effects {
            apply_effect(effect, board, roster, ctx, log, rng, gold);
        }
    }
}

fn subject_unit<'a>(board: &'a Board, ctx: &Ctx) -> Option<&'a Unit> {
    let (side, slot) = ctx.subject?;
    board.side(side).get(slot)
}

fn eval_condition(condition: &Condition, board: &Board, ctx: &Ctx) -> bool {
    match condition {
        Condition::SubjectHasTribe(tribe) => {
            subject_unit(board, ctx).is_some_and(|u| u.has_tribe(*tribe))
        }
        Condition::SubjectHasKeyword(keyword) => {
            subject_unit(board, ctx).is_some_and(|u| u.has(*keyword))
        }
        Condition::TierAtLeast(tier) => ctx.tier >= *tier,
        Condition::All(cs) => cs.iter().all(|c| eval_condition(c, board, ctx)),
        Condition::Any(cs) => cs.iter().any(|c| eval_condition(c, board, ctx)),
        Condition::Not(c) => !eval_condition(c, board, ctx),
    }
}

fn matches_filter(unit: &Unit, slot: usize, ctx: &Ctx, target_side: Side, filter: &Filter) -> bool {
    let is_self = target_side == ctx.side && Some(slot) == ctx.acting_slot;
    if is_self && !filter.include_self {
        return false;
    }
    if let Some(tribe) = filter.tribe
        && !unit.has_tribe(tribe)
    {
        return false;
    }
    if let Some(keyword) = filter.keyword
        && !unit.has(keyword)
    {
        return false;
    }
    true
}

fn matching(board: &Board, side: Side, ctx: &Ctx, filter: &Filter) -> Vec<(Side, usize)> {
    board
        .side(side)
        .iter()
        .filter(|(i, u)| matches_filter(u, *i, ctx, side, filter))
        .map(|(i, _)| (side, i))
        .collect()
}

fn pick_random(
    board: &Board,
    side: Side,
    ctx: &Ctx,
    filter: &Filter,
    count: u32,
    rng: &mut Rng,
) -> Vec<(Side, usize)> {
    let mut candidates = matching(board, side, ctx, filter);
    let mut out = Vec::new();
    for _ in 0..count {
        match rng.swap_take(&mut candidates) {
            Some(pick) => out.push(pick),
            None => break,
        }
    }
    out
}

fn resolve_selector(
    selector: &Selector,
    board: &Board,
    ctx: &Ctx,
    rng: &mut Rng,
) -> Vec<(Side, usize)> {
    match selector {
        Selector::This => ctx
            .acting_slot
            .map(|slot| vec![(ctx.side, slot)])
            .unwrap_or_default(),
        Selector::Subject => ctx.subject.into_iter().collect(),
        Selector::Adjacent => ctx
            .acting_slot
            .map(|slot| {
                board
                    .side(ctx.side)
                    .neighbours(slot)
                    .into_iter()
                    .map(|i| (ctx.side, i))
                    .collect()
            })
            .unwrap_or_default(),
        Selector::RandomFriendly { count, filter } => {
            pick_random(board, ctx.side, ctx, filter, *count, rng)
        }
        Selector::AllFriendly { filter } => matching(board, ctx.side, ctx, filter),
        Selector::RandomEnemy { count, filter } => {
            pick_random(board, ctx.side.other(), ctx, filter, *count, rng)
        }
        Selector::AllEnemy { filter } => matching(board, ctx.side.other(), ctx, filter),
    }
}

fn apply_modifier(unit: &mut Unit, modifier: &Modifier) {
    match modifier {
        Modifier::Attack(delta) => unit.attack += delta,
        Modifier::Health(delta) => {
            unit.max_health += delta;
            unit.health += delta;
        }
        Modifier::Grant(keyword) => {
            unit.keywords.insert(*keyword);
        }
    }
}

/// A Unit dealt damage by an ability, respecting Divine Shield and Poisonous
/// exactly as a combat blow does -- absorption, not breakage, and a
/// Poisonous mark that only takes if the blow actually lands. Mirrors
/// `action_phase::hit`, kept separate because that one also logs a combat
/// Struck/StruckBack pair an ability never causes.
fn damage(board: &mut Board, side: Side, slot: usize, amount: i32, poisonous: bool) {
    let Some(unit) = board.side_mut(side).get_mut(slot) else {
        return;
    };
    if unit.has(crate::units::Keyword::DivineShield) {
        unit.shield_spent = true;
        return;
    }
    unit.health -= amount;
    if poisonous {
        unit.poisoned = true;
    }
}

/// The first open Slot on `side`, searching from `from` (inclusive) rightward
/// and then wrapping to the front -- "immediately to its right", without a
/// shift-everything-else operation this codebase has no other use for. A
/// Deathrattle's own vacated Slot is already open by the time this runs, so
/// summoning there first is exactly "returns where it fell."
fn first_open_slot(board: &Board, side: Side, from: usize) -> Option<usize> {
    let party = board.side(side);
    (from..crate::party::SLOTS)
        .chain(0..from)
        .find(|&i| party.get(i).is_none())
}

#[allow(clippy::too_many_arguments)]
fn apply_effect(
    effect: &Effect,
    board: &mut Board,
    roster: &[UnitDef],
    ctx: &Ctx,
    log: &mut Vec<Event>,
    rng: &mut Rng,
    gold: &mut u32,
) {
    match effect {
        Effect::Buff {
            target,
            attack,
            health,
        } => {
            for (side, slot) in resolve_selector(target, board, ctx, rng) {
                if let Some(unit) = board.side_mut(side).get_mut(slot) {
                    unit.attack += attack;
                    unit.max_health += health;
                    unit.health += health;
                }
            }
        }
        Effect::Grant { target, keyword } => {
            for (side, slot) in resolve_selector(target, board, ctx, rng) {
                if let Some(unit) = board.side_mut(side).get_mut(slot) {
                    unit.keywords.insert(*keyword);
                }
            }
        }
        Effect::Revoke { target, keyword } => {
            for (side, slot) in resolve_selector(target, board, ctx, rng) {
                if let Some(unit) = board.side_mut(side).get_mut(slot) {
                    unit.keywords.remove(keyword);
                }
            }
        }
        Effect::Damage { target, amount } => {
            for (side, slot) in resolve_selector(target, board, ctx, rng) {
                let poisonous = board
                    .side(ctx.side)
                    .get(ctx.acting_slot.unwrap_or(usize::MAX))
                    .is_some_and(|u| u.has(crate::units::Keyword::Poisonous));
                damage(board, side, slot, *amount, poisonous);
            }
        }
        Effect::Summon {
            unit: unit_id,
            count,
            modify,
        } => {
            let Some(def) = def_by_id(roster, unit_id) else {
                return;
            };
            let start = ctx.acting_slot.unwrap_or(0);
            for _ in 0..*count {
                let Some(slot) = first_open_slot(board, ctx.side, start) else {
                    break;
                };
                let mut unit = Unit::new(def);
                for modifier in modify {
                    apply_modifier(&mut unit, modifier);
                }
                board.side_mut(ctx.side).put(slot, unit);
                fire_broadcast(
                    board,
                    roster,
                    ctx.side,
                    slot,
                    Trigger::AfterFriendlySummon,
                    ctx.tier,
                    log,
                    rng,
                    gold,
                );
            }
        }
        Effect::GainGold(amount) => {
            *gold = gold.saturating_add_signed(*amount);
        }
        Effect::AddToHand {
            unit: unit_id,
            count,
        } => {
            // No separate hand exists in this engine (buying already places a
            // Unit straight onto the board) -- the closest faithful meaning
            // available is placing it directly, the same as a purchase does.
            let Some(def) = def_by_id(roster, unit_id) else {
                return;
            };
            for _ in 0..*count {
                let Some(slot) = first_open_slot(board, ctx.side, 0) else {
                    break;
                };
                board.side_mut(ctx.side).put(slot, Unit::new(def));
            }
        }
        Effect::IfThen {
            condition,
            then,
            otherwise,
        } => {
            let branch = if eval_condition(condition, board, ctx) {
                then
            } else {
                otherwise
            };
            for effect in branch {
                apply_effect(effect, board, roster, ctx, log, rng, gold);
            }
        }
        Effect::Repeat { times, effects } => {
            for _ in 0..*times {
                for effect in effects {
                    apply_effect(effect, board, roster, ctx, log, rng, gold);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_phase::board_of;
    use crate::rng::{Domain, Seed};
    use crate::units::{Ability, Keyword, Tribe};

    fn rng() -> Rng {
        Seed(1).stream(Domain::Effect, 0)
    }

    fn def(id: &str, attack: i32, health: i32, abilities: Vec<Ability>) -> UnitDef {
        UnitDef {
            id: DefId::new(id),
            name: id.to_owned(),
            tier: 1,
            attack,
            health,
            tribes: vec![],
            all_tribes: false,
            keywords: vec![],
            abilities,
            token: false,
            text: String::new(),
        }
    }

    #[test]
    fn battlecry_buffs_a_neighbour_on_entry() {
        let caster = def(
            "caster",
            1,
            1,
            vec![Ability {
                trigger: Trigger::Battlecry,
                condition: None,
                effects: vec![Effect::Buff {
                    target: Selector::Adjacent,
                    attack: 1,
                    health: 1,
                }],
            }],
        );
        let neighbour = def("neighbour", 1, 1, vec![]);
        let roster = vec![caster.clone(), neighbour.clone()];
        let mut board = board_of(vec![Unit::new(&neighbour), Unit::new(&caster)], vec![]);
        let mut log = Vec::new();
        fire_own(
            &mut board,
            &roster,
            Side::Player,
            1,
            Trigger::Battlecry,
            0,
            &mut log,
            &mut rng(),
            &mut 0,
        );
        assert_eq!(board.player.get(0).unwrap().attack, 2);
        assert_eq!(board.player.get(0).unwrap().health, 2);
    }

    #[test]
    fn deathrattle_fires_after_the_unit_is_already_gone() {
        let token = def("token", 1, 1, vec![]);
        let caster = def(
            "caster",
            1,
            1,
            vec![Ability {
                trigger: Trigger::Deathrattle,
                condition: None,
                effects: vec![Effect::Summon {
                    unit: DefId::new("token"),
                    count: 2,
                    modify: vec![],
                }],
            }],
        );
        let roster = vec![caster.clone(), token];
        let mut board = board_of(vec![], vec![]);
        // Simulate `bury`'s own order: the Unit is already removed from the
        // Board by the time Deathrattle fires.
        let mut log = Vec::new();
        fire_deathrattle(
            &mut board,
            &roster,
            Side::Player,
            &caster.id,
            &mut log,
            &mut rng(),
            &mut 0,
        );
        assert_eq!(board.player.len(), 2);
        assert_eq!(board.player.get(0).unwrap().def, DefId::new("token"));
    }

    #[test]
    fn a_condition_gates_the_ability() {
        let watcher = def(
            "watcher",
            1,
            1,
            vec![Ability {
                trigger: Trigger::AfterFriendlyPlayed,
                condition: Some(Condition::SubjectHasTribe(Tribe::Beast)),
                effects: vec![Effect::Buff {
                    target: Selector::Subject,
                    attack: 1,
                    health: 0,
                }],
            }],
        );
        let mut beast = def("beast", 1, 1, vec![]);
        beast.tribes = vec![Tribe::Beast];
        let mut fish = def("fish", 1, 1, vec![]);
        fish.tribes = vec![];
        let roster = vec![watcher.clone(), beast.clone(), fish.clone()];

        let mut board = board_of(
            vec![Unit::new(&watcher), Unit::new(&beast), Unit::new(&fish)],
            vec![],
        );
        let mut log = Vec::new();
        fire_broadcast(
            &mut board,
            &roster,
            Side::Player,
            1, // the Beast played
            Trigger::AfterFriendlyPlayed,
            0,
            &mut log,
            &mut rng(),
            &mut 0,
        );
        assert_eq!(
            board.player.get(1).unwrap().attack,
            2,
            "the Beast got buffed"
        );

        fire_broadcast(
            &mut board,
            &roster,
            Side::Player,
            2, // the non-Beast played
            Trigger::AfterFriendlyPlayed,
            0,
            &mut log,
            &mut rng(),
            &mut 0,
        );
        assert_eq!(
            board.player.get(2).unwrap().attack,
            1,
            "the condition held the ability back for a non-Beast subject"
        );
    }

    #[test]
    fn gain_gold_credits_the_threaded_sink() {
        let payer = def(
            "payer",
            1,
            1,
            vec![Ability {
                trigger: Trigger::OnSell,
                condition: None,
                effects: vec![Effect::GainGold(2)],
            }],
        );
        let roster = vec![payer.clone()];
        let mut board = board_of(vec![Unit::new(&payer)], vec![]);
        let mut log = Vec::new();
        let mut gold = 3u32;
        fire_own(
            &mut board,
            &roster,
            Side::Player,
            0,
            Trigger::OnSell,
            0,
            &mut log,
            &mut rng(),
            &mut gold,
        );
        assert_eq!(gold, 5);
    }

    #[test]
    fn if_then_picks_the_right_branch() {
        let unit = def(
            "unit",
            1,
            1,
            vec![Ability {
                trigger: Trigger::StartOfTurn,
                condition: None,
                effects: vec![Effect::IfThen {
                    condition: Condition::TierAtLeast(3),
                    then: vec![Effect::Buff {
                        target: Selector::This,
                        attack: 5,
                        health: 0,
                    }],
                    otherwise: vec![Effect::Buff {
                        target: Selector::This,
                        attack: 1,
                        health: 0,
                    }],
                }],
            }],
        );
        let roster = vec![unit.clone()];
        let mut board = board_of(vec![Unit::new(&unit)], vec![]);
        let mut log = Vec::new();
        fire_own(
            &mut board,
            &roster,
            Side::Player,
            0,
            Trigger::StartOfTurn,
            1, // Tier 1: below the gate
            &mut log,
            &mut rng(),
            &mut 0,
        );
        assert_eq!(board.player.get(0).unwrap().attack, 2, "otherwise branch");
    }

    #[test]
    fn repeat_runs_the_effect_the_stated_number_of_times() {
        let unit = def(
            "unit",
            1,
            1,
            vec![Ability {
                trigger: Trigger::StartOfTurn,
                condition: None,
                effects: vec![Effect::Repeat {
                    times: 3,
                    effects: vec![Effect::Buff {
                        target: Selector::This,
                        attack: 1,
                        health: 0,
                    }],
                }],
            }],
        );
        let roster = vec![unit.clone()];
        let mut board = board_of(vec![Unit::new(&unit)], vec![]);
        let mut log = Vec::new();
        fire_own(
            &mut board,
            &roster,
            Side::Player,
            0,
            Trigger::StartOfTurn,
            0,
            &mut log,
            &mut rng(),
            &mut 0,
        );
        assert_eq!(board.player.get(0).unwrap().attack, 4);
    }

    #[test]
    fn summoning_broadcasts_after_friendly_summon() {
        let watcher = def(
            "watcher",
            1,
            1,
            vec![Ability {
                trigger: Trigger::AfterFriendlySummon,
                condition: None,
                effects: vec![Effect::Buff {
                    target: Selector::This,
                    attack: 0,
                    health: 1,
                }],
            }],
        );
        let summoner = def(
            "summoner",
            1,
            1,
            vec![Ability {
                trigger: Trigger::Battlecry,
                condition: None,
                effects: vec![Effect::Summon {
                    unit: DefId::new("token"),
                    count: 1,
                    modify: vec![],
                }],
            }],
        );
        let token = def("token", 1, 1, vec![]);
        let roster = vec![watcher.clone(), summoner.clone(), token];
        let mut board = board_of(vec![Unit::new(&watcher), Unit::new(&summoner)], vec![]);
        let mut log = Vec::new();
        fire_own(
            &mut board,
            &roster,
            Side::Player,
            1,
            Trigger::Battlecry,
            0,
            &mut log,
            &mut rng(),
            &mut 0,
        );
        assert_eq!(
            board.player.get(0).unwrap().health,
            2,
            "the watcher heard the summon and buffed itself"
        );
    }

    #[test]
    fn grant_is_idempotent_and_revoke_is_silent_on_absence() {
        let unit = def("unit", 1, 1, vec![]);
        let roster = vec![unit.clone()];
        let mut board = board_of(vec![Unit::new(&unit)], vec![]);
        let ctx_side = Side::Player;
        let mut log = Vec::new();
        apply_effect(
            &Effect::Grant {
                target: Selector::This,
                keyword: Keyword::Taunt,
            },
            &mut board,
            &roster,
            &Ctx {
                side: ctx_side,
                acting_slot: Some(0),
                subject: None,
                tier: 0,
            },
            &mut log,
            &mut rng(),
            &mut 0,
        );
        assert!(board.player.get(0).unwrap().has(Keyword::Taunt));
        apply_effect(
            &Effect::Revoke {
                target: Selector::This,
                keyword: Keyword::DivineShield,
            },
            &mut board,
            &roster,
            &Ctx {
                side: ctx_side,
                acting_slot: Some(0),
                subject: None,
                tier: 0,
            },
            &mut log,
            &mut rng(),
            &mut 0,
        );
        assert!(!board.player.get(0).unwrap().has(Keyword::DivineShield));
    }
}
