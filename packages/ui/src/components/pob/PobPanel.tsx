import { Component, createSignal, onMount, Show } from "solid-js";
import {
  decodePobCode,
  detectPob,
  launchPobApp,
  type BuildSummary,
} from "../../lib/tauri";

const PobPanel: Component = () => {
  const [input, setInput] = createSignal("");
  const [summary, setSummary] = createSignal<BuildSummary | null>(null);
  const [pobPath, setPobPath] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");

  onMount(async () => {
    try {
      const path = await detectPob();
      setPobPath(path);
    } catch {
      // PoB not found — that's fine
    }
  });

  const decode = async () => {
    const code = input().trim();
    if (!code) return;

    setLoading(true);
    setError("");
    setSummary(null);

    try {
      const result = await decodePobCode(code);
      setSummary(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const openInPob = async () => {
    const path = pobPath();
    if (!path) return;

    try {
      await launchPobApp(path, input().trim() || undefined);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div class="flex flex-col gap-4">
      <div class="flex flex-col gap-2">
        <label class="text-poe-muted text-sm">
          Paste a build code, pobb.in URL, or pastebin URL
        </label>
        <div class="flex gap-2">
          <input
            type="text"
            placeholder="eNrtPQl0..."
            class="flex-1 px-3 py-2 bg-poe-surface border border-poe-border rounded text-poe-text placeholder-poe-muted focus:border-poe-accent focus:outline-none font-mono text-sm"
            value={input()}
            onInput={(e) => setInput(e.currentTarget.value)}
            onKeyDown={(e) => e.key === "Enter" && decode()}
          />
          <button
            class="px-4 py-2 bg-poe-accent text-poe-bg rounded font-bold text-sm hover:opacity-90 disabled:opacity-50"
            onClick={decode}
            disabled={loading() || !input().trim()}
          >
            {loading() ? "Decoding..." : "Decode"}
          </button>
        </div>
      </div>

      <Show when={error()}>
        <div class="text-red-500 text-sm">{error()}</div>
      </Show>

      <Show when={summary()}>
        {(s) => (
          <div class="bg-poe-surface border border-poe-border rounded p-4 flex flex-col gap-3">
            <div class="flex items-center justify-between">
              <div>
                <h2 class="text-poe-unique text-lg font-bold">
                  {s().ascendancy || s().class_name}
                </h2>
                <div class="text-poe-muted text-sm">
                  {s().class_name} — Level {s().level}
                </div>
              </div>
              <Show when={pobPath()}>
                <button
                  class="px-4 py-2 bg-poe-accent text-poe-bg rounded font-bold text-sm hover:opacity-90"
                  onClick={openInPob}
                >
                  Open in PoB
                </button>
              </Show>
            </div>

            <Show when={s().main_skill}>
              <div class="text-sm">
                <span class="text-poe-muted">Main Skill: </span>
                <span class="text-poe-gem">{s().main_skill}</span>
              </div>
            </Show>

            <div class="grid grid-cols-3 gap-2 text-sm">
              <StatBox label="Life" value={s().total_stats.life} color="text-red-400" />
              <StatBox label="ES" value={s().total_stats.energy_shield} color="text-blue-400" />
              <StatBox label="Mana" value={s().total_stats.mana} color="text-blue-300" />
              <StatBox label="STR" value={s().total_stats.str_val} color="text-red-300" />
              <StatBox label="DEX" value={s().total_stats.dex_val} color="text-green-300" />
              <StatBox label="INT" value={s().total_stats.int_val} color="text-blue-300" />
            </div>

            <Show when={!pobPath()}>
              <div class="text-poe-muted text-xs">
                Path of Building not detected. Install it to enable one-click import.
              </div>
            </Show>
          </div>
        )}
      </Show>

      <Show when={!summary() && !loading() && !error()}>
        <div class="text-poe-muted text-sm text-center py-8">
          Paste a PoB build code to see a summary
        </div>
      </Show>
    </div>
  );
};

const StatBox: Component<{
  label: string;
  value: string | null;
  color: string;
}> = (props) => {
  return (
    <Show when={props.value}>
      <div class="bg-poe-bg rounded px-3 py-2 text-center">
        <div class="text-poe-muted text-xs">{props.label}</div>
        <div class={`font-bold ${props.color}`}>
          {Math.round(Number(props.value)).toLocaleString()}
        </div>
      </div>
    </Show>
  );
};

export default PobPanel;
