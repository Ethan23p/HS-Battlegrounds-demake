//! The Prep Phase: shop a party together between fights.
//!
//! See `docs/DESIGN.md`. A run is a sequence of rounds, each one Prep Phase
//! (this module) followed by one Action Phase (`crate::action_phase`). Runs
//! are best of 3: two wins takes the run, two losses ends it.
//!
//! **Every Unit the shop can offer draws from, and returns to, a scarce
//! pool.** Buying a Unit removes its copy from the pool for as long as it's in
//! play; selling it, or an unbought offer being cleared by a reroll or a new
//! round, returns that copy. This is Ethan's framing, not a mechanic borrowed
//! from Battlegrounds for its own sake -- an informal discipline of treating
//! every in-game object as drawing from and returning to a shared, finite
//! resource, the way a board game's physical pieces do. Gold is deliberately
//! exempt: it's an abstract resource, not something with a copy count to
//! conserve. A token (`UnitDef::token`) is the other exception, by
//! construction: it never enters the pool at all, so there's nothing for it to
//! draw or return.
//!
//! The opposing Party each round is procedural (`RunState::matchmake`), not
//! itself a participant with a pool of its own -- `party.rs` is explicit that
//! play is single-player and the opposing Party is data, and that's the model
//! this follows: matchmaking samples the roster fresh each round rather than
//! competing with the player for the same copies.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::abilities;
use crate::action_phase::Outcome;
use crate::party::Unit;
use crate::party::{Board, Party, SLOTS, Side};
use crate::rng::{Domain, Rng, Seed};
use crate::units::{DefId, Trigger, UnitDef};

/// Tavern tiers run 1..=6, same range `UnitDef::tier` already uses.
pub const MAX_TAVERN_TIER: u8 = 6;

/// Wins needed to take the run.
pub const WINS_TO_TAKE_RUN: u8 = 2;
/// Losses needed to end the run.
pub const LOSSES_TO_END_RUN: u8 = 2;

const BUY_COST: u32 = 3;
const SELL_REFUND: u32 = 1;
const REROLL_COST: u32 = 1;

/// How many copies of a Tier's Units start in the pool. Tapered so early,
/// cheap Units are plentiful and late, expensive ones are scarce -- the same
/// shape Battlegrounds itself uses. Placeholder numbers, tunable once the loop
/// can be felt (see `docs/scratchpad/roadmap.md`).
fn copies_for_tier(tier: u8) -> u32 {
    match tier {
        1 => 18,
        2 => 15,
        3 => 13,
        4 => 11,
        5 => 9,
        _ => 6,
    }
}

/// How many offers the shop shows at a given Tavern Tier.
fn shop_size_for_tier(tier: u8) -> usize {
    match tier {
        1 | 2 => 3,
        3 | 4 => 4,
        5 => 5,
        _ => 6,
    }
}

/// Gold cost to upgrade from `tier` to `tier + 1`.
fn tavern_upgrade_cost(tier: u8) -> u32 {
    tier as u32 * 5
}

/// Gold granted at the start of a round. Ramps early, caps at 10 -- same
/// shape as Battlegrounds' own curve. Placeholder, like the costs above.
fn gold_for_round(round: u32) -> u32 {
    (round + 2).min(10)
}

fn def_by_id<'a>(roster: &'a [UnitDef], id: &DefId) -> Option<&'a UnitDef> {
    roster.iter().find(|d| &d.id == id)
}

/// A multiset of Unit copies, keyed by Definition. `BTreeMap` rather than
/// `HashMap`: nothing here iterates it in a way that reaches a log, but every
/// other collection in this codebase keying on `DefId` is ordered for the same
/// reason, and there's no cause to be the exception.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pool {
    counts: BTreeMap<DefId, u32>,
}

impl Pool {
    /// Build a full pool from a roster, tapered by Tier. Tokens are excluded
    /// -- they never enter the shop pool (`UnitDef::token`).
    pub fn new(roster: &[UnitDef]) -> Self {
        let counts = roster
            .iter()
            .filter(|def| !def.token)
            .map(|def| (def.id.clone(), copies_for_tier(def.tier)))
            .collect();
        Pool { counts }
    }

    /// Copies of `id` currently available to be drawn.
    pub fn available(&self, id: &DefId) -> u32 {
        self.counts.get(id).copied().unwrap_or(0)
    }

