# Phase 5c: Stash Polish — Test Checklist

## Data Persistence
- [x] Portfolio data persists when switching tabs (navigate away and back)
- [x] Portfolio data persists across app restarts
- [x] "Last updated" timestamp shown in connected header
- [x] New scan updates the timestamp

## Pagination
- [x] Items table shows 50 items per page (no scroll)
- [x] Page controls appear when >50 items
- [] Prev/Next buttons work, page numbers highlight current
- [ ] Changing search/filter resets to page 1

## Refresh Prices Status
- [x] "Refreshing prices..." shown while refresh in progress
- [x] "Prices refreshed" shown on success
- [x] "Price fetch failed: ..." shown on error

## Scan Progress
- [x] Scanning text shows tab type (e.g. "Currency (Currency)", "Gems (Normal)")

## Tab Search
- [x] Tab search matches by tab type (e.g. typing "currency" shows Currency tabs)

## Settings Persistence
- [x] Min chaos threshold saved on blur, restored on next app launch | park for move to settings page, fine for now
- [x] Tab selection persisted (already tested)

## Rate Limit Cooldown
- [x] Cooldown is compact text (no large progress bar)

## Startup Safety
- [x] App launches with no GGG stash API calls (check logs for no 429/stash errors)
- [x] Saved credentials restore "Connected as ..." without hitting GGG

## Notes
<!-- Add any notes, bugs, or observations here -->
