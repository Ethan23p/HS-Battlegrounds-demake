//! The Unit vocabulary: the types every data file is written in.
//!
//! Nothing here executes. These are the *nouns* -- what a unit is allowed to
//! say. The engine's job is to be the only thing that knows what they mean, so
//! that adding a Unit is a data change and adding a *kind* of Unit is the only
//! thing that touches Rust.
//!
//! The vocabulary is deliberately small. Every variant below costs the engine a
//! match arm somewhere, so a new one has to earn its place by expressing
//! something a combination of existing ones cannot.

use serde::{Deserialize, Serialize};

/// A stable, human-written identifier for a unit or hero, e.g. `"rat_pack"`.
///
/// Data files reference each other by this, never by index, so the files stay
/// diffable and reorderable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DefId(pub String);

impl DefId {
    pub fn new(s: impl Into<String>) -> Self {
        DefId(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DefId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for DefId {
    fn from(s: &str) -> Self {
        DefId(s.to_owned())
    }
}

/// Unit families. Tribe membership drives most "whenever you play a X" abilities.
///
/// The set lives in code rather than data because tribes are a closed
/// vocabulary that Shop odds, pool construction, and ability text all agree on;
/// a tribe that nothing names is not a tribe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Tribe {
    Beast,
    Demon,
    Dragon,
    Elemental,
    Mech,
    Murloc,
    Naga,
    Pirate,
    Quilboar,
    Undead,
}

impl Tribe {
    /// Every tribe, in a fixed order. Used for pool construction and display.
    pub const ALL: [Tribe; 10] = [
        Tribe::Beast,
        Tribe::Demon,
        Tribe::Dragon,
        Tribe::Elemental,
        Tribe::Mech,
        Tribe::Murloc,
        Tribe::Naga,
        Tribe::Pirate,
        Tribe::Quilboar,
        Tribe::Undead,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Tribe::Beast => "Beast",
            Tribe::Demon => "Demon",
            Tribe::Dragon => "Dragon",
            Tribe::Elemental => "Elemental",
            Tribe::Mech => "Mech",
            Tribe::Murloc => "Murloc",
            Tribe::Naga => "Naga",
            Tribe::Pirate => "Pirate",
            Tribe::Quilboar => "Quilboar",
            Tribe::Undead => "Undead",
        }
    }
}

/// Persistent properties a unit can hold.
///
/// The set is closed, and it is Battlegrounds' own: Taunt, Divine Shield,
/// Poisonous, Windfury, Reborn and Rally, each unchanged. Keywords are the one
/// place the demake deliberately departs from nothing.
///
/// Keywords are a set, not a list: granting Taunt twice is granting it once.
/// They are separate from [`Effect`]s because the Action Phase consults them directly
/// rather than running them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Keyword {
    /// While this Unit or another with Taunt stands in its Party, every attack
    /// against that Party must target a Taunt holder.
    Taunt,
    /// Absorbs the next damage dealt to this Unit entirely, then is spent.
    DivineShield,
    /// This Unit attacks a second time, in a Beat of its own directly after its
    /// first -- so the dead from the first attack are cleared in between.
    Windfury,
    /// Any damage it deals to a Unit kills that Unit, however much health it has.
    Poisonous,
    /// The first time it dies, it returns in its Slot with 1 health.
    Reborn,
    /// This Unit's abilities fire when it attacks -- [`Trigger::OnAttack`] made
    /// visible on the Unit, so a Player can read it off the board.
    Rally,
}

impl Keyword {
    pub fn name(self) -> &'static str {
        match self {
            Keyword::Taunt => "Taunt",
            Keyword::DivineShield => "Divine Shield",
            Keyword::Windfury => "Windfury",
            Keyword::Poisonous => "Poisonous",
            Keyword::Reborn => "Reborn",
            Keyword::Rally => "Rally",
        }
    }

    /// Single-letter badge for compact Party rendering. Rally takes `L` because
    /// Reborn already holds `R`.
    pub fn badge(self) -> char {
        match self {
            Keyword::Taunt => 'T',
            Keyword::DivineShield => 'D',
            Keyword::Windfury => 'W',
            Keyword::Poisonous => 'P',
            Keyword::Reborn => 'R',
            Keyword::Rally => 'L',
        }
    }
}

/// The moment at which an ability fires.
///
/// Every trigger names a point the engine already passes through, so adding one
/// means finding an existing point in the resolution loop -- not inventing a
/// new one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trigger {
    /// When played from hand into a Slot.
    Battlecry,
    /// When this Unit dies, resolved from the Slot it occupied.
    Deathrattle,
    /// Once, before the first attack of an Action Phase.
    StartOfActionPhase,
    /// When this Unit is declared as an attacker, before damage. A Unit carrying
    /// abilities on this trigger wears [`Keyword::Rally`].
    OnAttack,
    /// When this unit takes damage and survives it.
    OnSurviveDamage,
    /// When any other friendly Unit enters a Slot, in either phase.
    AfterFriendlySummon,
    /// When any other friendly unit dies.
    AfterFriendlyDeath,
    /// When another friendly unit is played from hand. The played unit is
    /// the ability's *subject*.
    AfterFriendlyPlayed,
    /// When this unit is sold.
    OnSell,
    /// When this unit is bought into hand.
    OnBuy,
    /// At the close of the Prep Phase, before the Action Phase.
    EndOfTurn,
    /// At the start of the Prep Phase.
    StartOfTurn,
}

