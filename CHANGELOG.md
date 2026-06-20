# Changelog

## Unreleased

### Added
- **League mechanic tracking (Phase 6.8)** — TraXile-style per-map mechanic detection + stats, all from `Client.txt` (no OCR, no new APIs, no DB migration — reuses the `map_encounters` model).
  - **Area-based detection ("Mechanism B")** — new `areas::mechanic_for_area`: entering a dedicated area with no NPC line (Legion `Domain of Timeless Conflict`, the 5 Simulacrum waves, Breachstone Domains, Sanctum floors, Temple of Atzoatl, Expedition logbook zones, Lab trials, pinnacle boss arenas, Abyssal Depths) records a `MapEncounter` on the current run (the parent map for sub-areas), deduped per category. Previously area names drove only run lifecycle, never mechanic tagging.
  - **NPC dialogue expansion ("Mechanism A")** — `data/encounters.json` grown from 15 NPCs/2 quotes to cover Heist rogues (9), Sanctum (Lycia), Ultimatum (Trialmaster), Ancestor (Navali), Memory Tear (Eagon), and all 6 Einhar beast-capture lines (`kind:"capture"`, yellow/red detail, counted individually). Ported from TraXile + Exile Diary Reborn (both MIT).
  - **Per-mechanic stats** — `get_mechanic_stats` (DB `GROUP BY category` over `map_encounters` ⋈ `map_runs`, collapsed to one row per (category, run) so capture-style multi-row mechanics don't inflate per-run figures) → `MechanicStat { category, encounter_count, maps_with, pct_of_maps, avg_duration_secs, avg_loot_chaos, total_deaths }`. New **League Mechanics** table in the Maps tab.
  - **History filter** — `get_map_history_by_mechanic` (WHERE EXISTS); clicking a mechanic row or a run's encounter chip filters Recent Runs to maps containing it, with an active-filter chip to clear/toggle.
  - **Detection-signal catalog** — `docs/poe-mechanics-resources.md` gains a per-mechanic signal table (NPC presence/quote vs area entry, exact strings, category) plus an explicit "untrackable — no Client.txt output" list (in-map Breach/Legion/Ritual/Metamorph/Abyss) and a "candidates pending live verification" list. Fixed the Simulacrum area name (`Hysteriagate`, not the earlier guess).
  - 12 new unit tests (area detection, state-machine tagging + dedupe, mechanic-stats aggregation, history filter, new encounter categories).
- **Items/hour polish (Phase 6.7a)** — the three deferred polish items on the Items/hr panel:
  - **DateRange scope** — new `ItemRateScope::DateRange { start, end }` (calendar-day range, inclusive both ends) filtering runs by `substr(started_at,1,10) BETWEEN ?1 AND ?2` (lexical prefix compare — timezone-safe, no SQLite `date()` interpretation). Surfaced as a **Custom…** pill with from/to date inputs. The scope→params resolver was refactored from a fixed `Vec<i64>` (`match params.len()`) to `Vec<rusqlite::types::Value>` + `params_from_iter`, so a scope can bind mixed-type / multi params. 1 new DB unit test (boundary-day inclusivity).
  - **Client-side column sort** — clickable Items/hr table headers; clicking a column sorts by it and toggles asc/desc (default Chaos/hr desc); a new column resets to its sensible default (name asc, numbers desc).
  - **Pinned items** — a per-row pin toggle floats an item to the top of the table in every scope; the pin set is keyed by item name and persisted in `localStorage` (client-only). Pinned rows always sit above unpinned, with the active sort applied within each group.
- **Items/hour view — Tier-1 (Phase 6.7a)** — answers "what is dropping at what rate" using data already captured by 6.3's inventory diff. No new APIs, no OCR.
  - `get_items_per_hour(scope)` Tauri command (`scope` = `CurrentSession | Session{id} | LastSessions{n} | AllTime`; `CurrentSession` falls back to `AllTime` when no session is open).
  - DB aggregation: `GROUP BY name` over `loot_items` filtered by scope-resolved run IDs; per-row `items_per_hour` and `chaos_per_hour` use `SUM(duration_secs)` (active map time, idle excluded) as the denominator. Returned ordered by `chaos_per_hour` DESC.
  - `ItemRate { name, source, stacks, drops, total_chaos, active_secs, items_per_hour, chaos_per_hour }` — `source` is `"inventory"` for now; later 6.7 increments will add `"stash:bestiary"` (red beasts) and `"ocr:*"` (Hiveblood, Kingsmarch gold) using the same shape.
  - **Items / hr** panel in the Maps tab (between Per-Map Stats and Recent Runs): scope-toggle pills (`This session | Last 5 | All time`), denominator displayed, empty state explaining the credential/character/loot requirements.
  - 5 unit tests on the DB query (AllTime aggregation + ordering, Session scope isolation, CurrentSession fallback, LastSessions window, zero-active-secs guard).
  - Fixed a latent test-only build bug in `poe-pricing/src/cache.rs` (`make_record` test helper missing `count: None` since 6.5c added the field).
- **OCR capture spike + `resource_snapshots` foundation (Phase 6.6a)** — first 6.6 increment; gated on a 1-click verify:
  - `capture_poe_test` command tries `PrintWindow + PW_RENDERFULLCONTENT` on the PoE client area and returns `{width, height, non_black_fraction}` — no file I/O, no heavy deps. Declares `PrintWindow` via `extern` because the metadata-trimmed `windows` bindings don't expose it; adds the `Win32_Graphics_Gdi` feature.
  - Debug section in Settings → **Test PoE capture** button shows e.g. `1920×1080 — 87% non-black`.
  - `resource_snapshots` table (DB migration v4) + `MapTracker.record_resource_snapshot` / `get_resource_snapshots`. Generic `(source, value, timestamp)` keyed by source — same shape for OCR writes (`ocr:hiveblood`, `ocr:kingsmarch_gold`) and future character-API XP/kills.
- **Net-worth time series + noise filters + league decoupling (Phase 6.5)** — three increments:
  - **6.5a:** `portfolio_snapshots` (DB migration v3) recorded every time a stash scan finalizes (manual scan + auto-session start/end); `get_net_worth_history` command; net-worth Sparkline in the Maps Trends panel. `record_portfolio_snapshot` carries chrono internally so `src-tauri` stays chrono-free.
  - **6.5b:** `StashTracker.set_min_stack_chaos` filter — stacks below the chaos threshold are excluded from the snapshot total (items list still shows them). Snapshot retention cap (auto-prune oldest beyond 1000 on each insert). Settings: **Snapshot noise filter (chaos)** input.
  - **6.5c:** poe.ninja `count` (listing count) parsed into `PriceRecord` (`serde(default)` — harmless if absent); `PricedItem.listing_count` propagated by the matcher; `set_min_listing_count` excludes low-confidence prices from the total. New `price_league` setting decouples pricing from the game league (price a dead/private league against `Standard`).
- **Stats table + trend sparklines (Phase 6.4)** — two increments:
  - **6.4a:** `get_map_type_stats` aggregation (GROUP BY internal area id, fallback name → run count / avg time / avg loot / total deaths); Per-Map Stats table in the Maps tab with a derived Loot-per-hr column.
  - **6.4b:** inline-SVG `Sparkline` component (no chart-lib dependency; scales to container via `viewBox` + `preserveAspectRatio="none"`); Trends panel with three sparklines — run duration, currency/hour by session, net worth — all oldest → newest.
- **Per-map loot capture (Phase 6.3)** — the headline 6.3 feature in three increments:
  - **6.3a:** pure `diff_inventory(prev, curr)` returning loot deltas (new items + stack-size growth; `MainInventory` only). `fetch_character_inventory` against `character-window/get-items` (separate rate budget from stash). 5 unit tests for the diff.
  - **6.3b:** `loot_items` table + `map_runs.loot_chaos` (DB migration v2); `MapRun.loot_chaos` + `LootItem`; `set_run_loot` / `get_run_loot`; new Loot column on Recent Runs.
  - **6.3c — town-leak fix:** new `StashTracker.pending_end_inventory` + `snapshot_character_at_suspend`; `capture_loot` prefers the suspend snapshot (taken at InMap → Idle in the poll loop) over a fresh fetch, so town purchases/crafts between maps no longer leak into the prior map's loot.
  - End-to-end orchestration: `MapTracker.poll_events` exposes the inserted run id; src-tauri poll loop baselines inventory at map start, captures + prices + writes loot on `MapCompleted`, emits `map-tracker:loot`.
- **Polish, testability & process**
  - Timer overlay: `focus: false` on the timer window stops a focus-fight that caused show/hide/show flicker after a quick alt-tab return; redundant per-tick `show()` guarded; 600 ms hide debounce; default position now top-left.
  - **Clear** button on Recent Runs (`clear_map_history` command + `clear_history` DB).
  - `save_settings` shallow-merges so panels don't clobber each other's keys.
  - Testability refactors: pure `merge_settings(base, incoming)` (unit-tested) extracted from `save_settings`; pure `next_session_action(state, has_active_session, idle_elapsed, idle_timeout)` (`SessionAction` enum, six branches unit-tested) extracted from the async poll loop.
  - Integration tests: legacy-v0 DB migration upgrade; Client.txt replay (full session — runs/types/encounters/town-resume/attribution) using a fixture log.
  - Auto-journal hook (`.claude/settings.local.json`, gitignored, personal): `PostToolUse`/Bash matcher that fires on `git commit` and reminds the agent to store a session-journal note — so journaling happens automatically at commits.
  - **Working Rules** section in `CLAUDE.md` (research agents → Sonnet; test eval + commit-first at every stopping point).
  - `CHECKLIST.md` is a running per-PR test list (pending fixes + per-sub-phase verify items + carry-forward note for credential-gated flows).
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
