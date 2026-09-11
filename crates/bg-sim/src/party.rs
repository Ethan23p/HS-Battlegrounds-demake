//! Parties, Slots, and the Units standing in them.
//!
//! A [`UnitDef`] is immutable data shared by every Unit made from it; a [`Unit`]
//! is one mutable instance standing in one Slot. Keeping them apart is what lets
//! two copies of the same Definition take damage independently.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::units::{DefId, Keyword, Tribe, UnitDef};

/// Slots per Party. Eight, and the number is deliberate: a Party is built to
/// fill it, so every Slot left empty is a decision.
pub const SLOTS: usize = 8;

/// Which Party a Unit belongs to.
///
/// Play is offline and single-player: there is one Player, and the opposing
/// Party is data rather than a participant. `Side` distinguishes the two Parties
/// on the Board, not two players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    Player,
    Opposing,
}

impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Player => Side::Opposing,
            Side::Opposing => Side::Player,
        }
    }

    /// How to refer to this side's Party.
    pub fn name(self) -> &'static str {
        match self {
            Side::Player => "player",
            Side::Opposing => "opposing",
        }
    }

    /// How to refer to the person behind this side, as the object of a sentence.
    pub fn player_name(self) -> &'static str {
        match self {
            Side::Player => "you",
            Side::Opposing => "the opposing player",
        }
    }
}

/// One Unit standing in one Slot.
///
/// Stats are current values, not modifiers over the Definition: a Unit that has
/// been buffed and then damaged has no memory of how it got where it is. That is
/// deliberate -- it keeps a Unit's state readable on its own, and there is no
/// recomputation order to get wrong.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    /// The Definition this was made from. Abilities are looked up through it.
    pub def: DefId,
    pub name: String,
    pub attack: i32,
    pub health: i32,
    pub max_health: i32,
    /// A set, not a list: granting Taunt twice grants it once. `BTreeSet` rather
    /// than `HashSet` because iteration order reaches the log, and the log has to
    /// be identical across runs.
    pub keywords: BTreeSet<Keyword>,
    pub tribes: Vec<Tribe>,
    pub all_tribes: bool,
    /// Set once Reborn has been spent, so it cannot fire twice.
    pub reborn_spent: bool,
    /// Marked when a Poisonous blow has landed, which is lethal independently of
    /// health. Cleared only by the Unit leaving play.
    pub poisoned: bool,
    /// Set once this Unit's Divine Shield has absorbed something.
    ///
    /// The shield goes on absorbing for the rest of the Beat and breaks at the
    /// top of the next -- the same shape as a Unit that runs out in one Beat and
    /// dies at the top of the next. A Beat is one slice of time, so two blows
    /// arriving in it arrive together; which one the engine reaches first is an
    /// artefact of walking the Board, and nothing may turn on it.
    pub shield_spent: bool,
    /// Intents this Unit still holds: how many more times it means to act before
    /// the Board renews them.
    ///
    /// This is where "whose Beat is it" lives. Because it rides on the Unit
    /// rather than in a cursor over Slots, a Party can close ranks the moment a
    /// hole opens -- a Unit carries its Intents with it when it slides.
    pub intents: u32,
}

impl Unit {
    /// Instantiate a Unit from its Definition at full health.
    pub fn new(def: &UnitDef) -> Self {
        Unit {
            def: def.id.clone(),
            name: def.name.clone(),
            attack: def.attack,
            health: def.health,
            max_health: def.health,
            keywords: def.keywords.iter().copied().collect(),
            tribes: def.tribes.clone(),
            all_tribes: def.all_tribes,
            reborn_spent: false,
            poisoned: false,
            shield_spent: false,
            intents: 0,
        }
    }

    pub fn has(&self, keyword: Keyword) -> bool {
        self.keywords.contains(&keyword)
    }

    pub fn has_tribe(&self, tribe: Tribe) -> bool {
        self.all_tribes || self.tribes.contains(&tribe)
    }

    /// Whether this Unit has run out -- 0 health, or struck by Poisonous.
    ///
    /// Dying is not dead. A Unit that runs out in one Beat stands there for the
    /// rest of it and dies at the top of the next one. Only the Action Phase
    /// clears it.
    pub fn is_dying(&self) -> bool {
        self.poisoned || self.health <= 0
    }

