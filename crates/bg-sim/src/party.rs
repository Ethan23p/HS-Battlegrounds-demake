//! Parties, Slots, and the Units standing in them.
//!
//! A [`UnitDef`] is immutable data shared by every Unit made from it; a [`Unit`]
//! is one mutable instance standing in one Slot. Keeping them apart is what lets
//! two copies of the same Definition take damage independently.
//!
//! **A Party has no gaps.** It is an ordered run of Units anchored on its
//! left-most, and a Slot is simply where a Unit stands in that run -- so "Slot 3
//! is empty" is not a state this type can be in. Nothing has to be compacted,
//! because nothing is ever uncompacted. What the Action Phase's clock walks is
//! not Slot numbers but the Units' own [`Unit::ready`] flag, so a death shifting
//! everyone left cannot make the clock skip a Unit or visit one twice
//! (ADR 0010).

use std::collections::BTreeSet;

use crate::units::{DefId, Keyword, Tribe, UnitDef};

/// Slots per Party. Eight, and the number is deliberate: a Party is built to
/// fill it, so every Slot left unfilled is a decision.
pub const SLOTS: usize = 8;

/// Which Party a Unit belongs to.
///
/// There is one Player; the opposing Party is data rather than a participant
/// (ADR 0006). `Side` distinguishes the two Parties on the Board, not two
/// players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
/// recomputation order to get wrong. [`Unit::ready`] follows the same principle:
/// whether this Unit still owes the clock a turn is written on the Unit, not
/// inferred from a cursor kept somewhere else.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Marked when a lethal effect (Poisonous) has struck, independently of
    /// health. Cleared only by the Unit leaving play.
    pub doomed: bool,
    /// Whether this Unit has yet to act since the Board last came Ready.
    ///
    /// This is the whole of the Action Phase's turn order. Each Beat, the
    /// left-most Ready Unit of each Party acts and stops being Ready; when
    /// neither Party has a Ready Unit left, every Unit becomes Ready again --
    /// the moment Ethan named *before the first Unit acts* (ADR 0010).
    pub ready: bool,
}

impl Unit {
    /// Instantiate a Unit from its Definition at full health, owing a turn.
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
            doomed: false,
            ready: true,
        }
    }

    pub fn has(&self, keyword: Keyword) -> bool {
        self.keywords.contains(&keyword)
    }

    pub fn has_tribe(&self, tribe: Tribe) -> bool {
        self.all_tribes || self.tribes.contains(&tribe)
    }

    /// Whether this Unit should be removed at the end of the current Beat.
    pub fn is_dead(&self) -> bool {
        self.doomed || self.health <= 0
    }

    /// How many times this Unit acts within a single Beat.
    ///
    /// Windfury is exactly this: whatever a Unit would do once in a Beat, it
    /// does twice.
    pub fn actions_per_beat(&self) -> u32 {
        if self.has(Keyword::Windfury) { 2 } else { 1 }
    }
}

/// The Units a Player brings, in the Slots they occupy.
///
/// **Invariant, always:** a run of Units with no interior gap, at most [`SLOTS`]
/// long. A death closes up behind it the instant it is applied. The clock is not
/// disturbed by that, because the clock reads [`Unit::ready`] rather than
/// counting Slots (ADR 0010).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Party {
    units: Vec<Unit>,
}

impl Party {
    /// An empty Party.
    pub fn new() -> Self {
        Party::default()
    }

    /// Build a Party. Units beyond [`SLOTS`] are refused.
    pub fn from_units(units: Vec<Unit>) -> Result<Self, PartyFull> {
        if units.len() > SLOTS {
            return Err(PartyFull {
                attempted: units.len(),
            });
        }
        Ok(Party { units })
    }

    pub fn get(&self, slot: usize) -> Option<&Unit> {
        self.units.get(slot)
    }

    pub fn get_mut(&mut self, slot: usize) -> Option<&mut Unit> {
        self.units.get_mut(slot)
    }

    /// Remove whatever occupies a Slot. Everything to its right closes up.
    pub fn take(&mut self, slot: usize) -> Option<Unit> {
        (slot < self.units.len()).then(|| self.units.remove(slot))
    }

