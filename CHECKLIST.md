# PoeScout - Implementation Checklist

## Phase 1: Foundation + Affix Lookup
- [x] Rust toolchain installed (rustc 1.95.0, cargo 1.95.0)
- [x] pnpm installed (11.0.8)
- [x] Cargo workspace (`Cargo.toml` — 6 crates + src-tauri)
- [x] pnpm workspace (`pnpm-workspace.yaml` — ui + shared packages)
- [x] `poe-core` crate — types, config, error
  - [x] `Mod`, `BaseItem`, `SearchQuery`, `SearchResult` types
  - [x] `AppConfig` with auto-detect Client.txt path
  - [x] `PoeError` with serde::Serialize for Tauri IPC
- [x] `poe-data` crate — data engine
  - [x] SQLite schema (`mods`, `mod_stats`, `mod_spawn_weights`, `base_items`, FTS5)
  - [x] `Database` — open, migrate, batch insert, search queries
  - [x] `MemIndex` — HashMap indexes (mod-by-ID, mods-by-tag)
  - [x] `DataEngine` — orchestrates DB + indexes + ingestion
  - [x] RePoE JSON download (`mods.json`, `base_items.json`)
  - [x] Ingestion pipeline (serde parse -> batch SQLite insert -> FTS5 rebuild)
- [x] Tauri 2.x app (`src-tauri`)
  - [x] `AppState` with `DataEngine` managed state
  - [x] IPC commands: `search_mods`, `search_bases`, `get_mod_by_id`
  - [x] Windows icon for resource generation
  - [x] `tauri.conf.json` configured
- [x] SolidJS frontend (`packages/ui`)
  - [x] Vite 6 + vite-plugin-solid + UnoCSS
  - [x] PoE-themed dark color palette (gold accents, dark surfaces)
  - [x] `ModSearch` — FTS5 search, domain/type filters, debounced, result table
  - [x] `BaseSearch` — name search, item class filter, result table
  - [x] Typed Tauri IPC wrappers (`lib/tauri.ts`)
  - [x] Tab navigation (Affixes | Bases)
- [x] Shared TS types package (`packages/shared`)
- [x] `scripts/fetch-repoe.sh`
- [x] `cargo check` passes clean
- [x] `pnpm install` complete
- [ ] End-to-end test: `pnpm tauri dev` launches, data ingests, search works

## Phase 2: PoB Integration
- [x] `poe-pob` crate implementation
  - [x] `codec.rs` — base64 decode -> zlib inflate -> XML parse (BuildSummary with class/ascendancy/level/stats)
  - [x] `launch.rs` — find + launch PoB.exe (3 common paths + PATH search)
- [x] Tauri commands: `decode_pob_code`, `detect_pob`, `launch_pob_app`
- [x] Frontend: PobPanel — code input, summary card (class, ascendancy, level, main skill, stat boxes), "Open in PoB" button
- [x] Auto-detect PoB install path
- [x] Tab added to App (Affixes | Bases | PoB)
- [x] `cargo check` passes clean

## Phase 3: Map Timer
- [ ] `poe-maps` crate implementation
  - [ ] `tail.rs` — async file tail via `notify` crate
  - [ ] `parser.rs` — regex: area transitions, deaths
  - [ ] `session.rs` — state machine: IDLE -> IN_MAP -> COMPLETE
  - [ ] `history.rs` — SQLite persistence (`map_runs` table)
- [ ] Tauri commands + event emitters
- [ ] Frontend: live timer + history table (avg clear, deaths/map, maps/hour)
- [ ] Auto-detect Client.txt path

## Phase 4: Stash / Currency Tracking
- [ ] `poe-stash` crate implementation
  - [ ] `oauth.rs` — PoE OAuth 2.1 PKCE flow
  - [ ] `stash_api.rs` — GET /stash endpoints (rate-limited 45/60s)
  - [ ] `tracker.rs` — currency snapshots, per-hour calc
- [ ] `poe-pricing` crate implementation
  - [ ] `ninja.rs` — poe.ninja HTTP fetch + rate limiting
  - [ ] `cache.rs` — in-memory with 5-min TTL
- [ ] Currency aggregation (chaos, divines, harvest juice, hiveblood)
- [ ] SQLite snapshots every 5 min
- [ ] Frontend: stash viewer + currency charts + per-hour rates

## Phase 5: Overlay Mode + Polish
- [ ] Transparent overlay window (`always_on_top`, `decorations: false`)
- [ ] Global hotkey toggle (Ctrl+Space default, configurable)
- [ ] Click-through mode (`WS_EX_TRANSPARENT` on Windows)
- [ ] Compact overlay widgets (lookup bar, timer, currency ticker)
- [ ] System tray with quick actions
- [ ] Settings page (league, paths, hotkeys, OAuth)
- [ ] Auto-updater (Tauri plugin + GitHub Releases)

## Phase 6 (Future): WASM Plugin System
- [ ] Deferred — clean crate boundaries ready for it
