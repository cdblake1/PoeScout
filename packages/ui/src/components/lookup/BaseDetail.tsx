import { Component, createSignal, createEffect, createResource, For, Show } from "solid-js";
import {
  getAffixesForBase,
  type BaseItem,
  type AffixEntry,
} from "../../lib/tauri";
import { formatMs } from "../../lib/format";

// Influence detection: mod spawn_weight tags → influence source
const INFLUENCE_TAG_SUFFIXES: [string, string][] = [
  ["_shaper", "shaper"],
  ["_elder", "elder"],
  ["_crusader", "crusader"],
  ["_adjudicator", "warlord"],
  ["_basilisk", "hunter"],
  ["_eyrie", "redeemer"],
];

function detectSource(entry: AffixEntry): string {
  const domain = entry.mod_data.domain;
  if (domain === "crafted") return "crafted";
  if (domain === "delve") return "delve";
  if (domain === "unveiled") return "unveiled";
  if (domain === "veiled") return "veiled";
  for (const sw of entry.mod_data.spawn_weights) {
    if (sw.weight <= 0) continue;
    for (const [suffix, source] of INFLUENCE_TAG_SUFFIXES) {
      if (sw.tag.endsWith(suffix)) return source;
    }
  }
  return "base";
}

const SOURCE_ORDER = [
  "base", "shaper", "elder", "crusader", "redeemer", "hunter", "warlord",
  "delve", "crafted", "unveiled", "veiled",
];

const SOURCE_LABELS: Record<string, { label: string; color: string }> = {
  base:      { label: "Base",              color: "#c0c0c0" },
  shaper:    { label: "Shaper",            color: "#6688cc" },
  elder:     { label: "Elder",             color: "#8866aa" },
  crusader:  { label: "Crusader",          color: "#ddaa44" },
  redeemer:  { label: "Redeemer",          color: "#44bbdd" },
  hunter:    { label: "Hunter",            color: "#44aa66" },
  warlord:   { label: "Warlord",           color: "#cc4444" },
  delve:     { label: "Delve",             color: "#ccaa44" },
  crafted:   { label: "Crafting Bench",    color: "#b4b4ff" },
  unveiled:  { label: "Unveiled",          color: "#99cc66" },
  veiled:    { label: "Veiled",            color: "#778899" },
};

const GEN_TYPE_ORDER = [
  "prefix", "suffix", "corrupted", "eater_of_worlds_implicit", "searing_exarch_implicit",
  "scourge_benefit", "scourge_detriment", "crucible_tree", "enchantment",
  "unique", "talisman",
];

const GEN_TYPE_LABELS: Record<string, string> = {
  prefix: "Prefix",
  suffix: "Suffix",
  corrupted: "Corrupted",
  eater_of_worlds_implicit: "Eater of Worlds",
  searing_exarch_implicit: "Searing Exarch",
  scourge_benefit: "Scourge (Benefit)",
  scourge_detriment: "Scourge (Detriment)",
  crucible_tree: "Crucible",
  enchantment: "Enchantment",
  unique: "Unique",
  talisman: "Talisman",
};

const OTHER_COLORS: Record<string, string> = {
  corrupted: "#d02020",
  eater_of_worlds_implicit: "#4488aa",
  searing_exarch_implicit: "#cc6622",
  scourge_benefit: "#cc8800",
  scourge_detriment: "#884400",
  crucible_tree: "#dd6633",
  enchantment: "#b4b4ff",
  unique: "#af6025",
  talisman: "#44aa44",
};

interface ModGroup {
  groupName: string;
  tiers: { entry: AffixEntry; tierNum: number }[];
  totalWeight: number;
}