    fn take_one(&mut self, id: &DefId) {
        if let Some(n) = self.counts.get_mut(id) {
            *n = n.saturating_sub(1);
        }
    }

    /// Return one copy to the pool. The one place the conservation law is
    /// enforced -- sell, an unbought offer clearing, a reroll -- all funnel
    /// through this rather than each remembering to credit the pool back.
    fn return_one(&mut self, id: &DefId) {
        if let Some(n) = self.counts.get_mut(id) {
            *n += 1;
        }
    }

    /// Draw up to `count` copies at or below `max_tier`, weighted by how much
    /// stock remains of each. Drawn copies leave the pool immediately -- a
    /// shown offer is a reservation, not a free look.
    fn draw(
        &mut self,
        roster: &[UnitDef],
        max_tier: u8,
        count: usize,
        rng: &mut Rng,
    ) -> Vec<DefId> {
        let mut bag: Vec<DefId> = Vec::new();
        for def in roster {
            if def.tier > max_tier {
                continue;
            }
            for _ in 0..self.available(&def.id) {
                bag.push(def.id.clone());
            }
        }
        let mut drawn = Vec::with_capacity(count);
        for _ in 0..count {
            let Some(id) = rng.swap_take(&mut bag) else {
                break;
            };
            self.take_one(&id);
            drawn.push(id);
        }
        drawn
    }
}

/// One Unit sitting in the shop, and whether the player froze it against the
/// next reroll.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopSlot {
    pub def: DefId,
    pub frozen: bool,
}

/// Everything a run needs to remember between rounds: gold, Tavern Tier, the
/// board being built, the shop on offer, the pool it draws from, and the
/// running best-of-3 score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunState {
    pub board: Party,
    pub gold: u32,
    pub tavern_tier: u8,
    pub wins: u8,
    pub losses: u8,
    pub round: u32,
    pub shop: Vec<ShopSlot>,
    pool: Pool,
    seed: Seed,
}

/// Rejected shop and Tavern actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopError {
    NotEnoughGold,
    BoardFull,
    SlotEmpty,
    InvalidOffer,
    MaxTavernTier,
}

impl std::fmt::Display for ShopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ShopError::NotEnoughGold => "not enough gold",
            ShopError::BoardFull => "board is full",
            ShopError::SlotEmpty => "slot is empty",
            ShopError::InvalidOffer => "no such shop offer",
            ShopError::MaxTavernTier => "already at the highest Tavern Tier",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for ShopError {}

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunOutcome {
    PlayerWinsRun,
    PlayerLosesRun,
}

impl RunState {
    /// Start a fresh run: Tavern Tier 1, round 1's gold, an empty board, a
    /// full pool, and the first shop already drawn.
    pub fn new(seed: Seed, roster: &[UnitDef]) -> Self {
        let mut state = RunState {
            board: Party::new(),
            gold: gold_for_round(1),
            tavern_tier: 1,
            wins: 0,
            losses: 0,
            round: 1,
            shop: Vec::new(),
            pool: Pool::new(roster),
            seed,
        };
        state.refresh_shop(roster);
        state
    }

    fn rng(&self, domain: Domain) -> Rng {
        self.seed.stream(domain, self.round as u64)
    }

    /// Run a self-triggered ability against `self.board` alone. Prep Phase has
    /// no opposing Party to fight, so a scratch empty one stands in;
    /// cross-side selectors (RandomEnemy/AllEnemy) are documented in
    /// `units::Selector` as meaningless outside the Action Phase, and simply
    /// find nothing against it.
    fn fire_own(&mut self, roster: &[UnitDef], slot: usize, trigger: Trigger) {
        let mut board = Board::new(self.board.clone(), Party::new());
        let mut log = Vec::new();
        let mut rng = self.rng(Domain::Effect);
        abilities::fire_own(
            &mut board,
            roster,
            Side::Player,
            slot,
            trigger,
            self.tavern_tier,
            &mut log,
            &mut rng,
            &mut self.gold,
        );
        self.board = board.player;
    }

