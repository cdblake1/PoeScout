# Phase 6 Pass 1 — Manual Test Checklist

Run `npx --prefix packages/ui tauri dev` with Path of Exile open.
**Tip:** for faster session testing, set `"session_idle_timeout_secs": 30` in
`%APPDATA%\PoeScout\settings.json` (default is 900 = 15 min).

## Runs / UI
- [ ] Map a zone — live timer shows map name + correct tier
- [ ] Completed run appears in "Recent Runs" with correct tier and time
- [ ] Run row shows encounter chips when a league mechanic was present
- [ ] All-time stats (runs, avg time, maps/hr, deaths) update

## Instance-resume (town portal)
- [ ] Portal to town mid-map, then return to the SAME map → still ONE run (not split)
- [ ] That run's idle/hideout time is counted (not added to map duration)
- [ ] Entering a DIFFERENT map completes the previous run

## Hub classification (regression)
- [ ] Enter Kingsmarch → stays Idle, NO map run created
- [ ] Enter The Rogue Harbour → stays Idle, NO map run created
- [ ] Enter Azurite Mine → stays Idle, NO map run created

## Encounters
- [ ] Run a Delve (Niko) → "Delve" chip on the run
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

## Notes
<!-- Add any bugs, observations, or values seen during testing here -->
