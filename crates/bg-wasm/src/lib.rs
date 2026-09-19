//! WASM bindings for `bg-sim`, so a browser calls the real engine instead of
//! reimplementing its rules a second time in JS.
//!
//! Every function takes and returns plain strings (JSON) rather than
//! `JsValue`, so the wire format is exactly what `bg-cli` already produces --
//! there is nothing WASM-specific for a caller to learn, and the same JSON
//! could as easily have come from the CLI.
//!
//! Every Prep Phase function also takes `roster_json`: the shop's Units are
//! data (`assets/roster.ron`), not compiled in, so the browser is the one
//! holding it -- it fetches the RON once, hands it through [`parse_roster`]
//! to get JSON back, and passes that same JSON into everything else here.

use wasm_bindgen::prelude::*;

use bg_sim::action_phase::{Resolution, resolve_with_roster};
use bg_sim::fixtures::showcase_board;
use bg_sim::party::Board;
use bg_sim::prep_phase::RunState;
use bg_sim::rng::{Domain, Seed};
use bg_sim::units::{UnitDef, load_roster};

/// The placeholder Board `bg-cli` also resolves, as JSON. Stands in for a real
/// shop/party builder -- see `bg_sim::fixtures`.
#[wasm_bindgen]
pub fn showcase_board_json() -> String {
    serde_json::to_string(&showcase_board()).expect("a Board always serializes")
}

/// Parse a roster written in RON (`assets/roster.ron`) into the JSON every
/// other function here expects. The one place RON parsing happens -- a
/// browser only ever needs to `fetch` the text and hand it here once.
#[wasm_bindgen]
pub fn parse_roster(ron_text: &str) -> Result<String, JsError> {
    let roster = load_roster(ron_text).map_err(|e| JsError::new(&format!("bad roster: {e}")))?;
    serde_json::to_string(&roster)
        .map_err(|e| JsError::new(&format!("a UnitDef list always serializes: {e}")))
}

fn parse_roster_json(roster_json: &str) -> Result<Vec<UnitDef>, JsError> {
    serde_json::from_str(roster_json).map_err(|e| JsError::new(&format!("bad roster JSON: {e}")))
}

/// Resolve one Action Phase and return the `Resolution` as JSON.
///
/// `board_json` is a `Board` (see `bg_sim::party::Board`), typically one a
/// caller got from [`showcase_board_json`] and then rearranged.
#[wasm_bindgen]
pub fn resolve(roster_json: &str, board_json: &str, seed: u64) -> Result<String, JsError> {
    let roster = parse_roster_json(roster_json)?;
    let board: Board =
        serde_json::from_str(board_json).map_err(|e| JsError::new(&format!("bad Board: {e}")))?;
    let mut rng = Seed(seed).stream(Domain::Combat, 0);
    let resolution = resolve_with_roster(board, &roster, &mut rng);
    serde_json::to_string(&resolution)
        .map_err(|e| JsError::new(&format!("a Resolution always serializes: {e}")))
}

// ---------------------------------------------------------------------------
// Prep Phase: a run's shop, told directly rather than left for the front end
// to re-derive -- every function here takes the current `RunState` and hands
// back the next one (or an error naming exactly why the action was refused),
// the same tell-don't-ask shape `resolve` above already uses for a fight.
// ---------------------------------------------------------------------------

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
pub fn start_run(roster_json: &str, seed: u64) -> Result<String, JsError> {
    let roster = parse_roster_json(roster_json)?;
    let run = RunState::new(Seed(seed), &roster);
    run_json(&run)
}

/// Buy shop offer `offer` (0-based) into hand.
#[wasm_bindgen]
pub fn shop_buy(roster_json: &str, run_json_in: &str, offer: usize) -> Result<String, JsError> {
    let roster = parse_roster_json(roster_json)?;
    let mut run = parse_run(run_json_in)?;
    run.buy(&roster, offer)
        .map_err(|e| JsError::new(&e.to_string()))?;
    run_json(&run)
}

