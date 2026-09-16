# Front-end roadmap

Not canon — see `README.md`. Current as of 2026-09-15.

## Where things stand

- `bg-sim`'s Action Phase is adapted to DESIGN.md: Board/Party/Slot/Unit/Beat/Intent,
  concurrent interactions within a beat, no ordering advantage between sides
  (`docs/DESIGN.md` Ongoing has both decisions).
- `bg-sim` runs in the browser via `bg-wasm` (`wasm-bindgen`) — one copy of the rules,
  no rebuild step between changing a party and fighting it. `bg-cli` stays for a
  no-browser look at the same fixture.
- `Resolution` carries `boards`, a fully-resolved Board snapshot per Beat. The front end
  never reconstructs state from the event log — that's "tell, don't ask" (Ethan's name
  for it): the engine hands over complete, ordered fact, nothing downstream re-derives a
  bg-sim rule to interpret it. Holds for every feature built since.
- 0.1 and 0.2 are both done and merged into one working fight viewer, "Beat Ledger":
  resolve a fight, drag your party into order first, step/play/skip/reset through the
  result, single arrow + stacked damage numbers + a snap-and-recoil clash per exchange.
  Published as a Claude artifact so it's viewable on a phone with no build step —
  <https://claude.ai/artifact/DJeHAUvz8WVM419w3be221>, republished in place on every
  iteration rather than a new link each time.
- The rendering-technology question (hand-rolled DOM/CSS vs. an immediate-mode GUI like
  egui) was raised and settled by actually building and running both, not by guessing —
  see git history on `claude/frontend-0-1` and `claude/css-graphics-approach-tgzfh9`
  (closed, unmerged, kept as the record) if the comparison needs revisiting.
- The layout went through several real passes with Ethan directly — no header, one
  full-viewport stage, playfield anchored center-left, info floating top-right,
  proportional sizing with a letter-width floor rather than pixel constants, forced-
  landscape on mobile instead of a rotate prompt. Also settled: git history has the
  blow-by-blow (CSS specificity/cascade gotchas, the containment-vs-proportional
  distinction, a couple of real regressions caught by screenshot rather than assumed
  away) — worth a skim before touching `web/style.css` again, not reproduced here.
