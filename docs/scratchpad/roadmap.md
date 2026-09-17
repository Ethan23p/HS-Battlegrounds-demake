# Front-end roadmap

Not canon — see `README.md`. Current as of 2026-09-17.

## Where things stand

- 0.1 through 0.4 are all done: fight viewer, drag-to-reorder, the shop/persistent-run
  loop, and now abilities actually executing against a data-driven roster
  (`assets/roster.ron`) rather than keyword-only placeholder Units. See each iteration's
  own section below for what shipped and what bugs actually playing it surfaced.

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

**Follow-up round, from Ethan actually playing it on his phone:**

- A real coordinate bug, not a design question: on a portrait phone, arrows and the
  attack "clash" motion were computed from `getBoundingClientRect` (screen space, already
  rotated by the forced-landscape trick in `style.css`) and then fed into coordinate
  spaces (the SVG overlay's local space, a card's own `translate()`) that get that same
  ambient rotation applied a second time by the browser -- net double rotation, which is
  exactly the diagonal/crossed arrows Ethan's screenshot showed. Fixed by computing
  positions from the `offsetParent` chain instead (`offsetRelativeTo` in `app.js`), which
  is layout-space and transform-immune, matching what both the SVG's local space and a
  card's own pre-ambient-transform space actually need.
- Freezing is now board-wide, one button (`#freeze` in `.info`, next to Reroll), not a
  per-card toggle -- `RunState::toggle_freeze` takes no offer index anymore: it freezes
  every current offer if any is unfrozen, unfreezes all if the whole shop is already
  frozen. Matches Battlegrounds itself, and Ethan's correction that per-card freezing
  wasn't the intended shape.
- The reroll cost readout was showing "3g" while `REROLL_COST` is actually 1 -- caught
  while touching that line for the freeze button, fixed alongside it.