/// Play the hand Unit at `hand_index` (0-based) onto the first open board
/// Slot.
#[wasm_bindgen]
pub fn shop_play(
    roster_json: &str,
    run_json_in: &str,
    hand_index: usize,
) -> Result<String, JsError> {
    let roster = parse_roster_json(roster_json)?;
    let mut run = parse_run(run_json_in)?;
    run.play(&roster, hand_index)
        .map_err(|e| JsError::new(&e.to_string()))?;
    run_json(&run)
}

/// Sell the Unit in board Slot `slot` (0-based), refunding gold and returning
/// its copy to the pool.
#[wasm_bindgen]
pub fn shop_sell(roster_json: &str, run_json_in: &str, slot: usize) -> Result<String, JsError> {
    let roster = parse_roster_json(roster_json)?;
    let mut run = parse_run(run_json_in)?;
    run.sell(&roster, slot)
        .map_err(|e| JsError::new(&e.to_string()))?;
    run_json(&run)
}

/// Spend gold to clear and redraw every unfrozen offer.
#[wasm_bindgen]
pub fn shop_reroll(roster_json: &str, run_json_in: &str) -> Result<String, JsError> {
    let roster = parse_roster_json(roster_json)?;
    let mut run = parse_run(run_json_in)?;
    run.reroll(&roster)
        .map_err(|e| JsError::new(&e.to_string()))?;
    run_json(&run)
}

/// Toggle whether the whole shop survives the next reroll.
#[wasm_bindgen]
pub fn shop_toggle_freeze(run_json_in: &str) -> Result<String, JsError> {
    let mut run = parse_run(run_json_in)?;
    run.toggle_freeze();
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

/// Fire EndOfTurn and package this round's Board (the player's board against
/// a procedural opponent) as JSON, ready for [`resolve`]. Returns the
/// *updated* `RunState` alongside it -- EndOfTurn can change the board (a
/// Unit growing at the close of Prep), and that has to carry into next
/// round the same as everything else `RunState` remembers.
#[wasm_bindgen]
pub fn end_turn(roster_json: &str, run_json_in: &str) -> Result<String, JsError> {
    let roster = parse_roster_json(roster_json)?;
    let mut run = parse_run(run_json_in)?;
    let board = run.end_turn(&roster);
    #[derive(serde::Serialize)]
    struct EndTurnResult {
        run: RunState,
        board: Board,
    }
    serde_json::to_string(&EndTurnResult { run, board })
        .map_err(|e| JsError::new(&format!("an EndTurnResult always serializes: {e}")))
}

/// Record a fight's `Resolution` against the run's best-of-3 score. The
/// board is untouched by this -- a fight's own changes (damage, a casualty)
/// aren't permanent unless something specifies otherwise, and nothing does
/// yet, so the party that entered the fight is exactly the one the next
/// round starts from.
#[wasm_bindgen]
pub fn apply_fight_result(
    roster_json: &str,
    run_json_in: &str,
    resolution_json: &str,
) -> Result<String, JsError> {
    let roster = parse_roster_json(roster_json)?;
    let mut run = parse_run(run_json_in)?;
    let resolution: Resolution = serde_json::from_str(resolution_json)
        .map_err(|e| JsError::new(&format!("bad Resolution: {e}")))?;
    run.apply_fight_result(&roster, resolution.outcome);
    run_json(&run)
}

/// Advance to the next round: more gold, a fresh (frozen-respecting) shop.
/// Refuses once the run is already decided -- check `wins`/`losses` on the
/// `RunState` first (two of either ends it).
#[wasm_bindgen]
pub fn start_new_round(roster_json: &str, run_json_in: &str) -> Result<String, JsError> {
    let roster = parse_roster_json(roster_json)?;
    let mut run = parse_run(run_json_in)?;
    if run.run_outcome().is_some() {
        return Err(JsError::new("the run is already decided"));
    }
    run.start_new_round(&roster);
    run_json(&run)
}
