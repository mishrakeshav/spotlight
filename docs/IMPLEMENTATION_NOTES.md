# Implementation Notes

Current status: Phase 0 + Phase 1 complete, Phase 2 started.

## Completed
- Rust workspace with modular crates and binaries.
- Shared query/result contract in `shared-types`.
- Linux indexer in `core-indexer`:
  - `.desktop` app discovery from standard locations.
  - Filesystem filename indexing from `$HOME`.
  - Hidden-file filtering and max file cap.
- Ranked search in `core-search`:
  - Exact, prefix, contains, subtitle, and token hit boosts.
- Action execution in `core-actions`:
  - App launch via desktop `Exec` command.
  - File/folder open via `xdg-open`.
- `spotlightd` daemon flow:
  - Build index -> execute query -> print ranked results.

## Phase 2 (in progress)
- Added Tauri + Svelte app shell under `apps/desktop-ui`.
- Added Tauri Rust backend under `apps/desktop-ui/src-tauri`.
- Added IPC commands:
  - `search(query, limit)`
  - `open_result(id)`
- Added rich overlay UI baseline:
  - search input,
  - keyboard navigation,
  - result list,
  - preview panel,
  - action button.

## Verification
- Frontend build passes: `npm run build`.
- Core Rust crates pass tests/checks.
- Tauri crate currently blocked by missing system GTK/WebKit development libraries.

## Required system packages (Ubuntu)
Install these to build and run Tauri desktop app:

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential pkg-config \
  libgtk-3-dev libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev librsvg2-dev
```

## Next
1. Install missing Linux packages above.
2. Run `cargo check -p desktop-ui-tauri`.
3. Run `cd apps/desktop-ui && npm run tauri:dev`.
4. Continue UI polish: staged animations, command strip, category filters.