    /// Broadcast a trigger to the rest of `self.board`, with `subject_slot`
    /// as the ability's Subject. See [`Self::fire_own`] on the scratch board.
    fn fire_broadcast(&mut self, roster: &[UnitDef], subject_slot: usize, trigger: Trigger) {
        let mut board = Board::new(self.board.clone(), Party::new());
        let mut log = Vec::new();
        let mut rng = self.rng(Domain::Effect);
        abilities::fire_broadcast(
            &mut board,
            roster,
            Side::Player,
            subject_slot,
            trigger,
            self.tavern_tier,
            &mut log,
            &mut rng,
            &mut self.gold,
        );
        self.board = board.player;
    }

    /// Fire `trigger` for every Unit already on the board, each for itself --
    /// StartOfTurn, EndOfTurn.
    fn fire_all_own(&mut self, roster: &[UnitDef], trigger: Trigger) {
        for slot in (0..SLOTS)
            .filter(|&i| self.board.get(i).is_some())
            .collect::<Vec<_>>()
        {
            self.fire_own(roster, slot, trigger);
        }
    }

    /// Return every unfrozen offer to the pool, then draw a fresh shop up to
    /// this Tavern Tier's size. Sequential mutable borrows of `self.shop` and
    /// `self.pool`, not simultaneous -- deliberately not a `retain` closure,
    /// which would need to hold both at once.
    fn refresh_shop(&mut self, roster: &[UnitDef]) {
        let mut kept = Vec::new();
        for slot in self.shop.drain(..) {
            if slot.frozen {
                kept.push(slot);
            } else {
                self.pool.return_one(&slot.def);
            }
        }
        self.shop = kept;

        let need = shop_size_for_tier(self.tavern_tier).saturating_sub(self.shop.len());
        let mut rng = self.rng(Domain::Shop);
        let drawn = self.pool.draw(roster, self.tavern_tier, need, &mut rng);
        self.shop
            .extend(drawn.into_iter().map(|def| ShopSlot { def, frozen: false }));
    }

    /// Spend gold to clear and redraw every unfrozen offer.
    pub fn reroll(&mut self, roster: &[UnitDef]) -> Result<(), ShopError> {
        if self.gold < REROLL_COST {
            return Err(ShopError::NotEnoughGold);
        }
        self.gold -= REROLL_COST;
        self.refresh_shop(roster);
        Ok(())
    }

    /// Toggle whether the whole shop survives the next reroll -- one button
    /// for the whole lineup, the way Battlegrounds itself does it, not a
    /// per-card choice. Freezing when anything is still unfrozen freezes
    /// everything; freezing again (once the whole shop is already frozen)
    /// unfreezes it.
    pub fn toggle_freeze(&mut self) {
        let freeze = self.shop.iter().any(|slot| !slot.frozen);
        for slot in &mut self.shop {
            slot.frozen = freeze;
        }
    }

    /// Buy an offer onto the first open board Slot.
    pub fn buy(&mut self, roster: &[UnitDef], offer: usize) -> Result<(), ShopError> {
        if self.board.len() >= SLOTS {
            return Err(ShopError::BoardFull);
        }
        if self.gold < BUY_COST {
            return Err(ShopError::NotEnoughGold);
        }
        let slot = self.shop.get(offer).ok_or(ShopError::InvalidOffer)?;
        let def = def_by_id(roster, &slot.def).ok_or(ShopError::InvalidOffer)?;
        let unit = Unit::new(def);

        self.gold -= BUY_COST;
        self.shop.remove(offer);
        let empty = (0..SLOTS)
            .find(|&i| self.board.get(i).is_none())
            .expect("checked board.len() < SLOTS above");
        self.board.put(empty, unit);
        // Buying is both moments `units::Trigger` names separately: bought
        // into hand (OnBuy) and played into a Slot (Battlecry). This engine
        // has no hand step in between, so both fire together, right here.
        self.fire_own(roster, empty, Trigger::OnBuy);
        self.fire_own(roster, empty, Trigger::Battlecry);
        self.fire_broadcast(roster, empty, Trigger::AfterFriendlyPlayed);
        Ok(())
    }

    /// Sell a Unit off the board, refunding gold and returning its copy to
    /// the pool. OnSell fires while it's still standing there -- there is
    /// nothing left for its own `Selector::This` to reach once it's gone.
    pub fn sell(&mut self, roster: &[UnitDef], board_slot: usize) -> Result<(), ShopError> {
        if self.board.get(board_slot).is_none() {
            return Err(ShopError::SlotEmpty);
        }
        self.fire_own(roster, board_slot, Trigger::OnSell);
        let unit = self.board.take(board_slot).ok_or(ShopError::SlotEmpty)?;
        self.board.compact();
        self.gold += SELL_REFUND;
        self.pool.return_one(&unit.def);
        Ok(())
    }

