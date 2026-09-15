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

### 0.3 — A shop and a run — next
Buy/sell/reroll, gold, multiple rounds. First version that's a game rather than a fight
viewer. All three blockers from the original draft are already resolved:

- **Opponent source:** procedurally generated, kept light — not a hand-authored
  bestiary, not run history (`docs/DESIGN.md` Ongoing). What "kept light" means for the
  actual generation rule is still Claude's to design as engineering.
- **Economy:** a simple placeholder to start — fixed gold-per-turn, flat reroll/buy/tier
  costs, just enough to make the loop testable. Real numbers get tuned once the loop
  exists and can be felt, not designed on paper first. Sequencing choice, not a design
  ruling, so it isn't in DESIGN.md.
- **Persistence:** none yet. A run is one sitting; closing the tab ends it. Also
  sequencing, not a design fact about the eventual offline model.

This is the resume point for "recenter on gameplay."

### 0.4 — Data-driven content, for real
The WASM binding already exists (0.2), so this is narrower than it once was: Units/
abilities move from `bg-cli`'s hardcoded fixture to RON assets the page loads at
runtime, so adding a Unit is a file edit, not a recompile.

**Open (design, only if it comes up before this point):** ability authoring beyond
keywords — Departure 4 explicitly deferred this to "a dedicated discussion."

### 0.5+ — Polish, VFX, juice
Deliberately last: juice on rules that might still change is wasted work. The rendering
pass just finished (arrows, damage numbers, clash motion, proportional layout) covers the
fight viewer; a shop screen (0.3) will want its own pass once it exists to look at.
