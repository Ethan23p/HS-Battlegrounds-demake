---
status: accepted
---

# The frontend is a web page; the engine ships as WebAssembly

`bg-sim` compiles to `wasm32-unknown-unknown` and is driven by an HTML page. The page is
the frontend for every device — there is no separate mobile build, and the terminal
frontend stops being the plan of record for playing the game.

A phone and a desktop had to be equally first-class, and that requirement alone settles
most of it. A terminal frontend cannot be used on a phone at all. A native GUI means two
or three toolchains, two app stores, and a signing story, for a project whose whole
engine is 2,200 lines. A web page is the only option where *one* artefact runs everywhere
without asking anybody to install anything.

The engine already compiles to wasm with no changes and no new dependencies, and the
resulting module **imports nothing** — it is a pure function of its input, which is what
[ADR 0001](0001-own-the-random-number-generator.md) and the no-I/O rule bought us. So the
loader is `WebAssembly.instantiate(bytes, {})` and twenty lines of JavaScript, with no
`wasm-bindgen` and no codegen step in the build.

**Delta 4 (offline-capable) becomes literally true rather than aspirational.** `web/bg.html`
is one self-contained file — the page and a base64 engine, 261 KB — that runs from a
`file://` URL with no server and no network. A phone can save it and play on a plane.

The seam is the one the engine already had. `resolve` returns an ordered `Vec<Event>`,
documented as the thing a frontend animates and needs to know nothing else. The page
takes that literally: the log is an ordered list of mutations, so replaying the first *k*
onto the starting Board gives the Board at step *k*. That is what makes the page a
**transport** — play, pause, step, scrub, step backwards — without the engine ever
storing a frame history. A rule bug stops being an anecdote at the point where you can
scrub to the Beat and look at it.

## Consequences

- **`bg-wasm` is a third crate, and shallow like `bg-cli`.** One function crosses the
  boundary: JSON in, resolved Action Phase out. Anything in it that looks like a game
  decision is a bug. `bg-sim` still does not know a browser exists.
- **The Action Phase's output types now derive `Serialize`.** Serde was already a
  dependency and the log was already documented as the frontend's interface; this makes
  that interface crossable. It is a data trait, not I/O, so the no-I/O promise holds.
- **The log's completeness is now testable, and it is very nearly complete.** One rough
  edge: `Struck` is logged before the `ShieldAbsorbed` that cancels it, so a renderer must
  look one event ahead to know whether the damage landed. Worth reconsidering when
  Abilities add events — an absorbed hit could carry that on the `Struck` event itself.
- **`bg-cli` is now a debugging tool, not the road to a playable game.** It keeps its
  place: a terminal narration is the fastest way to read a fight while working on rules.
- **A build step exists where there was none.** `scripts/build_web.py` runs cargo and
  inlines the result. Its output is not committed.
- **Presentation decisions are now cheap to try.** Cards are DOM and CSS, so the move
  from "near Battlegrounds" toward Marvel Snap is a stylesheet, not a renderer.

## Considered and rejected

- **A terminal frontend (ratatui) as the primary target** — excellent for reading rules
  while building them, and it will keep being used for exactly that. It fails the
  requirement outright: nobody plays this on a phone.
- **A native GUI in egui or Bevy** — egui does reach the web, but a card game is layout,
  type, and transitions, which DOM and CSS do better than an immediate-mode canvas, and
  Bevy is a large engine to carry for eight rectangles in a row.
- **wasm-bindgen** — the standard choice, and the right one when the boundary is wide.
  Here the boundary is one string in and one string out, and `wasm-bindgen` would add a
  `cargo install` and a codegen step to the build for a seam that is four `extern "C"`
  functions.
- **A server that runs the engine, with a thin web client** — kills delta 4, adds hosting,
  and buys nothing for a single-player deterministic game.
- **Rendering to canvas rather than DOM** — more control over animation, but layout, text
  fitting, focus, and accessibility all get rebuilt by hand.
