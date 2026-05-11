import { Component, createSignal } from "solid-js";
import ModSearch from "./components/lookup/ModSearch";
import BaseSearch from "./components/lookup/BaseSearch";
import PobPanel from "./components/pob/PobPanel";

const App: Component = () => {
  const [tab, setTab] = createSignal<"mods" | "bases" | "pob">("mods");

  return (
    <div class="min-h-screen bg-poe-bg text-poe-text font-mono">
      <header class="flex items-center gap-4 px-4 py-3 bg-poe-surface border-b border-poe-border">
        <h1 class="text-poe-accent font-bold text-lg tracking-wide">PoeScout</h1>
        <nav class="flex gap-1 ml-4">
          <button
            class={`px-3 py-1 text-sm rounded ${tab() === "mods" ? "bg-poe-accent text-poe-bg" : "text-poe-muted hover:text-poe-text"}`}
            onClick={() => setTab("mods")}
          >
            Affixes
          </button>
          <button
            class={`px-3 py-1 text-sm rounded ${tab() === "bases" ? "bg-poe-accent text-poe-bg" : "text-poe-muted hover:text-poe-text"}`}
            onClick={() => setTab("bases")}
          >
            Bases
          </button>
          <button
            class={`px-3 py-1 text-sm rounded ${tab() === "pob" ? "bg-poe-accent text-poe-bg" : "text-poe-muted hover:text-poe-text"}`}
            onClick={() => setTab("pob")}
          >
            PoB
          </button>
        </nav>
      </header>

      <main class="p-4">
        {tab() === "mods" ? <ModSearch /> : tab() === "bases" ? <BaseSearch /> : <PobPanel />}
      </main>
    </div>
  );
};

export default App;
