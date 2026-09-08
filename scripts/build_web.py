#!/usr/bin/env python3
"""Build the web frontend.

Produces two things from `web/index.html`:

  web/bg_wasm.wasm  the engine, beside the page, for local development
  web/bg.html       the same page with the engine inlined -- one file, no
                    server, no network, which is what makes delta 4
                    (offline-capable) true rather than aspirational

The page prefers the inlined engine and falls back to fetching the sibling
`.wasm`, so the same source works both ways.
"""

from __future__ import annotations

import base64
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
WEB = ROOT / "web"
TARGET = "wasm32-unknown-unknown"
PROFILE = "web"
ARTEFACT = ROOT / "target" / TARGET / PROFILE / "bg_wasm.wasm"
MARKER = '<script>\n"use strict";'


def build_engine() -> pathlib.Path:
    subprocess.run(
        ["cargo", "build", "-p", "bg-wasm", "--target", TARGET, "--profile", PROFILE],
        cwd=ROOT,
        check=True,
    )
    if not ARTEFACT.exists():
        sys.exit(f"cargo reported success but {ARTEFACT} is missing")
    return ARTEFACT


def main() -> None:
    wasm = build_engine()
    WEB.mkdir(exist_ok=True)
    shutil.copy2(wasm, WEB / "bg_wasm.wasm")

    source = (WEB / "index.html").read_text(encoding="utf-8")
    if MARKER not in source:
        sys.exit("web/index.html no longer has the script marker this build expects")

    encoded = base64.b64encode(wasm.read_bytes()).decode("ascii")
    inlined = source.replace(
        MARKER,
        f'<script type="application/wasm-base64" id="wasm-b64">{encoded}</script>\n\n{MARKER}',
        1,
    )
    (WEB / "bg.html").write_text(inlined, encoding="utf-8")

    print(f"engine   {wasm.stat().st_size / 1024:7.1f} KB")
    print(f"bg.html  {len(inlined.encode()) / 1024:7.1f} KB  (self-contained)")


if __name__ == "__main__":
    main()