impl Trigger {
    /// Whether this trigger fires during the Action Phase rather than the Prep Phase.
    ///
    /// The Action Phase and Prep Phase share the same ability list; this is how the engine
    /// keeps a `Battlecry` from firing when a Deathrattle summons a token
    /// mid-fight.
    pub fn fires_in_action_phase(self) -> bool {
        matches!(
            self,
            Trigger::Deathrattle
                | Trigger::StartOfActionPhase
                | Trigger::OnAttack
                | Trigger::OnSurviveDamage
                | Trigger::AfterFriendlySummon
                | Trigger::AfterFriendlyDeath
        )
    }
}

/// A gate on an ability, checked when its trigger fires.
///
/// Conditions describe the *subject* -- the other unit that caused the
/// trigger -- because that is the only thing a trigger cannot already express.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    /// The subject belongs to this tribe.
    SubjectHasTribe(Tribe),
    /// The subject holds this keyword.
    SubjectHasKeyword(Keyword),
    /// The Player's Tier is at least this high.
    TierAtLeast(u8),
    /// Every listed condition holds.
    All(Vec<Condition>),
    /// At least one listed condition holds.
    Any(Vec<Condition>),
    /// The listed condition does not hold.
    Not(Box<Condition>),
}

/// Which units an effect lands on.
///
/// Selectors resolve against a *context* -- the unit running the ability, the
/// subject that triggered it, and the two Parties on the Board. Anything a
/// selector needs beyond that would be a new concept, not a new selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selector {
    /// The unit whose ability this is.
    This,
    /// The unit that caused the trigger (the one summoned, played, or killed).
    Subject,
    /// The Units in the Slots immediately left and right of this one.
    Adjacent,
    /// Friendly units matching the filter, chosen at random.
    RandomFriendly {
        #[serde(default = "one")]
        count: u32,
        #[serde(default)]
        filter: Filter,
    },
    /// Every friendly unit matching the filter.
    AllFriendly {
        #[serde(default)]
        filter: Filter,
    },
    /// Enemy units chosen at random. Only meaningful in the Action Phase.
    RandomEnemy {
        #[serde(default = "one")]
        count: u32,
        #[serde(default)]
        filter: Filter,
    },
    /// Every enemy unit matching the filter. Only meaningful in the Action Phase.
    AllEnemy {
        #[serde(default)]
        filter: Filter,
    },
}

fn one() -> u32 {
    1
}

/// A predicate narrowing a [`Selector`]'s candidates.
///
/// Defaults to "any unit other than the one acting", which is what almost
/// every unit means, so most data files can omit it entirely.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    /// Restrict to a tribe. `None` matches every unit.
    #[serde(default)]
    pub tribe: Option<Tribe>,
    /// Restrict to holders of a keyword.
    #[serde(default)]
    pub keyword: Option<Keyword>,
    /// Whether the acting unit may select itself.
    #[serde(default)]
    pub include_self: bool,
}

impl Filter {
    /// A filter that matches anything, including the acting unit.
    pub fn any() -> Self {
        Filter {
            include_self: true,
            ..Filter::default()
        }
    }

    /// A filter restricted to one tribe.
    pub fn tribe(tribe: Tribe) -> Self {
        Filter {
            tribe: Some(tribe),
            ..Filter::default()
        }
    }
}

/// A single thing an ability does.
///
/// Effects are declarative: they name *what* changes, never *how* to find it.
/// Ordering within an ability is significant -- effects resolve in listed order,
/// each seeing the Party the previous one left behind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    /// Permanently change attack and health. Health increases raise the maximum
    /// and heal by the same amount; decreases can kill.
    Buff {
        target: Selector,
        #[serde(default)]
        attack: i32,
        #[serde(default)]
        health: i32,
    },
    /// Add a keyword. Idempotent.
    Grant { target: Selector, keyword: Keyword },
    /// Remove a keyword. Silently does nothing if absent.
    Revoke { target: Selector, keyword: Keyword },
    /// Deal damage, respecting Divine Shield and Poisonous.
    Damage { target: Selector, amount: i32 },
    /// Put new units onto the acting unit's side, immediately to its right.
    /// Capped by available Party space.
    Summon {
        unit: DefId,
        #[serde(default = "one")]
        count: u32,
        /// Stat and keyword changes applied to each summoned copy.
        #[serde(default)]
        modify: Vec<Modifier>,
    },
    /// Give the owning player gold. Prep Phase only; ignored in the Action Phase.
    GainGold(i32),
    /// Add a Unit to the Player's hand. Prep Phase only.
    AddToHand {
        unit: DefId,
        #[serde(default = "one")]
        count: u32,
    },
    /// Run the listed effects only when the condition holds.
    IfThen {
        condition: Condition,
        then: Vec<Effect>,
        #[serde(default)]
        otherwise: Vec<Effect>,
    },
    /// Run the listed effects `times` times over.
    Repeat { times: u32, effects: Vec<Effect> },
}