- The sell button's real hit target was 14x14px, fine for a mouse but too small for a
  thumb on a real phone -- likely what Ethan actually hit ("I tried selling something and
  it didn't seem to work"). Given an invisible `::before` padding out to a ~40x40 hit
  area without changing the visible glyph or card layout.
- Board-state healing between rounds (survivors return at full health, deaths are
  permanent) was already correct from 0.3's `apply_fight_result` fix -- Ethan's message
  read as confirming the intended design, not reporting a regression, and a fresh
  Playwright check (buy, fight to a damaged win, continue) confirmed it still holds.
- Added `scripts/build_web.sh` for the two-step `cargo build -p bg-wasm` +
  `wasm-bindgen` rebuild -- the repetitive step Ethan asked about.

Ethan also suggested a standing discipline for future changes to this loop: keep a
scripted Playwright sequence that plays through several mechanics at once (buy, sell,
freeze, reroll, upgrade, fight, continue across rounds) and actually watch it run before
committing, rather than trusting unit tests alone for what's fundamentally a feel-driven
UI. Built this round as `scripts/playtest.js` (see 0.4 below) -- running it against the
0.4 changes is exactly what caught the run-over-screen bug documented there.

### 0.4 — Abilities execute, and the roster is data -- done

Planned in chat first, same structure as 0.3 (abstract functions, then interfaces, then
the tangible surface). Two real design questions were open going in, both settled and
recorded in DESIGN.md Ongoing:

- **Ability authoring is a fixed vocabulary, extended on request, not a scripting
  layer.** `units::Trigger`/`Condition`/`Selector`/`Effect` already existed as an
  unexecuted vocabulary; the question was whether to build the engine for it (extending
  it by hand as new abilities need something it can't express) or embed something like
  Rhai so any mechanic is expressible without an engine change. Ethan's framing: Claude
  itself, asked to add the variant a new ability needs, *is* the extensibility
  mechanism -- there's no expectation of hand-authored or human-scriptable abilities.
- **Roster content is named from psychology vocabulary**, at Ethan's suggestion, purely
  as flavor text the engine never reads for anything but display -- not encoded meaning,
  just a more interesting naming scheme than the placeholder animal names 0.3 shipped
  with.

**Landed as `bg-sim::abilities`** (`fire_own`/`fire_deathrattle`/`fire_broadcast`/
`fire_all`): walks a Unit's `Ability` list for a fired `Trigger`, checks its `Condition`,
resolves each `Effect`'s `Selector` into concrete Units, and applies it. Wired at points
`action_phase::resolve_with_roster` and `prep_phase::RunState` already pass through --
Deathrattle and AfterFriendlyDeath in `bury`, OnAttack before a blow lands (so Rally-style
self-buffs affect that same attack), OnSurviveDamage after one, StartOfActionPhase once
before the first Beat, OnBuy/Battlecry/AfterFriendlyPlayed on `buy`, OnSell on `sell`,
StartOfTurn on `start_new_round`, EndOfTurn on `end_turn`. `resolve` (no abilities, `&[]`
roster) stays as a thin wrapper so none of `action_phase`'s 38 existing combat-mechanics
tests needed to change -- `resolve_with_roster` is the new entry point that also executes
abilities, used everywhere abilities should actually run.

`GainGold` is the one Effect the module can't apply on its own (gold lives on
`RunState`, not `Board`); every entry point threads a `gold: &mut u32` through for it,
and an Action Phase call passes a throwaway scratch value -- which is exactly
`GainGold`'s own documented "ignored in the Action Phase." `AddToHand` has no real hand
to add to (buying already places a Unit straight onto the board), so it places directly,
the same as a purchase does.

**The roster moved from `fixtures::shop_roster` (deleted) to `assets/roster.ron`** -- a
`Vec<UnitDef>` written as data, parsed by `units::load_roster` (the one place RON parsing
happens). `bg-wasm` exposes `parse_roster(ron_text) -> json`; every other Prep Phase wasm
function now takes `roster_json` as a parameter instead of reaching for a hardcoded
roster internally, since the roster is something the browser fetches
(`web/roster.ron`, copied from `assets/roster.ron` by `scripts/build_web.sh`) rather than
something compiled in. 13 Units (12 shop offers plus one Deathrattle token), one per
`Trigger` the engine executes, named from psychology vocabulary (Instinct, Placebo,
Habituation, Extinction, Repression, Confirmation Bias, Sublimation, Groupthink,
Catharsis, Transference, Self-Actualization, Individuation, Suppressed Impulse).

A real bug surfaced immediately by actually playing it, not left standing: Placebo's
Battlecry never fired, because `buy()` only fired `Trigger::OnBuy` -- `units::Trigger`
names Battlecry ("played from hand") and OnBuy ("bought into hand") as two separate
moments, and this engine collapses hand and board into one buy step, so both need to
fire together there. A second, smaller one: the run-over screen's sidebar Record readout
went stale one fight behind the actual final score, since `enterRunOver` never repainted
it (and calling the shop's full `render()` there would have overwritten the final fight's
board with shop-offer styling instead) -- fixed by refreshing just that one field.

Verified per-ability with direct `bg-wasm` calls in-browser (buy/sell/fight sequences
engineered to exercise each `Trigger` deterministically) and end-to-end with
`scripts/playtest.js` (buy, freeze/reroll, sell, drag-reorder, fight across rounds to a
run-over) -- the latter is the standing discipline Ethan asked for, kept as a committed
script rather than an ad hoc one-off this time.

Still placeholder, deliberately: ability effects are unbalanced hand-picked numbers, same
spirit as 0.3's costs. `fixtures::showcase_board` (a separate, small hardcoded fixture
for `bg-cli`/`showcase_board_json`) is untouched -- it was never the shop roster and
isn't part of what 0.4 moved to data.

### 0.5 — Polish, VFX, juice
Deliberately last: juice on rules that might still change is wasted work. The rendering
pass finished before 0.3 (arrows, damage numbers, clash motion, proportional layout)
covers the fight viewer; the shop screen built in 0.3 is plain by comparison (click to
buy/sell/freeze, no animation) and will want its own pass once the loop itself is settled.
