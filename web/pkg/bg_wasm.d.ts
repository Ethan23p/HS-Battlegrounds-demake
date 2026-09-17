/* tslint:disable */
/* eslint-disable */

/**
 * Record a fight's `Resolution` against the run's best-of-3 score, and sync
 * the board with what the fight actually left standing -- damage and
 * fight-only state don't carry into the next round, but death does.
 */
export function apply_fight_result(roster_json: string, run_json_in: string, resolution_json: string): string;

/**
 * Fire EndOfTurn and package this round's Board (the player's board against
 * a procedural opponent) as JSON, ready for [`resolve`]. Returns the
 * *updated* `RunState` alongside it -- EndOfTurn can change the board (a
 * Unit growing at the close of Prep), and that has to carry into next
 * round the same as everything else `RunState` remembers.
 */
export function end_turn(roster_json: string, run_json_in: string): string;

/**
 * Parse a roster written in RON (`assets/roster.ron`) into the JSON every
 * other function here expects. The one place RON parsing happens -- a
 * browser only ever needs to `fetch` the text and hand it here once.
 */
export function parse_roster(ron_text: string): string;

/**
 * Resolve one Action Phase and return the `Resolution` as JSON.
 *
 * `board_json` is a `Board` (see `bg_sim::party::Board`), typically one a
 * caller got from [`showcase_board_json`] and then rearranged.
 */
export function resolve(roster_json: string, board_json: string, seed: bigint): string;

/**
 * Buy shop offer `offer` (0-based) onto the first open board Slot.
 */
export function shop_buy(roster_json: string, run_json_in: string, offer: number): string;

/**
 * Spend gold to clear and redraw every unfrozen offer.
 */
export function shop_reroll(roster_json: string, run_json_in: string): string;

/**
 * Sell the Unit in board Slot `slot` (0-based), refunding gold and returning
 * its copy to the pool.
 */
export function shop_sell(roster_json: string, run_json_in: string, slot: number): string;

/**
 * Toggle whether the whole shop survives the next reroll.
 */
export function shop_toggle_freeze(run_json_in: string): string;

/**
 * Spend gold to raise the Tavern Tier, widening what the shop can offer.
 */
export function shop_upgrade_tavern(run_json_in: string): string;

/**
 * The placeholder Board `bg-cli` also resolves, as JSON. Stands in for a real
 * shop/party builder -- see `bg_sim::fixtures`.
 */
export function showcase_board_json(): string;

/**
 * Advance to the next round: more gold, a fresh (frozen-respecting) shop.
 * Refuses once the run is already decided -- check `wins`/`losses` on the
 * `RunState` first (two of either ends it).
 */
export function start_new_round(roster_json: string, run_json_in: string): string;

/**
 * Start a fresh run: Tavern Tier 1, an empty board, a full pool, the first
 * shop already drawn.
 */
export function start_run(roster_json: string, seed: bigint): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly apply_fight_result: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly end_turn: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly parse_roster: (a: number, b: number) => [number, number, number, number];
    readonly resolve: (a: number, b: number, c: number, d: number, e: bigint) => [number, number, number, number];
    readonly shop_buy: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly shop_reroll: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly shop_sell: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly shop_toggle_freeze: (a: number, b: number) => [number, number, number, number];
    readonly shop_upgrade_tavern: (a: number, b: number) => [number, number, number, number];
    readonly showcase_board_json: () => [number, number];
    readonly start_new_round: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly start_run: (a: number, b: number, c: bigint) => [number, number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
