# Spotlight for Ubuntu/Linux - Product & Engineering Plan

## 1) Goal
Build a fast, Spotlight-like launcher and search experience for Ubuntu/Linux with a polished, rich UI and high-quality keyboard-first workflows.

## 2) Product Principles
- Instant response: search updates as user types, target 10-50ms query latency.
- Keyboard-first: every action accessible without mouse.
- Rich visual feedback: smooth animations, preview panel, clear hierarchy, strong typography.
- Reliable indexing: incremental updates, crash-safe index state, resumable background indexing.
- Linux-native behavior: works well on Ubuntu GNOME, with clear Wayland/X11 behavior.

## 3) Recommended Tech Stack
### Core (Rust)
- Runtime: `tokio`
- Search index: `tantivy`
- Metadata DB/state: `rusqlite`
- File watcher: `notify`
- Text extraction adapters: `poppler` (PDF), optional external tools (`pandoc`, `catdoc`, `ripgrep` fallback)

### UI (choose one)
1. GTK4 + libadwaita (`relm4`) - best Linux-native feel.
2. Tauri + Svelte - best for highly custom animation and visual richness.

Recommendation for this project goal: **Tauri + Svelte** for richer visual design and motion control.

## 4) System Architecture
- `daemon/indexer`
  - Crawls configured paths
  - Builds and updates search index incrementally
  - Emits status events (index progress, errors)
- `core/search`
  - Parses query intent (file/app/calc/command)
  - Executes fuzzy + ranked search
  - Returns top N in latency budget
- `core/actions`
  - Open file/folder/url
  - Launch app from `.desktop`
  - Copy path, reveal in folder, quick actions
- `ui/app`
  - Search overlay window
  - Results list + right-side preview pane
  - Category filters and action strip

## 5) UI/UX Scope (Rich Experience)
### Visual Direction
- Glass-like overlay with soft shadows and subtle background blur.
- Distinct typography scale: title, subtitle, metadata, keyboard hints.
- Color tokens and spacing system in design variables.
- Animated entry/exit, list stagger, hover/focus transitions.

### Core Interactions
- Global invoke hotkey (with Wayland fallback options).
- Type-to-search with immediate feedback.
- Arrow key navigation + enter to open.
- `Tab`/`Right Arrow` to open action panel for selected item.
- Preview pane for files/apps (metadata, recent usage, quick actions).
- Inline calculator and unit conversions.

### Accessibility
- Full keyboard navigation.
- Screen-reader labels for controls.
- High-contrast mode token set.

## 6) Feature Roadmap
## Phase 0: Project Setup (Week 1)
- Create Rust workspace and crate boundaries.
- Add CI (build + tests + lint).
- Define config format (`~/.config/spotlight/config.toml`).
- Create empty UI shell and IPC contract.

Deliverable:
- App launches, empty search box visible, daemon starts.

## Phase 1: Search MVP (Weeks 2-3)
- Index filenames + paths + app desktop entries.
- Basic fuzzy search and ranking.
- Open file/app actions.
- Results list with keyboard navigation.

Deliverable:
- Functional launcher for apps/files with responsive search.

## Phase 2: Rich UI + Preview (Weeks 4-5)
- Final visual system (theme tokens, typography, animation).
- Preview pane (file metadata, app details, last opened).
- Action panel (open, reveal, copy path, open with...).
- Recents and usage-based boost.

Deliverable:
- Polished UI with preview and rich interactions.

## Phase 3: Indexing Quality (Week 6)
- Incremental watch-based indexing.
- File type adapters (PDF + text formats).
- Ignore rules and include/exclude path controls.
- Reindex controls and health diagnostics.

Deliverable:
- Stable long-running indexing and improved relevance.

## Phase 4: Linux Integration (Week 7)
- Tray integration and settings page.
- Wayland/X11 activation strategy.
- Startup-on-login support.
- Packaging (`.deb`, AppImage).

Deliverable:
- Ubuntu-friendly install/run experience.

## Phase 5: Advanced Features (Week 8+)
- Plugins/providers (browser history, clipboard, terminal history).
- Command palette mode.
- Semantic ranking enhancements.
- Optional AI-powered action suggestions.

## 7) Functional Requirements
- Search domains:
  - Apps (`.desktop`)
  - Files/folders
  - Recent items
  - Calculator expressions
- Query behavior:
  - Typo tolerance
  - Prefix boost
  - Path/token matching
- Actions:
  - Open
  - Reveal in folder
  - Copy path
  - Open with selected app
- Settings:
  - Include/exclude directories
  - Ignore file extensions
  - Reindex now
  - Toggle result categories

## 8) Non-Functional Requirements
- Keystroke-to-result p95 < 50ms for warm index.
- Cold start UI < 500ms.
- Indexing must not block UI.
- Crash-safe index updates.
- Memory target < 350MB on typical developer machine.

## 9) Wayland/X11 Strategy
- X11: support global hotkey directly.
- Wayland: provide explicit alternatives:
  - GNOME extension integration (recommended for full global invoke)
  - Tray click / app shortcut fallback
  - User guidance in setup wizard

## 10) Project Structure
```text
spotlight/
  Cargo.toml
  crates/
    core-search/
    core-indexer/
    core-actions/
    shared-types/
  apps/
    desktop-ui/
  daemon/
    spotlightd/
  configs/
  docs/
```

## 11) Testing Plan
- Unit tests:
  - Tokenization
  - Ranking logic
  - Query parser
- Integration tests:
  - Index build and incremental updates
  - Action execution safety checks
- UI tests:
  - Keyboard navigation flows
  - Focus/selection behavior
- Performance tests:
  - Large directory indexing benchmark
  - Query latency benchmark

## 12) Risks and Mitigations
- Wayland hotkey limitations:
  - Mitigation: fallback activation methods + optional GNOME extension.
- Index bloat/perf degradation:
  - Mitigation: ignore rules, size caps, periodic compaction.
- File type extraction complexity:
  - Mitigation: plugin-style extractors, staged rollout.

## 13) First Build Tasks (Immediate)
1. Scaffold workspace with 4 Rust crates + daemon + UI app.
2. Define IPC schema for `search(query) -> results[]`.
3. Implement app indexing from `.desktop` files.
4. Implement filename indexing for user home directory.
5. Ship basic UI list with keyboard selection + open action.

## 14) Definition of Done (MVP)
- User can invoke launcher quickly.
- User can find installed apps and files by typing partial names.
- User can open selected result from keyboard.
- UI is visually polished (animation, spacing, hierarchy) and stable.
- Basic settings and reindex controls exist.
