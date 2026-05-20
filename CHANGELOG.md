# Changelog

## Unreleased

### Added
- **Item Capture Hotkey** — Press Ctrl+Q while hovering an item in PoE to automatically look up its base item. Simulates Ctrl+C (via `enigo`), reads clipboard (via `arboard`), parses PoE item text, and navigates to BaseDetail. Supports Normal, Magic (substring match), Rare, and Unique items.
- **Keybinds Panel** — Click "Keybinds" button in header to see a reference of all keyboard shortcuts (Ctrl+Q, F2, Esc).
- **Overlay Mode** — Press F2 (or click "Overlay" button) to toggle a compact, semi-transparent always-on-top window (480x600) for in-game lookup. Searches both mods and bases. Escape or F2 exits overlay and restores the full window.
- Tauri 2 capabilities file (`src-tauri/capabilities/default.json`) with window and global-shortcut permissions
- `tauri-plugin-global-shortcut` for F2 hotkey registration

### Parked
- **Unique Item Lookup** — Finding which unique items share a given affix is not feasible with current repoe-fork data. `uniques.json` has metadata only (name, item_class, visual_identity) with no mod/affix data, and there is no unique→mod mapping available. Revisit when a better data source becomes available.

### Changed
- **Overlay PoE-relative sizing & centering** — Overlay now sizes to ~40% width / ~60% height of the PoE window (min 480x600) and centers on it, instead of using fixed dimensions centered on screen. Uses new `get_poe_window_rect` Rust command (Win32 `FindWindowW` + `GetWindowRect`).
- **F2 toggle** — F2 toggles between overlay and standalone mode (simple 2-state). Previous 3-state cycle (enter → hide → show) removed to avoid Tauri transparency bug.
- **Overlay exit minimizes** — Exiting overlay (F2 or Esc) now minimizes PoeScout instead of restoring previous size/position, getting completely out of the way so PoE has full focus. Eliminates resize/reposition race condition.
- **Esc exits overlay fully** — Esc now minimizes PoeScout and exits overlay mode, rather than just toggling.
- **Ctrl+Q uses `enterOverlay()`** — Capture shortcut enters overlay without toggling, preserving hide/show state if already in overlay.
- **Unified Overlay UI** — Overlay mode now renders the full app UI (tabs, BaseDetail, search) instead of a separate simplified view. Ctrl+Q capture automatically enters overlay mode. Esc exits overlay from anywhere.
- All `implicit_tags` now display as pills (no filtering) — curated tags use hand-picked colors, unknown tags get auto-generated colors via deterministic hue hashing (`BaseDetail.tsx`)
- Expanded tier rows now show PoE-readable `text` (e.g. `+(8-12) to Strength`) instead of raw stat IDs

### Fixed
- **Overlay first-launch sizing** — `enterOverlay()` now calls `win.unminimize()` before resizing, ensuring the window is in a normal state before applying overlay dimensions. Fixes issue where first F2 press didn't cover the screen properly.
- **Overlay exit stealing focus from PoE** — `focus_poe_window` now uses `AttachThreadInput` trick (standard Win32 workaround) to reliably call `SetForegroundWindow`. Added 100ms settle delay after window resize/move to avoid race condition. Debug logging added to both Rust and TypeScript sides.
- **Overlay → Standalone window disappearing** — Removed `win.hide()`/`win.show()` calls from overlay toggle, which triggered Tauri 2 bug #13530 on transparent windows. The window is now only resized/repositioned, never hidden, so it reliably returns to standalone mode on F2/Esc.
- `scripts/fetch-repoe.sh` URL updated from legacy `brather1ng/RePoE` to `repoe-fork.github.io` to match runtime data source

### Removed
- Dead `tags` field from `RawMod` struct in `ingest.rs` — upstream `mods.json` has no `tags` field, only `implicit_tags`