    /// Whether this Unit still means to act.
    pub fn holds_intent(&self) -> bool {
        self.intents > 0
    }

    /// How many Intents this Unit takes when the Board renews them. Windfury is
    /// exactly this: a Unit that means to act twice where others mean to act
    /// once.
    pub fn intents_renewed(&self) -> u32 {
        if self.has(Keyword::Windfury) { 2 } else { 1 }
    }

    /// Give this Unit its Intents back.
    pub fn renew_intents(&mut self) {
        self.intents = self.intents_renewed();
    }

    /// Spend one Intent. Acting is spending it, whether or not the act lands.
    pub fn spend_intent(&mut self) {
        self.intents = self.intents.saturating_sub(1);
    }
}

/// The Units one side brings to the Board, in the Slots they occupy.
///
/// **Invariant:** Units are packed to the left with no interior gaps -- board
/// logic anchors on the left-most position. Taking a Unit out opens a hole, and
/// [`Party::compact`] closes it; the Action Phase does both in the same Beat.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Party {
    slots: [Option<Unit>; SLOTS],
}

impl Party {
    /// An empty Party.
    pub fn new() -> Self {
        Party::default()
    }

    /// Build a left-packed Party. Units beyond [`SLOTS`] are refused.
    pub fn from_units(units: Vec<Unit>) -> Result<Self, PartyFull> {
        if units.len() > SLOTS {
            return Err(PartyFull {
                attempted: units.len(),
            });
        }
        let mut party = Party::new();
        for (i, unit) in units.into_iter().enumerate() {
            party.slots[i] = Some(unit);
        }
        Ok(party)
    }

    pub fn get(&self, slot: usize) -> Option<&Unit> {
        self.slots.get(slot)?.as_ref()
    }

    pub fn get_mut(&mut self, slot: usize) -> Option<&mut Unit> {
        self.slots.get_mut(slot)?.as_mut()
    }

    /// Remove whatever occupies a Slot, leaving a hole.
    pub fn take(&mut self, slot: usize) -> Option<Unit> {
        self.slots.get_mut(slot)?.take()
    }

    /// Place a Unit in a Slot, returning whatever it displaced.
    pub fn put(&mut self, slot: usize, unit: Unit) -> Option<Unit> {
        self.slots[slot].replace(unit)
    }

    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(Option::is_none)
    }

    pub fn len(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// Occupied Slots, in Slot order, as `(slot, unit)`.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &Unit)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|u| (i, u)))
    }

    /// The Slots immediately left and right of `slot` that hold a Unit.
    pub fn neighbours(&self, slot: usize) -> Vec<usize> {
        let mut out = Vec::with_capacity(2);
        if slot > 0 && self.get(slot - 1).is_some() {
            out.push(slot - 1);
        }
        if slot + 1 < SLOTS && self.get(slot + 1).is_some() {
            out.push(slot + 1);
        }
        out
    }

    /// Close interior gaps, preserving order. Restores the left-packed invariant.
    pub fn compact(&mut self) {
        let mut write = 0;
        for read in 0..SLOTS {
            if self.slots[read].is_some() {
                if read != write {
                    self.slots[write] = self.slots[read].take();
                }
                write += 1;
            }
        }
    }

    /// Whether the left-packed invariant currently holds.
    pub fn is_packed(&self) -> bool {
        let mut seen_gap = false;
        for slot in &self.slots {
            match slot {
                Some(_) if seen_gap => return false,
                None => seen_gap = true,
                _ => {}
            }
        }
        true
    }
}

/// Returned when more Units are offered than a Party has Slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartyFull {
    pub attempted: usize,
}

impl std::fmt::Display for PartyFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Units offered for {SLOTS} Slots", self.attempted)
    }
}

impl std::error::Error for PartyFull {}

/// The two Parties contesting an Action Phase. Targeting is random, so Slots do
/// not face one another.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Board {
    pub player: Party,
    pub opposing: Party,
}

impl Board {
    pub fn new(player: Party, opposing: Party) -> Self {
        Board { player, opposing }
    }