function genericModText(text: string): string {
  return text
    .replace(/\((\d+)-(\d+)\)/g, "#")
    .replace(/\+#/g, "#")
    .replace(/(?<![a-zA-Z])(\d+(\.\d+)?)(?![a-zA-Z])/g, "#")
    .replace(/#+/g, "#");
}

function statSignature(entry: AffixEntry): string {
  return entry.mod_data.stats.map((s) => s.id).sort().join("|");
}

function buildModGroups(entries: AffixEntry[]): ModGroup[] {
  const byStats = new Map<string, AffixEntry[]>();
  for (const e of entries) {
    const key = statSignature(e) || e.mod_data.id;
    if (!byStats.has(key)) byStats.set(key, []);
    byStats.get(key)!.push(e);
  }

  const groups: ModGroup[] = [];
  for (const [_key, items] of byStats) {
    items.sort((a, b) => b.mod_data.required_level - a.mod_data.required_level);
    const tiers = items.map((entry, i) => ({ entry, tierNum: i + 1 }));
    const totalWeight = items.reduce((sum, e) => sum + e.effective_weight, 0);
    const modText = items[0].mod_data.text;
    const displayName = modText ? genericModText(modText) : (items[0].mod_data.stats.map((s) => s.id).join(", ") || items[0].mod_data.group || items[0].mod_data.id);
    groups.push({ groupName: displayName, tiers, totalWeight });
  }
  groups.sort((a, b) => b.totalWeight - a.totalWeight);
  return groups;
}

function formatStatRange(stat: { id: string; min: number; max: number }): string {
  let label = stat.id.replace(/_/g, " ").replace(/local /i, "").replace(/base /i, "");
  if (stat.min === stat.max) return `${stat.min} ${label}`;
  return `(${stat.min}–${stat.max}) ${label}`;
}

// Tag color mapping — WCAG AA compliant (≥4.5:1 contrast ratio)
// Each tag has a visually distinct hue to avoid confusion
const TAG_COLORS: Record<string, { bg: string; text: string }> = {
  // Elements — distinct hues
  fire:        { bg: "#7f1d1d", text: "#fecaca" },  // red
  cold:        { bg: "#1e3a5f", text: "#bfdbfe" },  // blue
  lightning:   { bg: "#713f12", text: "#fef08a" },  // yellow
  chaos:       { bg: "#581c87", text: "#e9d5ff" },  // purple
  physical:    { bg: "#78350f", text: "#fed7aa" },  // amber/orange
  elemental:   { bg: "#5b2100", text: "#ffcba4" },  // burnt orange (distinct from fire red)
  // Combat roles — warm vs cool tones
  attack:      { bg: "#6b2142", text: "#fbb6ce" },  // rose/magenta (distinct from fire red)
  caster:      { bg: "#312e81", text: "#c7d2fe" },  // indigo
  speed:       { bg: "#064e3b", text: "#a7f3d0" },  // emerald
  critical:    { bg: "#854d0e", text: "#fef08a" },  // dark gold
  // Defences — each unique
  life:        { bg: "#9f1239", text: "#fda4af" },  // pink-red (distinct from fire's darker red)
  mana:        { bg: "#1e3a8a", text: "#bfdbfe" },  // royal blue
  resistance:  { bg: "#166534", text: "#bbf7d0" },  // green
  defences:    { bg: "#374151", text: "#e5e7eb" },  // gray
  armour:      { bg: "#6b4c1e", text: "#fde68a" },  // dark khaki
  evasion:     { bg: "#1a5c3a", text: "#a7f3d0" },  // teal-green (distinct from resistance green)
  energy_shield: { bg: "#1e40af", text: "#bfdbfe" }, // deeper blue (distinct from mana)
  // Resources & misc — all distinct
  attribute:   { bg: "#581c87", text: "#e9d5ff" },  // purple
  gem:         { bg: "#134e4a", text: "#99f6e4" },  // teal
  minion:      { bg: "#365314", text: "#d9f99d" },  // lime
  aura:        { bg: "#164e63", text: "#a5f3fc" },  // cyan
  curse:       { bg: "#831843", text: "#fbcfe8" },  // pink
  damage:      { bg: "#7c2d12", text: "#fed7aa" },  // orange
  elemental_damage: { bg: "#5b2100", text: "#ffcba4" },  // burnt orange (matches elemental)
  physical_damage:  { bg: "#78350f", text: "#fed7aa" },  // amber (matches physical)
  block:       { bg: "#3f4f3f", text: "#c8e6c8" },  // muted sage
};

// Generate a deterministic color from a tag string using hue hashing
function hashTagColor(tag: string): { bg: string; text: string } {
  let hash = 0;
  for (let i = 0; i < tag.length; i++) {
    hash = tag.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue = ((hash % 360) + 360) % 360;
  // Dark, saturated bg + light text for contrast
  return {
    bg: `hsl(${hue}, 40%, 20%)`,
    text: `hsl(${hue}, 60%, 85%)`,
  };
}

function getTagColor(tag: string): { bg: string; text: string } {
  return TAG_COLORS[tag] ?? hashTagColor(tag);
}

function friendlyGroupName(group: string): string {
  return group
    .split(", ")
    .map((s) =>
      s.replace(/_/g, " ").replace(/local /i, "").replace(/base /i, "")
        .replace(/ \+%$/, " %").replace(/ \+$/, "").trim()
    )
    .join(" + ");
}

// Detect blocked mod categories from implicit text (e.g. "Cannot roll Caster Modifiers")
function blockedModCategories(item: BaseItem): string[] {
  const blocked: string[] = [];
  for (const text of item.implicit_text) {
    const m = text.match(/Cannot roll (\w+) Modifiers/i);
    if (m) blocked.push(m[1].toLowerCase());
  }
  return blocked;
}

// ---- Components ----

const BaseDetail: Component<{
  item: BaseItem;
  initialItemLevel?: number | null;
  onBack: () => void;
}> = (props) => {
  const [itemLevel, setItemLevel] = createSignal(props.initialItemLevel ?? 84);
  createEffect(() => {
    if (props.initialItemLevel != null) setItemLevel(props.initialItemLevel);
  });
  const blocked = () => blockedModCategories(props.item);
  const DEFAULT_HIDDEN_TAGS = new Set([
    "resource", "physical_damage", "flat_life_regen",
    "elemental", "influence_mod", "unveiled_mod",
    "elemental_damage", "red_herring", "support",
    "chaos_damage", "damage", "caster_damage", "dot_multi",
  ]);
  const [hiddenTags, setHiddenTags] = createSignal<Set<string>>(DEFAULT_HIDDEN_TAGS);

  const hideTag = (tag: string) => {
    setHiddenTags(prev => { const next = new Set(prev); next.add(tag); return next; });
  };
  const restoreTag = (tag: string) => {
    setHiddenTags(prev => { const next = new Set(prev); next.delete(tag); return next; });
  };

  const [affixData] = createResource(
    () => props.item.tags,
    (tags) => getAffixesForBase(tags)
  );

  interface SourceSection {
    source: string;
    sourceLabel: string;
    color: string;
    modGroups: ModGroup[];
  }

  const layout = () => {
    const data = affixData();
    if (!data) return { prefixes: [] as SourceSection[], suffixes: [] as SourceSection[], other: [] as { genType: string; genTypeLabel: string; color: string; modGroups: ModGroup[] }[] };

    const buckets = new Map<string, Map<string, AffixEntry[]>>();

    for (const a of data.affixes) {
      const source = detectSource(a);
      const gt = a.mod_data.generation_type;
      if (!buckets.has(source)) buckets.set(source, new Map());
      const inner = buckets.get(source)!;
      if (!inner.has(gt)) inner.set(gt, []);
      inner.get(gt)!.push(a);
    }

    const buildColumn = (genType: string): SourceSection[] => {
      const result: SourceSection[] = [];
      const orderedSources = [...SOURCE_ORDER.filter(s => buckets.has(s)), ...[...buckets.keys()].filter(s => !SOURCE_ORDER.includes(s))];
      for (const source of orderedSources) {
        const inner = buckets.get(source)!;
        const entries = inner.get(genType);
        if (entries && entries.length > 0) {
          const info = SOURCE_LABELS[source] ?? { label: source, color: "#a0a0a0" };
          result.push({
            source,
            sourceLabel: info.label,
            color: info.color,
            modGroups: buildModGroups(entries),
          });
        }
      }
      return result;
    };

    const prefixes = buildColumn("prefix");
    const suffixes = buildColumn("suffix");

    const otherGenTypes = new Map<string, AffixEntry[]>();
    for (const [_source, inner] of buckets) {
      for (const [gt, entries] of inner) {
        if (gt === "prefix" || gt === "suffix") continue;
        if (!otherGenTypes.has(gt)) otherGenTypes.set(gt, []);
        otherGenTypes.get(gt)!.push(...entries);
      }
    }

    const OTHER_ORDER = GEN_TYPE_ORDER.filter(gt => gt !== "prefix" && gt !== "suffix");
    const otherKeys = [...OTHER_ORDER.filter(gt => otherGenTypes.has(gt)), ...[...otherGenTypes.keys()].filter(gt => !OTHER_ORDER.includes(gt))];

    const other = otherKeys.map(gt => {
      const label = GEN_TYPE_LABELS[gt] ?? gt;
      const color = OTHER_COLORS[gt] ?? "#a0a0a0";
      return { genType: gt, genTypeLabel: label, color, modGroups: buildModGroups(otherGenTypes.get(gt)!) };
    });

    return { prefixes, suffixes, other };
  };

  return (
    <div class="flex flex-col gap-5">
      {/* Header */}
      <div class="flex items-center gap-4">
        <button class="text-poe-accent hover:underline text-base" onClick={props.onBack}>
          &larr; Back
        </button>
        <div class="flex items-center gap-4">
          <Show when={props.item.image_url}>
            <img
              src={props.item.image_url!}
              alt={props.item.name}
              class="h-12 object-contain"
              style={{ "image-rendering": "pixelated" }}
            />
          </Show>
          <div>
            <h2 class="text-poe-normal font-bold text-xl">{props.item.name}</h2>
            <span class="text-poe-muted text-sm">
              {props.item.item_class} &middot; Base Level {props.item.drop_level}
            </span>
          </div>
        </div>
        <div class="ml-auto flex items-center gap-2">
          <label class="text-poe-muted text-sm">iLvl</label>
          <input
            type="number"
            min="1"
            max="100"
            value={itemLevel()}
            onInput={(e) => {
              const v = parseInt(e.currentTarget.value);
              if (v >= 1 && v <= 100) setItemLevel(v);
            }}
            class="w-16 px-2 py-1.5 bg-poe-bg border border-poe-border rounded text-poe-accent text-base text-center font-bold focus:border-poe-accent focus:outline-none"
          />
        </div>
      </div>

      <Show when={blocked().length > 0}>
        <div class="flex items-center gap-2 text-xs text-red-400/80">
          <span>&#x2298; Cannot roll:</span>
          <For each={blocked()}>
            {(cat) => <span class="px-1.5 py-0.5 rounded bg-red-950/40 border border-red-800/30 capitalize">{cat}</span>}
          </For>
        </div>
      </Show>

      <Show when={hiddenTags().size > 0}>
        <div class="flex items-center gap-2 flex-wrap text-xs">
          <span class="text-poe-muted">Hidden:</span>
          <For each={[...hiddenTags()]}>
            {(tag) => (
              <button
                class="px-1.5 py-0.5 rounded bg-poe-bg border border-poe-border/50 text-poe-muted hover:text-poe-text hover:border-poe-border line-through"
                onClick={() => restoreTag(tag)}
                title={`Click to restore "${tag.replace(/_/g, " ")}"`}
              >
                {tag.replace(/_/g, " ")}
              </button>
            )}
          </For>
        </div>
      </Show>

      <Show when={affixData.loading}>
        <div class="text-poe-muted text-base text-center py-4">Loading affixes...</div>
      </Show>

      <Show when={affixData()}>
        <div class="text-poe-muted text-sm">
          {affixData()!.affixes.length} affixes in {formatMs(affixData()!.query_ms)}
        </div>

        {/* Section 1: Prefix / Suffix two-column layout */}
        <Show when={layout().prefixes.length > 0 || layout().suffixes.length > 0}>
          <div class="grid grid-cols-2 gap-5">
            {/* Left column: Prefixes */}
            <div class="flex flex-col gap-4">
              <h2 class="text-base font-bold uppercase tracking-wider text-[#d4aa70] border-b border-[#d4aa7044] pb-1">
                Prefixes
              </h2>
              <For each={layout().prefixes}>
                {(section) => (
                  <SourceGroup section={section} itemLevel={itemLevel()} hiddenTags={hiddenTags()} onHideTag={hideTag} blockedCategories={blocked()} />
                )}
              </For>
            </div>

            {/* Right column: Suffixes */}
            <div class="flex flex-col gap-4">
              <h2 class="text-base font-bold uppercase tracking-wider text-[#70aad4] border-b border-[#70aad444] pb-1">
                Suffixes
              </h2>
              <For each={layout().suffixes}>
                {(section) => (
                  <SourceGroup section={section} itemLevel={itemLevel()} hiddenTags={hiddenTags()} onHideTag={hideTag} blockedCategories={blocked()} />
                )}
              </For>
            </div>
          </div>
        </Show>

        {/* Section 2: Other mod types (single column) */}
        <Show when={layout().other.length > 0}>
          <For each={layout().other}>
            {(section) => (
              <div class="flex flex-col gap-3">
                <h3
                  class="text-base font-bold uppercase tracking-wider border-b pb-1"
                  style={{ color: section.color, "border-color": section.color + "44" }}
                >
                  {section.genTypeLabel}
                  <span class="text-poe-muted font-normal ml-2 normal-case">
                    ({section.modGroups.reduce((s, g) => s + g.tiers.length, 0)})
                  </span>
                </h3>
                <For each={section.modGroups}>
                  {(group) => <ModGroupRow group={group} itemLevel={itemLevel()} hiddenTags={hiddenTags()} onHideTag={hideTag} blockedCategories={blocked()} />}
                </For>
                <div class="flex items-center px-3 py-1.5 text-sm text-poe-muted border-t border-poe-border/30">
                  <span class="font-semibold flex-1">Total</span>
                  <span class="flex-shrink-0 tabular-nums">{section.modGroups.reduce((s, g) => s + g.tiers.length, 0)}</span>
                  <span class="font-mono flex-shrink-0 w-14 text-right tabular-nums text-amber-400/70">
                    {section.modGroups.reduce((s, g) => s + g.totalWeight, 0)}
                  </span>
                </div>
              </div>
            )}
          </For>
        </Show>
      </Show>
    </div>
  );
};

const SourceGroup: Component<{
  section: { source: string; sourceLabel: string; color: string; modGroups: ModGroup[] };
  itemLevel: number;
  hiddenTags: Set<string>;
  onHideTag: (tag: string) => void;
  blockedCategories: string[];
}> = (props) => {
  const totalTiers = () => props.section.modGroups.reduce((s, g) => s + g.tiers.length, 0);
  const totalWeight = () => props.section.modGroups.reduce((s, g) => s + g.totalWeight, 0);

  return (
    <div class="flex flex-col gap-2">
      <h4
        class="text-sm font-semibold uppercase tracking-wide pl-1"
        style={{ color: props.section.color }}
      >
        {props.section.sourceLabel}
        <span class="text-poe-muted font-normal ml-1 normal-case">
          ({totalTiers()})
        </span>
      </h4>
      <For each={props.section.modGroups}>
        {(group) => <ModGroupRow group={group} itemLevel={props.itemLevel} hiddenTags={props.hiddenTags} onHideTag={props.onHideTag} blockedCategories={props.blockedCategories} />}
      </For>
      <div class="flex items-center px-3 py-1.5 text-sm text-poe-muted border-t border-poe-border/30">
        <span class="font-semibold flex-1">Total</span>
        <span class="flex-shrink-0 tabular-nums">{totalTiers()}</span>
        <span class="font-mono flex-shrink-0 w-14 text-right tabular-nums text-amber-400/70">
          {totalWeight()}
        </span>
      </div>
    </div>
  );
};

const ModGroupRow: Component<{
  group: ModGroup;
  itemLevel: number;
  hiddenTags: Set<string>;
  onHideTag: (tag: string) => void;
  blockedCategories: string[];
}> = (props) => {
  const [expanded, setExpanded] = createSignal(false);

  const modTags = () => {
    const tags = props.group.tiers[0]?.entry.mod_data.tags ?? [];
    return tags.filter((t) => !props.hiddenTags.has(t));
  };

  const isBlocked = () => {
    const tags = props.group.tiers[0]?.entry.mod_data.tags ?? [];
    return props.blockedCategories.some(cat => tags.includes(cat));
  };

  const availableCount = () => props.group.tiers.filter((t) => t.entry.mod_data.required_level <= props.itemLevel).length;
  const allUnavailable = () => availableCount() === 0 || isBlocked();
  const availableWeight = () =>
    isBlocked() ? 0 :
    props.group.tiers
      .filter((t) => t.entry.mod_data.required_level <= props.itemLevel)
      .reduce((sum, t) => sum + t.entry.effective_weight, 0);

  return (
    <div
      class="rounded text-sm border transition-colors"
      classList={{
        "bg-poe-surface border-poe-border": !allUnavailable(),
        "bg-poe-bg/50 border-poe-border/40": allUnavailable(),
      }}
    >
      <button
        class="w-full px-3 py-2 flex items-center gap-2 text-left transition-colors"
        classList={{
          "hover:bg-poe-bg": !allUnavailable(),
          "opacity-40": allUnavailable(),
        }}
        onClick={() => setExpanded(!expanded())}
      >
        <span class="text-poe-muted w-5 text-center flex-shrink-0">
          {expanded() ? "\u25BE" : "\u25B8"}
        </span>
        <span
          class="font-medium flex-1 min-w-0 flex items-center gap-2 overflow-hidden"
          classList={{
            "text-poe-text": !allUnavailable(),
            "text-poe-muted line-through": allUnavailable(),
          }}
        >
          <span class="truncate">{friendlyGroupName(props.group.groupName)}</span>
          <span class="flex gap-1 flex-shrink-0">
            <For each={modTags()}>
              {(tag) => {
                const c = getTagColor(tag);
                return (
                  <span
                    class="px-1 py-0 rounded text-[10px] font-normal leading-tight cursor-pointer hover:opacity-70"
                    style={{ "background-color": c.bg, color: c.text }}
                    title={`Right-click to hide "${tag.replace(/_/g, " ")}"`}
                    onContextMenu={(e) => { e.preventDefault(); props.onHideTag(tag); }}
                  >
                    {tag.replace(/_/g, " ")}
                  </span>
                );
              }}
            </For>
          </span>
        </span>
        <span class="text-poe-muted flex-shrink-0 tabular-nums">
          {availableCount()}/{props.group.tiers.length}
        </span>
        <span
          class="font-mono flex-shrink-0 w-14 text-right tabular-nums"
          classList={{
            "text-amber-400": !allUnavailable(),
            "text-poe-muted/50": allUnavailable(),
          }}
        >
          {availableWeight()}
        </span>
      </button>

      <Show when={expanded()}>
        <div class="border-t border-poe-border">
          <For each={props.group.tiers}>
            {(tier) => {
              const available = () => !isBlocked() && tier.entry.mod_data.required_level <= props.itemLevel;
              return (
                <div
                  class="px-3 py-1.5 flex items-start gap-3 border-b border-poe-border/50 last:border-b-0 transition-all relative"
                  classList={{
                    "hover:bg-poe-bg": available(),
                    "bg-red-950/20": !available(),
                  }}
                >
                  <Show when={!available()}>
                    <div class="absolute left-0 top-0 bottom-0 w-0.5 bg-red-800/60" />
                  </Show>

                  <span
                    class="w-8 flex-shrink-0 text-right font-bold text-sm"
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
                    <div class="text-poe-text text-sm">
                      {tier.entry.mod_data.text || tier.entry.mod_data.name || tier.entry.mod_data.id}
                    </div>
                  </div>

                  <span
                    class="flex-shrink-0 w-14 text-right text-xs font-mono"
                    classList={{
                      "text-poe-muted": available(),
                      "text-red-400/50": !available(),
                    }}
                  >
                    iLvl {tier.entry.mod_data.required_level}
                  </span>

                  <span
                    class="font-mono flex-shrink-0 w-14 text-right tabular-nums text-sm"
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
