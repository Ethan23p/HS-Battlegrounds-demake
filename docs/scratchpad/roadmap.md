# Front-end roadmap

Not canon — see `README.md`. Current as of 2026-09-11, from
[0006](../transcripts/0006-the-concurrent-beat.md).

## Where things stand

- `bg-sim`'s Action Phase is adapted to DESIGN.md: Board/Party/Slot/Unit/Beat/Intent,
  concurrent interactions within a beat, no ordering advantage between sides
  (`docs/DESIGN.md` Ongoing has both decisions).
- `Resolution` is `Serialize`/`Deserialize` and carries `initial_board`, so a
  `Resolution` is a self-contained replay. Done in anticipation of 0.1 below.
- 0.1 is built: `bg-cli` emits a `Resolution` as JSON/JS, `web/` plays it back. See
  the 0.1 entry below.
- Both the docs-restart work and this audit are on open PR
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
`bg-cli` resolves a fixture fight and emits it as JSON (`bg` binary, `json` or `js`
format). `web/` is a static HTML/CSS/JS page, no build step: open `index.html`,
click Play, watch units strike, shields flash, units die and revive, parties
compact. Skip-to-end and a speed selector exist. Verified with Playwright
(screenshots + a DOM/console check) since this environment can't see a native
window; outcome matches what `bg-cli` itself resolves.

Settled as engineering, no design input needed: pacing is a dropdown (slow/normal/
fast) rather than a fixed rate; units are colored boxes with name/stats/keyword
badges, no art yet — revisit only if the 0.5+ polish pass wants real sprites.

**Known debt, not a design question:** `app.js` reconstructs board state by
replaying the log against rules read off `bg-sim`'s source (see the comment at the
top of the file) — there's no shared code between the two, so a change to
`bg-sim`'s Action Phase can silently desync the viewer. 0.4 (WASM) removes this by
letting the page call the real engine instead of reimplementing its rules.

### 0.2 — Touch a unit
Drag-to-reorder a party before a fight, via Pointer Events (mouse/touch/pen in one
path). Still one-shot: arrange, then fight, no persistent run.

**Open (engineering):** whether client-side drag preview re-implements left-packing or
stays a dumb ordered list and defers all packing rules to `bg-sim` — leaning toward the
latter, to avoid a second copy of `Party::compact`'s logic drifting from the real one.

### 0.3 — A shop and a run
Buy/sell/reroll, gold, multiple rounds, a persistent run. First version that's a game
rather than a fight viewer.

**Blocked on design, not engineering — needs Ethan:**
- Where the opponent comes from each round. DESIGN.md's Initial records that a
  random-draw-from-a-pool opponent was *mentioned*, not confirmed as current model.
- The economy: gold per turn, reroll/buy/tier-up costs. Departure 5 discarded the
  three-resource framing and left "internal physics" open.
- What persists between sessions (a run save), if anything, given offline
  single-player play.

### 0.4 — Data-driven content, for real
`bg-sim` compiled to `wasm32` (target installs cleanly; confirmed in-session), page
calls the real engine directly — no more shelling out to a binary. Units/abilities as
RON assets the page loads at runtime; editing a unit is a file edit, not a recompile.

**Open (design, only if it comes up before this point):** ability authoring beyond
keywords — Departure 4 explicitly deferred this to "a dedicated discussion."

### 0.5+ — Polish, VFX, juice
Deliberately last: juice on rules that might still change is wasted work.
