# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Development
bun run tauri dev          # full app dev mode (starts Vite + Tauri)
bun run dev                # Vite frontend only (port 1420)

# Build
bun run tauri build        # production .dmg bundle

# Frontend type checking
bun run check              # svelte-kit sync + svelte-check

# Tests (frontend only — no Rust test suite)
bun run test               # vitest watch
bun run test:run           # vitest single run
bun run test:coverage      # vitest + istanbul coverage → ./coverage/

# Version bump (updates package.json, tauri.conf.json, Cargo.toml, commits + tags)
./scripts/bump.sh patch|minor|major
```

## Architecture

This is a Tauri v2 desktop app with two binaries:

1. **`symptom-test-tracker`** — the main app (default-run in Cargo.toml)
2. **`mcp-server`** — a standalone MCP server that Claude/other AI clients connect to via stdio

### Frontend (`src/`)

SvelteKit in SPA/static mode (adapter-static, fallback to `index.html`). Svelte 5 with runes (`$state`, `$derived`, `$effect`). No router — navigation is a single `currentView` state variable in `src/routes/+page.svelte` that swaps between views.

- `src/lib/types.ts` — all shared TypeScript types
- `src/lib/db.ts` — thin wrappers around `invoke()` calls to Tauri commands
- `src/lib/utils.ts` — date helpers, flag styling, and the `LAB_PANELS` array (built-in panel/test definitions with reference ranges)
- `src/lib/glossary.ts` — lab test explanations for the Glossary panel
- `src/lib/views/*.svelte` — one file per view, rendered by `+page.svelte`

The Glossary panel appears inline (beside content) when viewport is wide enough (`CONTENT_MAX + GLOSSARY_WIDTH + 48px`), otherwise as an overlay.

### Rust backend (`src-tauri/src/`)

- `db.rs` — opens SQLite at `~/Library/Application Support/symptom-test-tracker/tracker.db`, runs `PRAGMA WAL + foreign_keys`, runs inline `CREATE TABLE IF NOT EXISTS` migrations on startup, seeds default symptoms if table is empty
- `lib.rs` — Tauri app builder; registers all commands via `invoke_handler!`
- `commands/labs.rs`, `commands/symptoms.rs`, `commands/export.rs`, `commands/settings.rs` — Tauri command implementations; each receives `State<Database>` and locks `db.conn` (Mutex)

### MCP server (`src-tauri/src/bin/mcp_server.rs`)

Standalone binary using `rmcp` v0.16 with stdio transport. Reads/writes the same SQLite file as the main app. Access is gated by `settings` table rows:
- `mcp_enabled = 'true'` — read tools work
- `mcp_write_enabled = 'true'` — `insert_lab_session` tool works

Tools: `get_recent_labs`, `get_abnormal_labs`, `get_symptom_history`, `get_trends`, `get_daily_summaries`, `insert_lab_session`.

## Key Gotchas

- **Two binaries**: `Cargo.toml` has `default-run = "symptom-test-tracker"` so `cargo run` works without `--bin`
- **rmcp version**: uses `rmcp = "0.16"` with `Implementation::from_build_env()` (v0.16 has many required fields — don't construct `Implementation` manually)
- **schemars**: rmcp 0.16 uses `schemars` 1.x; Tauri internally uses 0.8/0.9 — both coexist fine in Cargo.toml
- **Svelte 5 state init**: compute values before passing to `$state()` — `$state((() => ...)())` is invalid syntax
- **Tauri plugins config**: `tauri.conf.json` `"plugins"` section must be `{}` (empty object), not `{"dialog": {}}` — the dialog plugin rejects a map value
- **Version bumps**: must update `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` in sync — use `./scripts/bump.sh`
- **Frontend tests only**: vitest covers `src/lib/*.ts` utilities; `src/lib/db.ts` is excluded from coverage (requires Tauri runtime)
