# Changelog

## Unreleased

### Changed
- All `implicit_tags` now display as pills (no filtering) — curated tags use hand-picked colors, unknown tags get auto-generated colors via deterministic hue hashing (`BaseDetail.tsx`)
- Expanded tier rows now show PoE-readable `text` (e.g. `+(8-12) to Strength`) instead of raw stat IDs

### Fixed
- `scripts/fetch-repoe.sh` URL updated from legacy `brather1ng/RePoE` to `repoe-fork.github.io` to match runtime data source

### Removed
- Dead `tags` field from `RawMod` struct in `ingest.rs` — upstream `mods.json` has no `tags` field, only `implicit_tags`
