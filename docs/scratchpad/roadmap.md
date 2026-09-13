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

**Debt, not a design question:** `app.js` reconstructs board state *per animation
step* by replaying the log against rules read off `bg-sim`'s source (see the comment
at the top of the file) — there's no shared code between the two for that part, so a
change to `bg-sim`'s Action Phase can silently desync the viewer's frame-by-frame
replay even after 0.2. WASM (below) removes the duplication in *deciding* the fight
(the page calls the real `resolve`, not a reimplementation of it) but not in
*animating* one step at a time from its log, which is a narrower and more durable
problem: the log states changes, not states, so *something* has to know the rules to
step through it visually. Only worth solving further if it actually bites.

### 0.2 — Touch a unit, and the engine moves into the browser
Two things land together, because the second is what the first actually needs.

**`bg-sim` compiled to `wasm32` — done.** A thin `bg-wasm` crate (`showcase_board_json`,
`resolve`) wraps the engine for `wasm-bindgen`; `app.js` now calls it directly instead
of loading `bg-cli`'s generated output, verified to resolve the identical fixture fight
(`PlayerWins`, 7 beats) as `bg-cli` itself. Motivated by 0.2 specifically, not deferred
from 0.4: drag-to-reorder-then-refight needs the engine to answer a question the moment
a player drags a unit, and reimplementing `Party::compact` a second time in JS on top of
what `app.js` already reimplements for replay would only compound the drift risk. One
copy of the rules now, no rebuild step between "drag" and "fight." `bg-cli` stays for a
no-browser look at the same fixture. Mechanical notes for next time: `wasm-bindgen`'s
CLI must match the crate version exactly (`cargo tree -p bg-wasm -i wasm-bindgen`,
`cargo install wasm-bindgen-cli --version <that>`) and doesn't ship in the base
toolchain; the seed argument is a JS `BigInt` (`1n`), not a `Number`, since it's a Rust
`u64`; and `app.js` being a module now means `file://` no longer works at all for local
testing — `python3 -m http.server` in `web/` (documented in `web/README.md`).

**Drag-to-reorder a party before a fight — not yet started.** Via Pointer Events
(mouse/touch/pen in one path), snapping to the engine's own left-packing rather than a
client-side guess at it. Still one-shot: arrange, then fight, no persistent run.

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
