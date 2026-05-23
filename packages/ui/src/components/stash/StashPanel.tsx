import { Component, createSignal, createMemo, For, onMount, Show } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import {
  loadCredentials,
  loadSettings,
  saveSettings,
  saveCredentials,
  deleteCredentials,
  getStashTabs,
  takeSelectiveSnapshot,
  refreshPrices,
  getPrice,
  getCurrentLeague,
  savePortfolio,
  loadPortfolio,
  type PortfolioSummary,
  type PriceRecord,
  type PricedItem,
  type StashTab,
} from "../../lib/tauri";
import { setActiveTab } from "../../lib/navigation";

type Phase = "idle" | "listing" | "selecting" | "scanning";

const SKIP_TAB_TYPES = new Set(["MapStash", "UniqueStash"]);
const PAGE_SIZE = 50;

const StashPanel: Component = () => {
  const [league, setLeague] = createSignal("");
  const [leagueLoading, setLeagueLoading] = createSignal(true);
  const [sessid, setSessid] = createSignal("");
  const [account, setAccount] = createSignal("");
  const [connected, setConnected] = createSignal(false);
  const [portfolio, setPortfolio] = createSignal<PortfolioSummary | null>(null);
  const [lastUpdated, setLastUpdated] = createSignal<string | null>(null);
  const [error, setError] = createSignal("");
  const [priceLookup, setPriceLookup] = createSignal("");
  const [lookupResult, setLookupResult] = createSignal<PriceRecord | null>(null);
  const [lookupError, setLookupError] = createSignal("");
  const [lookupLoading, setLookupLoading] = createSignal(false);
  const [priceStatus, setPriceStatus] = createSignal("");

  const [phase, setPhase] = createSignal<Phase>("idle");
  const [tabs, setTabs] = createSignal<StashTab[]>([]);
  const [selectedTabs, setSelectedTabs] = createSignal<Set<number>>(new Set());
  const [scanProgress, setScanProgress] = createSignal<{
    current: number;
    total: number;
    tab_name: string;
    tab_type: string;
  } | null>(null);

  const [itemSearch, setItemSearch] = createSignal("");
  const [minChaos, setMinChaos] = createSignal(0);
  const [tabSearch, setTabSearch] = createSignal("");
  const [rateLimitCooldown, setRateLimitCooldown] = createSignal(0);
  const [page, setPage] = createSignal(0);
  let cooldownInterval: ReturnType<typeof setInterval> | undefined;

  const filteredItems = createMemo(() => {
    const p = portfolio();
    if (!p) return [];
    const search = itemSearch().toLowerCase();
    const min = minChaos();
    return p.items.filter((pi) => {
      if (min > 0 && (pi.total_price ?? 0) < min) return false;
      if (search) {
        const name = (pi.item.name || pi.item.type_line).toLowerCase();
        if (!name.includes(search)) return false;
      }
      return true;
    });
  });

  const pagedItems = createMemo(() => {
    const start = page() * PAGE_SIZE;
    return filteredItems().slice(start, start + PAGE_SIZE);
  });

  const totalPages = createMemo(() =>
    Math.max(1, Math.ceil(filteredItems().length / PAGE_SIZE)),
  );

  const filteredTabs = createMemo(() => {
    const search = tabSearch().toLowerCase();
    if (!search) return tabs();
    return tabs().filter(
      (t) =>
        t.id.toLowerCase().includes(search) ||
        t.tab_type.toLowerCase().includes(search),
    );
  });

  const startCooldown = (seconds: number) => {
    setRateLimitCooldown(seconds);
    if (cooldownInterval) clearInterval(cooldownInterval);
    cooldownInterval = setInterval(() => {
      const remaining = rateLimitCooldown() - 1;
      if (remaining <= 0) {
        clearInterval(cooldownInterval);
        cooldownInterval = undefined;
        setRateLimitCooldown(0);
      } else {
        setRateLimitCooldown(remaining);
      }
    }, 1000);
  };

  const formatLastUpdated = () => {
    const ts = lastUpdated();
    if (!ts) return null;
    const ms = parseInt(ts, 10);
    if (isNaN(ms)) return null;
    const date = new Date(ms);
    return date.toLocaleString();
  };

  onMount(async () => {
    let savedTabSelection: number[] | undefined;
    let savedMinChaos: number | undefined;
    try {
      const [settings, detectedLeague] = await Promise.all([
        loadSettings().catch(() => null),
        getCurrentLeague().catch(() => "Standard"),
      ]);
      const leagueName = settings?.league || detectedLeague;
      setLeague(leagueName);
      savedTabSelection = settings?.selected_tabs;
      savedMinChaos = settings?.min_chaos;
      if (savedMinChaos != null && savedMinChaos > 0) setMinChaos(savedMinChaos);
      setPriceStatus("Loading prices...");
      refreshPrices(leagueName)
        .then(() => setPriceStatus("Prices loaded"))
        .catch((e) => setPriceStatus(`Price fetch failed: ${e}`));
    } catch (e) {
      setLeague("Standard");
      console.warn("Failed to detect league:", e);
    } finally {
      setLeagueLoading(false);
    }

    try {
      const saved = await loadPortfolio();
      if (saved && saved.portfolio) {
        setPortfolio(saved.portfolio);
        setLastUpdated(saved.last_updated);
      }
    } catch {}

    try {
      const creds = await loadCredentials();
      if (creds && creds.poesessid && creds.account_name) {
        setSessid(creds.poesessid);
        setAccount(creds.account_name);
        setConnected(true);
        if (savedTabSelection && savedTabSelection.length > 0) {
          setSelectedTabs(new Set(savedTabSelection));
        }
      }
    } catch {}
  });

  const connect = async () => {
    if (!sessid().trim() || !account().trim()) return;
    setError("");
    try {
      await saveCredentials(sessid().trim(), account().trim());
      setConnected(true);
    } catch (e) {
      setError(String(e));
    }
  };

  const disconnect = async () => {
    try {
      await deleteCredentials();
    } catch (e) {
      console.error("Failed to delete credentials:", e);
    }
    setSessid("");
    setAccount("");
    setConnected(false);
    setPortfolio(null);
    setLastUpdated(null);
    setTabs([]);
    setPhase("idle");
  };

  const persistSettings = () => {
    saveSettings({
      league: league(),
      selected_tabs: Array.from(selectedTabs()),
      min_chaos: minChaos() > 0 ? minChaos() : undefined,
    }).catch(() => {});
  };

  const listTabs = async () => {
    setPhase("listing");
    setError("");
    try {
      const fetchedTabs = await getStashTabs(league());
      setTabs(fetchedTabs);
      const saved = selectedTabs();
      if (saved.size > 0) {
        const validSaved = new Set(
          Array.from(saved).filter((idx) =>
            fetchedTabs.some((t) => t.index === idx),
          ),
        );
        if (validSaved.size > 0) {
          setSelectedTabs(validSaved);
          setPhase("selecting");
          return;
        }
      }
      const autoSelected = new Set(
        fetchedTabs
          .filter((t) => !SKIP_TAB_TYPES.has(t.tab_type))
          .map((t) => t.index),
      );
      setSelectedTabs(autoSelected);
      setPhase("selecting");
    } catch (e) {
      setError(String(e));
      setPhase("idle");
    }
  };

  const scanSelected = async () => {
    const indices = Array.from(selectedTabs());
    if (indices.length === 0) return;
    if (rateLimitCooldown() > 0) return;
    setPhase("scanning");
    setError("");
    setScanProgress(null);

    persistSettings();

    const unlisten = await listen<{
      current: number;
      total: number;
      tab_name: string;
      tab_type: string;
    }>("stash:scan-progress", (e) => setScanProgress(e.payload));

    try {
      const result = await takeSelectiveSnapshot(league(), indices);
      setPortfolio(result);
      const now = Date.now().toString();
      setLastUpdated(now);
      savePortfolio(result).catch(() => {});
      if (result.rate_limited) {
        setError(
          "Rate limited by GGG — partial results shown. Cooldown active.",
        );
        startCooldown(60);
      }
      setPhase("idle");
      setTabs([]);
    } catch (e) {
      const msg = String(e);
      if (msg.includes("Rate limited")) {
        setError("Rate limited by GGG — try again in a minute");
        startCooldown(60);
      } else {
        setError(msg);
      }
      setPhase("idle");
    } finally {
      unlisten();
      setScanProgress(null);
    }
  };

  const toggleTab = (index: number) => {
    const current = new Set(selectedTabs());
    if (current.has(index)) {
      current.delete(index);
    } else {
      current.add(index);
    }
    setSelectedTabs(current);
  };

  const selectAll = () => setSelectedTabs(new Set(tabs().map((t) => t.index)));
  const deselectAll = () => setSelectedTabs(new Set<number>());

  const doRefreshPrices = async () => {
    setError("");
    setPriceStatus("Refreshing prices...");
    try {
      await refreshPrices(league());
      setPriceStatus("Prices refreshed");
    } catch (e) {
      setPriceStatus(`Price fetch failed: ${e}`);
    }
  };

  const lookupPrice = async () => {
    const name = priceLookup().trim();
    if (!name || lookupLoading()) return;
    setLookupError("");
    setLookupResult(null);
    setLookupLoading(true);
    try {
      const result = await getPrice(name, league());
      if (result) {
        setLookupResult(result);
      } else {
        setLookupError("No price found");
      }
    } catch (e) {
      setLookupError(String(e));
    } finally {
      setLookupLoading(false);
    }
  };

  const stashButtonLabel = () => {
    switch (phase()) {
      case "listing":
        return "Loading tabs...";
      case "scanning":
        return "Scanning...";
      default:
        return rateLimitCooldown() > 0
          ? `Cooldown (${rateLimitCooldown()}s)`
          : "Refresh Stash";
    }
  };

  return (
    <div class="flex flex-col gap-4">
      {/* League indicator + price status */}
      <div class="flex items-center gap-4 text-sm">
        <div class="flex items-center gap-2">
          <span class="text-poe-muted">League:</span>
          <Show
            when={!leagueLoading()}
            fallback={<span class="text-poe-muted">detecting...</span>}
          >
            <span class="text-poe-accent font-bold">{league()}</span>
            <button
              class="text-poe-muted hover:text-poe-text text-xs underline"
              onClick={() => setActiveTab("settings")}
            >
              change
            </button>
          </Show>
        </div>
        <Show when={priceStatus()}>
          <span
            class={
              priceStatus().startsWith("Price fetch failed")
                ? "text-red-400"
                : "text-poe-muted"
            }
          >
            {priceStatus()}
          </span>
        </Show>
      </div>

      {/* Credentials */}
      <Show
        when={connected()}
        fallback={
          <div class="bg-poe-surface border border-poe-border rounded p-4 flex flex-col gap-3">
            <h2 class="text-poe-accent font-bold text-sm">Connect to PoE</h2>
            <p class="text-poe-muted text-xs">
              Enter your POESESSID cookie and account name. Your session ID is
              stored locally in the app data directory.
            </p>
            <div class="flex flex-col gap-2">
              <input
                type="password"
                placeholder="POESESSID"
                class="px-3 py-2 bg-poe-bg border border-poe-border rounded text-poe-text placeholder-poe-muted focus:border-poe-accent focus:outline-none text-sm font-mono"
                value={sessid()}
                onInput={(e) => setSessid(e.currentTarget.value)}
              />
              <input
                type="text"
                placeholder="Account name"
                class="px-3 py-2 bg-poe-bg border border-poe-border rounded text-poe-text placeholder-poe-muted focus:border-poe-accent focus:outline-none text-sm font-mono"
                value={account()}
                onInput={(e) => setAccount(e.currentTarget.value)}
                onKeyDown={(e) => e.key === "Enter" && connect()}
              />
            </div>
            <button
              class="px-4 py-2 bg-poe-accent text-poe-bg rounded font-bold text-sm hover:opacity-90 disabled:opacity-50 self-start"
              onClick={connect}
              disabled={!sessid().trim() || !account().trim()}
            >
              Connect
            </button>
          </div>
        }
      >
        <div class="flex items-center justify-between bg-poe-surface border border-poe-border rounded px-4 py-2">
          <div class="flex items-center gap-3">
            <span class="text-poe-muted text-sm">
              Connected as{" "}
              <span class="text-poe-text font-bold">{account()}</span>
            </span>
            <Show when={formatLastUpdated()}>
              <span class="text-poe-muted text-xs">
                Last updated: {formatLastUpdated()}
              </span>
            </Show>
          </div>
          <div class="flex gap-2">
            <Show when={phase() !== "selecting"}>
              <button
                class="px-3 py-1 text-sm rounded bg-poe-accent text-poe-bg font-bold hover:opacity-90 disabled:opacity-50"
                onClick={listTabs}
                disabled={
                  phase() === "listing" ||
                  phase() === "scanning" ||
                  rateLimitCooldown() > 0
                }
              >
                {stashButtonLabel()}
              </button>
            </Show>
            <button
              class="px-3 py-1 text-sm rounded border border-poe-border text-poe-muted hover:text-poe-text"
              onClick={doRefreshPrices}
            >
              Refresh Prices
            </button>
            <button
              class="px-3 py-1 text-sm rounded border border-poe-border text-poe-muted hover:text-red-400"
              onClick={disconnect}
            >
              Disconnect
            </button>
          </div>
        </div>
      </Show>

      <Show when={error()}>
        <div class="text-red-500 text-sm">{error()}</div>
      </Show>

      {/* Rate limit cooldown — compact inline */}
      <Show when={rateLimitCooldown() > 0}>
        <div class="text-red-400 text-xs">
          Rate limit cooldown: {rateLimitCooldown()}s remaining
        </div>
      </Show>

      {/* Tab Selection */}
      <Show when={phase() === "selecting"}>
        <div class="bg-poe-surface border border-poe-border rounded p-4 flex flex-col gap-3">
          <div class="flex items-center justify-between">
            <h3 class="text-poe-accent font-bold text-sm">
              Select Tabs to Scan
            </h3>
            <div class="flex gap-2 text-xs">
              <button
                onClick={selectAll}
                class="text-poe-muted hover:text-poe-text underline"
              >
                Select All
              </button>
              <button
                onClick={deselectAll}
                class="text-poe-muted hover:text-poe-text underline"
              >
                Deselect All
              </button>
            </div>
          </div>
          <input
            type="text"
            placeholder="Search tabs by name or type..."
            class="px-3 py-1.5 bg-poe-bg border border-poe-border rounded text-poe-text placeholder-poe-muted focus:border-poe-accent focus:outline-none text-sm"
            value={tabSearch()}
            onInput={(e) => setTabSearch(e.currentTarget.value)}
          />
          <div class="grid grid-cols-3 gap-1 max-h-64 overflow-y-auto">
            <For each={filteredTabs()}>
              {(tab) => {
                const unsupported = SKIP_TAB_TYPES.has(tab.tab_type);
                return (
                  <label
                    class={`flex items-center gap-2 px-2 py-1 rounded text-sm ${unsupported ? "opacity-40 cursor-not-allowed" : "hover:bg-poe-bg/50 cursor-pointer"}`}
                    title={unsupported ? "Not supported" : ""}
                  >
                    <input
                      type="checkbox"
                      checked={selectedTabs().has(tab.index)}
                      onChange={() => !unsupported && toggleTab(tab.index)}
                      disabled={unsupported}
                      class="accent-poe-accent"
                    />
                    <span
                      class="truncate"
                      style={
                        tab.color
                          ? {
                              color: `rgb(${tab.color.r},${tab.color.g},${tab.color.b})`,
                            }
                          : {}
                      }
                    >
                      {tab.id}
                    </span>
                    <span class="text-poe-muted text-xs ml-auto shrink-0">
                      {tab.tab_type.replace("Stash", "")}
                    </span>
                  </label>
                );
              }}
            </For>
          </div>
          <div class="flex items-center gap-2">
            <button
              class="px-4 py-2 bg-poe-accent text-poe-bg rounded font-bold text-sm hover:opacity-90 disabled:opacity-50"
              onClick={scanSelected}
              disabled={selectedTabs().size === 0 || rateLimitCooldown() > 0}
            >
              {rateLimitCooldown() > 0
                ? `Cooldown (${rateLimitCooldown()}s)`
                : `Scan Selected (${selectedTabs().size} tabs)`}
            </button>
            <button
              class="px-3 py-1 text-sm rounded border border-poe-border text-poe-muted hover:text-poe-text"
              onClick={() => {
                setPhase("idle");
                setTabs([]);
                setTabSearch("");
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      </Show>

      {/* Scan Progress */}
      <Show when={phase() === "scanning"}>
        <div class="bg-poe-surface border border-poe-border rounded px-4 py-3">
          <div class="text-sm text-poe-text">
            {scanProgress()
              ? `Scanning tab ${scanProgress()!.current}/${scanProgress()!.total}: ${scanProgress()!.tab_name} (${scanProgress()!.tab_type.replace("Stash", "")})`
              : "Preparing scan..."}
          </div>
          <Show when={scanProgress()}>
            <div class="w-full bg-poe-bg rounded-full h-1.5 mt-2">
              <div
                class="bg-poe-accent h-1.5 rounded-full transition-all"
                style={{
                  width: `${(scanProgress()!.current / scanProgress()!.total) * 100}%`,
                }}
              />
            </div>
          </Show>
        </div>
      </Show>

      {/* Portfolio Summary */}
      <Show when={portfolio()}>
        {(p) => (
          <>
            <div class="grid grid-cols-4 gap-2">
              <StatCard
                label="Total Value"
                value={formatChaos(p().total_chaos)}
                color="text-yellow-400"
              />
              <StatCard
                label="Divine Value"
                value={formatDivine(p().total_divine)}
                color="text-poe-accent"
              />
              <StatCard
                label="Chaos/hr"
                value={
                  p().chaos_per_hour != null
                    ? formatChaos(p().chaos_per_hour!)
                    : "—"
                }
                color="text-green-400"
              />
              <StatCard
                label="Snapshots"
                value={String(p().snapshot_count)}
                color="text-poe-muted"
              />
            </div>

            {/* Items */}
            <div class="bg-poe-surface border border-poe-border rounded">
              <div class="flex items-center gap-3 px-4 py-2 border-b border-poe-border">
                <h3 class="text-poe-accent font-bold text-sm shrink-0">
                  Items
                  <span class="text-poe-muted font-normal ml-2">
                    {filteredItems().length}
                    {filteredItems().length !== p().items.length &&
                      ` / ${p().items.length}`}
                  </span>
                </h3>
                <input
                  type="text"
                  placeholder="Search items..."
                  class="flex-1 px-2 py-1 bg-poe-bg border border-poe-border rounded text-poe-text placeholder-poe-muted focus:border-poe-accent focus:outline-none text-xs"
                  value={itemSearch()}
                  onInput={(e) => {
                    setItemSearch(e.currentTarget.value);
                    setPage(0);
                  }}
                />
                <div class="flex items-center gap-1 shrink-0">
                  <span class="text-poe-muted text-xs">Min:</span>
                  <input
                    type="number"
                    placeholder="0"
                    class="w-16 px-2 py-1 bg-poe-bg border border-poe-border rounded text-poe-text placeholder-poe-muted focus:border-poe-accent focus:outline-none text-xs"
                    value={minChaos() || ""}
                    onInput={(e) => {
                      const val = Number(e.currentTarget.value) || 0;
                      setMinChaos(val);
                      setPage(0);
                    }}
                    onBlur={persistSettings}
                  />
                  <span class="text-poe-muted text-xs">c</span>
                </div>
              </div>
              <div class="overflow-x-auto">
                <table class="w-full text-sm">
                  <thead class="bg-poe-surface">
                    <tr class="text-poe-muted text-left border-b border-poe-border">
                      <th class="px-4 py-2">Item</th>
                      <th class="px-4 py-2">Type</th>
                      <th class="px-4 py-2 text-right">Qty</th>
                      <th class="px-4 py-2 text-right">Unit Price</th>
                      <th class="px-4 py-2 text-right">Total</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={pagedItems()}>
                      {(pi: PricedItem) => (
                        <tr class="border-b border-poe-border/30 hover:bg-poe-bg/50">
                          <td class="px-4 py-1.5">
                            <div class="flex items-center gap-2">
                              <img
                                src={pi.item.icon}
                                alt=""
                                class="w-6 h-6 object-contain"
                                loading="lazy"
                              />
                              <span class={frameTypeColor(pi.item.frame_type)}>
                                {pi.item.name || pi.item.type_line}
                              </span>
                            </div>
                          </td>
                          <td class="px-4 py-1.5 text-poe-muted text-xs">
                            {formatCategory(pi.price_source)}
                          </td>
                          <td class="px-4 py-1.5 text-right text-poe-muted">
                            {pi.item.stack_size ?? 1}
                          </td>
                          <td class="px-4 py-1.5 text-right text-poe-muted">
                            {pi.unit_price != null
                              ? formatChaos(pi.unit_price)
                              : "—"}
                          </td>
                          <td class="px-4 py-1.5 text-right text-yellow-400">
                            {pi.total_price != null
                              ? formatChaos(pi.total_price)
                              : "—"}
                          </td>
                        </tr>
                      )}
                    </For>
                  </tbody>
                </table>
              </div>
              {/* Pagination */}
              <Show when={totalPages() > 1}>
                <div class="flex items-center justify-between px-4 py-2 border-t border-poe-border text-xs">
                  <span class="text-poe-muted">
                    Showing {page() * PAGE_SIZE + 1}–
                    {Math.min((page() + 1) * PAGE_SIZE, filteredItems().length)}{" "}
                    of {filteredItems().length}
                  </span>
                  <div class="flex gap-1">
                    <button
                      class="px-2 py-1 rounded border border-poe-border text-poe-muted hover:text-poe-text disabled:opacity-30"
                      disabled={page() === 0}
                      onClick={() => setPage(page() - 1)}
                    >
                      Prev
                    </button>
                    <For
                      each={Array.from({ length: Math.min(totalPages(), 7) }, (_, i) => {
                        if (totalPages() <= 7) return i;
                        if (page() < 4) return i;
                        if (page() > totalPages() - 5)
                          return totalPages() - 7 + i;
                        return page() - 3 + i;
                      })}
                    >
                      {(p) => (
                        <button
                          class={`px-2 py-1 rounded border text-xs ${
                            page() === p
                              ? "border-poe-accent text-poe-accent"
                              : "border-poe-border text-poe-muted hover:text-poe-text"
                          }`}
                          onClick={() => setPage(p)}
                        >
                          {p + 1}
                        </button>
                      )}
                    </For>
                    <button
                      class="px-2 py-1 rounded border border-poe-border text-poe-muted hover:text-poe-text disabled:opacity-30"
                      disabled={page() >= totalPages() - 1}
                      onClick={() => setPage(page() + 1)}
                    >
                      Next
                    </button>
                  </div>
                </div>
              </Show>
            </div>
          </>
        )}
      </Show>

      {/* Price Lookup */}
      <div class="bg-poe-surface border border-poe-border rounded p-4 flex flex-col gap-2">
        <h3 class="text-poe-accent font-bold text-sm">Price Lookup</h3>
        <div class="flex gap-2">
          <input
            type="text"
            placeholder="Type an item name..."
            class="flex-1 px-3 py-2 bg-poe-bg border border-poe-border rounded text-poe-text placeholder-poe-muted focus:border-poe-accent focus:outline-none text-sm font-mono"
            value={priceLookup()}
            onInput={(e) => setPriceLookup(e.currentTarget.value)}
            onKeyDown={(e) => e.key === "Enter" && lookupPrice()}
          />
          <button
            class="px-4 py-2 bg-poe-accent text-poe-bg rounded font-bold text-sm hover:opacity-90 disabled:opacity-50"
            onClick={lookupPrice}
            disabled={!priceLookup().trim() || lookupLoading()}
          >
            {lookupLoading() ? "Loading..." : "Lookup"}
          </button>
        </div>
        <Show when={lookupResult()}>
          {(r) => (
            <div class="flex items-center gap-4 text-sm">
              <span class="text-poe-text font-bold">{r().name}</span>
              <span class="text-yellow-400">{formatChaos(r().chaos_value)}</span>
              <Show when={r().divine_value != null}>
                <span class="text-poe-accent">
                  {formatDivine(r().divine_value!)}
                </span>
              </Show>
              <span class="text-poe-muted text-xs">({r().category})</span>
            </div>
          )}
        </Show>
        <Show when={lookupError()}>
          <div class="text-poe-muted text-sm">{lookupError()}</div>
        </Show>
      </div>

      {/* Empty state */}
      <Show when={!portfolio() && connected() && phase() === "idle"}>
        <div class="text-poe-muted text-sm text-center py-8">
          Click "Refresh Stash" to load your stash tabs
        </div>
      </Show>
    </div>
  );
};

const StatCard: Component<{ label: string; value: string; color: string }> = (
  props,
) => (
  <div class="bg-poe-surface border border-poe-border rounded px-3 py-2 text-center">
    <div class="text-poe-muted text-xs">{props.label}</div>
    <div class={`font-bold ${props.color}`}>{props.value}</div>
  </div>
);

function formatChaos(value: number): string {
  if (value >= 1000) return `${(value / 1000).toFixed(1)}k c`;
  return `${Math.round(value)} c`;
}

function formatDivine(value: number): string {
  return `${value.toFixed(1)} div`;
}

function formatCategory(source: string | null): string {
  if (!source) return "—";
  return source
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace("Unique", "Uniq.");
}

function frameTypeColor(frameType: number | null): string {
  switch (frameType) {
    case 3:
      return "text-poe-unique";
    case 5:
      return "text-yellow-400";
    case 6:
      return "text-cyan-400";
    case 4:
      return "text-blue-400";
    case 1:
      return "text-blue-300";
    case 2:
      return "text-yellow-300";
    default:
      return "text-poe-text";
  }
}

export default StashPanel;
