import { Component, createSignal, createResource, For, Show } from "solid-js";
import {
  getAffixesForBase,
  type BaseItem,
  type AffixEntry,
} from "../../lib/tauri";
import { formatMs } from "../../lib/format";

type AffixSource = "base" | "crafted" | "shaper" | "elder" | "crusader" | "hunter" | "redeemer" | "warlord" | "veiled" | "delve";

const SOURCE_ORDER: AffixSource[] = ["base", "crafted", "shaper", "elder", "crusader", "hunter", "redeemer", "warlord", "veiled", "delve"];

const SOURCE_LABELS: Record<AffixSource, string> = {
  base: "Base", crafted: "Crafted", shaper: "Shaper", elder: "Elder",
  crusader: "Crusader", hunter: "Hunter", redeemer: "Redeemer",
  warlord: "Warlord", veiled: "Veiled", delve: "Delve",
};

const SOURCE_COLORS: Record<AffixSource, string> = {
  base: "#a0a0a0", crafted: "#b4b4ff", shaper: "#6888cc", elder: "#909090",
  crusader: "#e8d44d", hunter: "#4da84d", redeemer: "#6699cc",
  warlord: "#cc4444", veiled: "#8866cc", delve: "#d4aa00",
};

function detectSource(entry: AffixEntry): AffixSource {
  const mod = entry.mod_data;
  if (mod.domain === "crafted") return "crafted";
  if (mod.domain === "unveiled" || mod.domain === "veiled") return "veiled";
  if (mod.domain === "delve") return "delve";
  const swTags = mod.spawn_weights.map((sw) => sw.tag).join(" ");
  if (swTags.includes("shaper")) return "shaper";
  if (swTags.includes("elder")) return "elder";
  if (swTags.includes("crusader")) return "crusader";
  if (swTags.includes("hunter")) return "hunter";
  if (swTags.includes("redeemer")) return "redeemer";
  if (swTags.includes("warlord")) return "warlord";
  return "base";
}

interface ModGroup {
  groupName: string;
  tiers: { entry: AffixEntry; tierNum: number }[];
  totalWeight: number;
}

function buildModGroups(entries: AffixEntry[]): ModGroup[] {
  const byGroup = new Map<string, AffixEntry[]>();
  for (const e of entries) {
    const g = e.mod_data.group || e.mod_data.id;
    if (!byGroup.has(g)) byGroup.set(g, []);
    byGroup.get(g)!.push(e);
  }

  const groups: ModGroup[] = [];
  for (const [groupName, items] of byGroup) {
    items.sort((a, b) => b.mod_data.required_level - a.mod_data.required_level);
    const tiers = items.map((entry, i) => ({ entry, tierNum: i + 1 }));
    const totalWeight = items.reduce((sum, e) => sum + e.effective_weight, 0);
    groups.push({ groupName, tiers, totalWeight });
  }
  groups.sort((a, b) => b.totalWeight - a.totalWeight);
  return groups;
}

function formatStatRange(stat: { id: string; min: number; max: number }): string {
  let label = stat.id.replace(/_/g, " ").replace(/local /i, "").replace(/base /i, "");
  if (stat.min === stat.max) return `${stat.min} ${label}`;
  return `(${stat.min}–${stat.max}) ${label}`;
}

function friendlyGroupName(group: string): string {
  return group
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/^Local\s*/i, "")
    .replace(/Percent$/i, " %");
}

// ---- Components ----

