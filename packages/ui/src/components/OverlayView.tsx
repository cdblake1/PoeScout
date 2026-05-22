import { Component, createSignal, onCleanup, onMount, Show } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import { exitOverlay, resetOverlayLayout } from "../lib/overlay";
import { setNavigateToBase, setCapturedItemLevel } from "../lib/navigation";
import BaseSearch from "./lookup/BaseSearch";
import MapTimer from "./maps/MapTimer";

type OverlayTab = "bases" | "maps";

const OverlayView: Component = () => {
  const [tab, setTab] = createSignal<OverlayTab>("bases");

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      exitOverlay();
    }
  };
  document.addEventListener("keydown", onKeyDown);
  onCleanup(() => {
    document.removeEventListener("keydown", onKeyDown);
  });

  let unlistenOverlay: (() => void) | undefined;
  onMount(async () => {
    unlistenOverlay = await listen<{ item: any; itemLevel: number | null }>("overlay-show-base", (event) => {
      setCapturedItemLevel(event.payload.itemLevel ?? null);
      setNavigateToBase(event.payload.item);
      setTab("bases");
    });
  });
  onCleanup(() => {
    unlistenOverlay?.();
  });

  return (
    <div
      class="h-screen flex flex-col font-mono text-sm text-white"
      style={{ background: "rgba(12, 12, 14, 0.85)" }}
    >
      {/* Drag bar */}
      <div
        class="flex items-center justify-between px-3 py-1.5 shrink-0 cursor-move"
        data-tauri-drag-region
      >
        <div class="flex items-center gap-3" data-tauri-drag-region>
          <span class="text-poe-accent font-bold text-xs tracking-wide" data-tauri-drag-region>
            PoeScout
          </span>
          <div class="flex gap-1">
            <button
              class={`text-xs px-2 py-0.5 rounded ${
                tab() === "bases"
                  ? "bg-poe-accent/20 text-poe-accent"
                  : "text-poe-muted hover:text-poe-text"
              }`}
              onClick={() => setTab("bases")}
            >
              Bases
            </button>
            <button
              class={`text-xs px-2 py-0.5 rounded ${
                tab() === "maps"
                  ? "bg-poe-accent/20 text-poe-accent"
                  : "text-poe-muted hover:text-poe-text"
              }`}
              onClick={() => setTab("maps")}
            >
              Maps
            </button>
          </div>
        </div>
        <div class="flex items-center gap-1">
          <button
            class="text-poe-muted hover:text-poe-text text-xs px-1"
            onClick={() => resetOverlayLayout()}
            title="Reset position and size"
          >
            ↺
          </button>
          <button
            class="text-poe-muted hover:text-poe-text text-xs px-1"
            onClick={() => exitOverlay()}
            title="Exit overlay (F2)"
          >
            ✕
          </button>
        </div>
      </div>

      <div class="flex-1 overflow-y-auto px-3 pb-2">
        <Show when={tab() === "bases"}>
          <BaseSearch />
        </Show>
        <Show when={tab() === "maps"}>
          <MapTimer />
        </Show>
      </div>
    </div>
  );
};

export default OverlayView;
