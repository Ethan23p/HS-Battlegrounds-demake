//! The card vocabulary: the types every data file is written in.
//!
//! Nothing here executes. These are the *nouns* -- what a card is allowed to
//! say. The engine's job is to be the only thing that knows what they mean, so
//! that adding a card is a data change and adding a *kind* of card is the only
//! thing that touches Rust.
//!
//! The vocabulary is deliberately small. Every variant below costs the engine a
//! match arm somewhere, so a new one has to earn its place by expressing
//! something a combination of existing ones cannot.

use serde::{Deserialize, Serialize};

/// A stable, human-written identifier for a card or hero, e.g. `"rat_pack"`.
///
/// Data files reference each other by this, never by index, so the files stay
/// diffable and reorderable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CardId(pub String);

impl CardId {
    pub fn new(s: impl Into<String>) -> Self {
        CardId(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for CardId {
    fn from(s: &str) -> Self {
        CardId(s.to_owned())
    }
}

/// Minion families. Tribe membership drives most "whenever you play a X" cards.
///
/// The set lives in code rather than data because tribes are a closed
/// vocabulary that shop odds, pool construction, and card text all agree on;
/// a tribe that no card names is not a tribe.
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

/// Persistent properties a minion can hold.
///
/// Keywords are a set, not a list: granting Taunt twice is granting it once.
/// They are separate from [`Effect`]s because combat consults them directly
/// rather than running them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Keyword {
    /// Must be attacked before its non-taunt allies.
    Taunt,
    /// Absorbs the next instance of damage entirely.
    DivineShield,
    /// Attacks twice per turn.
    Windfury,
    /// Any damage it deals to a minion destroys that minion.
    Poisonous,
    /// The first time it dies, returns with 1 health.
    Reborn,
}

impl Keyword {
    pub fn name(self) -> &'static str {
        match self {
            Keyword::Taunt => "Taunt",
            Keyword::DivineShield => "Divine Shield",
            Keyword::Windfury => "Windfury",
            Keyword::Poisonous => "Poisonous",
            Keyword::Reborn => "Reborn",
        }
    }

    /// Single-letter badge for compact board rendering.
    pub fn badge(self) -> char {
        match self {
            Keyword::Taunt => 'T',
            Keyword::DivineShield => 'D',
            Keyword::Windfury => 'W',
            Keyword::Poisonous => 'P',
            Keyword::Reborn => 'R',
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
    /// When played from hand onto the board.
    Battlecry,
    /// When this minion dies, resolved from its position on the board.
    Deathrattle,
    /// Once, before the first attack of a combat.
    StartOfCombat,
    /// When this minion is declared as an attacker, before damage.
    OnAttack,
    /// When this minion takes damage and survives it.
    OnSurviveDamage,
    /// When any other friendly minion enters the board, in combat or tavern.
    AfterFriendlySummon,
    /// When any other friendly minion dies.
    AfterFriendlyDeath,
    /// When another friendly minion is played from hand. The played minion is
    /// the ability's *subject*.
    AfterFriendlyPlayed,
    /// When this minion is sold.
    OnSell,
    /// When this minion is bought into hand.
    OnBuy,
    /// At the close of the recruit phase, before combat.
    EndOfTurn,
    /// At the start of the recruit phase.
    StartOfTurn,
}

impl Trigger {
    /// Whether this trigger fires during combat rather than the tavern.
    ///
    /// Combat and tavern share the same ability list; this is how the engine
    /// keeps a `Battlecry` from firing when a Deathrattle summons a token
    /// mid-fight.
    pub fn fires_in_combat(self) -> bool {
        matches!(
            self,
            Trigger::Deathrattle
                | Trigger::StartOfCombat
                | Trigger::OnAttack
                | Trigger::OnSurviveDamage
                | Trigger::AfterFriendlySummon
                | Trigger::AfterFriendlyDeath
        )
    }
}

/// A gate on an ability, checked when its trigger fires.
///
/// Conditions describe the *subject* -- the other minion that caused the
/// trigger -- because that is the only thing a trigger cannot already express.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    /// The subject belongs to this tribe.
    SubjectHasTribe(Tribe),
    /// The subject holds this keyword.
    SubjectHasKeyword(Keyword),
    /// The owner's tavern tier is at least this high.
    TavernTierAtLeast(u8),
    /// Every listed condition holds.
    All(Vec<Condition>),
    /// At least one listed condition holds.
    Any(Vec<Condition>),
    /// The listed condition does not hold.
    Not(Box<Condition>),
}

/// Which minions an effect lands on.
///
/// Selectors resolve against a *context* -- the minion running the ability, the
/// subject that triggered it, and the two sides of the board. Anything a
/// selector needs beyond that would be a new concept, not a new selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selector {
    /// The minion whose ability this is.
    This,
    /// The minion that caused the trigger (the one summoned, played, or killed).
    Subject,
    /// The minions immediately left and right of this one.
    Adjacent,
    /// Friendly minions matching the filter, chosen at random.
    RandomFriendly {
        #[serde(default = "one")]
        count: u32,
        #[serde(default)]
        filter: Filter,
    },
    /// Every friendly minion matching the filter.
    AllFriendly {
        #[serde(default)]
        filter: Filter,
    },
    /// Enemy minions chosen at random. Only meaningful in combat.
    RandomEnemy {
        #[serde(default = "one")]
        count: u32,
        #[serde(default)]
        filter: Filter,
    },
    /// Every enemy minion matching the filter. Only meaningful in combat.
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
/// Defaults to "any minion other than the one acting", which is what almost
/// every card means, so most data files can omit it entirely.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    /// Restrict to a tribe. `None` matches every minion.
    #[serde(default)]
    pub tribe: Option<Tribe>,
    /// Restrict to holders of a keyword.
    #[serde(default)]
    pub keyword: Option<Keyword>,
    /// Whether the acting minion may select itself.
    #[serde(default)]
    pub include_self: bool,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            tribe: None,
            keyword: None,
            include_self: false,
        }
    }
}

