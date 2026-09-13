//! Fixture generator for the web front end.
//!
//! Resolves one Action Phase and writes it to stdout as either a JS assignment
//! (`const FIGHT = {...};`, loadable via a `<script>` tag with no server) or
//! plain JSON. `bg-sim` does all the resolving; this only calls `resolve` on
//! `bg_sim::fixtures::showcase_board` and serializes what comes back.
//!
//! `bg-wasm` resolves the same fixture from inside the browser -- see
//! `docs/scratchpad/roadmap.md` for why both exist rather than one replacing
//! the other outright.

use std::env;
use std::process::ExitCode;

use bg_sim::action_phase::resolve;
use bg_sim::fixtures::showcase_board;
use bg_sim::rng::{Domain, Seed};

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
    let resolution = resolve(showcase_board(), &mut rng);
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
