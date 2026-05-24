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

## Phase 6: Advanced Map Tracking & Currency/Hour (IN PROGRESS)

> **Pass 1 (6.1 + 6.2) implemented & tested 2026-05-23** on branch `feature/phase6-map-tracking-sessions`.
> Enhanced Client.txt parsing + run model (internal area id, instance-resume, `AreaType` classifier,
> idle/AFK, quote-level encounters, character attribution) and **automatic sessions** (auto stash-snapshot
> on first map / idle-timeout end; profit + idle-excluded currency/hour; survives restart). 35 poe-maps
> tests pass; workspace builds; UI type-checks. Locked decisions: scope 6.1+6.2, quote-level encounters,
> fully-automatic sessions, character set in Settings (reusable for a future "open in PoB").
> **Remaining:** 6.3 per-map loot · 6.4 charts · 6.5 net-worth · 6.6 OCR. Auto-snapshot path needs live
> testing with the game running + GGG credentials + selected stash tabs.

### Goal
Fuse Client.txt run data with stash/inventory snapshot diffs to capture rich per-run data
(currency, encounters, tiers, idle time, deaths) and compute a *true* active-play currency/hour.
**Differentiator vs existing tools:** per-map profit attribution + c/hr that excludes idle/AFK time.

### Research — DONE (2026-05-23)
- [x] **TraXile** (C#/WinForms, github.com/dermow/TraXile) — pure Client.txt parser, *no* currency tracking. Best **run-lifecycle** model.
- [x] **Exile Diary** (Electron/TS, github.com/Qt-dev/exile-diary) — Client.txt + **GGG character-inventory diff** for per-map loot, **date-accurate** poe.ninja pricing.
- [x] **Exilence Next** (Electron/TS + .NET, github.com/viktorgullmark/exilence-next) — stash-snapshot **net-worth** engine, item-level diff, two-tier retention.
- [x] Parsed real Client.txt (962k lines, 2025-07 → 2026-05) — found several unused signals (below).
- [x] Rate-limit research — stash API is per-session-feasible; the character-inventory API is a *separate* budget usable per-map.

#### What each tool gives us
| | TraXile | Exile Diary | Exilence Next |
|---|---|---|---|
| Loot / currency | ❌ manual spreadsheet only | ✅ char-inventory diff, **per-map** | ✅ stash snapshot, **net worth** |
| Pricing | none | poe.ninja, **date-accurate per drop** | poe.ninja, chaos-denominated |
| Run identity | **instance endpoint (IP)** | timestamp window | n/a |
| Mechanics | NPC substring → counters | **719-entry quote table** (`events.json`) | n/a |
| c/hr | none | profit ÷ Σ run-durations | net-worth Δ ÷ hardcoded 1h (naive) |
| **Steal** | lifecycle, side-area nesting, tags, in-game cmds, replay backfill | inventory diff, quote table, historical pricing, run-boundary guard list | item-level diff, two-tier retention, noise filters, decoupled price-league |

#### Client.txt signals we DON'T capture yet (real log, with counts)
- **Internal area ID** — `Generating level N area "MapWorldsDunes"` (today we keep only the level, discard the id). Canonical map identity: distinguishes tiers/unique maps, flags league hubs (`ChayulaLeague`, `KalguuranSettlersLeague`), towns (`1_1_town`). **Highest-value gap.**
- **`Connecting to instance server at <ip>:<port>`** — 22,823 hits. Instance identity → resume the same map after a town portal instead of starting a new run. Used by both TraXile and Exile Diary.
- **NPC league-mechanic dialogue** — present in *my* log: Niko 3186 (Delve), Jun 2943 (Betrayal), Maven 2898, Einhar 2628 (Bestiary), Sister Cassia 722 (Blight), Strange Voice 538 (Delirium), Expedition NPCs (Tujen/Rog/Dannig/Gwennen), Sirus 218, The Envoy 473. Format: `] NPC, Title: quote`.
- **AFK mode ON/OFF** — 196 / 189. Auto-pause idle time (we have an unused `hideout_secs` column).
- **Area-classification gaps** — Kingsmarch (375), The Rogue Harbour (70), Azurite Mine (86), Domain of Timeless Conflict (9) are currently **mis-counted as maps**. Need an internal-id-based classifier (current `areas.rs` only knows hideouts + 10 act towns).
- **Atlas passive (un)allocation** — 11,121 / 7,181 (`Successfully allocated/unallocated passive skill id: …`). Atlas-change history (optional).
- **Abnormal disconnect** — 51 (`Abnormal disconnect: …`). DC / death-by-DC tracking (optional).
- **Self-whisper `@To`** — 2,139. In-game annotation channel (`end` / `note`) à la Exile Diary / TraXile (optional).
- **Bugs found in current parser:**
  - Our `You have died` branch is **dead code** — 0 occurrences. Real deaths are `<char> has been slain` (137 hits). The `has been slain` branch works.
  - `has been slain` *and* `is now level` also fire for **party members** → need an active-character-name filter for correct attribution (solo today, but wrong in parties).

### Architecture decision — two currency layers
1. **Session-level (MVP — reuses Phase 5).** "Start Session" → stash snapshot; runs accrue; "End Session" → snapshot; profit = Δchaos. **c/hr = profit ÷ active map time** (idle excluded — beats Exilence's naive 1h divisor). No new auth scope, low risk.
2. **Per-map (differentiator — 6.3).** Diff the GGG **character inventory** API on each map exit (Exile Diary model): new items + stack-size deltas → per-map loot lines, date-accurate pricing. Separate rate budget from stash; needs a persistent price cache.

**Recommendation:** ship **6.1 + 6.2** first (immediate value, no new risk); **6.3** is the headline feature after. 6.4 UI rides on top; 6.5 is optional polish.

### Stretch goal: mechanics & resources with no item id
Researched how all three tools handle data with no item/currency id (Bestiary, Delve, Expedition, Ultimatum, …).
**Hard boundary — confirmed against our real log AND all three sources:** PoE's Client.txt *never* emits resource
*quantities*, only NPC dialogue. Every keyword hit in our log (`Sulphite`, `Azurite`, `Artifact`, `Lifeforce`,
`dust`) is an atlas-passive *node name*, an asset path, or flavour dialogue — never an amount. Three capability tiers:

| Tier | Examples | Source | Achievable? |
|---|---|---|---|
| **1. Encounter / count** | "ran Expedition", "captured 3 beasts", boss-fight timing | Client.txt NPC dialogue `] NPC, Title: quote` | ✅ count + presence + detail (beast yellow/red, simulacrum wave #, boss start/finish) |
| **2. Item amount** | lifeforce, scarabs, oils, fossils, **Expedition artifacts** (stackable items), Metamorph organs | inventory/stash diff (6.3) | ✅ exact amount + chaos value |
| **3. Side-panel resource** | Delve **sulphite / azurite / depth**, Kingsmarch **gold / ore**, Sanctum **resolve / aureus**, Breach **Hiveblood** | nowhere in log/API — but rendered **on screen** | ❌ from data; ⚠️ **best-effort via screen OCR (6.6)** (TraXile & Exile Diary give up on these) |

**The one escape hatch for Tier 3** (Exile Diary's only trick, used just 3×): poll the GGG **character API**
`/character/{name}`, read a numeric field/property, store `(timestamp, value)`, diff over the run window. Works
*only* for values GGG actually surfaces: `experience` (→ XP/hr) and per-item charge **properties** like
`Graftblood: {0}/{1}` on equipped items (`equipment[].properties`, `values[0]` = current/max).
NB on naming: **"graftblood"** (a Brequel-graft charge, a real recent-league resource) is *not* the same as
"hiveblood" — no tool tracks anything called hiveblood. Whatever resource you name is trackable **iff** it appears
as a character/item property at that endpoint; sulphite/azurite/gold do not, so they stay out of scope.

**Build mapping:** Tier 1 → folds into 6.1 (`map_encounters` + tags). Tier 2 → falls out of 6.3 inventory diff for
free. Tier 3 → two paths into the shared `resource_snapshots` table: **char-API snapshot-diff (6.3)** covers
XP/kills/item-charge properties (closes the "no XP-rate" gap); **on-screen OCR (6.6)** is the *only* way to capture
pure side-panel resources (Hiveblood, sulphite, Kingsmarch gold) — best-effort + user-calibrated.

---

### 6.1 — Enhanced parsing & richer run model (no auth; do first)
**`crates/poe-maps/src/parser.rs`**
- [ ] Extend `RE_GENERATING` to also capture the internal area id: `Generating level (\d+) area "([^"]+)"` → new `LogEvent::AreaLevelHint { area_level, area_id }` (carry both).
- [ ] New `LogEvent::InstanceConnected { endpoint }` from `Connecting to instance server at (\S+)`.
- [ ] New `LogEvent::Afk { on: bool }` from `AFK mode is now (ON|OFF)`.
- [ ] New `LogEvent::NpcLine { npc, text }` from `] ([^:]+): (.+)$` (used for mechanic detection).
- [ ] Drop the dead `You have died` alternative; keep `has been slain`. Add optional active-char capture for attribution.
- [ ] (Optional) `Abnormal disconnect`, atlas `Successfully (allocated|unallocated)`, self-whisper `@To … : (end|note …)`.

**`crates/poe-maps/src/areas.rs`** — replace the 2-list check with a proper classifier
- [ ] `enum AreaType { Map, Town, Hideout, Campaign, Heist, Delve, Lab, Sanctum, LeagueHub, Boss, Unknown }`.
- [ ] `classify(area_id: Option<&str>, area_name: &str) -> AreaType` — prefer internal id prefixes (`MapWorlds*`/`MapMetamorph*` → Map; `*_town`/`*endgame_town` → Town; `Hideout*`/`*Hideout` → Hideout; `Azurite Mine` → Delve; `*League`/`Kingsmarch` → LeagueHub; Heist/Lab/Sanctum lists), fall back to display name.
- [ ] `is_idle_zone` = Town | Hideout | LeagueHub (these stop a run); add `map_tier(area_level)` = `clamp(level - 67, 1..=17)`.

**`crates/poe-maps/src/state.rs`**
- [ ] Carry `area_id`, `instance_id`, `area_type`, `map_tier`, `idle_secs` through `InnerState`/`MapRun`.
- [ ] Use `instance_id`: re-entering the *same* endpoint resumes the open run (handles town-portal-and-back) — don't emit a new run.
- [ ] Track idle: on `InMap → Idle` start an idle clock; accumulate across town hops; write `hideout_secs`/`idle_secs` onto the next run + session. `Afk{on}` pauses/resumes the active clock.
- [ ] Don't end a run on sub-areas (lab trial, Vaal side-area, Abyssal Depths, Delve) — port Exile Diary's guard list. Full side-area *nesting* (TraXile) deferred.
- [ ] New `StateEvent::Encounter { run, category, detail }` fed from `NpcLine` via a lookup table.

**League-mechanic data table** (`crates/poe-maps/data/encounters.json`, loaded once → `HashMap`) — Tier 1 of the stretch goal
- [ ] **MVP: NPC-name → category** (Einhar→Bestiary, Niko→Delve, Jun→Betrayal, Cassia→Blight, Strange Voice→Delirium, Tujen/Rog/Dannig/Gwennen→Expedition, Alva→Incursion, Oshabi→Harvest, Maven/Envoy/Sirus→Pinnacle). Gives per-run presence (boolean **tag**) + encounter count. This alone matches what TraXile stores for most mechanics.
- [ ] **Detail extraction (later):** adopt Exile Diary's quote-level schema — `{quote → {category, type, arguments}}` (their `events.json` is now ~1,287 entries). Dispatcher idiom: `arguments` either increments a **count bucket** (beast `yellow`/`red`) or appends to a **timestamped list** (simulacrum waves; boss fights paired via "start / else-close-last-open-entry" → fight durations). Note Ultimatum *rounds* are a real gap even in Exile Diary (quotes parsed but dropped) — easy win if we add the rule.
- [ ] Store detail as `map_encounters` rows (category + detail) and surface per-run tags; do NOT attempt Tier-3 amounts here (not in the log).

**`crates/poe-maps/src/db.rs`** — add a `user_version` migration step (current `CREATE … IF NOT EXISTS` can't add columns)
- [ ] `ALTER TABLE map_runs ADD COLUMN area_id TEXT / instance_id TEXT / area_type TEXT / map_tier INTEGER / league TEXT / session_id INTEGER / loot_chaos REAL`; populate `hideout_secs`.
- [ ] New table:
```sql
CREATE TABLE map_encounters (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id INTEGER NOT NULL REFERENCES map_runs(id),
  category TEXT NOT NULL,           -- Bestiary/Delve/Betrayal/Blight/Delirium/Expedition/Pinnacle/…
  detail TEXT,                      -- boss name, beast type, etc.
  timestamp TEXT NOT NULL
);
```
- [ ] Tests: new regexes, classifier (Kingsmarch/Rogue Harbour/Azurite Mine → not Map), instance-resume, idle accounting, encounter mapping.

### 6.2 — Sessions + session-level currency/hour
**`crates/poe-maps/src/db.rs`**
```sql
CREATE TABLE map_sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  label TEXT,
  league TEXT,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  start_chaos REAL,                 -- portfolio snapshot at start
  end_chaos REAL,
  profit_chaos REAL,                -- end - start
  active_secs REAL,                 -- Σ run duration (idle excluded)
  notes TEXT
);
```
**`crates/poe-maps/src/lib.rs` + `src-tauri/src/commands/maps.rs`**
- [ ] `start_session(label)` → call existing `take_selective_snapshot` (reuse Phase 5 stash + pricing), create session row, mark active.
- [ ] Runs created while a session is active get `session_id` set in `poll_events`.
- [ ] `end_session()` → second snapshot; `profit = end - start`; `c/hr = profit ÷ (Σ active map secs)`; finalize row.
- [ ] `get_sessions()`, `get_session_detail(id)` (runs + encounters + profit + c/hr).
- [ ] Persist the start/end `PortfolioSummary` (JSON column or `portfolio.json`-style) so a crash mid-session can still diff.

### 6.3 — Per-map inventory-diff loot (differentiator)
**`crates/poe-stash`** — add character API
- [ ] `fetch_character_inventory(account, character)` → GGG `/character-window/get-items` (POESESSID-auth, separate rate budget). Add `get_characters(account, league)`.
- [ ] `diff_inventory(prev, curr)`: added items + **stack-size deltas** (the currency trick) + transformed items; only `MainInventory` (ignore equipment/gem swaps). Mirror Exile Diary's `compareElements`.
**`crates/poe-maps`**
- [ ] On map exit, snapshot inventory, diff vs previous, price each line, write `loot_items(run_id, name, type_line, stack_size, unit_chaos, total_chaos, drop_time, ignored)`; set `map_runs.loot_chaos`.
**`crates/poe-pricing`** — persistent, date-accurate cache
- [ ] Add `price_history(date TEXT, league TEXT, name TEXT, chaos REAL)` so re-pricing uses the rate from the drop's date; keep `original_value` vs re-computable `value` + per-item `ignored` flag.

**Resource snapshot-diff (Tier 3 escape hatch + XP/hr)** — reuses the same `/character/{name}` poll as the inventory diff
- [ ] Generic append-only table (TraXile's `tx_stats` shape):
```sql
CREATE TABLE resource_snapshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source TEXT NOT NULL,             -- 'experience' | 'kills' | 'graftblood' | <item-property>
  value INTEGER NOT NULL,           -- absolute reading (running total)
  timestamp TEXT NOT NULL
);
```
- [ ] On each character poll, record: `experience` (top-level), kill proxy (max incubator `progress` delta), and any equipped-item charge property (parse `Graftblood: {0}/{1}` → `values[0]`). Per-run gain = `value@last − value@first` between the run's `started_at`/`ended_at` → **XP/hr** (closes the existing "no XP-rate tracking" gap) + per-run XP/kills.
- [ ] Property-driven and generic: a new resource that GGG surfaces as a character/item property is added by config, no schema change. (Pure side-panel resources — sulphite/azurite/gold — remain out of scope: no source exists.)

### 6.4 — Stats & charts UI
**`packages/ui/src/components/maps/MapTimer.tsx`** + **`packages/ui/src/lib/tauri.ts`** bindings
- [ ] Session controls: Start/End Session, live profit + c/hr, current-session run list.
- [ ] Per-map-type table (group by `area_id`/name): run count, avg duration, avg profit, c/hr — "Strand: 2m30s, 45c/hr".
- [ ] Encounter chips on each run row (Bestiary/Delve/Betrayal/…).
- [ ] Charts: duration over time, c/hr trend, deaths per map type (lightweight; reuse stash-tab chart approach).
- [ ] Session history list (past sessions: profit, maps, duration, c/hr).

### 6.5 — Net-worth-over-time (optional, later)
- [ ] Two-tier retention (Exilence): full item rows for last N snapshots; value-only rows long-tail; cap history. Net-worth graph + per-tab "which tab earns" series.
- [ ] Noise filters: poe.ninja `count > 10`, per-item + per-stack chaos thresholds (user-configurable).
- [ ] Decouple game-league from price-league (price a dead league vs Standard).

### 6.6 — On-screen OCR for Tier-3 resources (stretch; the *only* way to capture untrackable amounts)
These resources have no item id and no API but ARE rendered on screen (Hiveblood, Kingsmarch gold/ore, Delve
sulphite/azurite, Sanctum resolve/aureus — see `docs/poe-mechanics-resources.md`). **Best-effort, opt-in, off by
default, user-calibrated.** Same category as accepted overlay tools (passive screen-read, no input automation).

**Capture — extend `src-tauri/src/commands/capture.rs`** (we already resolve the PoE HWND + window rect)
- [ ] Grab a window sub-rect via **`Windows.Graphics.Capture`** (WinRT, already in the `windows` crate). **NOT** GDI `BitBlt`/`PrintWindow` — those return **black frames** for DirectX games. ⚠️ **Derisk first:** a throwaway "capture PoE window → PNG" spike before building anything else.
- [ ] Capture only the calibrated region, not the whole frame (keeps OCR fast).

**OCR — new `src-tauri/src/ocr.rs` (or `crates/poe-ocr`)**
- [ ] Use **`Windows.Media.Ocr`** (built-in WinRT; no Tesseract bundle). Preprocess crop → grayscale → threshold → upscale; digit whitelist. Return `{ value: Option<i64>, confidence: f32, raw_text: String }`. Integers only; reject non-numeric / low-confidence reads.

**Calibration & settings**
- [ ] Store per-resource regions in `settings.json` as `{ resource_key, x, y, w, h }` **relative to the PoE window rect** (survives window moves; still resolution/UI-scale specific).
- [ ] UI `packages/ui/src/components/maps/OcrCalibration.tsx` — pick a resource, drag a box over the on-screen number, **"Test OCR"** shows the read value + confidence.

**Triggering & storage — reuse the 6.3 `resource_snapshots` table**
- [ ] Trigger OCR at **session Start/End** (6.2) — the realistic granularity, since most panels are only on screen intermittently. Optionally auto-trigger when the relevant zone/panel is detected from Client.txt area events (e.g. entered Kingsmarch).
- [ ] Write reads with `source = 'ocr:hiveblood' | 'ocr:kingsmarch_gold' | …`; per-session gain = `value@end − value@start`; show in the session report beside currency profit. Always allow **manual correction** (best-effort data).

**Scope & risks**
- [ ] MVP: **Hiveblood + Kingsmarch gold/ore** at session boundaries. Extend to sulphite / sanctum by adding `{resource_key, region}` entries — no code change.
- [ ] Known limits (documented in `docs/poe-mechanics-resources.md`): resolution/UI-scale fragility, visibility-gating (no true per-map deltas), UI moves between patches, OCR misreads → manual override.
- [ ] Tests: OCR digit parsing on sample crops; region-relative-to-window math; snapshot diff.

### Critical files
- `crates/poe-maps/src/parser.rs` — new `LogEvent` variants (area_id, instance, AFK, NPC lines); fix death regex
- `crates/poe-maps/src/areas.rs` — `AreaType` classifier (id-based), tier mapping
- `crates/poe-maps/src/state.rs` — instance-resume, idle/AFK accounting, encounters, session linkage
- `crates/poe-maps/src/db.rs` — `user_version` migrations; `map_sessions`, `map_encounters`, `loot_items`; new `map_runs` columns
- `crates/poe-maps/src/lib.rs` — session lifecycle, inventory-diff hook
- `crates/poe-maps/data/encounters.json` — NPC/quote → mechanic table (new)
- `crates/poe-stash/src/{lib,client}.rs` — character inventory API + diff (6.3)
- `crates/poe-pricing/src/cache.rs` — persistent date-accurate price history (6.3)
- `src-tauri/src/commands/maps.rs` — session + stats commands
- `src-tauri/src/lib.rs` — register new commands
- `packages/ui/src/components/maps/MapTimer.tsx` — session UI, per-map stats, charts
- `packages/ui/src/lib/tauri.ts` — new bindings
- `src-tauri/src/commands/capture.rs` — `Windows.Graphics.Capture` window-region grab (6.6)
- `src-tauri/src/ocr.rs` (or `crates/poe-ocr`) — `Windows.Media.Ocr` digit OCR (6.6)
- `packages/ui/src/components/maps/OcrCalibration.tsx` — OCR region calibration + "Test OCR" (6.6)