    /// Put a Unit into a Slot, pushing whatever stood there rightward. A `slot`
    /// past the end appends. Refused if the Party is already full.
    pub fn insert(&mut self, slot: usize, unit: Unit) -> Result<(), PartyFull> {
        if self.units.len() >= SLOTS {
            return Err(PartyFull {
                attempted: self.units.len() + 1,
            });
        }
        self.units.insert(slot.min(self.units.len()), unit);
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }

    /// Every Unit, left to right, as `(slot, unit)`.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &Unit)> {
        self.units.iter().enumerate()
    }

    /// The Slots either side of `slot`. Adjacency is Slot arithmetic and always
    /// has been -- with no gaps to skip over, it needs no qualification.
    pub fn neighbours(&self, slot: usize) -> Vec<usize> {
        let mut out = Vec::with_capacity(2);
        if slot > 0 && slot <= self.units.len() {
            out.push(slot - 1);
        }
        if slot + 1 < self.units.len() {
            out.push(slot + 1);
        }
        out
    }

    /// The left-most Unit that still owes the clock a turn, if any.
    pub fn first_ready(&self) -> Option<usize> {
        self.units.iter().position(|u| u.ready)
    }

    /// Whether any Unit here still owes the clock a turn.
    pub fn has_ready(&self) -> bool {
        self.units.iter().any(|u| u.ready)
    }

    /// Every Unit owes a turn again. The Board's only clock boundary.
    pub fn ready_all(&mut self) {
        for unit in &mut self.units {
            unit.ready = true;
        }
    }

    /// Remove every dead Unit, returning what a Reborn holder spends its Reborn
    /// to come back as, at the Slot it now stands in.
    ///
    /// One sweep, so the survivors close up exactly once and every Slot number
    /// this returns is already the post-sweep one.
    pub fn sweep_dead(&mut self) -> Vec<(usize, String)> {
        let mut kept: Vec<Unit> = Vec::with_capacity(self.units.len());
        let mut returned = Vec::new();
        for unit in std::mem::take(&mut self.units) {
            if !unit.is_dead() {
                kept.push(unit);
                continue;
            }
            if unit.has(Keyword::Reborn) && !unit.reborn_spent {
                let mut back = unit;
                back.reborn_spent = true;
                back.keywords.remove(&Keyword::Reborn);
                back.health = 1;
                back.doomed = false;
                returned.push((kept.len(), back.name.clone()));
                kept.push(back);
            }
        }
        self.units = kept;
        returned
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

/// The two Parties contesting an Action Phase. Targeting is random (ADR 0008), so
/// Slots do not face one another.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

    /// Whether anybody anywhere still owes the clock a turn.
    pub fn has_ready(&self) -> bool {
        self.player.has_ready() || self.opposing.has_ready()
    }

    /// Everybody owes a turn again.
    pub fn ready_all(&mut self) {
        self.player.ready_all();
        self.opposing.ready_all();
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
    fn a_new_unit_starts_at_full_health_and_owing_a_turn() {
        let u = unit("wisp", 1, 3);
        assert_eq!(u.health, 3);
        assert_eq!(u.max_health, 3);
        assert!(!u.is_dead());
        assert!(u.ready);
    }

    #[test]
    fn windfury_doubles_actions_per_beat() {
        assert_eq!(unit("plain", 1, 1).actions_per_beat(), 1);
        let wf = Unit::new(&def("gusty", 1, 1, &[Keyword::Windfury]));
        assert_eq!(wf.actions_per_beat(), 2);
    }

    #[test]
    fn a_doomed_unit_is_dead_regardless_of_health() {
        let mut u = unit("stout", 1, 9);
        assert!(!u.is_dead());
        u.doomed = true;
        assert!(u.is_dead(), "Poisonous kills independently of health");
    }

    #[test]
    fn a_party_occupies_the_slots_from_one_upward() {
        let p = Party::from_units(vec![unit("a", 1, 1), unit("b", 1, 1)]).unwrap();
        assert_eq!(p.len(), 2);
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
    fn taking_a_unit_closes_the_gap_at_once() {
        // There is no such thing as an empty interior Slot to close later.
        let mut p =
            Party::from_units(vec![unit("a", 1, 1), unit("b", 1, 1), unit("c", 1, 1)]).unwrap();
        assert_eq!(p.take(1).unwrap().name, "b");
        assert_eq!(p.len(), 2);
        assert_eq!(p.get(0).unwrap().name, "a");
        assert_eq!(
            p.get(1).unwrap().name,
            "c",
            "order survives, position does not"
        );
        assert!(p.get(2).is_none());
    }

    #[test]
    fn taking_past_the_end_takes_nothing() {
        let mut p = Party::from_units(vec![unit("a", 1, 1)]).unwrap();
        assert!(p.take(3).is_none());
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn inserting_pushes_the_rest_rightward() {
        let mut p = Party::from_units(vec![unit("a", 1, 1), unit("c", 1, 1)]).unwrap();
        p.insert(1, unit("b", 1, 1)).unwrap();
        let names: Vec<&str> = p.iter().map(|(_, u)| u.name.as_str()).collect();
        assert_eq!(names, ["a", "b", "c"]);
    }

    #[test]
    fn a_full_party_refuses_another_unit() {
        let full: Vec<Unit> = (0..SLOTS).map(|_| unit("x", 1, 1)).collect();
        let mut p = Party::from_units(full).unwrap();
        assert_eq!(
            p.insert(0, unit("late", 1, 1)).unwrap_err(),
            PartyFull {
                attempted: SLOTS + 1
            }
        );
    }

    #[test]
    fn neighbours_are_the_slots_either_side() {
        let mut p =
            Party::from_units(vec![unit("a", 1, 1), unit("b", 1, 1), unit("c", 1, 1)]).unwrap();
        assert_eq!(p.neighbours(1), vec![0, 2]);
        assert_eq!(p.neighbours(0), vec![1], "no Slot left of the first");
        assert_eq!(p.neighbours(2), vec![1], "no Slot right of the last");
        p.take(0);
        assert_eq!(p.neighbours(1), vec![0], "the survivors closed up");
    }

    #[test]
    fn readiness_is_walked_left_to_right_and_refreshed_together() {
        let mut p =
            Party::from_units(vec![unit("a", 1, 1), unit("b", 1, 1), unit("c", 1, 1)]).unwrap();
        assert_eq!(p.first_ready(), Some(0));
        p.get_mut(0).unwrap().ready = false;
        assert_eq!(p.first_ready(), Some(1));

        // A death ahead of the clock must not cost "b" its turn: readiness rides
        // on the Unit, so closing up cannot skip it.
        p.take(0);
        assert_eq!(p.first_ready(), Some(0));
        assert_eq!(p.get(0).unwrap().name, "b");

        p.get_mut(0).unwrap().ready = false;
        p.get_mut(1).unwrap().ready = false;
        assert!(!p.has_ready());
        p.ready_all();
        assert_eq!(p.first_ready(), Some(0));
    }

    #[test]
    fn sweeping_removes_the_dead_and_brings_reborn_back() {
        let mut p = Party::from_units(vec![
            unit("a", 1, 1),
            Unit::new(&def("phoenix", 1, 1, &[Keyword::Reborn])),
            unit("c", 1, 1),
        ])
        .unwrap();
        p.get_mut(0).unwrap().health = 0;
        p.get_mut(1).unwrap().health = 0;

        let returned = p.sweep_dead();
        assert_eq!(returned, vec![(0, "phoenix".to_owned())], "at its new Slot");
        let names: Vec<&str> = p.iter().map(|(_, u)| u.name.as_str()).collect();
        assert_eq!(names, ["phoenix", "c"]);
        assert_eq!(p.get(0).unwrap().health, 1);
        assert!(p.get(0).unwrap().reborn_spent);
        assert!(!p.get(0).unwrap().has(Keyword::Reborn));
    }

    #[test]
    fn reborn_is_spent_only_once() {
        let mut p =
            Party::from_units(vec![Unit::new(&def("phoenix", 1, 1, &[Keyword::Reborn]))]).unwrap();
        p.get_mut(0).unwrap().health = 0;
        assert_eq!(p.sweep_dead().len(), 1);
        p.get_mut(0).unwrap().health = 0;
        assert!(p.sweep_dead().is_empty());
        assert!(p.is_empty());
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
