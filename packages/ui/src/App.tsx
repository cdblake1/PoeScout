import { Component, createSignal, onMount, onCleanup, Show } from "solid-js";
import ModSearch from "./components/lookup/ModSearch";
import BaseSearch from "./components/lookup/BaseSearch";
import PobPanel from "./components/pob/PobPanel";
import KeybindsPanel from "./components/KeybindsPanel";
import { initOverlayShortcut, cleanupOverlayShortcut, toggleOverlay } from "./lib/overlay";
import { activeTab, setActiveTab } from "./lib/navigation";
import { initCaptureShortcut, cleanupCaptureShortcut } from "./lib/capture";

const App: Component = () => {
  const [showKeybinds, setShowKeybinds] = createSignal(false);

  onMount(async () => {
    await initOverlayShortcut();
    await initCaptureShortcut();
  });

  onCleanup(() => {
    cleanupOverlayShortcut();
    cleanupCaptureShortcut();
  });

  return (
    <div class="min-h-screen bg-poe-bg text-poe-text font-mono">
      <header
        class="flex items-center gap-4 px-4 py-3 bg-poe-surface border-b border-poe-border"
        data-tauri-drag-region
      >
        <h1 class="text-poe-accent font-bold text-lg tracking-wide" data-tauri-drag-region>
          PoeScout
        </h1>
        <nav class="flex gap-1 ml-4">
          <button
            class={`px-3 py-1 text-sm rounded ${activeTab() === "mods" ? "bg-poe-accent text-poe-bg" : "text-poe-muted hover:text-poe-text"}`}
            onClick={() => setActiveTab("mods")}
          >
            Affixes
          </button>
          <button
            class={`px-3 py-1 text-sm rounded ${activeTab() === "bases" ? "bg-poe-accent text-poe-bg" : "text-poe-muted hover:text-poe-text"}`}
            onClick={() => setActiveTab("bases")}
          >
            Bases
          </button>
          <button
            class={`px-3 py-1 text-sm rounded ${activeTab() === "pob" ? "bg-poe-accent text-poe-bg" : "text-poe-muted hover:text-poe-text"}`}
            onClick={() => setActiveTab("pob")}
          >
            PoB
          </button>
        </nav>
        <div class="ml-auto flex gap-2">
          <button
            class="px-3 py-1 text-sm rounded text-poe-muted hover:text-poe-text border border-poe-border hover:border-poe-accent"
            onClick={() => setShowKeybinds(!showKeybinds())}
          >
            Keybinds
          </button>
          <button
            class="px-3 py-1 text-sm rounded text-poe-muted hover:text-poe-text border border-poe-border hover:border-poe-accent"
            onClick={() => toggleOverlay()}
            title="Toggle overlay mode (F2)"
          >
            Overlay
          </button>
        </div>
      </header>

      <main class="p-4">
        {activeTab() === "mods" ? <ModSearch /> : activeTab() === "bases" ? <BaseSearch /> : <PobPanel />}
      </main>

      <Show when={showKeybinds()}>
        <KeybindsPanel onClose={() => setShowKeybinds(false)} />
      </Show>
    </div>
  );
};

export default App;
