import { Component, createSignal, createResource, For, Show, onMount } from "solid-js";
import {
  searchBases,
  listItemClasses,
  listBasesByClass,
  type BaseItem,
  type BaseSearchQuery,
  type BaseSearchResult,
} from "../../lib/tauri";
import { formatMs } from "../../lib/format";
import BaseDetail from "./BaseDetail";
import type { ImplicitStat } from "../../lib/tauri";

function formatImplicit(s: ImplicitStat): string {
  const label = s.stat_id
    .replace(/_/g, " ")
    .replace(/base /i, "")
    .replace(/ ?%$/, "%");
  if (s.min === s.max) return `+${s.min} ${label}`;
  return `+(${s.min}–${s.max}) ${label}`;
}

// Attribute tag → display info
const ATTRIBUTE_GROUPS: Record<string, { label: string; color: string }> = {
  str_armour:         { label: "Strength",              color: "#c74038" },
  dex_armour:         { label: "Dexterity",             color: "#36b34e" },
  int_armour:         { label: "Intelligence",          color: "#5090d0" },
  str_dex_armour:     { label: "Strength / Dexterity",  color: "#c7a038" },
  str_int_armour:     { label: "Strength / Intelligence", color: "#9060a0" },
  dex_int_armour:     { label: "Dexterity / Intelligence", color: "#40a0a0" },
  str_dex_int_armour: { label: "STR / DEX / INT",       color: "#a0a0a0" },
};

const ATTRIBUTE_TAG_KEYS = Object.keys(ATTRIBUTE_GROUPS);

function getAttributeTag(item: BaseItem): string | null {
  for (const tag of item.tags) {
    if (ATTRIBUTE_TAG_KEYS.includes(tag)) return tag;
  }
  return null;
}

function groupByAttribute(items: BaseItem[]): { tag: string | null; items: BaseItem[] }[] {
  const grouped = new Map<string | null, BaseItem[]>();
  for (const item of items) {
    const tag = getAttributeTag(item);
    if (!grouped.has(tag)) grouped.set(tag, []);
    grouped.get(tag)!.push(item);
  }
  // Sort: known attribute groups first (in defined order), then ungrouped
  const result: { tag: string | null; items: BaseItem[] }[] = [];
  for (const key of ATTRIBUTE_TAG_KEYS) {
    if (grouped.has(key)) result.push({ tag: key, items: grouped.get(key)! });
  }
  if (grouped.has(null)) result.push({ tag: null, items: grouped.get(null)! });
  return result;
}

const EQUIPMENT_CLASSES = [
  "Body Armour", "Helmet", "Gloves", "Boots", "Shield",
  "Ring", "Amulet", "Belt", "Quiver",
  "Bow", "Claw", "Dagger", "Rune Dagger", "Wand", "Sceptre",
  "One Hand Sword", "Thrusting One Hand Sword", "One Hand Axe", "One Hand Mace",
  "Two Hand Sword", "Two Hand Axe", "Two Hand Mace", "Staff", "Warstaff",
  "Jewel", "AbyssJewel",
  "LifeFlask", "ManaFlask", "HybridFlask", "UtilityFlask",
];