    pub fn side(&self, side: Side) -> &Party {
        match side {
            Side::Player => &self.player,
            Side::Opposing => &self.opposing,
        }
    }

    pub fn side_mut(&mut self, side: Side) -> &mut Party {
        match side {
            Side::Player => &mut self.player,
            Side::Opposing => &mut self.opposing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(name: &str, attack: i32, health: i32, keywords: &[Keyword]) -> UnitDef {
        UnitDef {
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
        }
    }

    fn unit(name: &str, attack: i32, health: i32) -> Unit {
        Unit::new(&def(name, attack, health, &[]))
    }

    #[test]
    fn a_new_unit_starts_at_full_health() {
        let u = unit("wisp", 1, 3);
        assert_eq!(u.health, 3);
        assert_eq!(u.max_health, 3);
        assert!(!u.is_dying());
    }

    #[test]
    fn a_poisoned_unit_is_dying_whatever_its_health() {
        let mut u = unit("stout", 1, 9);
        assert!(!u.is_dying());
        u.poisoned = true;
        assert!(u.is_dying(), "Poisonous kills independently of health");
    }

    #[test]
    fn a_party_is_built_left_packed() {
        let p = Party::from_units(vec![unit("a", 1, 1), unit("b", 1, 1)]).unwrap();
        assert_eq!(p.len(), 2);
        assert!(p.is_packed());
        assert_eq!(p.get(0).unwrap().name, "a");
        assert_eq!(p.get(1).unwrap().name, "b");
        assert!(p.get(2).is_none());
    }

    #[test]
    fn a_party_refuses_more_units_than_slots() {
        let too_many: Vec<Unit> = (0..SLOTS + 1).map(|_| unit("x", 1, 1)).collect();
        assert_eq!(
            Party::from_units(too_many).unwrap_err(),
            PartyFull {
                attempted: SLOTS + 1
            }
        );
    }

    #[test]
    fn taking_a_unit_leaves_a_hole_that_compaction_closes() {
        let mut p =
            Party::from_units(vec![unit("a", 1, 1), unit("b", 1, 1), unit("c", 1, 1)]).unwrap();
        p.take(1);
        assert!(!p.is_packed(), "taking a Unit opens a hole");
        assert!(
            p.get(2).is_some(),
            "and leaves the Units past it where they were"
        );

        p.compact();
        assert!(p.is_packed());
        assert_eq!(p.get(0).unwrap().name, "a");
        assert_eq!(p.get(1).unwrap().name, "c", "order survives compaction");
        assert!(p.get(2).is_none());
    }

    #[test]
    fn compaction_of_a_packed_party_changes_nothing() {
        let mut p = Party::from_units(vec![unit("a", 1, 1), unit("b", 1, 1)]).unwrap();
        let before = p.clone();
        p.compact();
        assert_eq!(p, before);
    }

    #[test]
    fn neighbours_are_the_occupied_slots_either_side() {
        let mut p =
            Party::from_units(vec![unit("a", 1, 1), unit("b", 1, 1), unit("c", 1, 1)]).unwrap();
        assert_eq!(p.neighbours(1), vec![0, 2]);
        assert_eq!(p.neighbours(0), vec![1], "no Slot left of 0");
        p.take(0);
        assert_eq!(p.neighbours(1), vec![2], "an empty Slot is not a neighbour");
    }

    #[test]
    fn neighbours_at_the_last_slot_do_not_run_off_the_end() {
        let full: Vec<Unit> = (0..SLOTS).map(|_| unit("x", 1, 1)).collect();
        let p = Party::from_units(full).unwrap();
        assert_eq!(p.neighbours(SLOTS - 1), vec![SLOTS - 2]);
    }

    #[test]
    fn an_empty_party_is_empty() {
        assert!(Party::new().is_empty());
        assert!(!Party::from_units(vec![unit("a", 1, 1)]).unwrap().is_empty());
    }

    #[test]
    fn sides_are_opposites() {
        assert_eq!(Side::Player.other(), Side::Opposing);
        assert_eq!(Side::Opposing.other(), Side::Player);
    }
}
