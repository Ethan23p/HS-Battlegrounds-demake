# Conversation Transcripts

This directory holds exported, human-readable Markdown transcripts of the
Claude Code conversations that produced this repository.

## Why this exists

The project's design decisions, rationale, and rejected alternatives live in
conversation, not just in the code and docs that resulted from it. Claude
Code keeps a raw JSONL log of every session on the machine it ran on
(`~/.claude/projects/...`), but that log is local, dense, and not part of the
repo — if the repo were deleted and rebuilt, or a fresh Claude instance
picked up the project with no memory of these conversations, that context
would be gone.

Committing a cleaned-up Markdown export of each significant session solves
that: a fresh instance (or a human) can read a transcript here and be
re-seeded with the actual discussion — what was asked, what was decided, and
why — even with zero other context. This is the durable archive of "how we
got here."

## Naming convention

Files are named `NNNN-slug.md`, sequential and zero-padded to four digits:

- `0001-project-kickoff.md`
- `0002-effect-model-decision.md`
- ...

`NNNN` increments once per exported session (not per day), in the order the
sessions happened. `slug` is a short, lowercase, hyphenated description of
what the session was mainly about. Pick the next number by looking at the
highest one already in this directory.

## How to add or regenerate an export

Exports are produced by `scripts/export_transcript.py`, which reads a
session's raw JSONL transcript and converts it to Markdown, stripping out
tool calls, tool results, thinking blocks, and harness noise (system
reminders, hook output, ambient tool-availability notices, etc.) so what's
left is the actual conversation.

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

Run it any time a session reaches a natural milestone worth preserving (a
major decision, the end of a work session, before a long gap). See the
script's module docstring for the full JSONL schema notes and known
limitations — read that first if the exports start looking wrong, since
Claude Code's on-disk transcript format is undocumented and can change
between versions.

## Caveat: these are point-in-time snapshots

An export captures a session exactly as it stood at the moment the script
ran. If the session was still active/ongoing, the export simply stops where
the transcript did at that moment — it is not a summary or a "final" version
of that conversation, just a snapshot. A session that continues after being
exported will need a fresh export (as a new, later-numbered file) to capture
what happened next; the old export is not overwritten or made obsolete, it's
just an earlier snapshot in the sequence.