const BaseSearch: Component = () => {
  const [query, setQuery] = createSignal("");
  const [selectedClass, setSelectedClass] = createSignal<string | null>(null);
  const [searchResults, setSearchResults] = createSignal<BaseSearchResult | null>(null);
  const [classItems, setClassItems] = createSignal<BaseItem[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");
  const [selectedItem, setSelectedItem] = createSignal<BaseItem | null>(null);
  const [classes] = createResource(listItemClasses);

  let debounceTimer: number | undefined;

  const doSearch = async () => {
    const text = query().trim();
    if (!text) {
      setSearchResults(null);
      return;
    }

    setLoading(true);
    setError("");
    setSelectedClass(null);

    try {
      const q: BaseSearchQuery = { text, limit: 200 };
      const res = await searchBases(q);
      setSearchResults(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const onInput = (val: string) => {
    setQuery(val);
    clearTimeout(debounceTimer);
    if (!val.trim()) {
      setSearchResults(null);
      return;
    }
    debounceTimer = window.setTimeout(doSearch, 150);
  };

  const selectClass = async (cls: string) => {
    setSelectedClass(cls);
    setQuery("");
    setSearchResults(null);
    setLoading(true);
    setError("");

    try {
      const items = await listBasesByClass(cls);
      setClassItems(items);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const clearSelection = () => {
    setSelectedClass(null);
    setClassItems([]);
  };

  // Which items to show in the card grid
  const displayItems = () => {
    if (searchResults()) return searchResults()!.items;
    if (selectedClass()) return classItems();
    return [];
  };

  const showCategoryBrowser = () => !query().trim() && !selectedClass();

  return (
    <Show
      when={!selectedItem()}
      fallback={
        <BaseDetail
          item={selectedItem()!}
          onBack={() => setSelectedItem(null)}
        />
      }
    >
    <div class="flex flex-col gap-4">
      {/* Search bar */}
      <input
        type="text"
        placeholder="Search base items... (e.g. Vaal Regalia, Hubris Circlet)"
        class="w-full px-3 py-2 bg-poe-surface border border-poe-border rounded text-poe-text placeholder-poe-muted focus:border-poe-accent focus:outline-none"
        value={query()}
        onInput={(e) => onInput(e.currentTarget.value)}
      />

      <Show when={error()}>
        <div class="text-red-500 text-sm">{error()}</div>
      </Show>

      {/* Breadcrumb when a class is selected */}
      <Show when={selectedClass()}>
        <div class="flex items-center gap-2 text-sm">
          <button
            class="text-poe-accent hover:underline"
            onClick={clearSelection}
          >
            All Categories
          </button>
          <span class="text-poe-muted">/</span>
          <span class="text-poe-text">{selectedClass()}</span>
          <span class="text-poe-muted">({classItems().length})</span>
        </div>
      </Show>

      {/* Search result info */}
      <Show when={searchResults()}>
        <div class="text-poe-muted text-xs">
          {searchResults()!.total} results in {formatMs(searchResults()!.query_ms)}
        </div>
      </Show>

      {/* Category browser — show when no search/selection active */}
      <Show when={showCategoryBrowser()}>
        <CategoryBrowser
          classes={classes() ?? []}
          onSelect={selectClass}
        />
      </Show>

      {/* Card grid — grouped by attribute when browsing a class */}
      <Show when={displayItems().length > 0}>
        <Show
          when={selectedClass() && !searchResults()}
          fallback={
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3">
              <For each={displayItems()}>
                {(item) => <BaseCard item={item} onClick={setSelectedItem} />}
              </For>
            </div>
          }
        >
          <For each={groupByAttribute(displayItems())}>
            {(group) => (
              <div class="flex flex-col gap-2 mb-4">
                <Show when={group.tag}>
                  <h3
                    class="text-sm font-bold uppercase tracking-wider border-b pb-1"
                    style={{
                      color: ATTRIBUTE_GROUPS[group.tag!].color,
                      "border-color": ATTRIBUTE_GROUPS[group.tag!].color + "44",
                    }}
                  >
                    {ATTRIBUTE_GROUPS[group.tag!].label}
                    <span class="text-poe-muted ml-2 font-normal normal-case">({group.items.length})</span>
                  </h3>
                </Show>
                <Show when={!group.tag}>
                  <h3 class="text-sm font-bold uppercase tracking-wider text-poe-muted border-b border-poe-border pb-1">
                    Other
                    <span class="ml-2 font-normal normal-case">({group.items.length})</span>
                  </h3>
                </Show>
                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3">
                  <For each={group.items}>
                    {(item) => <BaseCard item={item} onClick={setSelectedItem} />}
                  </For>
                </div>
              </div>
            )}
          </For>
        </Show>
      </Show>

      <Show when={loading()}>
        <div class="text-poe-muted text-sm text-center py-4">Loading...</div>
      </Show>
    </div>
    </Show>
  );
};

const CategoryBrowser: Component<{
  classes: [string, number][];
  onSelect: (cls: string) => void;
}> = (props) => {
  const equipmentClasses = () =>
    props.classes.filter(([cls]) => EQUIPMENT_CLASSES.includes(cls));

  const grouped = () => {
    const eq = equipmentClasses();
    const armour = eq.filter(([c]) => ["Body Armour", "Helmet", "Gloves", "Boots", "Shield"].includes(c));
    const accessories = eq.filter(([c]) => ["Ring", "Amulet", "Belt", "Quiver"].includes(c));
    const weapons = eq.filter(([c]) =>
      ["Bow", "Claw", "Dagger", "Rune Dagger", "Wand", "Sceptre",
       "One Hand Sword", "Thrusting One Hand Sword", "One Hand Axe", "One Hand Mace",
       "Two Hand Sword", "Two Hand Axe", "Two Hand Mace", "Staff", "Warstaff"].includes(c)
    );
    const jewels = eq.filter(([c]) => ["Jewel", "AbyssJewel"].includes(c));
    const flasks = eq.filter(([c]) => c.includes("Flask"));
    return { armour, accessories, weapons, jewels, flasks };
  };

  return (
    <div class="flex flex-col gap-4">
      <CategoryGroup title="Armour" items={grouped().armour} onSelect={props.onSelect} />
      <CategoryGroup title="Weapons" items={grouped().weapons} onSelect={props.onSelect} />
      <CategoryGroup title="Accessories" items={grouped().accessories} onSelect={props.onSelect} />
      <CategoryGroup title="Jewels" items={grouped().jewels} onSelect={props.onSelect} />
      <CategoryGroup title="Flasks" items={grouped().flasks} onSelect={props.onSelect} />
    </div>
  );
};

const CategoryGroup: Component<{
  title: string;
  items: [string, number][];
  onSelect: (cls: string) => void;
}> = (props) => {
  return (
    <Show when={props.items.length > 0}>
      <div>
        <h3 class="text-poe-accent text-sm font-bold mb-2 uppercase tracking-wider">{props.title}</h3>
        <div class="flex flex-wrap gap-2">
          <For each={props.items}>
            {([cls, count]) => (
              <button
                class="px-3 py-2 bg-poe-surface border border-poe-border rounded hover:border-poe-accent transition-colors text-sm"
                onClick={() => props.onSelect(cls)}
              >
                <span class="text-poe-text">{cls}</span>
                <span class="text-poe-muted ml-1">({count})</span>
              </button>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
};

const BaseCard: Component<{ item: BaseItem; onClick?: (item: BaseItem) => void }> = (props) => {
  const [imgError, setImgError] = createSignal(false);

  return (
    <div
      class="bg-poe-surface border border-poe-border rounded p-3 flex flex-col items-center gap-2 hover:border-poe-accent transition-colors cursor-pointer"
      onClick={() => props.onClick?.(props.item)}
    >
      {/* Item image */}
      <div class="w-full h-24 flex items-center justify-center">
        <Show
          when={props.item.image_url && !imgError()}
          fallback={
            <div class="w-12 h-12 bg-poe-border rounded flex items-center justify-center text-poe-muted text-xs">
              ?
            </div>
          }
        >
          <img
            src={props.item.image_url!}
            alt={props.item.name}
            class="max-h-24 max-w-full object-contain"
            style={{ "image-rendering": "pixelated" }}
            onError={() => setImgError(true)}
          />
        </Show>
      </div>

      {/* Item name */}
      <div class="text-poe-normal text-xs text-center font-bold leading-tight">
        {props.item.name}
      </div>

      {/* Base stats */}
      <div class="text-[10px] text-center space-y-0.5 w-full">
        <Show when={props.item.properties.armour_max}>
          <div class="text-poe-text">
            Armour: <span class="text-poe-muted">{props.item.properties.armour_min}–{props.item.properties.armour_max}</span>
          </div>
        </Show>
        <Show when={props.item.properties.evasion_max}>
          <div class="text-poe-text">
            Evasion: <span class="text-poe-muted">{props.item.properties.evasion_min}–{props.item.properties.evasion_max}</span>
          </div>
        </Show>
        <Show when={props.item.properties.energy_shield_max}>
          <div class="text-poe-text">
            ES: <span class="text-poe-muted">{props.item.properties.energy_shield_min}–{props.item.properties.energy_shield_max}</span>
          </div>
        </Show>
      </div>

      {/* Implicits */}
      <Show when={props.item.implicit_stats.length > 0}>
        <div class="text-[10px] text-center w-full">
          <For each={props.item.implicit_stats}>
            {(s) => (
              <div class="text-blue-300 italic">{formatImplicit(s)}</div>
            )}
          </For>
        </div>
      </Show>

      {/* Requirements */}
      <div class="flex flex-wrap gap-1.5 justify-center text-[10px]">
        <span class="text-poe-muted">Lvl {props.item.drop_level}</span>
        <Show when={props.item.requirements.strength}>
          <span class="text-red-400">{props.item.requirements.strength} Str</span>
        </Show>
        <Show when={props.item.requirements.dexterity}>
          <span class="text-green-400">{props.item.requirements.dexterity} Dex</span>
        </Show>
        <Show when={props.item.requirements.intelligence}>
          <span class="text-blue-400">{props.item.requirements.intelligence} Int</span>
        </Show>
      </div>
    </div>
  );
};

export default BaseSearch;