    /// Spend gold to raise the Tavern Tier, widening what the shop can offer.
    pub fn upgrade_tavern(&mut self) -> Result<(), ShopError> {
        if self.tavern_tier >= MAX_TAVERN_TIER {
            return Err(ShopError::MaxTavernTier);
        }
        let cost = tavern_upgrade_cost(self.tavern_tier);
        if self.gold < cost {
            return Err(ShopError::NotEnoughGold);
        }
        self.gold -= cost;
        self.tavern_tier += 1;
        Ok(())
    }

    /// A procedural opponent for this round -- "kept light" per
    /// `docs/DESIGN.md`: a handful of Units sampled at or below this run's own
    /// Tavern Tier, not itself a participant with a pool to draw down.
    pub fn matchmake(&self, roster: &[UnitDef]) -> Party {
        let eligible: Vec<&UnitDef> = roster
            .iter()
            .filter(|def| !def.token && def.tier <= self.tavern_tier)
            .collect();
        let count = (self.tavern_tier as usize + 1).min(SLOTS);
        let mut rng = self.rng(Domain::Matchmaking);
        let units: Vec<Unit> = (0..count)
            .filter_map(|_| rng.choose(&eligible).map(|def| Unit::new(def)))
            .collect();
        Party::from_units(units).expect("count is bounded by SLOTS above")
    }

    /// Fire EndOfTurn for the board being taken into the fight, then package
    /// it against a procedural opponent for `action_phase::resolve_with_roster`.
    pub fn end_turn(&mut self, roster: &[UnitDef]) -> Board {
        self.fire_all_own(roster, Trigger::EndOfTurn);
        Board::new(self.board.clone(), self.matchmake(roster))
    }

    /// Record a fight's result against the best-of-3 score. A Draw or
    /// Stalemate scores nobody -- rare enough (a Draw needs a mutual kill; a
    /// Stalemate needs `action_phase::MAX_BEATS`) that awarding either side a
    /// win for it would be a surprising rule with no rationale behind it.
    ///
    /// A fight's own changes to the party aren't permanent unless something
    /// specifically makes them so -- and nothing in this engine currently
    /// does, so the next round's board is rebuilt entirely from the party
    /// that *entered* the fight (`self.board`, untouched by it), not from
    /// the fight's own outcome: everyone who fought comes back fresh from
    /// its Definition, the same reset a newly bought Unit already gets,
    /// dead or not.
    pub fn apply_fight_result(&mut self, roster: &[UnitDef], outcome: Outcome) {
        match outcome {
            Outcome::PlayerWins => self.wins += 1,
            Outcome::OpposingWins => self.losses += 1,
            Outcome::Draw | Outcome::Stalemate => {}
        }

        let mut board = Party::new();
        for (i, unit) in self.board.iter() {
            if let Some(def) = def_by_id(roster, &unit.def) {
                board.put(i, Unit::new(def));
            }
        }
        self.board = board;
    }

    /// How the run stands, once it's decided.
    pub fn run_outcome(&self) -> Option<RunOutcome> {
        if self.wins >= WINS_TO_TAKE_RUN {
            Some(RunOutcome::PlayerWinsRun)
        } else if self.losses >= LOSSES_TO_END_RUN {
            Some(RunOutcome::PlayerLosesRun)
        } else {
            None
        }
    }

