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

**Checked against a hand-rolled immediate-mode GUI (egui), not just assumed — closed.**
A parallel session built a hand-painted `bg-gui` prototype (real `bg-sim` types, drawn
via `egui::Painter` rect/circle/text calls, `claude/css-graphics-approach-tgzfh9`,
evaluation only, not merged). Built and ran it rather than judging from the diff: cold
build ~56s; with no display attached at all it hard-crashes before reaching the app;
under `xvfb-run` it still crashed, missing a system library (`libxkbcommon-x11.so`) the
container didn't have — needed a root `apt-get install` to get a single screenshot out of
its headless hook. Confirms the original call — even the screenshot path that exists
specifically for a no-display sandbox depends on host libraries outside Claude's control,
and the hook itself is check-only: one static PNG per rebuild, no DOM-equivalent way to
query state or drive interaction, against Playwright's click/drag/computed-style/
screenshot loop that's driven every real bug fix this iteration. Local worktree removed;
`claude/css-graphics-approach-tgzfh9` kept on origin, unmerged, as the record.

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

**Landscape-first, proportional layout — done.** Ethan's own web-dev mental model, applied:
anchor to the screen, reason in fractions of it ("a card is X% of screen height, N fit end
to end"), not in pixels negotiated against a container. Three changes, all in `web/`:

- `.unit`'s size is now `clamp(64px, 11vh, 108px)` tall with a fixed `aspect-ratio`, and
  `flex: 0 0 auto` in `.row` -- nothing grows or shrinks it against a sibling or ancestor,
  so the `min-width: auto` renegotiation that caused three bugs last iteration has nothing
  left to negotiate. Verified: at 1000x560 a card resolves to exactly 48x64
  (`11vh` of 560, floor-clamped), matching the formula, not a guess.
- Portrait phones get an explicit "rotate to landscape" prompt (`.rotate-prompt`,
  `@media (max-width: 760px) and (orientation: portrait)`) instead of a squeezed layout --
  landscape is the one supported orientation now, matching real Battlegrounds. Verified a
  real landscape-phone viewport (844x390) fits all 8 slots with no scroll.
- `.row`'s old `overflow-x: auto` scroll-safety-net is gone. It's no longer needed (the
  above makes a full row fit by construction) and it turned out actively harmful: the CSS
  overflow spec forces `overflow-y` to `auto` the instant `overflow-x` isn't `visible`, on
  the *same element*, even when `overflow-y: visible` is set explicitly -- there is no
  combination of the two that scrolls one axis while leaving the other genuinely
  unclipped. This silently clipped the reworked damage-label anchor below before the
  `overflow-x` line was removed; caught by screenshot, not by reasoning about the CSS in
  the abstract.

**Rendering pass, Ethan's read after trying the egui prototype — done.** Three asks,
landed together in `web/app.js` + `style.css`:

- **One arrow per engagement, not one per blow.** A Struck and its StruckBack in the same
  Beat used to draw two arrows (or three, spread apart) for what is one clash seen from
  both sides. `drawBeatOverlay` now groups cues by the unordered {attacker, target} pair
  and draws a single, unidirectional arrow per pair -- the initiating blow's own
  direction -- while damage numbers stay one-per-blow (a trade still shows both figures).
- **Damage numbers as their own system, anchored on the card.** `.dmg-label` is now a
  child of the `.unit` it describes (`placeDamageLabel` appends directly to the struck
  card), positioned in that card's own box via `--stack` rather than computed from
  battlefield-relative `getBoundingClientRect` math against a shared overlay layer. Simpler
  and more robust to any future reflow -- the label moves with the card by construction, no
  recompute needed.
- **The BG attack motion -- cards snap together and rubber-band back.** Replaced the old
  scale/translateY pulse with `.unit.attacking`'s `clash` keyframe and `.unit.recoiling`'s
  smaller `recoil` keyframe, both driven by a real `--clash-x`/`--clash-y` vector (the same
  attacker->target direction `drawArrow` uses) set from JS, so a card visibly moves toward
  the card it's actually striking rather than a generic up/down nudge.

Verified end-to-end with Playwright, not just visually: drag-to-reorder still swaps
roster order; fight/skip-to-end/reset/play/rearrange all still transition correctly; a
Beat with a mutual trade renders exactly one arrow per engaged pair with each blow's own
stacked damage figure.

**Two-column layout, sidebar as one scrolling stack — done.** Ethan's explicit shape:
play field on the left, "information stuffs" on the right -- controls, the event log
(minimizable), and wherever a later addition lands -- as one vertically-scrolling column
rather than a slot per thing. `index.html`'s `<main>` still grids battlefield/sidebar
(unchanged), but the masthead no longer carries the transport controls; `.sidebar` is a
new `<aside>` holding `.controls` (now styled as its own card, matching `.dispatch`) and
the event log in order, `position: sticky` with a viewport-relative `max-height` so it
scrolls in place rather than growing the page past the fold. The log's own heading is now
a real `<button id="log-toggle">` toggling `aria-expanded`; collapsing it hides `.lines`
via a CSS sibling selector, no JS-held display state. Verified: collapse/expand round-trips
cleanly (`display: none` -> `block`), and drag/fight/skip/play/reset/rearrange all still
work after the DOM move (control element ids didn't change, only their container).

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
