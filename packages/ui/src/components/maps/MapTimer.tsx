import { Component, createSignal, onMount, onCleanup, For, Show } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import {
  getTrackerState,
  getMapHistory,
  getMapStats,
  type TrackerState,
  type MapRun,
  type MapStats,
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

function elapsedSince(isoTimestamp: string): number {
  const start = new Date(isoTimestamp).getTime();
  return Math.max(0, (Date.now() - start) / 1000);
}

const MapTimer: Component = () => {
  const [state, setState] = createSignal<TrackerState>({ kind: "Idle" });
  const [history, setHistory] = createSignal<MapRun[]>([]);
  const [stats, setStats] = createSignal<MapStats>({
    total_runs: 0,
    avg_duration_secs: 0,
    maps_per_hour: 0,
    total_deaths: 0,
  });
  const [elapsed, setElapsed] = createSignal(0);

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
      const [h, st] = await Promise.all([getMapHistory(50, 0), getMapStats()]);
      setHistory(h);
      setStats(st);
    } catch {}
  };

  let unlistenState: (() => void) | undefined;
  let unlistenComplete: (() => void) | undefined;

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
  });

  onCleanup(() => {
    if (tickInterval !== undefined) clearInterval(tickInterval);
    unlistenState?.();
    unlistenComplete?.();
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
      </div>

      {/* Live timer */}
      <div class="bg-poe-surface border border-poe-border rounded p-4">
        <Show when={state().kind === "InMap"}>
          <div class="text-center">
            <div class="text-poe-accent text-xl font-bold">
              {state().map_name}
              <Show when={state().area_level}>
                <span class="text-poe-muted text-sm ml-2">
                  (T{Math.max(1, (state().area_level || 83) - 67)})
                </span>
              </Show>
            </div>
            <div class="text-4xl font-bold mt-2 tabular-nums">
              {formatDuration(elapsed())}
            </div>
            <Show when={(state().deaths || 0) > 0}>
              <div class="text-red-400 text-sm mt-1">
                Deaths: {state().deaths}
              </div>
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

      {/* Session stats */}
      <Show when={stats().total_runs > 0}>
        <div class="grid grid-cols-4 gap-3">
          <div class="bg-poe-surface border border-poe-border rounded p-3 text-center">
            <div class="text-poe-muted text-xs">Runs</div>
            <div class="text-lg font-bold">{stats().total_runs}</div>
          </div>
          <div class="bg-poe-surface border border-poe-border rounded p-3 text-center">
            <div class="text-poe-muted text-xs">Avg Time</div>
            <div class="text-lg font-bold">
              {formatDuration(stats().avg_duration_secs)}
            </div>
          </div>
          <div class="bg-poe-surface border border-poe-border rounded p-3 text-center">
            <div class="text-poe-muted text-xs">Maps/hr</div>
            <div class="text-lg font-bold">
              {stats().maps_per_hour.toFixed(1)}
            </div>
          </div>
          <div class="bg-poe-surface border border-poe-border rounded p-3 text-center">
            <div class="text-poe-muted text-xs">Deaths</div>
            <div class="text-lg font-bold text-red-400">
              {stats().total_deaths}
            </div>
          </div>
        </div>
      </Show>

      {/* History table */}
      <div class="bg-poe-surface border border-poe-border rounded">
        <div class="px-3 py-2 border-b border-poe-border text-poe-muted text-xs uppercase tracking-wide">
          Recent Runs
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
                </tr>
              </thead>
              <tbody>
                <For each={history()}>
                  {(run) => (
                    <tr class="border-b border-poe-border/50 hover:bg-poe-bg/50">
                      <td class="px-3 py-1.5 text-poe-accent">{run.map_name}</td>
                      <td class="px-3 py-1.5 text-right text-poe-muted">
                        {run.area_level
                          ? `T${Math.max(1, run.area_level - 67)}`
                          : "-"}
                      </td>
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
