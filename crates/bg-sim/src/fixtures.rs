//! A placeholder Board, shared by every front end that needs one to resolve
//! against before a real shop or party builder exists.
//!
//! Delete this module once 0.4 (see `docs/scratchpad/roadmap.md`) replaces it
//! with Units loaded from real data files -- it exists only so `bg-cli` and
//! `bg-wasm` don't each hardcode their own copy of the same three-a-side
//! fight in the meantime.

use crate::party::{Board, Unit};
use crate::units::{DefId, Keyword, UnitDef};

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

/// Three Units a side, chosen to exercise Windfury, Divine Shield, Reborn and
/// Taunt in one fight.
pub fn showcase_board() -> Board {
    crate::action_phase::board_of(
        vec![
            u("bait", 1, 1, &[]),
            u("gusty", 3, 6, &[Keyword::Windfury]),
            u("brute", 4, 9, &[]),
        ],
        vec![
            u("shielded", 2, 4, &[Keyword::DivineShield]),
            u("phoenix", 1, 2, &[Keyword::Reborn]),
            u("wall", 1, 12, &[Keyword::Taunt]),
        ],
    )
}

fn def(id: &str, tier: u8, attack: i32, health: i32, keywords: &[Keyword]) -> UnitDef {
    UnitDef {
        id: DefId::new(id),
        name: id.to_owned(),
        tier,
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

/// The shop's roster: two Units per Tavern Tier, 1 through 6. No abilities --
/// the Action Phase doesn't execute any yet, so a shop Unit whose only
/// interesting thing was an ability would be a dead card. Keywords are enough
/// to exercise `prep_phase::Pool`'s tiering and the fight viewer alike.
/// Placeholder stats, like the shop's costs -- delete alongside the rest of
/// this module once 0.4 moves Units into real data files.
pub fn shop_roster() -> Vec<UnitDef> {
    vec![
        def("scrapling", 1, 1, 1, &[]),
        def("sentry_pup", 1, 2, 2, &[Keyword::Taunt]),
        def("rimewing", 2, 2, 3, &[Keyword::Windfury]),
        def("shellback", 2, 1, 4, &[Keyword::DivineShield]),
        def("marsh_reaper", 3, 4, 3, &[Keyword::Poisonous]),
        def("emberkin", 3, 3, 5, &[Keyword::Taunt]),
        def("ashfall_roc", 4, 4, 6, &[Keyword::Reborn]),
        def("stormcaller", 4, 5, 5, &[Keyword::Windfury]),
        def(
            "voidwarden",
            5,
            6,
            7,
            &[Keyword::Taunt, Keyword::DivineShield],
        ),
        def("nightbrand", 5, 7, 5, &[Keyword::Poisonous]),
        def("colossus_wyrm", 6, 8, 8, &[Keyword::DivineShield]),
        def("apex_tyrant", 6, 9, 9, &[Keyword::Taunt]),
    ]
}