- All of this is on open PR
  [#11](https://github.com/Ethan23p/HS-Battlegrounds-demake/pull/11), not yet merged to
  `main`.

## Why web, and why this order

Claude can drive a browser (Playwright: click, drag, assert on the DOM, screenshot) but
has no way to see or interact with a native window in this environment. Ethan wants
touch-capable drag input, fluid VFX, and fast data-driven iteration, all of which a
DOM+CSS front end gets close to for free. So: web, hand-rolled, no game engine, and
staged so the parts needing no design input come first.

## Iterations

### 0.1 — See a fight happen — done
A fight resolves and plays back Beat by Beat: step forward/back, replay, skip to end,
speed control. No design input needed here — pacing and card styling were engineering
calls, not design ones.

### 0.2 — Touch a unit, and the engine moves into the browser — done
`bg-sim` compiled to `wasm32`; drag-to-reorder a party before fighting it, verified on
real mobile-device emulation (Pointer Events, one code path for mouse/touch/pen). Still
one-shot: arrange, then fight, no persistent run — that's 0.3.

### 0.3 — A shop and a run — done
Buy/sell/reroll, gold, multiple rounds. First version that's a game rather than a fight
viewer. Planned in chat first (abstract functions, then interfaces, then the tangible
surface, Ethan's suggested order) before any code, and two design questions were settled
along the way, both recorded in DESIGN.md Ongoing:

- **Pool conservation.** Every Unit the shop offers draws from, and returns to, a scarce
  shared pool -- Ethan's framing, "an informal discipline" of treating in-game objects as
  drawing from and returning to a finite resource, applied specifically to Units (not
  gold, which stays a plain abstract resource; not tokens, which never enter the pool by
  construction). A shown offer reserves its copy immediately; buying converts that
  reservation into a permanent absence; selling, or an unbought offer clearing on reroll
  or a new round, returns it. `bg-sim::prep_phase::Pool`.
- **Runs are best of 3**, not a health total -- the damage-on-loss formula depends on a
  broader meta-game that isn't decided yet, so best of 3 sidesteps needing one entirely.
  `apply_fight_result` just tallies wins/losses; `run_outcome` fires at two of either.

Landed as `bg-sim::prep_phase` (`RunState`, `Pool`, `ShopSlot`, `RunOutcome`; buy/sell/
reroll/freeze/upgrade-tavern/matchmake/end-turn/apply-fight-result/start-new-round),
exposed to the browser via `bg-wasm` in the same plain-JSON tell-don't-ask shape
`resolve()` already used, and a `web/` UI that repurposes the existing fight-viewer
layout rather than building a second one: the opposing row is the shop while it's on
screen (relabeled "Shop", swapped back to "Opposing party" the moment a fight starts),
the player row is the actual persistent board, carrying over round to round with the
same drag-to-reorder Pointer Events code 0.2 built, just re-targeted at
`RunState.board.slots`.

A real gap surfaced and got fixed during testing, not left standing: `apply_fight_result`
initially only tallied wins/losses and never synced the board with what the fight
actually did, so a Unit that died in combat would return next round at full health --
undermining the entire point of a persistent run. Fixed by rebuilding the board from
`Resolution::final_board`'s survivors, each restored fresh from its Definition (the
same reset a newly bought Unit already gets, since nothing in this engine yet grants a
Unit a permanent change beyond its Definition) -- damage and fight-only state (Divine
Shield spent, Poisonous marks) don't carry into the next round, but death does, matching
Battlegrounds itself. A second, smaller bug: the sell button sits inside a draggable
card, and a plain click on it also started the drag machinery, whose pointerup handler
re-rendered the card (replacing the button's own DOM node) before the browser's `click`
event could land on it -- sells silently never fired. Fixed by having the drag handler
ignore presses starting on the sell button.

Verified end-to-end: buy/sell/freeze/reroll/upgrade-tavern each surfaces the engine's own
refusal reason on failure (not enough gold, board full, ...); a frozen offer survives a
reroll; drag-to-reorder produces a real permutation (checked directly against
`RunState.board.slots`, since the shop roster's low tier-1 variety made two on-screen
units coincidentally share a name more often than not); the full loop (shop -> fight ->
continue -> next round, or run-over at two wins/losses) runs clean through multiple
rounds with no console errors; a Draw correctly scores neither side.

Still placeholder, deliberately: flat buy/sell/reroll/tavern-upgrade costs, a simple
gold-per-round ramp, a tapered-by-tier pool size, a 12-Unit hardcoded roster
(`fixtures::shop_roster`, two per Tavern Tier, keywords only -- no abilities execute yet).
Real numbers get tuned once the loop can be felt, not designed on paper first.

### 0.4 — Data-driven content, for real
The WASM binding already exists (0.2), so this is narrower than it once was: Units/
abilities move from `fixtures::shop_roster`'s hardcoded roster to RON assets the page
loads at runtime, so adding a Unit is a file edit, not a recompile.

**Open (design, only if it comes up before this point):** ability authoring beyond
keywords — Departure 4 explicitly deferred this to "a dedicated discussion." Abilities
exist as data (`units::Ability`/`Effect`/`Trigger`) but nothing executes them yet --
0.3's roster is keyword-only for exactly that reason.

### 0.5+ — Polish, VFX, juice
Deliberately last: juice on rules that might still change is wasted work. The rendering
pass finished before 0.3 (arrows, damage numbers, clash motion, proportional layout)
covers the fight viewer; the shop screen built in 0.3 is plain by comparison (click to
buy/sell/freeze, no animation) and will want its own pass once the loop itself is settled.
