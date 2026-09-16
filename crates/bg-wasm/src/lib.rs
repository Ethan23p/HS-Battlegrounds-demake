//! WASM bindings for `bg-sim`, so a browser calls the real engine instead of
//! reimplementing its rules a second time in JS.
//!
//! Every function takes and returns plain strings (JSON) rather than
//! `JsValue`, so the wire format is exactly what `bg-cli` already produces --
//! there is nothing WASM-specific for a caller to learn, and the same JSON
//! could as easily have come from the CLI.

use wasm_bindgen::prelude::*;

use bg_sim::action_phase::{Resolution, resolve as resolve_action_phase};
use bg_sim::fixtures::{shop_roster, showcase_board};
use bg_sim::party::Board;
use bg_sim::prep_phase::RunState;
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

// ---------------------------------------------------------------------------
// Prep Phase: a run's shop, told directly rather than left for the front end
// to re-derive -- every function here takes the current `RunState` and hands
// back the next one (or an error naming exactly why the action was refused),
// the same tell-don't-ask shape `resolve` above already uses for a fight.
// ---------------------------------------------------------------------------

/// The shop's roster (`bg_sim::fixtures::shop_roster`) as JSON, so a front end
/// can look up a shop offer's name/stats/keywords by `DefId` without the
/// engine needing to repeat that data inside every `RunState`.
#[wasm_bindgen]
pub fn shop_roster_json() -> String {
    serde_json::to_string(&shop_roster()).expect("a UnitDef list always serializes")
}

fn parse_run(run_json: &str) -> Result<RunState, JsError> {
    serde_json::from_str(run_json).map_err(|e| JsError::new(&format!("bad RunState: {e}")))
}

fn run_json(run: &RunState) -> Result<String, JsError> {
    serde_json::to_string(run)
        .map_err(|e| JsError::new(&format!("a RunState always serializes: {e}")))
}

/// Start a fresh run: Tavern Tier 1, an empty board, a full pool, the first
/// shop already drawn.
#[wasm_bindgen]
pub fn start_run(seed: u64) -> String {
    let run = RunState::new(Seed(seed), &shop_roster());
    serde_json::to_string(&run).expect("a RunState always serializes")
}

/// Buy shop offer `offer` (0-based) onto the first open board Slot.
#[wasm_bindgen]
pub fn shop_buy(run_json_in: &str, offer: usize) -> Result<String, JsError> {
    let mut run = parse_run(run_json_in)?;
    run.buy(&shop_roster(), offer)
        .map_err(|e| JsError::new(&e.to_string()))?;
    run_json(&run)
}

/// Sell the Unit in board Slot `slot` (0-based), refunding gold and returning
/// its copy to the pool.
#[wasm_bindgen]
pub fn shop_sell(run_json_in: &str, slot: usize) -> Result<String, JsError> {
    let mut run = parse_run(run_json_in)?;
    run.sell(slot).map_err(|e| JsError::new(&e.to_string()))?;
    run_json(&run)
}

/// Spend gold to clear and redraw every unfrozen offer.
#[wasm_bindgen]
pub fn shop_reroll(run_json_in: &str) -> Result<String, JsError> {
    let mut run = parse_run(run_json_in)?;
    run.reroll(&shop_roster())
        .map_err(|e| JsError::new(&e.to_string()))?;
    run_json(&run)
}

/// Toggle whether shop offer `offer` survives the next reroll.
#[wasm_bindgen]
pub fn shop_toggle_freeze(run_json_in: &str, offer: usize) -> Result<String, JsError> {
    let mut run = parse_run(run_json_in)?;
    run.toggle_freeze(offer)
        .map_err(|e| JsError::new(&e.to_string()))?;
    run_json(&run)
}

/// Spend gold to raise the Tavern Tier, widening what the shop can offer.
#[wasm_bindgen]
pub fn shop_upgrade_tavern(run_json_in: &str) -> Result<String, JsError> {
    let mut run = parse_run(run_json_in)?;
    run.upgrade_tavern()
        .map_err(|e| JsError::new(&e.to_string()))?;
    run_json(&run)
}

/// Package this round's Board (the player's board against a procedural
/// opponent) as JSON, ready for [`resolve`].
#[wasm_bindgen]
pub fn end_turn(run_json_in: &str) -> Result<String, JsError> {
    let run = parse_run(run_json_in)?;
    let board = run.end_turn(&shop_roster());
    serde_json::to_string(&board)
        .map_err(|e| JsError::new(&format!("a Board always serializes: {e}")))
}

/// Record a fight's `Resolution` against the run's best-of-3 score, and sync
/// the board with what the fight actually left standing -- damage and
/// fight-only state don't carry into the next round, but death does.
#[wasm_bindgen]
pub fn apply_fight_result(run_json_in: &str, resolution_json: &str) -> Result<String, JsError> {
    let mut run = parse_run(run_json_in)?;
    let resolution: Resolution = serde_json::from_str(resolution_json)
        .map_err(|e| JsError::new(&format!("bad Resolution: {e}")))?;
    run.apply_fight_result(
        &shop_roster(),
        resolution.outcome,
        &resolution.final_board.player,
    );
    run_json(&run)
}

/// Advance to the next round: more gold, a fresh (frozen-respecting) shop.
/// Refuses once the run is already decided -- check `wins`/`losses` on the
/// `RunState` first (two of either ends it).
#[wasm_bindgen]
pub fn start_new_round(run_json_in: &str) -> Result<String, JsError> {
    let mut run = parse_run(run_json_in)?;
    if run.run_outcome().is_some() {
        return Err(JsError::new("the run is already decided"));
    }
    run.start_new_round(&shop_roster());
    run_json(&run)
}
