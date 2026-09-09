#!/usr/bin/env python3
"""Verify every quotation in the provenance ledger against its transcript.

The ledger's whole value is that its quotes are Ethan's actual words. That is a
claim a script can check, so it should be: this reads every entry in
`docs/vision.md`, finds the transcript it cites, and asserts the quoted text
appears there verbatim. Whitespace is normalised on both sides -- the ledger
rewraps long quotes -- but nothing else is.

Exit status 1 on any mismatch. Run it whenever the ledger or a transcript
changes.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / "docs" / "vision.md"
TRANSCRIPTS = ROOT / "docs" / "transcripts"

# `- **P6.1** [typed] -- 0002 L368`
ENTRY = re.compile(r"^- \*\*(P[\d.]+)\*\*\s+\[([a-z]+)\]\s+—\s+(\d{4})\s+L(\d+)")


def squash(text: str) -> str:
    """Collapse all runs of whitespace, so rewrapping cannot cause a failure."""
    return " ".join(text.split())


def transcript_for(number: str) -> Path:
    matches = sorted(TRANSCRIPTS.glob(f"{number}-*.md"))
    if len(matches) != 1:
        raise SystemExit(f"no unique transcript for {number}: {matches}")
    return matches[0]


def entries(ledger: str):
    """Yield (id, mode, transcript number, line, quote) for each ledger entry."""
    lines = ledger.split("\n")
    for i, line in enumerate(lines):
        match = ENTRY.match(line)
        if not match:
            continue
        quote = []
        for follow in lines[i + 1 :]:
            stripped = follow.strip()
            if stripped.startswith(">"):
                quote.append(stripped[1:].strip())
            elif not stripped:
                continue
            else:
                break
        yield (*match.groups(), " ".join(quote))


def main() -> int:
    ledger = LEDGER.read_text()
    cache: dict[str, str] = {}
    checked = failed = 0

    for pid, mode, number, line, quote in entries(ledger):
        checked += 1
        if not quote:
            print(f"FAIL {pid}: no quotation follows the entry")
            failed += 1
            continue
        if number not in cache:
            cache[number] = squash(transcript_for(number).read_text())
        if squash(quote) not in cache[number]:
            print(f"FAIL {pid} ({number} L{line}, {mode}): not found verbatim")
            print(f"     {squash(quote)[:120]}...")
            failed += 1

    print(f"\n{checked} quotations checked, {failed} not found verbatim.")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
