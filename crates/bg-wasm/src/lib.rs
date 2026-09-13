//! WASM bindings for `bg-sim`, so a browser calls the real engine instead of
//! reimplementing its rules a second time in JS.
//!
//! Every function takes and returns plain strings (JSON) rather than
//! `JsValue`, so the wire format is exactly what `bg-cli` already produces --
//! there is nothing WASM-specific for a caller to learn, and the same JSON
//! could as easily have come from the CLI.

use wasm_bindgen::prelude::*;

use bg_sim::action_phase::resolve as resolve_action_phase;
use bg_sim::fixtures::showcase_board;
use bg_sim::party::Board;
use bg_sim::rng::{Domain, Seed};

/// The placeholder Board `bg-cli` also resolves, as JSON. Stands in for a real
/// shop/party builder -- see `bg_sim::fixtures`.
#[wasm_bindgen]
pub fn showcase_board_json() -> String {
    serde_json::to_string(&showcase_board()).expect("a Board always serializes")
}

/// Resolve one Action Phase and return the `Resolution` as JSON.
///
/// `board_json` is a `Board` (see `bg_sim::party::Board`), typically one a
/// caller got from [`showcase_board_json`] and then rearranged.
#[wasm_bindgen]
pub fn resolve(board_json: &str, seed: u64) -> Result<String, JsError> {
    let board: Board =
        serde_json::from_str(board_json).map_err(|e| JsError::new(&format!("bad Board: {e}")))?;
    let mut rng = Seed(seed).stream(Domain::Combat, 0);
    let resolution = resolve_action_phase(board, &mut rng);
    serde_json::to_string(&resolution)
        .map_err(|e| JsError::new(&format!("a Resolution always serializes: {e}")))
}
