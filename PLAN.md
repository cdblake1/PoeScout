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

## Phase 4: PoB Integration
- [x] Decode build codes (base64 -> zlib -> XML) — `poe-pob/codec.rs`
- [x] Launch PoB from app — `poe-pob/launch.rs`
- [x] UI: paste/input build code — `PobPanel.tsx`
- [x] UI: display parsed build summary (class, ascendancy, level, main skill, stats)
- [x] Wire one-click PoB launch button (auto-detects PoB path)

## Phase 5: Stash & Currency Tracker
- [x] poe.ninja price integration (`poe-pricing` crate) — NinjaClient, PriceCache with 5-min TTL
- [x] POESESSID auth for GGG stash API (`poe-stash` crate) — rate-limited client
- [x] Fetch stash tabs + items with item-to-price matching
- [x] Portfolio snapshot — total chaos/divine value, tab breakdown, top 20 items
- [x] Currency-per-hour calculations (from snapshot deltas)
- [x] Credentials persistence (`credentials.json` in app data dir)
- [x] UI: Stash tab with credentials, portfolio, tab breakdown, top items, price lookup
- [ ] OAuth 2.1 PKCE (deferred — POESESSID approach for now)
