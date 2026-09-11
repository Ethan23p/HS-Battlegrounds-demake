//! Fixture generator for the web front end.
//!
//! Resolves one Action Phase and writes it to stdout as either a JS assignment
//! (`const FIGHT = {...};`, loadable via a `<script>` tag with no server) or
//! plain JSON. `bg-sim` does all the resolving; this only picks a Board, calls
//! `resolve`, and serializes what comes back.
//!
//! There is no shop or party builder yet, so the Board is a fixture. See
//! `docs/scratchpad/roadmap.md` for what replaces it.

use std::env;
use std::process::ExitCode;

use bg_sim::action_phase::{board_of, resolve};
use bg_sim::party::{Board, Unit};
use bg_sim::rng::{Domain, Seed};
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

/// The fixture fight 0.1's viewer plays back.
fn fixture_board() -> Board {
    board_of(
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

fn main() -> ExitCode {
    let mut args = env::args().skip(1);

    let seed = match args.next() {
        Some(raw) => match raw.parse::<Seed>() {
            Ok(seed) => seed,
            Err(_) => {
                eprintln!(
                    "bg: '{raw}' is not a valid seed (16 hex digits, 0x-prefixed hex, or decimal)"
                );
                return ExitCode::FAILURE;
            }
        },
        None => Seed(1),
    };
    let format = args.next().unwrap_or_else(|| "js".to_owned());

    let mut rng = seed.stream(Domain::Combat, 0);
    let resolution = resolve(fixture_board(), &mut rng);
    let json = serde_json::to_string(&resolution).expect("a Resolution always serializes");

    match format.as_str() {
        "json" => println!("{json}"),
        "js" => println!("const FIGHT = {json};"),
        other => {
            eprintln!("bg: unknown format '{other}' (expected 'json' or 'js')");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
