# PoeScout Roadmap

## Phase 1: Affix/Base Lookup
- [x] Ingest repoe-fork mods.json + base_items.json into SQLite
- [x] FTS5 search for mods
- [x] Base item detail view with affix display
- [x] Human-readable mod text in ModSearch
- [x] Implicit tag pills with colors
- [ ] Unique item lookup (PARKED — no data source)

## Phase 2: PoB Integration
- [x] Decode build codes (base64 -> zlib -> XML) — `poe-pob/codec.rs`
- [x] Launch PoB from app — `poe-pob/launch.rs`
- [ ] UI: paste/input build code
- [ ] UI: display parsed build summary
- [ ] Wire one-click PoB launch button

## Phase 3: Overlay Mode
- [x] Transparent always-on-top window mode (Tauri window config)
- [x] Global hotkey to toggle overlay vs full window
- [x] Minimal overlay UI (compact lookup)
- [x] Item capture hotkey (Ctrl+Q) — hover item in PoE, press hotkey, auto-lookup base
- [x] Keybinds reference panel
- [x] PoE window-relative sizing & centering (Win32 FindWindowW)
- [x] 3-state F2 toggle (enter/hide/show) with Esc to fully exit

## Phase 4: Map Timer
- [ ] Tail Client.txt via fs events
- [ ] Regex parse area transitions
- [ ] State machine for map enter/exit timing
- [ ] UI: timer display + map history

## Phase 5: Stash & Currency Tracker
- [ ] OAuth 2.1 PKCE for GGG stash API
- [ ] Fetch stash tabs
- [ ] poe.ninja price integration (`poe-pricing` crate)
- [ ] Currency-per-hour calculations
- [ ] UI: stash viewer + currency/hr dashboard
