# PoE 1 Mechanics & Resource Trackability — Context for Map/Currency Tracking

> **Purpose:** Reference for what PoeScout's map/currency tracker (Phase 6) *can* and *cannot* capture.
> Many league mechanics reward **resources that have no inventory item id** and therefore cannot be
> tracked the way currency/items are. This file records which mechanics those are, per patch, so we
> don't waste effort trying to track the impossible. Read alongside `PLAN.md` Phase 6.
>
> **Current patch when written:** **3.28 "Mirage"** (released 2026-03-06). Update the per-mechanic
> table each league — mechanics rotate in/out of core and resources get reworked (Breach gained
> Hiveblood in 3.28; Settlers Ore/Crops swapped roles in 3.28).

## The hard boundary (verified against a real 962k-line Client.txt + TraXile/Exile Diary/Exilence source)

PoE's `Client.txt` **never emits resource quantities** — only NPC dialogue, area transitions, deaths,
level-ups, and instance/system lines. Every resource keyword that appears in the log (`Sulphite`,
`Azurite`, `Artifact`, `Lifeforce`, `dust`…) is an **atlas/Genesis passive node name**, an asset path,
or NPC flavour text — never an amount. So resource amounts must come from elsewhere, and for many
mechanics they come from nowhere we can read.

## Three capability tiers

