#!/usr/bin/env python3
"""Export a Claude Code session transcript (raw JSONL) to readable Markdown.

WHY THIS EXISTS
----------------
Claude Code stores one JSONL "event log" per session at:

    ~/.claude/projects/<mangled-cwd>/<session-uuid>.jsonl

where <mangled-cwd> is the project's working directory with every "/"
replaced by "-" (e.g. /home/user/HS-Battlegrounds-demake ->
-home-user-HS-Battlegrounds-demake). These files are the authoritative
record of a conversation, but they are dense, line-oriented, and full of
harness bookkeeping that isn't part of the actual discussion. This script
converts one such file into a clean Markdown conversation, suitable for
committing into the repo as a durable, human-readable archive that can
re-seed a fresh Claude instance (or a human) even if the repo were deleted
and rebuilt from scratch.

SCHEMA NOTES (reverse-engineered empirically from a real transcript,
version "2.1.260" as of 2026-09-04 -- Claude Code's on-disk format is
undocumented and can change without notice; treat everything below as
observed fact about one snapshot, not a stable contract)
-------------------------------------------------------------------------
Each line is a standalone JSON object ("event"). The event's top-level
"type" field is the primary discriminator:

  "user"       - A turn attributed to the user role. This does NOT always
                 mean a human typed it -- see below. message.role=="user",
                 message.content is either:
                   * a plain string  -> the human's literal typed text
                     (this is the common case for real human input), or
                   * a list of content blocks, each with its own "type":
                       - "text": plain text (e.g. the literal string
                         "[Request interrupted by user]" when the user
                         hits ctrl-C mid-turn)
                       - "tool_result": the result of a tool call the
                         assistant made. Has "tool_use_id" (links back to
                         the assistant's tool_use block id), "content"
                         (either a string, or a list of sub-blocks such
                         as {"type": "text", ...} or
                         {"type": "tool_reference", ...}), and
                         "is_error" (bool). These are tool plumbing,
                         with ONE exception: the result of a
                         multiple-choice question (AskUserQuestion)
                         wraps the human's own answer, in the form
                         'The user answered: "<question>"="<answer>"'.
                         Those words were typed by a person and are
                         recovered as ordinary human turns -- see
                         extract_answers().
               Two harness signals mark a "user" line as NOT human-typed
               conversation:
                 - isMeta == true: injected context, not something the
                   person wrote. Observed cases: stop-hook feedback
                   ("Stop hook feedback: ..."), full skill-file dumps
                   injected when a skill loads, and the boilerplate
                   "<local-command-caveat>...</local-command-caveat>"
                   wrapper around slash-command output.
                 - content is a literal slash-command invocation of the
                   form "<command-name>/foo</command-name>\\n
                   <command-message>...</command-message>\\n
                   <command-args>...</command-args>" -- this one was
                   NOT marked isMeta in practice, so it needs its own
                   pattern match (see META_TEXT_MARKERS below).

  "assistant"  - A turn from Claude. message.content is a list of content
                 blocks:
                   - "text": Claude's visible reply text.
                   - "thinking": extended-thinking content. Has a
                     "thinking" string and an opaque "signature". In the
                     transcript this script was developed against, every
                     thinking block's "thinking" field was an EMPTY
                     string (the actual reasoning is redacted/omitted on
                     disk even though the block is present) -- so thinking
                     is unconditionally excluded from output; there is
                     nothing meaningful to show even with --include-tools.
                   - "tool_use": a tool invocation. Has "id", "name", and
                     "input" (a dict of tool-specific arguments).

  "system"     - Harness-level notices, not conversation. Observed
                 subtypes: "stop_hook_summary" (output of configured Stop
                 hooks, e.g. a git-check reminder) and "local_command"
                 (stdout of a slash command, wrapped in
                 <local-command-stdout> tags). Always skipped.

  "attachment" - Ambient context deltas the harness silently injects:
                 which tools just became available (deferred_tools_delta),
                 MCP server instructions (mcp_instructions_delta), skill
                 listings (skill_listing), agent listings
                 (agent_listing_delta), token-budget reminders
                 (total_tokens_reminder), auto-mode notices, etc. None of
                 this was written or read by a human. Always skipped.

  "queue-operation", "mode", "atis-latch", "last-prompt"
               - Pure session bookkeeping (message queue state, the
                 current permission mode, an idle-latch flag, a cache of
                 the last prompt for UI display). These frequently lack a
                 "timestamp" field entirely and carry no conversational
                 content. Always skipped.

Other structural facts:
  - Every event that represents real conversation (user/assistant/system/
    attachment) carries "timestamp" as ISO-8601 UTC with millisecond
    precision and a "Z" suffix, e.g. "2026-09-04T14:52:44.092Z". The
    bookkeeping types above often omit it.
  - "uuid" is this event's own id; "parentUuid" links to the event it
    followed (a loose linked-list/tree over the conversation). In
    practice the file is append-only and line order already matches
    chronological order, so this script simply reads top to bottom rather
    than reconstructing the tree.
  - "isSidechain": true marks an event as belonging to a SUBAGENT's own
    transcript. In this dataset, subagent transcripts are entirely
    separate files under a sibling "subagents/" directory
    (<session-dir>/subagents/agent-<id>.jsonl, plus a matching
    "*.meta.json" with agentType/description/model), sharing this same
    per-line schema plus an "agentId" field. A subagent invocation shows
    up in the PARENT transcript only as an ordinary "tool_use" block
    (name "Agent" or "Task") followed by its "tool_result" -- the child
    conversation is not inlined. This script exports one top-level
    session file at a time and does not walk into subagents/; see
    LIMITATIONS.
  - Every event also repeats ambient fields: sessionId, cwd, gitBranch,
    version, userType, entrypoint. Handy for the front matter, not for
    filtering.

FILTERING PHILOSOPHY
---------------------
The goal is a *readable conversation* that could re-seed a fresh Claude
instance, not a full replay. By default this script keeps only:
  - human-typed user text (verbatim, never paraphrased)
  - Claude's visible reply text

...and drops thinking blocks, tool calls, tool results, and all harness/
bookkeeping noise -- except answers to multiple-choice questions, which
are human words wearing a tool result's clothes and are always kept. Pass --include-tools to add back compact, one-line
summaries of tool calls (tool name + short description/first argument),
never full payloads.

The noise-detection rules live in the constants right below the imports
(LINE_TYPES_SKIPPED, META_TEXT_MARKERS, etc.) specifically so the next
person to touch this script can tune them in one place without reading
the rest of the code.

LIMITATIONS (known, deliberate)
--------------------------------
  - Subagent transcripts (subagents/*.jsonl) are not walked or merged in;
    a subagent invocation appears at most as a compact tool-call summary
    under --include-tools ("Agent: <description>"), never its full
    sub-conversation. Exporting those is a separate, un-implemented
    feature -- point --session at a subagents/*.jsonl file directly if
    you want to try (the per-line schema is compatible), but --list does
    not enumerate them and turn-merging heuristics are tuned for
    top-level sessions.
  - The mangled-cwd -> project-directory mapping is inferred by naively
    replacing "/" with "-" in the current working directory. Claude Code
    itself does the real mangling; if it ever changes scheme (e.g. to
    handle "." or other characters differently) this script's default
    --project-dir guess would miss, though --project-dir lets you point
    at the right folder directly.
  - This schema was reverse-engineered from ONE real transcript (Claude
    Code v2.1.260, single session, no compaction boundary, no images, no
    redacted_thinking blocks, no permission-denial events observed). Rarer
    shapes (compaction summaries, image content blocks, multi-turn
    sidechains inlined into the main file, etc.) are handled defensively
    -- unrecognized block/line shapes are skipped with a counted warning
    rather than crashing the export -- but have not been seen firsthand.
  - Tool-result content is never shown by default and only as a truncated
    one-liner under --include-tools; if you need the full tool output,
    read the raw JSONL.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

# =============================================================================
# Noise-filtering rules. Tune these as the upstream schema drifts.
# =============================================================================

# Top-level event "type" values that are pure harness/bookkeeping noise and
# never contain conversation. Skipped unconditionally.
LINE_TYPES_SKIPPED = {
    "queue-operation",
    "mode",
    "atis-latch",
    "last-prompt",
    "attachment",
    "system",
}

# Substrings that, if found at the START of a "user" event's string content,
# mark it as harness-injected rather than human-typed -- even when isMeta is
# not set (observed for slash-command invocation echoes).
META_TEXT_PREFIXES = (
    "<command-name>",
    "<local-command-caveat>",
    "Stop hook feedback:",
)

# Markers identifying harness-injected content that arrives with the "user"
# role but was never typed by a human: background-task completion notices and
# system reminders. Unlike META_TEXT_PREFIXES these are matched anywhere in the
# opening of the message, because the wrapper element is not always first.
# Only the head of the message is searched, so a human quoting one of these
# tags later in a long message is still exported.
META_TEXT_SUBSTRINGS = (
    "<task-notification>",
    "<system-reminder>",
    "[SYSTEM NOTIFICATION - NOT USER INPUT]",
)
META_SUBSTRING_SEARCH_CHARS = 400

# Literal text-block bodies that are harness markers, not conversation, but
# still worth a short note in the output rather than silent deletion.
INTERRUPT_MARKER = "[Request interrupted by user]"

# A multiple-choice answer arrives as a tool result wrapping the human's own
# words. These markers pick it out and strip the harness framing around it.
ANSWER_PREFIX = "The user answered:"
ANSWER_TRAILERS = (
    ". Read the answers carefully",
    "Read the answers carefully",
)

# Content-block "type" values we understand within a "user" line's content
# list. Anything else is counted and skipped rather than crashing the export.
# (The assistant-side equivalent -- text/thinking/tool_use/redacted_thinking
# -- is handled directly in render_transcript()'s per-block dispatch below.)
KNOWN_USER_BLOCK_TYPES = {"text", "tool_result"}

TRUNCATE_LEN = 100  # for --include-tools one-line summaries


# =============================================================================
# Data loading
# =============================================================================


class Stats:
    def __init__(self) -> None:
        self.lines_total = 0
        self.lines_parse_errors = 0
        self.lines_skipped_type = 0
        self.user_turns_human = 0
        self.user_turns_meta_skipped = 0
        self.assistant_turns = 0
        self.tool_calls_seen = 0
        self.tool_results_seen = 0
        self.answers_recovered = 0
        self.thinking_blocks_seen = 0
        self.unknown_block_types: dict[str, int] = {}
        self.unknown_line_shapes = 0

    def note_unknown_block(self, block_type: str) -> None:
        self.unknown_block_types[block_type] = self.unknown_block_types.get(block_type, 0) + 1


def default_project_dir(project_dir_arg: str | None) -> Path:
    if project_dir_arg:
        return Path(project_dir_arg).expanduser()
    cwd = str(Path.cwd())
    mangled = cwd.replace("/", "-")
    return Path.home() / ".claude" / "projects" / mangled


def find_session_files(project_dir: Path) -> list[Path]:
    """Top-level session transcripts only (not subagents/ children)."""
    if not project_dir.is_dir():
        return []
    return sorted(
        (p for p in project_dir.glob("*.jsonl") if p.is_file()),
        key=lambda p: p.stat().st_mtime,
        reverse=True,
    )


def resolve_session_path(session_arg: str | None, project_dir: Path) -> Path:
    if session_arg:
        candidate = Path(session_arg).expanduser()
        if candidate.exists():
            return candidate
        candidate = project_dir / f"{session_arg}.jsonl"
        if candidate.exists():
            return candidate
        raise SystemExit(f"error: could not resolve --session {session_arg!r} "
                          f"(tried it as a path and as {candidate})")
    files = find_session_files(project_dir)
    if not files:
        raise SystemExit(f"error: no session transcripts found under {project_dir}")
    return files[0]


def approx_turn_count(path: Path) -> int:
    count = 0
    try:
        with path.open("r", encoding="utf-8", errors="replace") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    d = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if d.get("type") in ("user", "assistant"):
                    count += 1
    except OSError:
        return -1
    return count


# =============================================================================
# Rendering helpers
# =============================================================================


def truncate(text: str, n: int = TRUNCATE_LEN) -> str:
    text = " ".join(text.split())  # collapse whitespace/newlines
    if len(text) <= n:
        return text
    return text[: n - 1].rstrip() + "…"


def is_meta_or_harness_text(text: str) -> bool:
    stripped = text.lstrip()
    if any(stripped.startswith(p) for p in META_TEXT_PREFIXES):
        return True
    head = stripped[:META_SUBSTRING_SEARCH_CHARS]
    return any(marker in head for marker in META_TEXT_SUBSTRINGS)


def summarize_tool_use(block: dict) -> str:
    name = block.get("name", "?")
    inp = block.get("input") or {}
    desc = None
    if isinstance(inp, dict):
        for key in ("description", "file_path", "pattern", "query", "url", "prompt", "command"):
            if key in inp and isinstance(inp[key], str) and inp[key].strip():
                desc = inp[key]
                break
        if desc is None:
            for v in inp.values():
                if isinstance(v, str) and v.strip():
                    desc = v
                    break
    if desc is None:
        return f"`{name}`"
    return f"`{name}`: {truncate(desc)}"


def tool_result_text(block: dict) -> str:
    """The plain text of a tool result, whatever shape it arrived in."""
    content = block.get("content")
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts = []
        for sub in content:
            if isinstance(sub, dict) and sub.get("type") == "text":
                parts.append(sub.get("text", ""))
        return " ".join(parts)
    return str(content)


def extract_answers(block: dict) -> list[tuple[str, str]]:
    """Human answers to a multiple-choice question, as (question, answer).

    These arrive as a *tool result* rather than a user message, because the
    harness asks the question on Claude's behalf -- but the words are the
    human's, and they are frequently where a decision actually gets made. An
    archive that drops them is not the record it claims to be, so they are
    recovered here and rendered as ordinary human turns.

    Returns an empty list for every other kind of tool result.
    """
    text = tool_result_text(block)
    if not text.lstrip().startswith(ANSWER_PREFIX):
        return []
    body = text.lstrip()[len(ANSWER_PREFIX) :]
    for tail in ANSWER_TRAILERS:
        cut = body.rfind(tail)
        if cut != -1:
            body = body[:cut]
    pairs = re.findall(r'"(.*?)"="(.*?)"(?=,\s*"|\s*\.?\s*$)', body, flags=re.DOTALL)
    if pairs:
        return [(q.strip(), a.strip()) for q, a in pairs]
    # Shape drifted; keep the words rather than losing them.
    return [("", body.strip().strip('".'))]


def summarize_tool_result(block: dict) -> str:
    content = block.get("content")
    is_error = bool(block.get("is_error"))
    if isinstance(content, str):
        text = content
    elif isinstance(content, list):
        parts = []
        for sub in content:
            if not isinstance(sub, dict):
                continue
            if sub.get("type") == "text":
                parts.append(sub.get("text", ""))
            elif sub.get("type") == "tool_reference":
                parts.append(sub.get("tool_name", ""))
            else:
                parts.append(f"<{sub.get('type', 'unknown')}>")
        text = " ".join(parts)
    else:
        text = str(content)
    prefix = "error" if is_error else "result"
    return f"→ {prefix}: {truncate(text)}"


# =============================================================================
# A "turn" the exporter emits is either a human message or an accumulated
# block of Claude output (text plus, optionally, tool-call summaries) that
# may span several raw assistant/tool-result lines in the source file.
# =============================================================================


class ClaudeBuffer:
    """Accumulates one logical Claude turn across consecutive assistant
    lines (and any interleaved tool-only, non-human user lines) until the
    next real human message forces a flush."""

    def __init__(self) -> None:
        self.parts: list[str] = []

    def add_text(self, text: str) -> None:
        if text.strip():
            self.parts.append(text.rstrip())

    def add_tool_line(self, line: str) -> None:
        self.parts.append(f"> {line}")

    def is_empty(self) -> bool:
        return not self.parts

    def render(self) -> str:
        return "\n\n".join(self.parts)

    def reset(self) -> None:
        self.parts = []


def render_transcript(events: list[dict], stats: Stats, include_tools: bool) -> str:
    # sections is a list of (role, text) pairs; adjacent same-role pairs are
    # merged at render time so an interrupt note immediately followed by the
    # user's real next message reads as one "## User" block, not two.
    sections: list[tuple[str, str]] = []
    buf = ClaudeBuffer()

    def flush_claude() -> None:
        if not buf.is_empty():
            sections.append(("Claude", buf.render()))
            buf.reset()

    def add_user(text: str) -> None:
        if sections and sections[-1][0] == "User":
            role, prev = sections[-1]
            sections[-1] = (role, prev + "\n\n" + text)
        else:
            sections.append(("User", text))

    for ev in events:
        etype = ev.get("type")

        if etype not in ("user", "assistant"):
            stats.lines_skipped_type += 1
            continue

        message = ev.get("message")
        if not isinstance(message, dict):
            stats.unknown_line_shapes += 1
            continue
        content = message.get("content")

        if etype == "user":
            if ev.get("isMeta"):
                stats.user_turns_meta_skipped += 1
                continue

            if isinstance(content, str):
                if is_meta_or_harness_text(content):
                    stats.user_turns_meta_skipped += 1
                    continue
                flush_claude()
                stats.user_turns_human += 1
                add_user(content)
                continue

            if isinstance(content, list):
                human_text_parts: list[str] = []
                tool_result_lines: list[str] = []
                saw_interrupt = False
                for block in content:
                    if not isinstance(block, dict):
                        stats.unknown_line_shapes += 1
                        continue
                    btype = block.get("type")
                    if btype == "text":
                        text = block.get("text", "")
                        if text == INTERRUPT_MARKER:
                            saw_interrupt = True
                        elif is_meta_or_harness_text(text):
                            pass  # harness echo inside a block; drop
                        else:
                            human_text_parts.append(text)
                    elif btype == "tool_result":
                        stats.tool_results_seen += 1
                        answers = extract_answers(block)
                        for question, answer in answers:
                            stats.answers_recovered += 1
                            if question:
                                human_text_parts.append(
                                    f"*(answering: {question})*\n\n{answer}"
                                )
                            else:
                                human_text_parts.append(answer)
                        if include_tools and not answers:
                            tool_result_lines.append(summarize_tool_result(block))
                    elif btype in KNOWN_USER_BLOCK_TYPES:
                        pass
                    else:
                        stats.note_unknown_block(f"user:{btype}")
                    continue

                if human_text_parts:
                    flush_claude()
                    stats.user_turns_human += 1
                    joined = "\n\n".join(human_text_parts)
                    add_user(joined)
                elif saw_interrupt:
                    flush_claude()
                    stats.user_turns_human += 1
                    add_user("*(user interrupted Claude's previous turn)*")
                else:
                    stats.user_turns_meta_skipped += 1

                if include_tools and tool_result_lines:
                    for line in tool_result_lines:
                        buf.add_tool_line(line)
                continue

            stats.unknown_line_shapes += 1
            continue

        # etype == "assistant"
        if not isinstance(content, list):
            stats.unknown_line_shapes += 1
            continue
        stats.assistant_turns += 1
        for block in content:
            if not isinstance(block, dict):
                stats.unknown_line_shapes += 1
                continue
            btype = block.get("type")
            if btype == "text":
                buf.add_text(block.get("text", ""))
            elif btype in ("thinking", "redacted_thinking"):
                stats.thinking_blocks_seen += 1
                # never rendered -- see module docstring
            elif btype == "tool_use":
                stats.tool_calls_seen += 1
                if include_tools:
                    buf.add_tool_line(summarize_tool_use(block))
            else:
                stats.note_unknown_block(f"assistant:{btype}")

    flush_claude()
    return "\n\n".join(f"## {role}\n\n{text}" for role, text in sections)


# =============================================================================
# Main
# =============================================================================


def build_front_matter(session_path: Path, session_id: str, stats: Stats, include_tools: bool) -> str:
    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    lines = [
        "---",
        f"session_id: {session_id}",
        f"export_date: {now}",
        f"source_path: {session_path}",
        f"include_tools: {str(include_tools).lower()}",
        "message_counts:",
        f"  lines_total: {stats.lines_total}",
        f"  user_turns: {stats.user_turns_human}",
        f"  assistant_turns: {stats.assistant_turns}",
        f"  user_lines_filtered_as_noise: {stats.user_turns_meta_skipped}",
        f"  tool_calls_omitted: {stats.tool_calls_seen}",
        f"  tool_results_omitted: {stats.tool_results_seen}",
        f"  answers_recovered: {stats.answers_recovered}",
        f"  thinking_blocks_omitted: {stats.thinking_blocks_seen}",
        f"  parse_errors: {stats.lines_parse_errors}",
        f"  unknown_block_shapes: {sum(stats.unknown_block_types.values())}",
        "---",
    ]
    return "\n".join(lines)


def load_events(path: Path, stats: Stats) -> list[dict]:
    events: list[dict] = []
    with path.open("r", encoding="utf-8", errors="replace") as f:
        for raw_line in f:
            raw_line = raw_line.strip()
            if not raw_line:
                continue
            stats.lines_total += 1
            try:
                events.append(json.loads(raw_line))
            except json.JSONDecodeError:
                stats.lines_parse_errors += 1
    return events


def cmd_list(project_dir: Path) -> None:
    files = find_session_files(project_dir)
    if not files:
        print(f"No session transcripts found under {project_dir}", file=sys.stderr)
        return
    print(f"Sessions in {project_dir}:\n")
    print(f"{'session_id':<38} {'modified':<21} {'size':>10}  turns")
    for p in files:
        st = p.stat()
        mtime = datetime.fromtimestamp(st.st_mtime, tz=timezone.utc).strftime("%Y-%m-%d %H:%M:%S")
        size = f"{st.st_size / 1024:.1f} KB"
        turns = approx_turn_count(p)
        print(f"{p.stem:<38} {mtime:<21} {size:>10}  {turns}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Export a Claude Code session JSONL transcript to readable Markdown."
    )
    parser.add_argument("--session", help="Session UUID or path to a .jsonl transcript file.")
    parser.add_argument("--out", help="Output Markdown file path. Defaults to stdout.")
    parser.add_argument("--list", action="store_true", help="List available sessions and exit.")
    parser.add_argument(
        "--include-tools",
        action="store_true",
        help="Include compact one-line summaries of tool calls/results (not full payloads).",
    )
    parser.add_argument(
        "--project-dir",
        help="Override the ~/.claude/projects/<mangled-cwd> directory to look in.",
    )
    args = parser.parse_args()

    project_dir = default_project_dir(args.project_dir)

    if args.list:
        cmd_list(project_dir)
        return

    session_path = resolve_session_path(args.session, project_dir)
    session_id = session_path.stem

    stats = Stats()
    events = load_events(session_path, stats)
    body = render_transcript(events, stats, include_tools=args.include_tools)
    front_matter = build_front_matter(session_path, session_id, stats, args.include_tools)

    title = f"# Transcript: {session_id}\n"
    output = front_matter + "\n\n" + title + "\n" + body + "\n"

    if args.out:
        out_path = Path(args.out).expanduser()
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(output, encoding="utf-8")
        print(f"Wrote {out_path} ({len(output):,} bytes)", file=sys.stderr)
    else:
        sys.stdout.write(output)

    if stats.lines_parse_errors:
        print(f"warning: {stats.lines_parse_errors} line(s) failed to parse as JSON", file=sys.stderr)
    if stats.unknown_line_shapes:
        print(f"warning: {stats.unknown_line_shapes} line(s)/block(s) had an unrecognized shape "
              f"and were skipped", file=sys.stderr)
    if stats.unknown_block_types:
        for k, v in sorted(stats.unknown_block_types.items()):
            print(f"warning: {v} unrecognized content block(s) of type {k!r} were skipped", file=sys.stderr)


if __name__ == "__main__":
    main()