impl Filter {
    /// A filter that matches anything, including the acting minion.
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
/// each seeing the board the previous one left behind.
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
    /// Put new minions onto the acting minion's side, immediately to its right.
    /// Capped by available board space.
    Summon {
        minion: CardId,
        #[serde(default = "one")]
        count: u32,
        /// Stat and keyword changes applied to each summoned copy.
        #[serde(default)]
        modify: Vec<Modifier>,
    },
    /// Give the owning player gold. Tavern only; ignored in combat.
    GainGold(i32),
    /// Add a card to the owning player's hand. Tavern only.
    AddToHand {
        minion: CardId,
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

/// A stat or keyword adjustment applied to a minion as it is created.
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

/// A minion as written in a data file.
///
/// This is the *definition*, shared and immutable. The mutable thing that sits
/// on a board is a separate type -- see [`crate::board::Minion`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinionDef {
    pub id: CardId,
    pub name: String,
    /// Tavern tier, 1..=6. Tokens use the tier they thematically belong to.
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
    /// Summoned only by other cards; never enters the shop pool.
    #[serde(default)]
    pub token: bool,
    /// Rules text, for display only. The engine never reads it.
    #[serde(default)]
    pub text: String,
}

impl MinionDef {
    /// Whether this minion belongs to `tribe`, accounting for all-tribes cards.
    pub fn has_tribe(&self, tribe: Tribe) -> bool {
        self.all_tribes || self.tribes.contains(&tribe)
    }

    /// Whether any ability uses this trigger.
    pub fn has_trigger(&self, trigger: Trigger) -> bool {
        self.abilities.iter().any(|a| a.trigger == trigger)
    }
}

/// A hero's once-per-turn activated power.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeroPower {
    pub name: String,
    /// Gold cost to activate.
    #[serde(default)]
    pub cost: i32,
    /// Activations allowed per recruit phase.
    #[serde(default = "one")]
    pub uses_per_turn: u32,
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub text: String,
}

/// A hero as written in a data file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeroDef {
    pub id: CardId,
    pub name: String,
    pub health: i32,
    #[serde(default)]
    pub armor: i32,
    #[serde(default)]
    pub power: Option<HeroPower>,
    #[serde(default)]
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_defaults_exclude_self() {
        assert!(!Filter::default().include_self);
        assert!(Filter::any().include_self);
    }

    #[test]
    fn all_tribes_matches_every_tribe() {
        let def = MinionDef {
            id: CardId::new("amalgam"),
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
    fn tavern_only_triggers_do_not_fire_in_combat() {
        assert!(!Trigger::Battlecry.fires_in_combat());
        assert!(!Trigger::OnSell.fires_in_combat());
        assert!(Trigger::Deathrattle.fires_in_combat());
        assert!(Trigger::StartOfCombat.fires_in_combat());
    }

    #[test]
    fn a_minion_parses_from_ron() {
        let src = r#"(
            id: "rat_pack",
            name: "Rat Pack",
            tier: 2,
            attack: 2,
            health: 2,
            tribes: [Beast],
            abilities: [(
                trigger: Deathrattle,
                effects: [Summon(minion: "rat", count: 2)],
            )],
            text: "Deathrattle: Summon two 1/1 Rats.",
        )"#;
        let def: MinionDef = ron::from_str(src).expect("parses");
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
