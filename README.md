# Spotlight (Linux)

Spotlight-like launcher for Ubuntu/Linux, built with Rust + Tauri + Svelte.

## Features

- Search installed desktop apps and files quickly
- Keyboard-first navigation
- Preview pane and action strip
- Open/focus app, reveal file location, copy path
- Linux bundles via CI (`.deb` and `.AppImage`)

## Project Structure

```text
spotlight/
  crates/
    core-search/
    core-indexer/
    core-actions/
    shared-types/
  daemon/spotlightd/
  apps/desktop-ui/
```

## Requirements (Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential pkg-config \
  libgtk-3-dev libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev librsvg2-dev \
  wmctrl
```

Also install:
- Rust toolchain (via rustup)
- Node.js 20+

## Run Locally

```bash
cd apps/desktop-ui
npm ci
npm run tauri:dev
```

## Build Local Bundles

```bash
cd apps/desktop-ui
npm ci
npm run tauri:build -- --bundles deb,appimage
```

Output artifacts:
- `apps/desktop-ui/src-tauri/target/release/bundle/deb/*.deb`
- `apps/desktop-ui/src-tauri/target/release/bundle/appimage/*.AppImage`

## GitHub Automation

GitHub Actions workflow: `.github/workflows/build-and-release.yml`

- On PR/push: builds and uploads CI artifacts
- On tag push `v*` (example `v0.1.0`): builds and attaches artifacts to GitHub Release

Create a release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## Notes

- App window focus for already-running apps uses `wmctrl` when available.
- On some Wayland setups, focusing existing windows may be limited by compositor security policy.