/// A stat or keyword adjustment applied to a unit as it is created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Modifier {
    Attack(i32),
    Health(i32),
    Grant(Keyword),
}

/// One trigger, one optional gate, one ordered list of effects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ability {
    pub trigger: Trigger,
    #[serde(default)]
    pub condition: Option<Condition>,
    pub effects: Vec<Effect>,
}

/// A unit as written in a data file.
///
/// This is the *definition*, shared and immutable. The mutable thing that sits
/// on a Party is a separate type -- see [`crate::Party::Unit`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitDef {
    pub id: DefId,
    pub name: String,
    /// Tier, 1..=6. Tokens use the Tier they thematically belong to.
    pub tier: u8,
    pub attack: i32,
    pub health: i32,
    #[serde(default)]
    pub tribes: Vec<Tribe>,
    /// Counts as every tribe at once (Amalgam-likes).
    #[serde(default)]
    pub all_tribes: bool,
    #[serde(default)]
    pub keywords: Vec<Keyword>,
    #[serde(default)]
    pub abilities: Vec<Ability>,
    /// Summoned only by other units; never enters the shop pool.
    #[serde(default)]
    pub token: bool,
    /// Rules text, for display only. The engine never reads it.
    #[serde(default)]
    pub text: String,
}

impl UnitDef {
    /// Whether this unit belongs to `tribe`, accounting for all-tribes units.
    pub fn has_tribe(&self, tribe: Tribe) -> bool {
        self.all_tribes || self.tribes.contains(&tribe)
    }

    /// Whether any ability uses this trigger.
    pub fn has_trigger(&self, trigger: Trigger) -> bool {
        self.abilities.iter().any(|a| a.trigger == trigger)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_keyword_has_its_own_badge() {
        let keywords = [
            Keyword::Taunt,
            Keyword::DivineShield,
            Keyword::Windfury,
            Keyword::Poisonous,
            Keyword::Reborn,
            Keyword::Rally,
        ];
        let badges: std::collections::BTreeSet<char> =
            keywords.into_iter().map(Keyword::badge).collect();
        assert_eq!(
            badges.len(),
            keywords.len(),
            "a badge has to name exactly one Keyword"
        );
    }

    #[test]
    fn filter_defaults_exclude_self() {
        assert!(!Filter::default().include_self);
        assert!(Filter::any().include_self);
    }

    #[test]
    fn all_tribes_matches_every_tribe() {
        let def = UnitDef {
            id: DefId::new("amalgam"),
            name: "Amalgam".into(),
            tier: 1,
            attack: 1,
            health: 1,
            tribes: vec![],
            all_tribes: true,
            keywords: vec![],
            abilities: vec![],
            token: true,
            text: String::new(),
        };
        assert!(Tribe::ALL.iter().all(|&t| def.has_tribe(t)));
    }

    #[test]
    fn prep_only_triggers_do_not_fire_in_action_phase() {
        assert!(!Trigger::Battlecry.fires_in_action_phase());
        assert!(!Trigger::OnSell.fires_in_action_phase());
        assert!(Trigger::Deathrattle.fires_in_action_phase());
        assert!(Trigger::StartOfActionPhase.fires_in_action_phase());
    }

    #[test]
    fn a_unit_parses_from_ron() {
        let src = r#"(
            id: "rat_pack",
            name: "Rat Pack",
            tier: 2,
            attack: 2,
            health: 2,
            tribes: [Beast],
            abilities: [(
                trigger: Deathrattle,
                effects: [Summon(unit: "rat", count: 2)],
            )],
            text: "Deathrattle: Summon two 1/1 Rats.",
        )"#;
        let def: UnitDef = ron::from_str(src).expect("parses");
        assert_eq!(def.name, "Rat Pack");
        assert!(def.has_tribe(Tribe::Beast));
        assert!(def.has_trigger(Trigger::Deathrattle));
        // Omitted fields fall back to their defaults.
        assert!(!def.token);
        assert!(def.keywords.is_empty());
    }

    #[test]
    fn nested_effects_parse_from_ron() {
        let src = r#"(
            trigger: AfterFriendlyPlayed,
            condition: Some(SubjectHasTribe(Murloc)),
            effects: [Buff(target: Subject, attack: 1, health: 1)],
        )"#;
        let ability: Ability = ron::from_str(src).expect("parses");
        assert_eq!(ability.trigger, Trigger::AfterFriendlyPlayed);
        assert_eq!(
            ability.condition,
            Some(Condition::SubjectHasTribe(Tribe::Murloc))
        );
    }
}
