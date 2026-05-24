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
