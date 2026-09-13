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