| Tier | What it is | Data source | Trackable? |
|---|---|---|---|
| **1. Encounter / count** | "ran Expedition", "captured 3 beasts", boss-fight timing | `Client.txt` NPC dialogue `] NPC, Title: quote` | ✅ presence + count + detail (e.g. beast yellow/red, simulacrum wave #, boss start/finish) |
| **2. Item amount** | resources that are real stackable items with an id | inventory diff (char API) or stash snapshot diff + poe.ninja | ✅ exact amount + chaos value |
| **3. Stored / run resource** | side-panel counters, account/league pools, in-run-only meters | none in `Client.txt`/API — but **rendered on screen** | ❌ from data; ⚠️ best-effort via **screen OCR** (PoeScout 6.6) |

**Tier-3 escape hatch (rare):** poll the GGG character API `/character/{name}` and diff a numeric field
over the run window. This works **only** for values GGG actually surfaces there:
- `experience` (→ XP/hr),
- kill proxy (max incubator `progress` delta),
- per-item charge **properties** on equipped gear, e.g. a `Name: {current}/{max}` property string
  (this is the pattern behind Exile Diary's `graftblood` tracker — a charge on an *equipped item*).

A Tier-3 resource is trackable **iff** it appears as a character/item property at that endpoint.
Account/league-stored pools (Hiveblood, Sulphite, Kingsmarch Gold) do **not** appear there.

**Second escape hatch — on-screen OCR (PoeScout Phase 6.6):** every Tier-3 resource is *displayed in-game*, so
it can be read with screen capture + OCR. This is the only way to get Hiveblood / sulphite / Kingsmarch gold
amounts. It is **best-effort, opt-in, and user-calibrated**, with hard limits: resolution/UI-scale specific
(needs a calibrated capture region per resource), visibility-gated (the panel must be on screen → realistically
captured at session boundaries, not per-map), and brittle across patches when GGG moves UI. Use
`Windows.Graphics.Capture` (GDI `BitBlt` returns black on DirectX games) + `Windows.Media.Ocr` (built-in, no
Tesseract). Treat reads as low-confidence and allow manual correction.

## 3.28 (Mirage) — per-mechanic resource trackability

### ❌ Tier-3 — UNTRACKABLE resources (no item id, no API)
| Mechanic | Resource | Nature |
|---|---|---|
| **Breach (3.28 rework)** | **Hiveblood** | Auto-stored counter, **cap 100,000**, earned from Hiveborn/Hive Fortress, spent as tribute at the **Genesis Tree** (Monastery of the Keepers). No item id, no known API. **This is the user's "hiveblood".** |
| **Breach / Genesis Tree** | Womb passive points | Account allocation (like atlas points); `Client.txt` logs `… Genesis Tree … Skill Point` allocations but not balances |
| **Delve** | Sulphite, Azurite, Depth | Niko side-panel / Delve city pools |
| **Settlers / Kingsmarch** (core, optional in 3.28) | Gold, Ore, Bars, Crops, Dust, worker/town state | Town-management panel. NB 3.28 swapped Ore↔Crops payouts (Ore→currency, Crops→equipment) |
| **Sanctum** | Resolve, Aureus | Resolve = per-run meter; Aureus = non-tradeable in-run gold, auto-collected, doesn't leave the run |
| **Ritual** | Tribute | Per-area run resource, spent at the altar in that map only; never an item |
| **Betrayal** | Syndicate intelligence / ranks / safehouse state | Progress panel, not a currency |
| **Atlas** | Passive points | Account allocation; allocate/unallocate logged in `Client.txt`, balance is not |

> For all of the above, the most we can capture is **Tier-1 encounter detection** ("this map had Breach /
> Delve / Ritual"), never the amount gained.

### ✅ Tier-2 — TRACKABLE as items (inventory/stash diff + poe.ninja)
These reward **real stackable items**, so the Phase 6.3 inventory/stash diff captures amount + value:
- **Mirage (3.28 league):** Coins (currency items that imbue a L20 gem with a support)
- **Breach:** Splinters, Blessings, **Wombgifts** (tradeable; sold for chaos / socketed at Genesis Tree)
- **Legion:** Splinters, Emblems
- **Expedition:** Artifacts (Astragali, Exotic Coinage, Sacred/Burial Medallion) — items; poe.ninja "Artifact"
- **Harvest:** Crystallised Lifeforce (Sacred/Primal/Vivid/Wild) — items
- **Sanctum:** lasting rewards (orbs, relics) are items — only the in-run Resolve/Aureus are Tier-3
- **Always items:** Scarabs, Essences, Fossils/Resonators, Oils (Blight), Delirium Orbs, Catalysts,
  Tattoos/Runegrafts, Divination Cards, Fragments, Maps, and all standard currency

### Tier-1 — encounter detection via NPC dialogue (works for any mechanic)
Detectable from `] NPC, Title: quote`. Key NPCs seen in our log: Einhar (Bestiary), Niko (Delve),
Jun/syndicate (Betrayal), Sister Cassia (Blight), Strange Voice (Delirium), Alva (Incursion),
Oshabi (Harvest), Tujen/Rog/Gwennen/Dannig (Expedition), The Trialmaster (Ultimatum),
Maven/The Envoy/Sirus (Pinnacle bosses). Detail (beast colour, wave #, boss phase timing) comes from
quote-level matching — see Exile Diary's `events.json` model referenced in `PLAN.md` 6.1.

## Detection-signal catalog (Phase 6.8)

How PoeScout decides a mechanic was present in a map. Two mechanisms, both from `Client.txt`:

- **NPC dialogue** — `] Name, Title: quote`. Matched in `crates/poe-maps/data/encounters.json`:
  `by_npc` keys on the NPC's **first name** (title-agnostic → survives league title changes) for
  *presence* (recorded once per category per map); `by_quote` matches the **exact** quote for detail
  events (e.g. beast captures, counted individually).
- **Area entry** — entering a dedicated area with no NPC line. Matched in
  `crates/poe-maps/src/areas.rs::mechanic_for_area` (`AREA_MECHANICS`). The mechanic is recorded on
  the *current run* (the parent map for sub-areas).

Both push a `MapEncounter { category, detail }` onto the run. Sourced from **TraXile**
(github.com/dermow/TraXile, `TrX_EventMapping.cs` / `TrX_DefaultMappings.cs`) and **Exile Diary
Reborn** (github.com/Qt-dev/exile-diary, `src/helpers/data/events.json`), both MIT — keep this table,
`encounters.json`, and `AREA_MECHANICS` in sync each league.

| Mechanic | Signal | Exact substring / area name | Category | Notes |
|---|---|---|---|---|
| Bestiary | NPC presence | `Einhar` | Bestiary | |
| Bestiary | NPC quote ×6 | capture lines (yellow ×3 / red ×3) | Bestiary | counted per capture (`kind:"capture"`) |
| Delve | NPC presence / area | `Niko` / `Azurite Mine` (hub) | Delve | Azurite Mine is a hub, not a run |
| Betrayal | NPC presence | `Jun` | Betrayal | |
| Blight | NPC presence | `Sister Cassia` | Blight | |
| Delirium | NPC presence | `Strange Voice`, `Eagon` | Delirium | Eagon = Memory Tear |
| Incursion | NPC presence / area | `Alva` / `The Temple of Atzoatl` | Incursion / Temple | |
| Harvest | NPC presence / area | `Oshabi` / `The Sacred Grove` | Harvest | |
| Expedition | NPC presence | `Tujen`, `Rog`, `Gwennen`, `Dannig` | Expedition | |
| Expedition | Area entry | 13 logbook zones (Volcanic Island, Bluffs, …) | Expedition `logbook` | name-collision zones (Cemetery, Vaal Temple) omitted |
| Heist | NPC presence | 9 rogues (`Karst`, `Tullina`, …) | Heist | |
| Sanctum | NPC presence / area | `Lycia` / 6 Sanctum floors | Sanctum | |
| Ultimatum | NPC presence / area | `The Trialmaster` / `The Tower of Ordeals` | Ultimatum | |
| Ancestor (ToTA) | NPC presence / area | `Navali` / `The Halls of the Dead` | Ancestor | |
| Legion | Area entry | `Domain of Timeless Conflict` | Legion `domain` | in-map monolith itself is invisible |
| Breach | Area entry | 5 Breachstone Domains (`Xoph's Domain`, …) | Breach `domain` | in-map Breach itself is invisible |
| Simulacrum | Area entry | `Lunacy's Watch`, `The Bridge Enraptured`, `The Syndrome Encampment`, `Hysteriagate`, `Oriath Delusion` | Simulacrum | |
| Abyss | Area entry | `Abyssal Depths` | Abyss | the pit only; in-map cracks invisible |
| Labyrinth | Area entry | 6 `Trial of …` | Lab `trial` | |
| Pinnacle bosses | Area entry | `Eye of the Storm` (Sirus), `The Shaper's Realm`, Absence-of-… arenas, etc. | Boss `<name>` | |
| Maven | NPC presence / area | `The Maven`, `The Envoy` / `The Maven's Crucible` | Maven / Boss | |

**Area names use apostrophe forms** (e.g. `Lunacy's Watch`, `The Shaper's Realm`) — that is what
Client.txt emits, per Exile Diary's `areas.json`. TraXile strips apostrophes; do not copy its forms.

### ❌ Untrackable — no `Client.txt` output at all
The mechanic happens entirely client-side; nothing reaches the log. We cannot know it was in the map.

| Mechanic | Why |
|---|---|
| **Breach** (in-map) | opening a Breach prints nothing; only a Breachstone *Domain* (separate area) is detectable |
| **Legion** (in-map) | opening a Monolith prints nothing; only `Domain of Timeless Conflict` is detectable |
| **Ritual** | altars/Tribute are pure UI; no dialogue, no area |
| **Metamorph** | in-map organ assembly prints nothing (Tane only speaks in his hideout lab) |
| **Abyss cracks** (in-map) | only the `Abyssal Depths` pit is detectable, not the crack |

### Candidates not yet shipped (documented, pending live-log verification)
- **Vaal corrupted side areas** (~55, e.g. `Side Chapel`, `Hidden Patch`) → could tag `Vaal`; held back
  because the generic names risk colliding with other areas — verify against a live log first.
- **Ultimatum / Sanctum / Ancestor outcome quotes** (win/loss/took-reward, boss-kill lines) — large
  `by_quote` tables exist in TraXile but several lines are unverified against a real client; presence is
  already covered by `by_npc`, so outcome detail is deferred.
- **Conqueror / Affliction / Settlers boss arenas** — present in one source only; verify before adding.

## Bottom line for the tracker
1. **Currency/profit** comes from Tier-2 item diffs (stash snapshot or character-inventory diff) priced
   via poe.ninja. This is the core of Phase 6.2/6.3.
2. **Mechanic data** is Tier-1 encounter detection (counts/tags/boss timing) from `Client.txt`.
3. **From data alone, do not promise** Hiveblood / Sulphite / Azurite / Kingsmarch Gold / Ritual Tribute /
   Sanctum Resolve+Aureus amounts — they are Tier-3 and unobtainable from log/API/diffs (the references we
   studied all give up on them). The *only* capture path is best-effort **screen OCR (Phase 6.6)**; absent that,
   surface them as "encountered", not as amounts.
4. Revisit this table every league: a reworked mechanic can move a resource between tiers (e.g. if GGG
   ever exposes Hiveblood as a character property, it becomes Tier-3-trackable).

## Sources
- PoE Wiki — Version 3.28.0
- Maxroll — 3.28 Mirage reveal summary; Kalguuran/Settlers & Breach farming guides
- poecurrency — 3.28 Hive Fortress farming (Hiveblood / Wombgifts)
- PoE Wiki — Breach, Ritual, The Forbidden Sanctum
