import { Component, createSignal, For, onMount, Show } from "solid-js";
import {
  getAllLeagues,
  getCurrentLeague,
  loadSettings,
  saveSettings,
  setTrackedCharacter,
  capturePoeTest,
  ocrRegion,
  recordResourceOcr,
  type CaptureTestResult,
} from "../../lib/tauri";

const SettingsPanel: Component = () => {
  const [leagues, setLeagues] = createSignal<string[]>([]);
  const [selectedLeague, setSelectedLeague] = createSignal("");
  const [savedLeague, setSavedLeague] = createSignal("");
  const [character, setCharacter] = createSignal("");
  const [savedCharacter, setSavedCharacter] = createSignal("");
  const [minStackChaos, setMinStackChaos] = createSignal("0");
  const [savedMinStackChaos, setSavedMinStackChaos] = createSignal(0);
  const [minListingCount, setMinListingCount] = createSignal("0");
  const [savedMinListingCount, setSavedMinListingCount] = createSignal(0);
  const [priceLeague, setPriceLeague] = createSignal("");
  const [savedPriceLeague, setSavedPriceLeague] = createSignal("");
  const [captureRunning, setCaptureRunning] = createSignal(false);
  const [captureResult, setCaptureResult] = createSignal<CaptureTestResult | null>(null);
  const [captureError, setCaptureError] = createSignal("");

  // OCR resource calibration (6.6b)
  const [ocrSource, setOcrSource] = createSignal("kingsmarch_gold");
  const [ocrX, setOcrX] = createSignal(0);
  const [ocrY, setOcrY] = createSignal(0);
  const [ocrW, setOcrW] = createSignal(160);
  const [ocrH, setOcrH] = createSignal(40);
  const [ocrBusy, setOcrBusy] = createSignal(false);
  const [ocrText, setOcrText] = createSignal<string | null>(null);
  const [ocrValue, setOcrValue] = createSignal<number | null>(null);
  const [ocrError, setOcrError] = createSignal("");

  const rect = () => [ocrX(), ocrY(), ocrW(), ocrH()] as const;

  const testOcr = async () => {
    setOcrBusy(true);
    setOcrError("");
    setOcrText(null);
    try {
      setOcrText(await ocrRegion(...rect()));
    } catch (e) {
      setOcrError(String(e));
    } finally {
      setOcrBusy(false);
    }
  };

  const readOcr = async () => {
    setOcrBusy(true);
    setOcrError("");
    setOcrValue(null);
    try {
      setOcrValue(await recordResourceOcr(ocrSource().trim() || "resource", ...rect()));
    } catch (e) {
      setOcrError(String(e));
    } finally {
      setOcrBusy(false);
    }
  };

  const runCaptureTest = async () => {
    setCaptureRunning(true);
    setCaptureError("");
    setCaptureResult(null);
    try {
      setCaptureResult(await capturePoeTest());
    } catch (e) {
      setCaptureError(String(e));
    } finally {
      setCaptureRunning(false);
    }
  };
  const [loading, setLoading] = createSignal(true);
  const [saving, setSaving] = createSignal(false);
  const [status, setStatus] = createSignal("");

  onMount(async () => {
    try {
      const [settings, allLeagues, defaultLeague] = await Promise.all([
        loadSettings(),
        getAllLeagues(),
        getCurrentLeague(),
      ]);

      setLeagues(allLeagues);

      const league = settings?.league || defaultLeague;
      setSelectedLeague(league);
      setSavedLeague(league);

      const char = settings?.character || "";
      setCharacter(char);
      setSavedCharacter(char);

      const msc = settings?.min_stack_chaos ?? 0;
      setMinStackChaos(String(msc));
      setSavedMinStackChaos(msc);

      const mlc = settings?.min_listing_count ?? 0;
      setMinListingCount(String(mlc));
      setSavedMinListingCount(mlc);

      const pl = settings?.price_league ?? "";
      setPriceLeague(pl);
      setSavedPriceLeague(pl);
    } catch (e) {
      setStatus(`Failed to load settings: ${e}`);
    } finally {
      setLoading(false);
    }
  });

  const save = async () => {
    setSaving(true);
    setStatus("");
    try {
      const char = character().trim();
      const msc = Math.max(0, parseFloat(minStackChaos()) || 0);
      const mlc = Math.max(0, parseInt(minListingCount(), 10) || 0);
      const pl = priceLeague().trim();
      await saveSettings({
        league: selectedLeague(),
        character: char,
        min_stack_chaos: msc,
        min_listing_count: mlc,
        price_league: pl,
      });
      await setTrackedCharacter(char || null);
      setSavedLeague(selectedLeague());
      setSavedCharacter(char);
      setSavedMinStackChaos(msc);
      setSavedMinListingCount(mlc);
      setSavedPriceLeague(pl);
      setStatus("Settings saved");
    } catch (e) {
      setStatus(`Failed to save: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const hasChanges = () =>
    selectedLeague() !== savedLeague() ||
    character().trim() !== savedCharacter() ||
    Math.max(0, parseFloat(minStackChaos()) || 0) !== savedMinStackChaos() ||
    Math.max(0, parseInt(minListingCount(), 10) || 0) !== savedMinListingCount() ||
    priceLeague().trim() !== savedPriceLeague();

  return (
    <div class="flex flex-col gap-6 max-w-lg">
      <h2 class="text-poe-accent font-bold text-lg">Settings</h2>

      <Show when={!loading()} fallback={<span class="text-poe-muted text-sm">Loading...</span>}>
        <div class="bg-poe-surface border border-poe-border rounded p-4 flex flex-col gap-4">
          <div class="flex flex-col gap-2">
            <label class="text-poe-muted text-sm font-bold">League</label>
            <select
              class="px-3 py-2 bg-poe-bg border border-poe-border rounded text-poe-text text-sm font-mono focus:border-poe-accent focus:outline-none"
              value={selectedLeague()}
              onChange={(e) => setSelectedLeague(e.currentTarget.value)}
            >
              <For each={leagues()}>
                {(league) => <option value={league}>{league}</option>}
              </For>
            </select>
            <p class="text-poe-muted text-xs">
              Used for price lookups and stash tracking. Defaults to the current softcore challenge league.
            </p>
          </div>

          <div class="flex flex-col gap-2">
            <label class="text-poe-muted text-sm font-bold">Character</label>
            <input
              type="text"
              class="px-3 py-2 bg-poe-bg border border-poe-border rounded text-poe-text text-sm font-mono focus:border-poe-accent focus:outline-none"
              placeholder="Your character name"
              value={character()}
              onInput={(e) => setCharacter(e.currentTarget.value)}
            />
            <p class="text-poe-muted text-xs">
              Attributes deaths and level-ups to you (so party members don't count). Leave blank
              to count everything (solo-accurate). Also used for future "open my character in PoB".
            </p>
          </div>

          <div class="flex flex-col gap-2">
            <label class="text-poe-muted text-sm font-bold">Snapshot noise filter (chaos)</label>
            <input
              type="number"
              min="0"
              step="0.1"
              class="px-3 py-2 bg-poe-bg border border-poe-border rounded text-poe-text text-sm font-mono focus:border-poe-accent focus:outline-none"
              value={minStackChaos()}
              onInput={(e) => setMinStackChaos(e.currentTarget.value)}
            />
            <p class="text-poe-muted text-xs">
              Stacks worth less than this (in chaos) are excluded from the snapshot total
              that drives the net-worth chart. Items table still shows everything. 0 = no filter.
            </p>
          </div>

          <div class="flex flex-col gap-2">
            <label class="text-poe-muted text-sm font-bold">poe.ninja listing-count threshold</label>
            <input
              type="number"
              min="0"
              step="1"
              class="px-3 py-2 bg-poe-bg border border-poe-border rounded text-poe-text text-sm font-mono focus:border-poe-accent focus:outline-none"
              value={minListingCount()}
              onInput={(e) => setMinListingCount(e.currentTarget.value)}
            />
            <p class="text-poe-muted text-xs">
              Stacks priced from poe.ninja entries with fewer than this many listings are
              excluded from the snapshot total (low listing counts = low confidence).
              Try <code>10</code>. 0 = no filter. Items without a count (uncommon) are not filtered.
            </p>
          </div>

          <div class="flex flex-col gap-2">
            <label class="text-poe-muted text-sm font-bold">Price-league override</label>
            <select
              class="px-3 py-2 bg-poe-bg border border-poe-border rounded text-poe-text text-sm font-mono focus:border-poe-accent focus:outline-none"
              value={priceLeague()}
              onChange={(e) => setPriceLeague(e.currentTarget.value)}
            >
              <option value="">Same as game league</option>
              <For each={leagues()}>
                {(league) => <option value={league}>{league}</option>}
              </For>
            </select>
            <p class="text-poe-muted text-xs">
              Fetch prices from a different league than the one your stash is in
              (e.g. price a dead/private league against <code>Standard</code>). Leave on
              <em> Same as game league</em> for normal use.
            </p>
          </div>

          <div class="flex items-center gap-3">
            <button
              class="px-4 py-2 bg-poe-accent text-poe-bg rounded font-bold text-sm hover:opacity-90 disabled:opacity-50"
              onClick={save}
              disabled={saving() || !hasChanges()}
            >
              {saving() ? "Saving..." : "Save"}
            </button>
            <Show when={status()}>
              <span class={status().startsWith("Failed") ? "text-red-400 text-sm" : "text-green-400 text-sm"}>
                {status()}
              </span>
            </Show>
          </div>
        </div>

        <div class="bg-poe-surface border border-poe-border rounded p-4 flex flex-col gap-2">
          <label class="text-poe-muted text-sm font-bold">Debug — OCR capture spike (6.6)</label>
          <p class="text-poe-muted text-xs">
            Tries to grab the PoE window with <code>PrintWindow + PW_RENDERFULLCONTENT</code> and
            reports the fraction of non-black pixels. Open PoE on screen first.
            Near <strong>1</strong> = capture works (we can build OCR on top).
            Near <strong>0</strong> = black frame; we'd need <code>Windows.Graphics.Capture</code>.
          </p>
          <div class="flex items-center gap-3 flex-wrap">
            <button
              class="px-4 py-2 bg-poe-bg border border-poe-border rounded text-poe-text text-sm hover:border-poe-accent disabled:opacity-50"
              onClick={runCaptureTest}
              disabled={captureRunning()}
            >
              {captureRunning() ? "Capturing..." : "Test PoE capture"}
            </button>
            <Show when={captureResult()}>
              <span
                class={
                  captureResult()!.non_black_fraction >= 0.5
                    ? "text-green-400 text-sm"
                    : "text-red-400 text-sm"
                }
              >
                {captureResult()!.width}×{captureResult()!.height}
                {" — "}
                {(captureResult()!.non_black_fraction * 100).toFixed(1)}% non-black
              </span>
            </Show>
            <Show when={captureError()}>
              <span class="text-red-400 text-sm">{captureError()}</span>
            </Show>
          </div>
        </div>

        <div class="bg-poe-surface border border-poe-border rounded p-4 flex flex-col gap-2">
          <label class="text-poe-muted text-sm font-bold">
            Experimental — OCR resource reader (6.6b)
          </label>
          <p class="text-poe-muted text-xs">
            Reads a number off the screen (e.g. Kingsmarch gold, Sulphite, Hiveblood) from a
            calibrated rectangle of the PoE client area, in pixels from its top-left. Open PoE,
            enter X/Y/Width/Height over the number, and <strong>Test</strong> until the text reads
            cleanly; then <strong>Read &amp; store</strong> saves it as an <code>ocr:&lt;key&gt;</code>
            time-series. Best-effort and resolution-specific — verify the value.
          </p>
          <div class="flex items-center gap-2 flex-wrap text-sm">
            <input
              class="px-2 py-1 bg-poe-bg border border-poe-border rounded text-poe-text w-40"
              placeholder="resource key"
              value={ocrSource()}
              onInput={(e) => setOcrSource(e.currentTarget.value)}
            />
            {(
              [
                ["X", ocrX, setOcrX],
                ["Y", ocrY, setOcrY],
                ["W", ocrW, setOcrW],
                ["H", ocrH, setOcrH],
              ] as const
            ).map(([label, get, set]) => (
              <label class="flex items-center gap-1 text-poe-muted">
                {label}
                <input
                  type="number"
                  class="px-2 py-1 bg-poe-bg border border-poe-border rounded text-poe-text w-20"
                  value={get()}
                  onInput={(e) => set(parseInt(e.currentTarget.value) || 0)}
                />
              </label>
            ))}
          </div>
          <div class="flex items-center gap-3 flex-wrap">
            <button
              class="px-4 py-2 bg-poe-bg border border-poe-border rounded text-poe-text text-sm hover:border-poe-accent disabled:opacity-50"
              onClick={testOcr}
              disabled={ocrBusy()}
            >
              {ocrBusy() ? "Working..." : "Test"}
            </button>
            <button
              class="px-4 py-2 bg-poe-bg border border-poe-border rounded text-poe-text text-sm hover:border-poe-accent disabled:opacity-50"
              onClick={readOcr}
              disabled={ocrBusy()}
            >
              Read &amp; store
            </button>
            <Show when={ocrText() !== null}>
              <span class="text-poe-muted text-sm">
                read: <span class="text-poe-text">"{ocrText()}"</span>
              </span>
            </Show>
            <Show when={ocrValue() !== null}>
              <span class="text-green-400 text-sm">stored {ocrSource()} = {ocrValue()}</span>
            </Show>
            <Show when={ocrError()}>
              <span class="text-red-400 text-sm">{ocrError()}</span>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default SettingsPanel;
