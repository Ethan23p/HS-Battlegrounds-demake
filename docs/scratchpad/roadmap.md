# Front-end roadmap

Not canon — see `README.md`. Current as of 2026-09-13, from
[0007](../transcripts/0007-the-front-end-and-the-scratchpad.md).

## Where things stand

- `bg-sim`'s Action Phase is adapted to DESIGN.md: Board/Party/Slot/Unit/Beat/Intent,
  concurrent interactions within a beat, no ordering advantage between sides
  (`docs/DESIGN.md` Ongoing has both decisions).
- `Resolution` is `Serialize`/`Deserialize` and carries `initial_board`, so a
  `Resolution` is a self-contained replay.
- 0.1 is built and redesigned: `bg-cli` emits a `Resolution` as JSON/JS, `web/` plays
  it back as "Beat Ledger" (masthead + two ranks + event dispatch, IBM Plex Sans/Mono,
  full light/dark). Published as a Claude artifact so Ethan can open it on a phone with
  no file-manager/build-step failure modes — republish the same artifact path on every
  later iteration rather than standing up a second copy.
- Both the docs-restart work and this audit are on open PR
  [#11](https://github.com/Ethan23p/HS-Battlegrounds-demake/pull/11), not yet merged to
  `main`.
- 0.3's three open questions are resolved (below); 0.4's WASM step moved up into 0.2,
  since 0.2 is the point where the page first needs to *ask* the engine something
  instead of replaying a canned log.

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
window; outcome matches what `bg-cli` itself resolves. Redesigned once ("Beat
Ledger") and published as an artifact after the first cut only worked as a plain
`file://` path and broke when opened through a phone file manager.

Settled as engineering, no design input needed: pacing is a dropdown (slow/normal/
fast) rather than a fixed rate; units are colored cards with name/stats/keyword
badges, no art yet — revisit only if the 0.5+ polish pass wants real sprites.

**Debt, now being paid off in 0.2, not a design question:** `app.js` reconstructs
board state by replaying the log against rules read off `bg-sim`'s source (see the
comment at the top of the file) — there's no shared code between the two, so a
change to `bg-sim`'s Action Phase can silently desync the viewer.

### 0.2 — Touch a unit, and the engine moves into the browser
Two things land together, because the second is what the first actually needs:

- **`bg-sim` compiled to `wasm32`**, called directly from the page instead of
  shelling out to `bg-cli`. Motivated by 0.2 specifically, not deferred from 0.4:
  drag-to-reorder-then-refight needs the engine to answer a question the moment a
  player drags a unit, and the alternative (reimplement `Party::compact` a second
  time in JS, on top of what `app.js` already reimplements for replay) only makes
  the drift risk above worse. One copy of the rules, no rebuild step between "drag"
  and "fight."
- **Drag-to-reorder a party before a fight**, via Pointer Events (mouse/touch/pen in
  one path), snapping to the engine's own left-packing rather than a client-side
  guess at it. Still one-shot: arrange, then fight, no persistent run yet.

**Mechanical, not risky:** a thin `bg-wasm` crate wrapping `resolve()` for
`wasm-bindgen`; `bg-sim` already has zero I/O and full serde support, so it's a clean
target. `wasm32-unknown-unknown` installs cleanly in this environment (confirmed
2026-09-11); `wasm-bindgen`'s CLI does not ship in the base toolchain and needs
installing when this is built.

### 0.3 — A shop and a run
Buy/sell/reroll, gold, multiple rounds. First version that's a game rather than a
fight viewer. All three blockers from the previous draft are resolved:

- **Opponent source:** procedurally generated, kept light — not a hand-authored
  bestiary, not run history. Recorded in `docs/DESIGN.md` Ongoing ("The opponent pool
  is procedural, kept light"). What "kept light" means for the actual generation
  rule is still Claude's to design as engineering, informed by that quotation, not a
  second design question.
- **Economy:** a simple placeholder now — fixed gold-per-turn, flat reroll/buy/tier
  costs, just enough to make the loop testable. The real numbers get tuned once the
  loop exists and can be felt, not designed on paper first. This is a sequencing
  choice, not a design ruling on what the numbers should be, so it isn't in
  DESIGN.md.
- **Persistence:** none yet. A run is one sitting; closing the tab ends it. Also a
  sequencing choice, not a design fact about the eventual offline model.

### 0.4 — Data-driven content, for real
The WASM binding already exists (0.2), so this is narrower than it was: Units/
abilities move from `bg-cli`'s hardcoded fixture to RON assets the page loads at
runtime, so adding a Unit is a file edit, not a recompile.

**Open (design, only if it comes up before this point):** ability authoring beyond
keywords — Departure 4 explicitly deferred this to "a dedicated discussion."

### 0.5+ — Polish, VFX, juice
Deliberately last: juice on rules that might still change is wasted work.
