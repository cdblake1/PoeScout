import { Component, createSignal, For, onMount, Show } from "solid-js";
import {
  getAllLeagues,
  getCurrentLeague,
  loadSettings,
  saveSettings,
  setTrackedCharacter,
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
      </Show>
    </div>
  );
};

export default SettingsPanel;
