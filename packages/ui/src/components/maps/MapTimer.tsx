import { Component, createSignal, onMount, onCleanup, For, Show } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import Sparkline from "./Sparkline";
import {
  getTrackerState,
  getMapHistory,
  getMapStats,
  getMapSessions,
  getMapTypeStats,
  getNetWorthHistory,
  getItemsPerHour,
  clearMapHistory,
  type TrackerState,
  type MapRun,
  type MapStats,
  type MapSession,
  type MapTypeStat,
  type MapEncounter,
  type PortfolioSnapshot,
  type ItemRate,
  type ItemRateScope,
} from "../../lib/tauri";

function formatDuration(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

function formatDurationLong(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  if (h > 0) return `${h}h ${m}m ${s}s`;
  return `${m}m ${s}s`;
}

function formatChaos(v: number | null): string {
  if (v == null) return "—";
  return `${Math.round(v).toLocaleString()}c`;
}

function lootPerHour(m: MapTypeStat): string {
  if (m.avg_loot_chaos == null || m.avg_duration_secs <= 0) return "—";
  return Math.round(m.avg_loot_chaos / (m.avg_duration_secs / 3600)).toLocaleString();
}

function tierOf(run: MapRun): string {
  if (run.map_tier != null) return `T${run.map_tier}`;
  if (run.area_level != null) return `T${Math.max(1, run.area_level - 67)}`;
  return "-";
}

function encounterCategories(encs: MapEncounter[]): string[] {
  return Array.from(new Set(encs.map((e) => e.category)));
}

function elapsedSince(isoTimestamp: string): number {
  const start = new Date(isoTimestamp).getTime();
  return Math.max(0, (Date.now() - start) / 1000);
}

const MapTimer: Component = () => {
  const [state, setState] = createSignal<TrackerState>({ kind: "Idle" });
  const [history, setHistory] = createSignal<MapRun[]>([]);
  const [sessions, setSessions] = createSignal<MapSession[]>([]);
  const [mapTypeStats, setMapTypeStats] = createSignal<MapTypeStat[]>([]);
  const [netWorth, setNetWorth] = createSignal<PortfolioSnapshot[]>([]);
  const [itemRates, setItemRates] = createSignal<ItemRate[]>([]);
  const [itemScope, setItemScope] = createSignal<ItemRateScope>({ kind: "current_session" });
  const [stats, setStats] = createSignal<MapStats>({
    total_runs: 0,
    avg_duration_secs: 0,
    maps_per_hour: 0,
    total_deaths: 0,
  });
  const [elapsed, setElapsed] = createSignal(0);

  const activeSession = () => sessions().find((s) => s.ended_at === null);

  let tickInterval: number | undefined;

  const startTick = () => {
    if (tickInterval !== undefined) return;
    tickInterval = window.setInterval(() => {
      const s = state();
      if (s.kind === "InMap" && s.started_at) {
        setElapsed(elapsedSince(s.started_at));
      } else if (s.kind === "Idle" && s.since) {
        setElapsed(elapsedSince(s.since));
      }
    }, 1000);
  };

  const refreshData = async () => {
    try {
      const [h, st, ses, mts, nw, ir] = await Promise.all([
        getMapHistory(50, 0),
        getMapStats(),
        getMapSessions(20, 0),
        getMapTypeStats(),
        getNetWorthHistory(50),
        getItemsPerHour(itemScope()),
      ]);
      setHistory(h);
      setStats(st);
      setSessions(ses);
      setMapTypeStats(mts);
      setNetWorth(nw);
      setItemRates(ir);
    } catch {}
  };

  const setScope = async (scope: ItemRateScope) => {
    setItemScope(scope);
    try {
      setItemRates(await getItemsPerHour(scope));
    } catch {}
  };

  const scopeLabel = () => {
    const s = itemScope();
    if (s.kind === "current_session") return "This session";
    if (s.kind === "last_sessions") return `Last ${s.n}`;
    if (s.kind === "all_time") return "All time";
    return `Session #${s.id}`;
  };

  const clearHistory = async () => {
    if (!window.confirm("Clear all recorded map runs? This cannot be undone.")) return;
    try {
      await clearMapHistory();
      await refreshData();
    } catch {}
  };

  let unlistenState: (() => void) | undefined;
  let unlistenComplete: (() => void) | undefined;
  let unlistenSessionStart: (() => void) | undefined;
  let unlistenSessionEnd: (() => void) | undefined;
  let unlistenLoot: (() => void) | undefined;

  onMount(async () => {
    const s = await getTrackerState();
    setState(s);
    startTick();
    await refreshData();

    unlistenState = await listen<TrackerState>("map-tracker:state-change", (e) => {
      setState(e.payload);
    });
    unlistenComplete = await listen<MapRun>("map-tracker:map-complete", async () => {
      await refreshData();
    });
    unlistenSessionStart = await listen("map-tracker:session-start", async () => {
      await refreshData();
    });
    unlistenSessionEnd = await listen("map-tracker:session-end", async () => {
      await refreshData();
    });
    unlistenLoot = await listen("map-tracker:loot", async () => {
      await refreshData();
    });
  });

  onCleanup(() => {
    if (tickInterval !== undefined) clearInterval(tickInterval);
    unlistenState?.();
    unlistenComplete?.();
    unlistenSessionStart?.();
    unlistenSessionEnd?.();
    unlistenLoot?.();
  });

  return (
    <div class="space-y-4">
      {/* Status bar */}
      <div class="flex items-center gap-3">
        <span
          class={`inline-block w-2 h-2 rounded-full ${
            state().kind === "InMap"
              ? "bg-green-400"
              : state().kind === "Idle"
              ? "bg-yellow-400"
              : "bg-gray-500"
          }`}
        />
        <span class="text-poe-muted text-sm">
          {state().kind === "InMap" ? "In Map" : state().kind === "Idle" ? "Idle" : "Waiting for zone..."}
        </span>
        <Show when={activeSession()}>
          <span class="text-green-400 text-xs ml-auto">
            ● Session: {activeSession()!.run_count} maps · {formatDurationLong(activeSession()!.active_secs)}
          </span>
        </Show>
      </div>

      {/* Live timer */}
      <div class="bg-poe-surface border border-poe-border rounded p-4">
        <Show when={state().kind === "InMap"}>
          <div class="text-center">
            <div class="text-poe-accent text-xl font-bold">
              {state().map_name}
              <Show when={state().area_level}>
                <span class="text-poe-muted text-sm ml-2">
                  (T{state().map_tier ?? Math.max(1, (state().area_level || 83) - 67)})
                </span>
              </Show>
            </div>
            <div class="text-4xl font-bold mt-2 tabular-nums">
              {formatDuration(elapsed())}
            </div>
            <Show when={(state().deaths || 0) > 0}>
              <div class="text-red-400 text-sm mt-1">Deaths: {state().deaths}</div>
            </Show>
          </div>
        </Show>
        <Show when={state().kind === "Idle"}>
          <div class="text-center">
            <div class="text-poe-muted text-lg">{state().zone_name || "Hideout"}</div>
            <div class="text-2xl font-bold mt-1 text-yellow-400 tabular-nums">
              {formatDuration(elapsed())}
            </div>
          </div>
        </Show>
      </div>

      {/* All-time stats */}
      <Show when={stats().total_runs > 0}>
        <div class="grid grid-cols-4 gap-3">
          <div class="bg-poe-surface border border-poe-border rounded p-3 text-center">
            <div class="text-poe-muted text-xs">Runs</div>
            <div class="text-lg font-bold">{stats().total_runs}</div>
          </div>
          <div class="bg-poe-surface border border-poe-border rounded p-3 text-center">
            <div class="text-poe-muted text-xs">Avg Time</div>
            <div class="text-lg font-bold">{formatDuration(stats().avg_duration_secs)}</div>
          </div>
          <div class="bg-poe-surface border border-poe-border rounded p-3 text-center">
            <div class="text-poe-muted text-xs">Maps/hr</div>
            <div class="text-lg font-bold">{stats().maps_per_hour.toFixed(1)}</div>
          </div>
          <div class="bg-poe-surface border border-poe-border rounded p-3 text-center">
            <div class="text-poe-muted text-xs">Deaths</div>
            <div class="text-lg font-bold text-red-400">{stats().total_deaths}</div>
          </div>
        </div>
      </Show>

      {/* Trends */}
      <Show
        when={
          history().length > 1 ||
          sessions().some((s) => s.chaos_per_hour != null) ||
          netWorth().length > 1
        }
      >
        <div class="bg-poe-surface border border-poe-border rounded p-3 grid grid-cols-3 gap-4">
          <Sparkline
            data={history().slice(0, 30).reverse().map((r) => r.duration_secs)}
            label={`Run duration — last ${Math.min(history().length, 30)} (oldest → newest)`}
            color="#5fb3ff"
          />
          <Sparkline
            data={sessions()
              .filter((s) => s.chaos_per_hour != null)
              .slice(0, 20)
              .reverse()
              .map((s) => s.chaos_per_hour!)}
            label="Currency/hour — by session (oldest → newest)"
            color="#7fd97f"
          />
          <Sparkline
            data={netWorth().slice().reverse().map((p) => p.total_chaos)}
            label={`Net worth (chaos) — last ${netWorth().length} snapshots`}
            color="#e8c170"
          />
        </div>
      </Show>

      {/* Sessions */}
      <Show when={sessions().length > 0}>
        <div class="bg-poe-surface border border-poe-border rounded">
          <div class="px-3 py-2 border-b border-poe-border text-poe-muted text-xs uppercase tracking-wide">
            Sessions
          </div>
          <div class="max-h-64 overflow-y-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="text-poe-muted text-xs border-b border-poe-border">
                  <th class="text-left px-3 py-1">Started</th>
                  <th class="text-right px-3 py-1">Maps</th>
                  <th class="text-right px-3 py-1">Active</th>
                  <th class="text-right px-3 py-1">Profit</th>
                  <th class="text-right px-3 py-1">c/hr</th>
                </tr>
              </thead>
              <tbody>
                <For each={sessions()}>
                  {(s) => (
                    <tr class="border-b border-poe-border/50 hover:bg-poe-bg/50">
                      <td class="px-3 py-1.5">
                        {new Date(s.started_at).toLocaleString()}
                        <Show when={s.ended_at === null}>
                          <span class="text-green-400 ml-1">●</span>
                        </Show>
                      </td>
                      <td class="px-3 py-1.5 text-right">{s.run_count}</td>
                      <td class="px-3 py-1.5 text-right tabular-nums">
                        {formatDurationLong(s.active_secs)}
                      </td>
                      <td
                        class={`px-3 py-1.5 text-right ${
                          (s.profit_chaos ?? 0) >= 0 ? "text-green-400" : "text-red-400"
                        }`}
                      >
                        {formatChaos(s.profit_chaos)}
                      </td>
                      <td class="px-3 py-1.5 text-right">
                        {s.chaos_per_hour != null
                          ? Math.round(s.chaos_per_hour).toLocaleString()
                          : "—"}
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </div>
      </Show>

      {/* Per-map stats */}
      <Show when={mapTypeStats().length > 0}>
        <div class="bg-poe-surface border border-poe-border rounded">
          <div class="px-3 py-2 border-b border-poe-border text-poe-muted text-xs uppercase tracking-wide">
            Per-Map Stats
          </div>
          <div class="max-h-64 overflow-y-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="text-poe-muted text-xs border-b border-poe-border">
                  <th class="text-left px-3 py-1">Map</th>
                  <th class="text-right px-3 py-1">Runs</th>
                  <th class="text-right px-3 py-1">Avg Time</th>
                  <th class="text-right px-3 py-1">Avg Loot</th>
                  <th class="text-right px-3 py-1">Loot/hr</th>
                  <th class="text-right px-3 py-1">Deaths</th>
                </tr>
              </thead>
              <tbody>
                <For each={mapTypeStats()}>
                  {(m) => (
                    <tr class="border-b border-poe-border/50 hover:bg-poe-bg/50">
                      <td class="px-3 py-1.5 text-poe-accent">{m.map_name}</td>
                      <td class="px-3 py-1.5 text-right">{m.run_count}</td>
                      <td class="px-3 py-1.5 text-right tabular-nums">
                        {formatDuration(m.avg_duration_secs)}
                      </td>
                      <td class="px-3 py-1.5 text-right text-green-400">
                        {m.avg_loot_chaos != null ? formatChaos(m.avg_loot_chaos) : "—"}
                      </td>
                      <td class="px-3 py-1.5 text-right">{lootPerHour(m)}</td>
                      <td class="px-3 py-1.5 text-right text-poe-muted">{m.total_deaths}</td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </div>
      </Show>

      {/* Items per hour (6.7a) */}
      <div class="bg-poe-surface border border-poe-border rounded">
        <div class="px-3 py-2 border-b border-poe-border flex items-center justify-between">
          <span class="text-poe-muted text-xs uppercase tracking-wide">Items / hr</span>
          <div class="flex gap-1">
            {(
              [
                ["This session", { kind: "current_session" }],
                ["Last 5", { kind: "last_sessions", n: 5 }],
                ["All time", { kind: "all_time" }],
              ] as [string, ItemRateScope][]
            ).map(([label, scope]) => (
              <button
                class={`text-xs px-2 py-0.5 rounded border ${
                  scopeLabel() === label
                    ? "border-poe-accent text-poe-accent"
                    : "border-poe-border text-poe-muted hover:text-poe-accent"
                }`}
                onClick={() => setScope(scope)}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
        <Show
          when={itemRates().length > 0}
          fallback={
            <div class="px-3 py-4 text-poe-muted text-sm text-center">
              No priced loot in this scope yet — needs a configured Character + stash credentials + at least one map completion.
            </div>
          }
        >
          <div class="px-3 py-1 text-poe-muted text-[11px]">
            Denominator: active map time ={" "}
            <span class="tabular-nums">
              {formatDurationLong(itemRates()[0]?.active_secs ?? 0)}
            </span>
          </div>
          <div class="max-h-72 overflow-y-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="text-poe-muted text-xs border-b border-poe-border">
                  <th class="text-left px-3 py-1">Item</th>
                  <th class="text-right px-3 py-1">Stacks</th>
                  <th class="text-right px-3 py-1">Items/hr</th>
                  <th class="text-right px-3 py-1">Chaos/hr</th>
                  <th class="text-right px-3 py-1">Drops</th>
                </tr>
              </thead>
              <tbody>
                <For each={itemRates()}>
                  {(ir) => (
                    <tr class="border-b border-poe-border/50 hover:bg-poe-bg/50">
                      <td class="px-3 py-1.5 text-poe-accent">{ir.name}</td>
                      <td class="px-3 py-1.5 text-right tabular-nums">{ir.stacks}</td>
                      <td class="px-3 py-1.5 text-right tabular-nums">
                        {ir.items_per_hour > 0 ? ir.items_per_hour.toFixed(2) : "—"}
                      </td>
                      <td class="px-3 py-1.5 text-right text-green-400 tabular-nums">
                        {ir.chaos_per_hour > 0
                          ? Math.round(ir.chaos_per_hour).toLocaleString()
                          : "—"}
                      </td>
                      <td class="px-3 py-1.5 text-right text-poe-muted tabular-nums">
                        {ir.drops}
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </div>

      {/* History table */}
      <div class="bg-poe-surface border border-poe-border rounded">
        <div class="px-3 py-2 border-b border-poe-border text-poe-muted text-xs uppercase tracking-wide flex items-center justify-between">
          <span>Recent Runs</span>
          <Show when={history().length > 0}>
            <button
              class="normal-case text-poe-muted hover:text-red-400 text-xs"
              onClick={clearHistory}
            >
              Clear
            </button>
          </Show>
        </div>
        <Show
          when={history().length > 0}
          fallback={
            <div class="px-3 py-4 text-poe-muted text-sm text-center">
              No map runs recorded yet
            </div>
          }
        >
          <div class="max-h-96 overflow-y-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="text-poe-muted text-xs border-b border-poe-border">
                  <th class="text-left px-3 py-1">Map</th>
                  <th class="text-right px-3 py-1">Tier</th>
                  <th class="text-right px-3 py-1">Time</th>
                  <th class="text-right px-3 py-1">Deaths</th>
                  <th class="text-right px-3 py-1">Loot</th>
                </tr>
              </thead>
              <tbody>
                <For each={history()}>
                  {(run) => (
                    <tr class="border-b border-poe-border/50 hover:bg-poe-bg/50">
                      <td class="px-3 py-1.5 text-poe-accent">
                        {run.map_name}
                        <Show when={run.encounters.length > 0}>
                          <div class="flex flex-wrap gap-1 mt-0.5">
                            <For each={encounterCategories(run.encounters)}>
                              {(c) => (
                                <span class="text-[10px] px-1 rounded bg-poe-bg text-poe-muted border border-poe-border">
                                  {c}
                                </span>
                              )}
                            </For>
                          </div>
                        </Show>
                      </td>
                      <td class="px-3 py-1.5 text-right text-poe-muted">{tierOf(run)}</td>
                      <td class="px-3 py-1.5 text-right tabular-nums">
                        {formatDuration(run.duration_secs)}
                      </td>
                      <td
                        class={`px-3 py-1.5 text-right ${
                          run.deaths > 0 ? "text-red-400" : "text-poe-muted"
                        }`}
                      >
                        {run.deaths}
                      </td>
                      <td class="px-3 py-1.5 text-right text-green-400">
                        {run.loot_chaos != null ? formatChaos(run.loot_chaos) : "—"}
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Show>
      </div>
    </div>
  );
};

export default MapTimer;
