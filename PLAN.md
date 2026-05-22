# PoeScout Roadmap

## Phase 1: Affix/Base Lookup
- [x] Ingest repoe-fork mods.json + base_items.json into SQLite
- [x] FTS5 search for mods
- [x] Base item detail view with affix display
- [x] Human-readable mod text in ModSearch
- [x] Implicit tag pills with colors
- [ ] Unique item lookup (PARKED — no data source)

## Phase 2: Overlay Mode
- [x] Transparent always-on-top window mode (Tauri window config)
- [x] Global hotkey to toggle overlay vs full window
- [x] Minimal overlay UI (compact lookup)
- [x] Item capture hotkey (Ctrl+Q) — hover item in PoE, press hotkey, auto-lookup base
- [x] Keybinds reference panel
- [x] PoE window-relative sizing & centering (Win32 FindWindowW)
- [x] 3-state F2 toggle (enter/hide/show) with Esc to fully exit

## Phase 3: Map Timer
- [x] Tail Client.txt via polling (500ms interval, seek-to-EOF on start)
- [x] Regex parse area transitions, deaths, level-ups
- [x] Two-event parsing: AreaLevelHint (level) + AreaChange (human name)
- [x] State machine (Stopped → Idle → InMap) with zone_name tracking
- [x] SQLite persistence for map runs (cross-session)
- [x] UI: Maps tab with live timer, session stats, history table
- [x] Separate timer mini-window (always-on-top, zone-gated visibility, PoE focus tracking)
- [x] Timer draggable in overlay mode (startDragging API), click-through otherwise
- [x] Overlay tabs (Bases | Maps) with full MapTimer view
- [x] Overlay position/size persistence (localStorage) with reset button
- [x] Debounced timer show to prevent flicker at character load
- [ ] Mirage/sub-zone run merging (deferred)

## Phase 4: Stash & Currency Tracker
- [ ] OAuth 2.1 PKCE for GGG stash API
- [ ] Fetch stash tabs
- [ ] poe.ninja price integration (`poe-pricing` crate)
- [ ] Currency-per-hour calculations
- [ ] UI: stash viewer + currency/hr dashboard

## Phase 5: PoB Integration
- [x] Decode build codes (base64 -> zlib -> XML) — `poe-pob/codec.rs`
- [x] Launch PoB from app — `poe-pob/launch.rs`
- [ ] UI: paste/input build code
- [ ] UI: display parsed build summary
- [ ] Wire one-click PoB launch button
