# Phase 6 — Manual Test Checklist

Run `npx --prefix packages/ui tauri dev` with Path of Exile open.
**Tip:** for faster session testing, set `"session_idle_timeout_secs": 30` in
`%APPDATA%\PoeScout\settings.json` (default is 900 = 15 min).

> **Status (2026-05-26):** Phase 6.1–6.5 + 6.6a are all merged on `main` (11 PRs).
> Backend/state-machine items (Runs/UI, Instance-resume, Hub fix, Delve chip)
> are verified. **Most credential-gated items below are unchecked** — they're
> the carry-forward for a focused live test session whenever you can run one.

## Runs / UI
- [x] Map a zone — live timer shows map name + correct tier | lets place it in the top left corner as default position
- [x] Completed run appears in "Recent Runs" with correct tier and time | need a way to clear recent runs
- [x] Run row shows encounter chips when a league mechanic was present
- [x] All-time stats (runs, avg time, maps/hr, deaths) update

## Instance-resume (town portal)
- [x] Portal to town mid-map, then return to the SAME map → still ONE run (not split)
- [x] That run's idle/hideout time is counted (not added to map duration)
- [x] Entering a DIFFERENT map completes the previous run

## Hub classification (regression)
- [x] Enter Kingsmarch → stays Idle, NO map run created
- [x] Enter The Rogue Harbour → stays Idle, NO map run created
- [x] Enter Azurite Mine → stays Idle, NO map run created

## Encounters
- [x] Run a Delve (Niko) → "Delve" chip on the run
- [ ] Run an Expedition (Tujen/Rog/Gwennen/Dannig) → "Expedition" chip
- [ ] Run Bestiary / Blight / Betrayal / Breach → matching chip
- [ ] Same mechanic NPC talking repeatedly → chip appears once (not duplicated)

## Automatic sessions (needs GGG credentials + selected stash tabs)
- [ ] Connect credentials + select tabs in the Stash tab first
- [ ] First map out of town auto-starts a session (Sessions panel shows it active)
- [ ] Map a few times → session "Maps" count and active time grow
- [ ] Go idle in town/hideout past the timeout → session auto-ends
- [ ] Ended session shows Profit (chaos) and c/hr in the Sessions panel
- [ ] With NO credentials/tabs: session still tracks maps/time, Profit blank (no crash)

## Settings persistence / merge
- [ ] Set Character in Settings, Save → persists after restart
- [ ] After saving Character, your selected stash tabs are STILL selected (not wiped)
- [ ] After changing tabs in Stash, your Character is STILL set (not wiped)

## Restart mid-session
- [ ] Start a session (map once), then close + relaunch the app
- [ ] The open session resumes (no duplicate session created)

## Attribution (party play, optional)
- [ ] With Character set, a party member's death does NOT increment your run deaths
- [ ] With Character set, a party member's level-up is NOT recorded on your run

## Pending fixes to verify (2026-05-24)
- [ ] Overlay flicker on quick alt-tab+return is gone (focus:false + hide debounce)
- [ ] Timer overlay defaults to the top-left corner
- [ ] "Clear" button on Recent Runs empties the list

## Phase 6.3 — Per-map loot (needs GGG credentials + a character)
- [ ] 6.3a: `character-window/get-items` returns expected JSON for your character (set Character in Settings)
- [ ] 6.3b: per-map loot value (loot_chaos) populated on map completion
- [ ] 6.3b: per-run loot line items stored and shown (Loot column on Recent Runs)
- [ ] 6.3c (town-leak fix): in a map → town → DIFFERENT map sequence, town
      purchases/crafts between maps do NOT leak into the prior map's loot

## Phase 6.5 — Net worth + noise filter (needs GGG credentials)
- [ ] 6.5a: stash scan records a portfolio snapshot (Net worth sparkline in Maps Trends populates)
- [ ] 6.5a: auto-session start/end also add data points (chaos only)
- [ ] 6.5b: "Snapshot noise filter (chaos)" in Settings > 0 excludes sub-threshold stacks from the snapshot total; items list still shows them
- [ ] 6.5c: "poe.ninja listing-count threshold" in Settings filters low-confidence prices from the snapshot total
- [ ] 6.5c: "Price-league override" in Settings fetches prices from a different league (e.g. `Standard`); stash fetches still use the game league

## Phase 6.7 — Items / hr (needs GGG credentials + a character + map runs with priced loot)
- [ ] 6.7a: Items/hr panel appears in the Maps tab (below Per-Map Stats, above Recent Runs)
- [ ] 6.7a: with no priced loot yet, the empty state explains the requirements
- [ ] 6.7a: scope toggle (This session / Last 5 / All time) re-queries — row counts and rates change
- [ ] 6.7a: ordering is `chaos_per_hour` desc (Divine on top in a typical session, then chaos stacks, then splinters/fossils)
- [ ] 6.7a: "Denominator: active map time = …" matches the sum of in-scope run durations (idle excluded)
- [ ] 6.7a: `Items/hr` for Chaos Orbs scales as expected — double the maps in scope → roughly half the per-hour rate (since total stacks doubled but time also doubled)
- [ ] 6.7a: switching scope to a session with no priced loot shows the empty state (not stale data)
- [x] 6.7a-polish (sort): clicking a column header sorts by it; same header toggles ▲/▼; default is Chaos/hr ▼
- [x] 6.7a-polish (pin): clicking ☆ pins an item to the top (★); it stays pinned across scope switches AND after an app restart (localStorage); unpin returns it to sorted position
- [x] 6.7a-polish (pin+sort): pinned rows always sit above unpinned; changing the sort reorders within each group
- [x] 6.7a-polish (DateRange): **Custom…** reveals from/to date inputs; Apply re-queries to only runs in that day-range (inclusive both ends); denominator matches those runs; reversed from/to still works
- [x] 6.7a-polish (DateRange): a custom range with no priced loot shows the empty state, not stale rows; clicking a preset pill hides the custom row and re-queries

## Phase 6.6 — OCR capture spike
- [ ] Open PoE on screen; in Settings → click **Test PoE capture** → reports e.g. `1920×1080 — 87% non-black`
- [ ] Non-black fraction ≥ 0.5 → `PrintWindow + PW_RENDERFULLCONTENT` works; safe to build OCR + calibration on top
- [ ] If close to 0% (black frame) → report back; we'd switch to the heavier `Windows.Graphics.Capture` path

## Notes
<!-- Add any bugs, observations, or values seen during testing here -->
map overlay got into a state where it would flicker, show,hide,show,hide continuously after alt+tab and return quickly
