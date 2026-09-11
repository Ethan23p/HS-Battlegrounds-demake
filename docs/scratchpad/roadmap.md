# Front-end roadmap

Not canon — see `README.md`. Current as of 2026-09-11, from
[0006](../transcripts/0006-the-concurrent-beat.md).

## Where things stand

- `bg-sim`'s Action Phase is adapted to DESIGN.md: Board/Party/Slot/Unit/Beat/Intent,
  concurrent interactions within a beat, no ordering advantage between sides
  (`docs/DESIGN.md` Ongoing has both decisions).
- `Resolution` is `Serialize`/`Deserialize` and carries `initial_board`, so a
  `Resolution` is a self-contained replay. Done in anticipation of 0.1 below.
- No front end exists yet. `bg-cli`'s `main` is a stub; `bg-sim/examples/watch.rs`
  prints a narrated log to a terminal, nothing more.
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

### 0.1 — See a fight happen
A Rust binary dumps a `Resolution` to JSON. A static HTML/CSS/JS page (no build step)
plays the event log back with sliding units, damage numbers, shield flashes, death
fades. Trigger: a button. No player input during the fight.

**Open (engineering, not design):** pacing/speed control; placeholder unit appearance
(colored box + name/stats) vs. planning for real art now.

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
