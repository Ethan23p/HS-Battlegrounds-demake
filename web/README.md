# The frontend

One page, driving `bg-sim` compiled to WebAssembly. See
[ADR 0010](../docs/adr/0010-the-frontend-is-a-web-page.md).

```sh
python3 scripts/build_web.py
```

That writes two build outputs, neither committed:

- **`bg.html`** — the page with the engine inlined. One file, ~260 KB. Open it directly
  from disk; no server, no network, nothing to install. This is the thing to put on a
  phone.
- **`bg_wasm.wasm`** — the engine on its own, for local development.

`index.html` is the source, and prefers the inlined engine when it is there, falling back
to fetching `bg_wasm.wasm` beside it. Editing the page during development therefore wants
a server, because browsers refuse to fetch WebAssembly over `file://`:

```sh
python3 -m http.server -d web     # then open http://localhost:8000
```

Rebuild after any change to `crates/` — the inlined engine is a snapshot, not a link.
