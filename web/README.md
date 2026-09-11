# web — 0.1 fight viewer

A static page, no build step, no server dependency: open `index.html` directly in a
browser. It plays back one resolved Action Phase as an animated board.

See `docs/scratchpad/roadmap.md` for what this is and isn't (a fixture viewer, not a
game yet) and what 0.2+ adds.

## Regenerating the fixture fight

`fight.js` is generated, not hand-written -- it's `bg-cli`'s output, committed so the
page works with no build step. Regenerate it after any change to `bg-sim`'s rules, or to
see a different fight (an argument is a seed):

```sh
cargo run -q -p bg-cli -- 1 js > web/fight.js
```

`bg-cli`'s Board is a fixture (three units a side, chosen to exercise Windfury, Divine
Shield, Reborn and Taunt in one fight) — there is no shop or party builder yet.

## How playback works

`app.js` reconstructs board state by replaying `FIGHT.log` against a client-side copy of
`FIGHT.initial_board`. The reconstruction rules (when a shield breaks, what Reborn
revives with, how compaction packs) are read directly off `bg-sim`'s source and kept in
sync with it by hand — there's no shared code between the two yet. See the comment at
the top of `app.js`. 0.4 (WASM) is what removes this duplication.
