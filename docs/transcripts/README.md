# Conversation Transcripts

This directory holds exported, human-readable Markdown transcripts of the Claude Code
conversations that produced this repository.

## Why this exists

Design decisions, rationale, and rejected alternatives live in conversation, not just in
the code and docs that resulted from it. Claude Code keeps a raw JSONL log of every
session (`~/.claude/projects/...`), but it's local, dense, and not part of the repo — if
the repo were deleted and rebuilt, or a fresh instance picked up the project with no
memory of these conversations, that context would be gone.

A cleaned-up Markdown export of each significant session solves that: a fresh instance
(or a human) can read a transcript here and be re-seeded with the actual discussion —
what was asked, decided, and why — with zero other context. This is the durable archive
of "how we got here."

## Naming convention

Files are named `NNNN-slug.md`, sequential and zero-padded to four digits
(`0001-project-kickoff.md`, `0002-effect-model-decision.md`, ...). `NNNN` increments once
per exported session (not per day), in the order sessions happened. `slug` is a short,
lowercase, hyphenated description. Pick the next number from the highest one already
here.

## How to add or regenerate an export

Exports are produced by `scripts/export_transcript.py`, which reads a session's raw
JSONL transcript and converts it to Markdown, stripping tool calls, tool results,
thinking blocks, and harness noise (system reminders, hook output, ambient
tool-availability notices) so what's left is the actual conversation.

```bash
# See what sessions are available for this project
python3 scripts/export_transcript.py --list

# Export the most recently modified session
python3 scripts/export_transcript.py --out docs/transcripts/0002-some-slug.md

# Export a specific session by UUID (see --list for the id)
python3 scripts/export_transcript.py --session <uuid> --out docs/transcripts/0002-some-slug.md

# Include compact one-line summaries of tool calls (still no raw payloads)
python3 scripts/export_transcript.py --include-tools --out docs/transcripts/0002-some-slug.md
```

Run it any time a session reaches a natural milestone (a major decision, end of a work
session, before a long gap). See the script's module docstring for the full JSONL schema
notes and known limitations if exports start looking wrong — Claude Code's on-disk
transcript format is undocumented and can change between versions.

## Known gap: answers to multiple-choice questions

Until 2026-09-06 the exporter dropped answers to multiple-choice questions
(`AskUserQuestion`). They arrive as *tool results* rather than user messages —
the harness asks on Claude's behalf — so the filter treated them as plumbing,
even though the words are Ethan's and are frequently where a decision was
actually made. The script now recovers them (`answers_recovered` in the front
matter counts them).

**`0001` and `0002` predate the fix and are known to be missing at least one.**
In `0002`, "And the technical decision you've been waiting on..." is followed
immediately by "Recorded." — the effect-model answer behind
[ADR 0007](../adr/0007-abilities-are-data-with-a-modifier-stage.md) fell in that
gap. Re-exporting fixes it, but that session's raw JSONL lives in the container
it ran in, so it has to be run from there:

```bash
python3 scripts/export_transcript.py \
  --session 7e14bc36-1a5c-562e-80d1-d6857cb7d317 \
  --out docs/transcripts/0002-vocabulary-and-action-phase.md
```

Regenerating an export *in place* like that is the one exception to the caveat
below: it covers the same span more faithfully, rather than capturing a later
one.

## Caveat: these are point-in-time snapshots

An export captures a session exactly as it stood when the script ran. If the session was
still active, the export simply stops there — not a summary or a "final" version, just a
snapshot. A session that continues after export needs a fresh, later-numbered export to
capture what happened next; the old one isn't overwritten.
