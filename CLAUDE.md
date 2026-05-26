# PoeScout — Project Context for Claude Code

## What Is This Project?
PoeScout is a desktop companion tool for Path of Exile 1. It provides affix/base item lookup, PoB integration, overlay mode, map timing, and stash/currency tracking — all in a single native app.

## Stack
- **Desktop**: Tauri 2 (Rust backend)
- **UI**: SolidJS + Vite (in `packages/ui`)
- **Data source**: `repoe-fork` GitHub org — `mods.json` + `base_items.json` from `https://repoe-fork.github.io/`
- **DB**: SQLite (via `poe-data` crate)

## Build / Launch
```bash
npx tauri dev       # Launch app (NOT cargo tauri dev)
cargo test          # Run Rust tests
```

## Key Crates
- `poe-data` — Data ingestion from repoe-fork, SQLite storage, FTS5 search
- `poe-core` — Shared types (items, mods, affixes)
- `poe-pob` — Path of Building integration (decode build codes, launch PoB)
- `poe-stash` — GGG stash API integration
- `poe-pricing` — poe.ninja price lookups
- `poe-maps` — Map timer (Client.txt parsing)

## Key UI Files
- `packages/ui/src/components/lookup/BaseDetail.tsx` — Base item detail + affix display
- `packages/ui/src/components/lookup/ModSearch.tsx` — Mod search with FTS5

## Git Workflow
- **Never push directly to `main`**. Always create a feature branch and open a GitHub PR.
- Branch naming: `feature/<short-description>` or `fix/<short-description>`
- Use `gh pr create` after pushing the branch

## Roadmap
See `PLAN.md` for the checklist roadmap. Update checkboxes when work is completed.

## Changelog
See `CHANGELOG.md` — update with every change (use Keep a Changelog format).

## Working Rules (for the agent)
- **Research agents → Sonnet.** Spawn research/summarization sub-agents (reading source, web research, codebase surveys) with the Sonnet model, not Opus — much cheaper for read-and-summarize work. Reserve Opus for the main thread and hard reasoning/implementation.
- **Test eval at every stopping point.** When wrapping up a chunk of work, evaluate and report what needs **unit**, **integration**, and **manual** testing — clearly separating what's already covered from what the user must run by hand (e.g. flows needing the live game + GGG credentials). **Commit** the work before planning/writing that eval.