const BaseDetail: Component<{
  item: BaseItem;
  onBack: () => void;
}> = (props) => {
  const [itemLevel, setItemLevel] = createSignal(84);

  const [affixData] = createResource(
    () => props.item.tags,
    (tags) => getAffixesForBase(tags)
  );

  const prefixes = () => affixData()?.affixes.filter((a) => a.mod_data.generation_type === "prefix") ?? [];
  const suffixes = () => affixData()?.affixes.filter((a) => a.mod_data.generation_type === "suffix") ?? [];

  return (
    <div class="flex flex-col gap-4">
      {/* Header */}
      <div class="flex items-center gap-3">
        <button class="text-poe-accent hover:underline text-sm" onClick={props.onBack}>
          &larr; Back
        </button>
        <div class="flex items-center gap-3">
          <Show when={props.item.image_url}>
            <img
              src={props.item.image_url!}
              alt={props.item.name}
              class="h-10 object-contain"
              style={{ "image-rendering": "pixelated" }}
            />
          </Show>
          <div>
            <h2 class="text-poe-normal font-bold text-lg">{props.item.name}</h2>
            <span class="text-poe-muted text-xs">
              {props.item.item_class} &middot; Base Level {props.item.drop_level}
            </span>
          </div>
        </div>
        <div class="ml-auto flex items-center gap-2">
          <label class="text-poe-muted text-xs">iLvl</label>
          <input
            type="number"
            min="1"
            max="100"
            value={itemLevel()}
            onInput={(e) => {
              const v = parseInt(e.currentTarget.value);
              if (v >= 1 && v <= 100) setItemLevel(v);
            }}
            class="w-14 px-2 py-1 bg-poe-bg border border-poe-border rounded text-poe-accent text-sm text-center font-bold focus:border-poe-accent focus:outline-none"
          />
        </div>
      </div>

      <Show when={affixData.loading}>
        <div class="text-poe-muted text-sm text-center py-4">Loading affixes...</div>
      </Show>

      <Show when={affixData()}>
        <div class="text-poe-muted text-xs">
          {affixData()!.affixes.length} affixes in {formatMs(affixData()!.query_ms)}
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <AffixColumn title="PREFIX" entries={prefixes()} itemLevel={itemLevel()} />
          <AffixColumn title="SUFFIX" entries={suffixes()} itemLevel={itemLevel()} />
        </div>
      </Show>
    </div>
  );
};

const AffixColumn: Component<{
  title: string;
  entries: AffixEntry[];
  itemLevel: number;
}> = (props) => {
  const sourceGroups = () => {
    const bySource = new Map<AffixSource, AffixEntry[]>();
    for (const entry of props.entries) {
      const src = detectSource(entry);
      if (!bySource.has(src)) bySource.set(src, []);
      bySource.get(src)!.push(entry);
    }
    const result: { source: AffixSource; modGroups: ModGroup[] }[] = [];
    for (const src of SOURCE_ORDER) {
      const entries = bySource.get(src);
      if (entries && entries.length > 0) {
        result.push({ source: src, modGroups: buildModGroups(entries) });
      }
    }
    return result;
  };

  return (
    <div class="flex flex-col gap-3">
      <h3 class="text-poe-accent font-bold text-sm uppercase tracking-wider border-b border-poe-border pb-1">
        {props.title}
        <span class="text-poe-muted font-normal ml-2">({props.entries.length})</span>
      </h3>
      <For each={sourceGroups()}>
        {(sg) => (
          <div class="flex flex-col gap-2">
            <h4 class="text-xs font-bold uppercase tracking-wide" style={{ color: SOURCE_COLORS[sg.source] }}>
              {SOURCE_LABELS[sg.source]}
              <span class="text-poe-muted font-normal ml-1">
                ({sg.modGroups.reduce((s, g) => s + g.tiers.length, 0)})
              </span>
            </h4>
            <For each={sg.modGroups}>
              {(group) => <ModGroupRow group={group} itemLevel={props.itemLevel} />}
            </For>
          </div>
        )}
      </For>
      <Show when={props.entries.length === 0}>
        <div class="text-poe-muted text-xs italic">None</div>
      </Show>
    </div>
  );
};