    /// Advance to the next round: more gold, a fresh (frozen-respecting) shop,
    /// and StartOfTurn for everything already on the board.
    pub fn start_new_round(&mut self, roster: &[UnitDef]) {
        self.round += 1;
        self.gold = gold_for_round(self.round);
        self.refresh_shop(roster);
        self.fire_all_own(roster, Trigger::StartOfTurn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(id: &str, tier: u8) -> UnitDef {
        UnitDef {
            id: DefId::new(id),
            name: id.to_owned(),
            tier,
            attack: tier as i32,
            health: tier as i32,
            tribes: vec![],
            all_tribes: false,
            keywords: vec![],
            abilities: vec![],
            token: false,
            text: String::new(),
        }
    }

    fn token_def(id: &str) -> UnitDef {
        UnitDef {
            token: true,
            ..def(id, 1)
        }
    }

    fn small_roster() -> Vec<UnitDef> {
        vec![def("a", 1), def("b", 1), def("c", 2), token_def("t")]
    }

    #[test]
    fn buying_fires_battlecry_not_just_onbuy() {
        use crate::units::{Ability, Effect, Selector};

        let mut caster = def("caster", 1);
        caster.abilities = vec![Ability {
            trigger: Trigger::Battlecry,
            condition: None,
            effects: vec![Effect::Buff {
                target: Selector::This,
                attack: 5,
                health: 0,
            }],
        }];
        let roster = vec![caster];
        let mut run = RunState::new(Seed(20), &roster);
        run.gold = 100;
        run.shop = vec![ShopSlot {
            def: DefId::new("caster"),
            frozen: false,
        }];
        run.buy(&roster, 0).unwrap();
        assert_eq!(
            run.board.get(0).unwrap().attack,
            6,
            "Battlecry fired the moment buying placed it on the board"
        );
    }

    #[test]
    fn a_fresh_pool_excludes_tokens() {
        let pool = Pool::new(&small_roster());
        assert_eq!(pool.available(&DefId::new("a")), copies_for_tier(1));
        assert_eq!(
            pool.available(&DefId::new("t")),
            0,
            "tokens never enter the pool"
        );
    }

    #[test]
    fn buying_removes_a_copy_and_selling_returns_it() {
        let roster = small_roster();
        let mut run = RunState::new(Seed(1), &roster);
        run.gold = 100;

        // The copy already left the pool the moment it was drawn into the
        // shop -- a shown offer is a reservation, not a free look -- so
        // buying it (converting that reservation into a permanent absence)
        // shouldn't change the pool count at all. Only selling should.
        let offer = run.shop[0].def.clone();
        let reserved = run.pool.available(&offer);
        run.buy(&roster, 0).unwrap();
        assert_eq!(
            run.pool.available(&offer),
            reserved,
            "buying an already-reserved offer doesn't touch the pool again"
        );
        assert_eq!(run.board.len(), 1);

        run.sell(&roster, 0).unwrap();
        assert_eq!(
            run.pool.available(&offer),
            reserved + 1,
            "selling returns the copy"
        );
        assert!(run.board.is_empty());
    }

    #[test]
    fn an_unfrozen_offer_returns_to_the_pool_on_reroll() {
        let roster = small_roster();
        let mut run = RunState::new(Seed(2), &roster);
        run.gold = 100;
        let total_before: u32 = roster
            .iter()
            .filter(|d| !d.token)
            .map(|d| copies_for_tier(d.tier))
            .sum();
        let outstanding = |run: &RunState| -> u32 {
            let in_pool: u32 = roster
                .iter()
                .filter(|d| !d.token)
                .map(|d| run.pool.available(&d.id))
                .sum();
            let in_shop = run.shop.len() as u32;
            in_pool + in_shop
        };
        assert_eq!(
            outstanding(&run),
            total_before,
            "every copy is either in the pool or on offer"
        );

        run.reroll(&roster).unwrap();
        assert_eq!(
            outstanding(&run),
            total_before,
            "a reroll returns unfrozen offers before drawing new ones -- nothing is created or lost"
        );
    }

    #[test]
    fn freezing_survives_a_reroll() {
        let roster = small_roster();
        let mut run = RunState::new(Seed(3), &roster);
        run.gold = 100;
        run.toggle_freeze();
        let frozen_shop = run.shop.clone();
        run.reroll(&roster).unwrap();
        assert_eq!(
            run.shop.iter().map(|s| &s.def).collect::<Vec<_>>(),
            frozen_shop.iter().map(|s| &s.def).collect::<Vec<_>>(),
            "the whole frozen shop survives in place"
        );
    }

    #[test]
    fn freezing_again_unfreezes_the_whole_shop() {
        let roster = small_roster();
        let mut run = RunState::new(Seed(12), &roster);
        run.gold = 100;
        run.toggle_freeze();
        assert!(run.shop.iter().all(|s| s.frozen));
        run.toggle_freeze();
        assert!(run.shop.iter().all(|s| !s.frozen));
    }

    #[test]
    fn buying_fails_without_enough_gold() {
        let roster = small_roster();
        let mut run = RunState::new(Seed(4), &roster);
        run.gold = 0;
        assert_eq!(run.buy(&roster, 0), Err(ShopError::NotEnoughGold));
    }

    #[test]
    fn buying_fails_on_a_full_board() {
        let roster = vec![def("only", 1)];
        let mut run = RunState::new(Seed(5), &roster);
        run.gold = 1000;
        for i in 0..SLOTS {
            run.shop = vec![ShopSlot {
                def: DefId::new("only"),
                frozen: false,
            }];
            run.buy(&roster, 0)
                .unwrap_or_else(|e| panic!("slot {i}: {e}"));
        }
        run.shop = vec![ShopSlot {
            def: DefId::new("only"),
            frozen: false,
        }];
        assert_eq!(run.buy(&roster, 0), Err(ShopError::BoardFull));
    }

    #[test]
    fn the_shop_never_offers_above_the_tavern_tier() {
        let roster = small_roster();
        let run = RunState::new(Seed(6), &roster);
        for slot in &run.shop {
            let tier = def_by_id(&roster, &slot.def).unwrap().tier;
            assert!(tier <= run.tavern_tier);
        }
    }

    #[test]
    fn best_of_three_ends_the_run() {
        let roster = small_roster();
        let mut run = RunState::new(Seed(7), &roster);
        assert_eq!(run.run_outcome(), None);
        run.apply_fight_result(&roster, Outcome::PlayerWins);
        assert_eq!(run.run_outcome(), None);
        run.apply_fight_result(&roster, Outcome::PlayerWins);
        assert_eq!(run.run_outcome(), Some(RunOutcome::PlayerWinsRun));
    }

    #[test]
    fn two_losses_ends_the_run_the_other_way() {
        let roster = small_roster();
        let mut run = RunState::new(Seed(8), &roster);
        run.apply_fight_result(&roster, Outcome::OpposingWins);
        run.apply_fight_result(&roster, Outcome::OpposingWins);
        assert_eq!(run.run_outcome(), Some(RunOutcome::PlayerLosesRun));
    }

    #[test]
    fn a_draw_or_stalemate_scores_neither_side() {
        let roster = small_roster();
        let mut run = RunState::new(Seed(9), &roster);
        run.apply_fight_result(&roster, Outcome::Draw);
        run.apply_fight_result(&roster, Outcome::Stalemate);
        assert_eq!((run.wins, run.losses), (0, 0));
    }

    #[test]
    fn a_fights_casualties_and_damage_are_not_permanent() {
        let roster = small_roster();
        let mut run = RunState::new(Seed(11), &roster);
        run.gold = 100;
        run.shop = vec![
            ShopSlot {
                def: DefId::new("a"),
                frozen: false,
            },
            ShopSlot {
                def: DefId::new("b"),
                frozen: false,
            },
        ];
        run.buy(&roster, 0).unwrap(); // "a" into slot 0
        run.buy(&roster, 0).unwrap(); // "b" into slot 1 ("a"'s offer is gone, "b" is now offer 0)

        // Nothing about the fight itself matters here -- only what the party
        // looked like going in. "a" damaged and "b" dead is exactly the kind
        // of change combat makes that isn't permanent unless something
        // specifies otherwise, and nothing here does.
        run.apply_fight_result(&roster, Outcome::PlayerWins);

        assert_eq!(
            run.board.len(),
            2,
            "a fight's casualty isn't permanent -- both return"
        );
        for (_, unit) in run.board.iter() {
            assert_eq!(
                unit.health, unit.max_health,
                "damage does not carry into the next round"
            );
        }
    }

    #[test]
    fn same_seed_same_round_is_deterministic() {
        let roster = small_roster();
        let a = RunState::new(Seed(42), &roster);
        let b = RunState::new(Seed(42), &roster);
        assert_eq!(a.shop, b.shop, "the same seed draws the same opening shop");
    }

    #[test]
    fn matchmaking_never_exceeds_board_slots() {
        let roster = small_roster();
        let mut run = RunState::new(Seed(10), &roster);
        run.tavern_tier = MAX_TAVERN_TIER;
        let opposing = run.matchmake(&roster);
        assert!(opposing.len() <= SLOTS);
    }
}
