# Scratchpad

This directory is Claude's own working notes — not Ethan's words, not part of
`docs/DESIGN.md`, not canon. It's where a session keeps its mental model of "where
things actually stand right now" so the next instance doesn't have to reconstruct a plan
from git log and code.

## Rules for this directory

- **Ground truth is still `docs/DESIGN.md` and nothing else.** If anything here
  conflicts with it, DESIGN.md wins — this is Claude's understanding and planning
  around it, not a replacement for it, and it is never cited as design authority.
- **Freely rewritten.** No ratification bar. Update it the moment something changes;
  don't wait for permission, and don't treat old entries as historical record the way a
  transcript is — stale content here is just deleted, not archived.
- **Flag Claude's own calls here, not as settled fact elsewhere.** An engineering
  decision Claude had to make without an explicit ruling (a data format, a module
  boundary, a default) belongs here as "Claude's call, open to correction" until it's
  actually been put to Ethan — not written into code comments or docs as if it were
  decided. This is the same discipline as the design-vs-code split from the 2026-09-11
  session ([0006](../transcripts/0006-the-concurrent-beat.md)): a code question, Claude
  owns; a design question goes to Ethan and gets recorded in DESIGN.md's Ongoing, not
  invented here and left unflagged.
- **Prune stale entries.** A question that gets answered moves into DESIGN.md's Ongoing
  (Ethan's words, quoted) and disappears from here. A plan that's been superseded gets
  overwritten, not kept for history.

## Files

- [`roadmap.md`](roadmap.md) — the iteration plan (0.1, 0.2, ...), what's built, what's
  next, and open questions queued for Ethan.
