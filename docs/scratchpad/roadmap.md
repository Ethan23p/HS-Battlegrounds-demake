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

## Principle: tell, don't ask, across the engine/front-end boundary

Ethan's framing, from the `boards`-snapshot fix below. The engine is the source of
truth for state and the event log; it computes both fully (this is instant — a
synchronous WASM call, nothing to stream) and hands the complete, ordered, immutable
result over as data. The front end never asks the engine "what happens next" and the
engine never pushes on its own clock — pacing is entirely the front end's presentation
choice, decoupled from how or when the engine produced the facts. Concretely: nothing
downstream of `resolve()` should ever need to reimplement a bg-sim rule to interpret
its output. If a future feature makes a consumer infer state from events again, that's
the signal to add another told fact, not another inference. Worth holding to as 0.2's
drag-to-reorder and later features get built.

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

**Playback intentionality pass — done.** Ethan: it only played through once until a
refresh, and asked for replayability, a step-back-and-forth switcher, and damage
numbers that read as part of the Beat rather than flashing and vanishing; also asked
Claude to design the rest itself rather than just those two asks. Landed together
since both hinge on the same idea: everything shown belongs to *the Beat currently
revealed*, not a timer — see `Player.revealStep(index)` in `app.js`, the single place
that renders a position, used by stepping, replaying, and skipping alike.

Added, watching as a spectator rather than just implementing the two literal asks:
attacker→target arrows (colored by side, dashed when absorbed, offset apart when a
blow and its answer share the same two cards — the exact "trading blows" case that
motivated the earlier narration fix); the struck stat itself flashes so the floating
number and the card read as one event; a distinct pulsing "critical" state for a Unit
sitting at 0 health awaiting burial (Departure 2's death-timing was otherwise
invisible — the card just read 0 and looked normal until it vanished a Beat later); a
shimmer on a rank when it closes ranks (left-anchoring had no visual of its own
either). Two real bugs found and fixed while building it, not left standing: `Skip to
end` jumping straight to the closing step without ever revealing the last Beat left
the *previous* overlay on screen instead of the finishing blow (now computed from
`steps`/`index` alone, path-independent); and an SVG with no explicit size clips to a
300×150 default regardless of its CSS box, silently truncating the second arrow of
almost every pair.

**Debt — resolved.** `Resolution` now carries `boards`, a Board snapshot already fully
resolved for every Beat (bury, shield-break, compaction, every Struck/StruckBack/
Reborn applied). `app.js` no longer reconstructs state from the log at all — narration
events are read only for animation timing (which slot to flash, what number to pop),
never for what a Unit's resulting stats or keywords are. This is the "tell, don't ask"
fix Ethan named for it: bg-sim tells the resulting state directly instead of leaving
every consumer to infer it by re-deriving the engine's own rules from a stream of
deltas.

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

**Drag-to-reorder a party before a fight — done. 0.2 is complete.** A `Prep` screen
(new default state; the fight viewer is now entered via a "Fight" button, and a
"Rearrange" button returns to it, keeping the current arrangement) with the player's
row draggable via Pointer Events -- one code path for mouse, touch and pen -- verified
against a real mobile-device emulation (iPhone 13), not just a desktop mouse. Still
one-shot: arrange, then fight, no persistent run.

The roadmap's open question about client-side packing logic turned out not to apply:
a reorder only *permutes* the Units already there, it never opens or closes a gap, so
there's nothing to compact and nothing of `Party::compact`'s rule to defer to bg-sim
or reimplement -- `Prep.boardJson()` is a plain array splice, packed by construction.
A fresh seed each fight (`Date.now()`), so re-fighting the same arrangement doesn't
replay identically -- `Reset` on the fight itself still replays that one exactly, only
"Fight" from Prep draws a new one.

Three real, non-obvious CSS/JS bugs found and fixed along the way, not left standing:
a grid item's implicit `min-width: auto` (its content's own min-content size) beat the
`minmax()` floor meant to constrain it -- fixed at every level of the flex chain
between the scrollable row and the page edge, not just the row itself; `[hidden]`
loses to any later same-specificity `display` rule regardless of matching, since
author styles always beat the UA stylesheet at equal specificity; and a media query
placed earlier in the file than the base rules it was meant to override always lost,
matching or not, since source order still decides equal-specificity ties. The mobile
layout itself was redone rather than patched once this surfaced: eight cards with real
content (name, stats, badges) can't be shrunk to fit a phone and stay legible, so the
row now scrolls horizontally at a fixed, always-legible card size instead of
compressing columns to fit -- the standard answer for more content than fits on
touch, and it stopped an entire class of "make it 1px narrower" chase.

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
