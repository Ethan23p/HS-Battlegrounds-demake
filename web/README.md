# web — Beat Ledger

Plays back a resolved Action Phase as an animated board. Also published as a Claude
artifact so it can be opened on a phone with no local setup at all — see
`docs/scratchpad/roadmap.md` for the link and for what this is and isn't (a fixture
viewer, not a game yet).

## Running it locally

`app.js` is an ES module (it imports `pkg/bg_wasm.js`), so it needs a real HTTP origin
— browsers refuse module imports over `file://`. Serve the directory with anything
static:

```sh
cd web && python3 -m http.server 8000
# then open http://localhost:8000/
```

## Regenerating `pkg/` (the WASM build)

`pkg/` is generated, not hand-written, and committed for the same reason `bg-cli`'s
output used to be: the page should need no build step to open, only to change.
Regenerate it after any change to `bg-sim` or `bg-wasm`:

```sh
cargo build -p bg-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/release/bg_wasm.wasm
```

`wasm-bindgen`'s CLI version must match the `wasm-bindgen` crate version exactly
(`cargo tree -p bg-wasm -i wasm-bindgen` shows it). If they drift:

```sh
cargo install wasm-bindgen-cli --version <that version>
```

The Board resolved is still a fixture (`bg_sim::fixtures::showcase_board` — three
units a side, chosen to exercise Windfury, Divine Shield, Reborn and Taunt in one
fight): there is no shop or party builder yet. `bg-cli` resolves the same fixture from
the command line, for a look with no browser involved.

## How playback works

`app.js` calls `bg-wasm`'s `resolve` directly — the real `bg-sim::action_phase::resolve`,
compiled to `wasm32` and run in the browser — so the fight it plays back is not a second
implementation of the rules. It also no longer reconstructs board state itself: the
`Resolution` carries `boards`, a Board snapshot already fully resolved for every Beat, so
the viewer reads the resulting state directly instead of deriving it from the log. `log`
is read here only for narration and animation timing — which slot to flash, what number
to pop — never for what a Unit's resulting stats or keywords are. One step of playback is
one Beat: its cues (strikes, shields, deaths, revivals) animate together, since a Beat's
interactions are concurrent, and the numbers on screen only change once, when that Beat's
board is revealed. See the comment at the top of `app.js`.