const ModGroupRow: Component<{ group: ModGroup; itemLevel: number }> = (props) => {
  const [expanded, setExpanded] = createSignal(false);

  // Count how many tiers are available at current iLvl
  const availableCount = () => props.group.tiers.filter((t) => t.entry.mod_data.required_level <= props.itemLevel).length;
  const allUnavailable = () => availableCount() === 0;
  // Weight sum of only available tiers
  const availableWeight = () =>
    props.group.tiers
      .filter((t) => t.entry.mod_data.required_level <= props.itemLevel)
      .reduce((sum, t) => sum + t.entry.effective_weight, 0);

  return (
    <div
      class="rounded text-xs border transition-colors"
      classList={{
        "bg-poe-surface border-poe-border": !allUnavailable(),
        "bg-poe-bg/50 border-poe-border/40": allUnavailable(),
      }}
    >
      {/* Group header */}
      <button
        class="w-full px-2 py-1.5 flex items-center gap-2 text-left transition-colors"
        classList={{
          "hover:bg-poe-bg": !allUnavailable(),
          "opacity-40": allUnavailable(),
        }}
        onClick={() => setExpanded(!expanded())}
      >
        <span class="text-poe-muted w-4 text-center flex-shrink-0">
          {expanded() ? "\u25BE" : "\u25B8"}
        </span>
        <span
          class="font-medium flex-1 truncate"
          classList={{
            "text-poe-text": !allUnavailable(),
            "text-poe-muted line-through": allUnavailable(),
          }}
        >
          {friendlyGroupName(props.group.groupName)}
        </span>
        <span class="text-poe-muted flex-shrink-0 tabular-nums">
          {availableCount()}/{props.group.tiers.length}
        </span>
        <span
          class="font-mono flex-shrink-0 w-12 text-right tabular-nums"
          classList={{
            "text-amber-400": !allUnavailable(),
            "text-poe-muted/50": allUnavailable(),
          }}
        >
          {availableWeight()}
        </span>
      </button>

      {/* Expanded tier list */}
      <Show when={expanded()}>
        <div class="border-t border-poe-border">
          <For each={props.group.tiers}>
            {(tier) => {
              const available = () => tier.entry.mod_data.required_level <= props.itemLevel;
              return (
                <div
                  class="px-2 py-1 flex items-start gap-2 border-b border-poe-border/50 last:border-b-0 transition-all relative"
                  classList={{
                    "hover:bg-poe-bg": available(),
                    "bg-red-950/20": !available(),
                  }}
                >
                  {/* Locked overlay bar on left edge */}
                  <Show when={!available()}>
                    <div class="absolute left-0 top-0 bottom-0 w-0.5 bg-red-800/60" />
                  </Show>

                  <span
                    class="w-6 flex-shrink-0 text-right font-bold"
                    classList={{
                      "text-poe-muted": available(),
                      "text-red-900/60": !available(),
                    }}
                  >
                    T{tier.tierNum}
                  </span>

                  <div
                    class="flex-1 min-w-0 transition-opacity"
                    classList={{
                      "opacity-100": available(),
                      "opacity-30 line-through decoration-red-800/40": !available(),
                    }}
                  >
                    <div class="text-poe-text truncate">
                      {tier.entry.mod_data.name || tier.entry.mod_data.id}
                    </div>
                    <div class="text-poe-muted">
                      <For each={tier.entry.mod_data.stats}>
                        {(stat) => <span class="mr-2">{formatStatRange(stat)}</span>}
                      </For>
                    </div>
                  </div>

                  <span
                    class="flex-shrink-0 w-12 text-right text-[10px] font-mono"
                    classList={{
                      "text-poe-muted": available(),
                      "text-red-400/50": !available(),
                    }}
                  >
                    iLvl {tier.entry.mod_data.required_level}
                  </span>

                  <span
                    class="font-mono flex-shrink-0 w-12 text-right tabular-nums"
                    classList={{
                      "text-amber-400": available(),
                      "text-amber-400/20": !available(),
                    }}
                  >
                    {tier.entry.effective_weight}
                  </span>
                </div>
              );
            }}
          </For>
        </div>
      </Show>
    </div>
  );
};

export default BaseDetail;
