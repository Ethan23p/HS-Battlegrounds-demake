//! Print a resolved Action Phase, so a human can read what the rules did.
use bg_sim::action_phase::{board_of, resolve};
use bg_sim::party::Unit;
use bg_sim::units::{DefId, Keyword, UnitDef};

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

fn main() {
    let r = resolve(board_of(
        vec![u("bait", 1, 1, &[]), u("gusty", 3, 6, &[Keyword::Windfury])],
        vec![
            u("shielded", 2, 4, &[Keyword::DivineShield]),
            u("phoenix", 1, 2, &[Keyword::Reborn]),
        ],
    ));
    println!("{}", r.narrate());
    println!("\ndamage to opposing player: {}", r.damage_to_opposing);
    println!("damage to player: {}", r.damage_to_player);
}
