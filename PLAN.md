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
- [x] Portfolio snapshot — total chaos/divine value, all items with search/filter/pagination
- [x] Currency-per-hour calculations (from snapshot deltas)
- [x] Credentials persistence (`credentials.json` in app data dir)
- [x] UI: Stash tab with credentials, portfolio, items table, price lookup
- [x] Two-phase selective scan with progress bar, partial results on 429
- [x] Portfolio persistence across sessions with last-updated timestamp
- [x] Tab selection persistence, tab search by name/type
- [x] Rate limit cooldown timer, no stash API calls on startup
- [ ] OAuth 2.1 PKCE (deferred — POESESSID approach for now)

## Phase 6: Advanced Map Tracking & Currency/Hour (LARGELY DONE)

> **Status (2026-05-26):** 11 PRs merged on `main`. 6.1–6.5 + 6.6a shipped;
> only **6.6b (real OCR + calibration UI)** is left, gated on the user clicking
> the spike's **Test PoE capture** button in Settings (see `CHECKLIST.md` →
> Phase 6.6). Credential-gated flows (sessions, loot, net-worth) are still in
> the running manual-test list — see `CHECKLIST.md` for the carry-forward.

### What shipped (per sub-phase = one or more PRs)
- [x] **6.1** enhanced parsing + run model — internal area-id capture, instance-resume across town portals, `AreaType` classifier (fixes Kingsmarch/Rogue Harbour/Azurite Mine being mis-counted as maps), idle/hideout accounting, character attribution for death/level-up, quote-level encounter detection (`crates/poe-maps/data/encounters.json`). DB migration v1 adds `map_runs` columns + `map_sessions` + `map_encounters`.
- [x] **6.2** auto sessions + idle-excluded c/hr — auto stash-snapshot on first map out of town; auto-end after a configurable idle timeout; manual override; survives an app restart. Decision logic extracted as the pure `next_session_action` (unit-tested).
- [x] **6.3** per-map loot — `diff_inventory` pure fn (new items + stack deltas, MainInventory only), `fetch_character_inventory` (separate rate budget from stash), `loot_items` table + `loot_chaos` (DB v2), full src-tauri orchestration (map-start baseline + map-completion capture), Loot column on Recent Runs. **6.3c town-leak fix:** suspend-time inventory snapshot taken on InMap → Idle, so town purchases/crafts between maps don't leak into the prior map's loot.
- [x] **6.4** stats UI — `get_map_type_stats` aggregation (group by area id, fallback name; avg time / avg loot / Loot-per-hr / deaths) shown as a Per-Map Stats table. Inline-SVG `Sparkline` (no chart-lib dep) for three trends in a Trends panel: run duration, currency/hour by session, net worth.
- [x] **6.5** net-worth time series + noise filters + league decoupling — `portfolio_snapshots` (DB v3) recorded on every scan (manual + auto-session); retention cap (1000); chaos threshold + count threshold both gate the snapshot total; new `price_league` setting decouples pricing from the game league.
- [x] **6.6a** OCR spike + storage foundation — `capture_poe_test` command tries `PrintWindow + PW_RENDERFULLCONTENT` and reports `non_black_fraction`; Debug button in Settings; `resource_snapshots` (DB v4) + `MapTracker.record_resource_snapshot` / `get_resource_snapshots`.

**Plus, shipped along the way:**
- Timer-overlay polish: `focus: false` on the timer window stops the alt-tab focus-fight flicker; redundant per-tick `show()` guarded; 600 ms hide-debounce; default position top-left.
- **Clear** button on Recent Runs (`clear_map_history`).
- `save_settings` shallow-merges so panels don't clobber each other.
- Testability refactors: pure `merge_settings` + pure `next_session_action`, both unit-tested.
- Integration tests: legacy-v0 DB migration upgrade; Client.txt replay (fixture log → runs/types/encounters/town-resume/attribution).
- Auto-journal hook (personal `.claude/settings.local.json`, gitignored): PostToolUse/Bash matches `git commit` and reminds the agent to journal.
- **Working Rules** in `CLAUDE.md` (research agents → Sonnet; test eval + commit-first at every stopping point).

### Research (historical — kept as reference)
Deep-dived TraXile, Exile Diary, Exilence Next and parsed a 962k-line real
`Client.txt`. Three-tier resource-trackability model:
**Tier-1** encounter/count (Client.txt dialogue) ·
**Tier-2** item amounts (inventory/stash diff) ·
**Tier-3** side-panel resources (Hiveblood, sulphite, Kingsmarch gold) only via
on-screen OCR (6.6). Full per-mechanic catalog for 3.28 Mirage lives in
[`docs/poe-mechanics-resources.md`](docs/poe-mechanics-resources.md) — update
that doc each league.

### Remaining
- [ ] **6.6b** real OCR + calibration UI — blocked on the spike result. If
      `non_black_fraction ≥ ~0.5` → `Windows.Media.Ocr` digit reader +
      drag-a-box calibration UI + session-boundary reads writing to
      `resource_snapshots`. If ≈ 0 → fall back to `Windows.Graphics.Capture`
      (D3D11 framepool, heavier).
- [ ] **6.5d** two-tier item retention (deferred) — store priced items per
      snapshot for the last N snapshots; "items gained" diff view; new
      `snapshot_items` table + retention prune + UI panel. Defer until there's
      a concrete use-case.
- [ ] **Manual verification pass** — credential-gated flows (sessions, loot,
      net-worth chart, noise filters) are still unchecked in `CHECKLIST.md`.
      One live session would catch any regressions before more code lands.
