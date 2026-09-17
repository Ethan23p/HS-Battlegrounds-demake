//! A small hardcoded Board, shared by `bg-cli` and `bg-wasm`'s
//! `showcase_board_json` so neither hardcodes its own copy. Unrelated to the
//! real shop roster, which now lives in `assets/roster.ron` and loads through
//! `units::load_roster` -- this one is just a fixed three-a-side fight for a
//! no-shop demo of the Action Phase alone.

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
