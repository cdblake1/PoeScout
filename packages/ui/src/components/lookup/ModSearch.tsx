import { Component, createSignal, For, Show } from "solid-js";
import {
  searchMods,
  type Mod,
  type SearchQuery,
  type SearchResult,
} from "../../lib/tauri";
import { formatMs, formatStatRange, generationLabel, generationColor } from "../../lib/format";

const ModSearch: Component = () => {
  const [query, setQuery] = createSignal("");
  const [domain, setDomain] = createSignal("");
  const [genType, setGenType] = createSignal("");
  const [results, setResults] = createSignal<SearchResult | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");

  let debounceTimer: number | undefined;

  const doSearch = async () => {
    const text = query().trim();
    if (!text) {
      setResults(null);
      return;
    }

    setLoading(true);
    setError("");

    try {
      const q: SearchQuery = {
        text,
        domain: domain() || undefined,
        generation_type: genType() || undefined,
        limit: 100,
      };
      const res = await searchMods(q);
      setResults(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const onInput = (val: string) => {
    setQuery(val);
    clearTimeout(debounceTimer);
    debounceTimer = window.setTimeout(doSearch, 150);
  };

  return (
    <div class="flex flex-col gap-3">
      <div class="flex gap-2 items-center">
        <input
          type="text"
          placeholder="Search affixes... (e.g. fire damage, life)"
          class="flex-1 px-3 py-2 bg-poe-surface border border-poe-border rounded text-poe-text placeholder-poe-muted focus:border-poe-accent focus:outline-none"
          value={query()}
          onInput={(e) => onInput(e.currentTarget.value)}
        />
        <select
          class="px-2 py-2 bg-poe-surface border border-poe-border rounded text-poe-text text-sm"
          value={genType()}
          onChange={(e) => {
            setGenType(e.currentTarget.value);
            doSearch();
          }}
        >
          <option value="">All Types</option>
          <option value="prefix">Prefix</option>
          <option value="suffix">Suffix</option>
        </select>
        <select
          class="px-2 py-2 bg-poe-surface border border-poe-border rounded text-poe-text text-sm"
          value={domain()}
          onChange={(e) => {
            setDomain(e.currentTarget.value);
            doSearch();
          }}
        >
          <option value="">All Domains</option>
          <option value="item">Item</option>
          <option value="flask">Flask</option>
          <option value="jewel">Jewel</option>
          <option value="abyss_jewel">Abyss Jewel</option>
        </select>
      </div>

      <Show when={error()}>
        <div class="text-red-500 text-sm">{error()}</div>
      </Show>

      <Show when={results()}>
        {(res) => (
          <>
            <div class="text-poe-muted text-xs">
              {res().total} results in {formatMs(res().query_ms)}
            </div>

            <div class="overflow-auto">
              <table class="w-full text-sm border-collapse">
                <thead>
                  <tr class="text-poe-muted text-left border-b border-poe-border">
                    <th class="px-2 py-1">Type</th>
                    <th class="px-2 py-1">Name</th>
                    <th class="px-2 py-1">Stats</th>
                    <th class="px-2 py-1">iLvl</th>
                    <th class="px-2 py-1">Weights</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={res().mods}>
                    {(mod) => <ModRow mod={mod} />}
                  </For>
                </tbody>
              </table>
            </div>
          </>
        )}
      </Show>

      <Show when={!results() && !loading()}>
        <div class="text-poe-muted text-sm text-center py-8">
          Type to search affixes by name, stat, or tag
        </div>
      </Show>

      <Show when={loading()}>
        <div class="text-poe-muted text-sm text-center py-4">Searching...</div>
      </Show>
    </div>
  );
};

const ModRow: Component<{ mod: Mod }> = (props) => {
  const topWeights = () =>
    props.mod.spawn_weights
      .filter((sw) => sw.weight > 0)
      .sort((a, b) => b.weight - a.weight)
      .slice(0, 3);

  return (
    <tr class="border-b border-poe-border/50 hover:bg-poe-surface/50">
      <td class={`px-2 py-1 ${generationColor(props.mod.generation_type)}`}>
        {generationLabel(props.mod.generation_type)}
      </td>
      <td class="px-2 py-1 text-poe-text">{props.mod.name}</td>
      <td class="px-2 py-1">
        <For each={props.mod.stats}>
          {(stat) => (
            <div class="text-poe-accent text-xs">
              {stat.id}: {formatStatRange(stat.min, stat.max)}
            </div>
          )}
        </For>
      </td>
      <td class="px-2 py-1 text-poe-muted">{props.mod.required_level}</td>
      <td class="px-2 py-1">
        <For each={topWeights()}>
          {(sw) => (
            <span class="inline-block mr-1 text-xs">
              <span class="text-poe-muted">{sw.tag}:</span>
              <span class="text-poe-currency">{sw.weight}</span>
            </span>
          )}
        </For>
      </td>
    </tr>
  );
};

export default ModSearch;
