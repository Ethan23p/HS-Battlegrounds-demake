# ADR conventions

Every ADR here is **Claude's**, not Ethan's — his own words never appear except inside a
cited, verbatim quote linked to a transcript. That distinction slipped once
([ADR 0003](0003-the-action-phase-is-a-simulation-of-beats.md), corrected by
[ADR 0008](0008-targeting-is-random-simultaneity-is-the-only-delta.md)): a previous
instance presented its own extrapolation from a real request as though it followed from
it, and it took a full design round to trace back where the drift actually started. Treat
a claim inside an ADR about what Ethan asked for as reliable only as far as the quote
backing it — no quote, no claim; an unquoted "Ethan wanted X" is Claude's reading, not a
fact.

The same caution applies one level down, even where nothing is misattributed: a
"Consequences" section is Claude's own downstream reasoning about a decision, not a
verified outcome — treat it as a read, not a settled fact, especially where it describes
a system that isn't built yet. (2026-09-08 found several of these across 0002, 0005,
0006, and 0009; the overreaching bits were trimmed, hedged, or moved to
[the scratchpad](../scratchpad/state.md#design-notes-for-unbuilt-systems).)

Numbered sequentially: `0001-slug.md`, `0002-slug.md`, ... Written only for decisions that
are hard to reverse, surprising without context, and the result of a real trade-off — see
`CLAUDE.md`'s working agreement. Not every choice gets one.
