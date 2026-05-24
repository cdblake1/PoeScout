# Changelog

## Unreleased

### Added
- **Advanced Map Tracking & Sessions (Phase 6, Pass 1)** — richer Client.txt parsing plus automatic farming sessions with currency/hour:
  - Internal area-id capture (`Generating level N area "Id"`) → canonical map identity, real map tier, and an `AreaType` classifier (map/town/hideout/hub/…) that fixes Kingsmarch, The Rogue Harbour, and Azurite Mine being mis-counted as map runs
  - Instance-endpoint tracking (`Connecting to instance server at`) so town-portalling back into a map *resumes* the same run instead of splitting it; idle/hideout time attributed to runs; sub-areas (Vaal/lab/abyss) no longer split a run
  - League-mechanic encounter detection from NPC dialogue (`data/encounters.json`, title-agnostic `by_npc` + exact `by_quote`), stored per run and shown as chips
  - Death/level-up attribution to your character (set in Settings); removed the dead `You have died` branch (real deaths are `has been slain`)
  - Automatic sessions: stash snapshot on the first map out of town, snapshot again after an idle timeout (default 15 min); profit = Δchaos; currency/hour over *active map time* (idle excluded); open sessions survive an app restart
  - SQLite: `user_version` migration framework; new `map_runs` columns (area_id, area_type, map_tier, instance_id, league, session_id, hideout_secs) plus `map_sessions` and `map_encounters` tables
  - UI: Sessions panel (profit, c/hr, maps, active time) and encounter chips in the Maps tab; Settings character field
  - `save_settings` now shallow-merges so panels no longer clobber each other's keys
  - New commands: `get_map_sessions`, `get_session_detail`, `set_tracked_character`; new events: `session-start`, `session-end`
  - 35 `poe-maps` unit tests
- **Stash & Currency Tracker** — New "Stash" tab for tracking stash value and item prices. Features:
  - poe.ninja integration: fetches prices for currency, fragments, div cards, uniques, gems, fossils, etc.
  - In-memory price cache with 5-minute TTL (auto-refreshes when stale)
  - POESESSID authentication: paste your session cookie to connect to GGG's stash API
  - Credentials stored securely in app data directory (`credentials.json`), auto-loaded on startup
  - Stash snapshot: fetches all tabs, prices every item, shows total chaos/divine value
  - Tab breakdown: per-tab value summary sorted by worth
  - Top 20 most valuable items table with PoE-style rarity coloring
  - Chaos/hour calculation from snapshot deltas over time
  - Standalone price lookup: type any item name to see its poe.ninja price instantly
  - Rate-limited GGG API client (min 1.1s between requests) to avoid 429s
  - `poe-pricing` crate: NinjaClient, PriceCache, PricingEngine
  - `poe-stash` crate: StashClient, item-to-price matcher, StashTracker
  - 7 Tauri commands: set_session_id, get_stash_tabs, take_stash_snapshot, refresh_prices, get_price, save_credentials, load_credentials
- **Map Timer** — New "Maps" tab tracks map runs in real time by parsing PoE's Client.txt log. Features:
  - Auto-starts on app launch (no start/stop button)
  - Human-readable map names from "You have entered" log lines
  - Area level capture from "Generating level" log lines
  - Live elapsed timer with map name, tier, and death count
  - Session stats: total runs, avg duration, maps/hour, total deaths
  - Scrollable history table of completed runs
  - SQLite persistence across sessions (map_runs table in poescout.db)
  - `poe-maps` crate: regex parser, area classifier (hideouts + towns), state machine, file watcher, SQLite database
  - 3 Tauri commands: get_tracker_state, get_map_history, get_map_stats
  - 3 Tauri events: state-change, map-complete, death
- **Timer Mini-Window** — Separate always-on-top floating bar (280x36) showing live map status:
  - Zone-gated visibility: only shows after entering a valid zone (not at main menu/character selection)
  - Auto-hides when PoE loses focus, reappears when PoE is focused
  - Debounced show (750ms) to prevent flicker during character load
  - Draggable when F2 overlay is open, click-through otherwise (via `startDragging()` API)
  - Shows: status dot (green=map, yellow=idle) + zone name + MM:SS timer + death count
- **Overlay Tabs** — F2 overlay now has Bases and Maps tabs in the drag bar
- **Overlay Position Persistence** — Overlay remembers position and size across F2 toggles (localStorage). Reset button (↺) re-centers on PoE.
- **PoE Focus Detection** — `is_poe_foreground` Rust command (Win32 `GetForegroundWindow`)
- **Item Capture Hotkey** — Press Ctrl+Q while hovering an item in PoE to automatically look up its base item. Simulates Ctrl+C (via `enigo`), reads clipboard (via `arboard`), parses PoE item text, and navigates to BaseDetail. Supports Normal, Magic (substring match), Rare, and Unique items.
- **Keybinds Panel** — Click "Keybinds" button in header to see a reference of all keyboard shortcuts (Ctrl+Q, F2, Esc).
- **Overlay Mode** — Press F2 to toggle a compact, semi-transparent always-on-top overlay for in-game lookup. Only activatable when PoE is running. Resizable and draggable.
- Tauri 2 capabilities file (`src-tauri/capabilities/default.json`) with window, global-shortcut, and dragging permissions
- `tauri-plugin-global-shortcut` for F2 hotkey registration
- Third Tauri window (timer) in `tauri.conf.json` — decorationless, transparent, always-on-top, skip-taskbar
- Window routing in `index.tsx` — main/overlay/timer windows share one SolidJS entry point, render different components based on window label

### Parked
- **Unique Item Lookup** — Finding which unique items share a given affix is not feasible with current repoe-fork data. `uniques.json` has metadata only (name, item_class, visual_identity) with no mod/affix data, and there is no unique→mod mapping available. Revisit when a better data source becomes available.

### Changed
- **Overlay PoE-relative sizing & centering** — Overlay now sizes to ~40% width / ~60% height of the PoE window (min 480x600) and centers on it, instead of using fixed dimensions centered on screen. Uses new `get_poe_window_rect` Rust command (Win32 `FindWindowW` + `GetWindowRect`).
- **F2 toggle** — F2 toggles between overlay and standalone mode (simple 2-state). Previous 3-state cycle (enter → hide → show) removed to avoid Tauri transparency bug.
- **Overlay exit restores PoE focus** — F2 close hides overlay and calls `focus_poe_window` to return focus to PoE.
- **Ctrl+Q uses `enterOverlay()`** — Capture shortcut enters overlay without toggling, preserving hide/show state if already in overlay.
- All `implicit_tags` now display as pills (no filtering) — curated tags use hand-picked colors, unknown tags get auto-generated colors via deterministic hue hashing (`BaseDetail.tsx`)
- Expanded tier rows now show PoE-readable `text` (e.g. `+(8-12) to Strength`) instead of raw stat IDs

### Fixed
- **Overlay first-launch sizing** — `enterOverlay()` now calls `win.unminimize()` before resizing, ensuring the window is in a normal state before applying overlay dimensions. Fixes issue where first F2 press didn't cover the screen properly.
- **Overlay exit stealing focus from PoE** — `focus_poe_window` now uses `AttachThreadInput` trick (standard Win32 workaround) to reliably call `SetForegroundWindow`.
- `scripts/fetch-repoe.sh` URL updated from legacy `brather1ng/RePoE` to `repoe-fork.github.io` to match runtime data source

### Removed
- Dead `tags` field from `RawMod` struct in `ingest.rs` — upstream `mods.json` has no `tags` field, only `implicit_tags`
