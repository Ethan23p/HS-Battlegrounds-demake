# Scratchpad

This directory is Claude's own working notes — not Ethan's words, not ratified
vocabulary, not an ADR. It's where a session keeps its mental model of "where things
actually stand right now" so the next instance doesn't have to reconstruct it from git
log and code.

## Rules for this directory

- **Freely rewritten.** No ratification bar, unlike `CONTEXT.md` or the deltas in
  `docs/design/vision.md`. Update it the moment something changes; don't wait for
  permission.
- **Not authoritative on design.** If something here conflicts with `docs/design/`,
  `CONTEXT.md`, or an ADR, those win — this is Claude's understanding of them, not a
  replacement for them.
- **Flag Claude's own calls here, not as settled fact elsewhere.** An engineering detail
  Claude had to invent (an attacker-cycling rule, a return-stat for Reborn, whatever)
  belongs here as "open, awaiting Ethan's correction" until he's actually weighed in —
  see [ADR 0008](../adr/0008-targeting-is-random-simultaneity-is-the-only-delta.md)'s
  "Two calls that were flagged, and how they landed" for what happens when that
  discipline slips: unflagged invention is exactly how three of that ADR's four
  corrections started.
- **Prune stale entries.** An open question that gets answered should move to
  `CONTEXT.md`, an ADR, or the deltas in `vision.md`, and disappear from here — not
  accumulate as dead weight.

## Files

- [`state.md`](state.md) — what's built, what's next, and every open item.
