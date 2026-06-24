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
- [x] **6.10** TraXile-parity detection — whole-line substring matcher
      (`LogEvent::SystemLine` + `encounters::by_line`/`match_line`) unlocking
      Mirage (`[Faridun]…`), Nameless Seer, Reflecting Mist, Simulacrum
      full-clear, twice-blessed; ported outcome-detail tables (Ultimatum/
      Ancestor/Sanctum win-loss) + Reliquary areas; timed sub-activities
      (`map_subactivities`, DB v5) for Vaal/Sanctum/lab-trial/… inside a map.
      UI surfacing of sub-activity/outcome detail deferred.
- [x] **6.9** detection accuracy + honesty + OCR + ToS strategy — chat-channel
      filter in the parser; removed misleading Maven/Envoy presence; ported
      TraXile's fuller area tables (Vaal/logbook-side/bosses/safehouse/lake);
      "not log-detectable" honesty note in the Maps UI; rate-limit-aware GGG API
      client (honors `X-Rate-Limit-*`/`Retry-After`, backs off on 429); opt-in
      OCR resource reader (`Windows.Media.Ocr` → `resource_snapshots`) with a
      Settings calibration panel — this completes **6.6b**. Also fixed: seed-based
      run identity (no more same-gateway map merges) + live in-progress run row.
      Research + guardrails in `docs/poe-mechanics-resources.md`.
- [x] **6.8** league mechanic tracking (TraXile-style) — per-map mechanic
      detection from `Client.txt` via two mechanisms: NPC dialogue
      (`data/encounters.json`, expanded to Heist/Sanctum/Ultimatum/Ancestor +
      beast-capture counting) and **new** area-entry detection
      (`areas::mechanic_for_area` → Legion/Simulacrum/Breachstone/Sanctum/Temple/
      logbook/lab/boss arenas). Stats via `get_mechanic_stats`; history filter via
      `get_map_history_by_mechanic`; **League Mechanics** table + clickable filter
      in the Maps tab. Reuses `map_encounters` (no migration). Detection-signal
      catalog + untrackables documented in `docs/poe-mechanics-resources.md`.
- [x] **6.6b** real OCR + calibration UI — DONE in 6.9 (`Windows.Media.Ocr`
      over a calibrated rectangle → `resource_snapshots`; Settings panel). Note
      below kept for history:
- [ ] ~~**6.6b** real OCR + calibration UI~~ — blocked on the spike result. If
      `non_black_fraction ≥ ~0.5` → `Windows.Media.Ocr` digit reader +
      drag-a-box calibration UI + session-boundary reads writing to
      `resource_snapshots`. If ≈ 0 → fall back to `Windows.Graphics.Capture`
      (D3D11 framepool, heavier).
- [ ] **6.7** items/hour view — see sub-spec below; **Tier-1 is queued as the
      next concrete deliverable**; Tier-2 / Tier-3 follow as new tabs / OCR land.
- [ ] **6.5d** two-tier item retention (deferred) — store priced items per
      snapshot for the last N snapshots; "items gained" diff view; new
      `snapshot_items` table + retention prune + UI panel. Defer until there's
      a concrete use-case.
- [ ] **Manual verification pass** — credential-gated flows (sessions, loot,
      net-worth chart, noise filters) are still unchecked in `CHECKLIST.md`.
      One live session would catch any regressions before more code lands.

### Phase 6.7 — Items/Hour view (spec)

**Why:** the net-worth sparkline and per-map loot total answer *how much* but not
*what*. Juicers picking between strategies need to know "am I getting more
chaos/hr from currency drops or div cards?", "how much Crystallised Lifeforce/hr
in this Harvest setup?", "Divines/hr in T17 vs T16?". The data for most of this
is already in `loot_items` — we just don't aggregate by item name.

**Data sources (three tiers, reuse what's already there):**
| Tier | Source                        | Examples                                                      | Status                                    |
|------|-------------------------------|---------------------------------------------------------------|-------------------------------------------|
| 1    | `loot_items` (inventory diff) | Chaos/Divine, Harvest Lifeforce, splinters, fossils, embers, Expedition artifacts, Hinekora's Locks | Data lives now; just needs aggregation    |
| 2    | New stash-tab fetches         | Bestiary (red beasts), Map tab, Currency tab                  | Needs new `fetch_stash_tab` path per type |
| 3    | `resource_snapshots` (OCR)    | Hiveblood, sulphite, Kingsmarch gold, Sanctum Sacred Water    | Blocked on 6.6b OCR shipping              |

#### 6.7a — Tier-1 aggregation + view ✅ SHIPPED
- [x] Backend: `get_items_per_hour(scope) -> Vec<ItemRate>` Tauri command in
      `src-tauri/src/commands/maps.rs`. `scope` is a tagged enum: `CurrentSession`,
      `Session { id }`, `LastSessions { n }`, `AllTime` (DateRange deferred — wasn't needed for the v1).
- [x] `poe-maps` query: `GROUP BY name` over `loot_items` filtered by scope-resolved run IDs; `active_secs = COALESCE(SUM(duration_secs), 0)` over the same set; divide-by-zero guarded.
- [x] `ItemRate { name, source: "inventory", stacks, drops, total_chaos, active_secs, items_per_hour, chaos_per_hour }`.
- [x] UI: **Items/hr** panel between Per-Map Stats and Recent Runs; columns Item · Stacks · Items/hr · Chaos/hr · Drops; scope toggle pills (`This session | Last 5 | All time`); empty state spells out the credential/character/loot requirements.
- [x] 5 DB unit tests (AllTime ordering, Session isolation, CurrentSession fallback, LastSessions window, zero-active-secs guard).
- [x] CHECKLIST entry under "Phase 6.7" for live verification.
- [x] (Polish) Pinned items (pin-to-top, localStorage); client-side header-click sort; DateRange scope (`substr(started_at,1,10) BETWEEN` filter + Custom-range date inputs).

#### 6.7b — Tier-2 special stash tabs (after 6.7a)
- [ ] Extend `StashClient::fetch_stash_tab` to handle Bestiary tab (and Map / Currency tabs if cheap). Reuse `diff_inventory` shape.
- [ ] New `loot_items.source` values: `"stash:bestiary"`, `"stash:map"`, etc. (column already in schema if we add it; otherwise migration v5).
- [ ] Beast rarity captured per row → enables "Red beasts/hr" filter.
- [ ] Filter chips in the Items/hr panel: `Currency | Fragments | Scarabs | Lifeforce | Beasts | All`.

#### 6.7c — Tier-3 OCR resources (after 6.6b)
- [ ] `get_items_per_hour` reads `resource_snapshots` deltas keyed `ocr:*` and folds them into the same `ItemRate` list with `source: "ocr:<key>"`.
- [ ] Calibration UI from 6.6b doubles as the "add a tracked OCR resource" UX — once calibrated, it shows up in the items/hr table automatically.

#### Open questions (decide during 6.7a)
- Rate denominator default: **active map time** (idle excluded, matches existing c/hr) vs session wall time vs world clock. Lean active map time for consistency; expose a toggle if users push back.
- Per-tier or unified view: ship 6.7a as one table sourced from `loot_items` only; add a `Source` column when 6.7b/c expand it. Don't gate 6.7a on the others.
