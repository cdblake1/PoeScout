import { Component, onCleanup, onMount } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import { exitOverlay } from "../lib/overlay";
import { setNavigateToBase, setCapturedItemLevel } from "../lib/navigation";
import BaseSearch from "./lookup/BaseSearch";

const OverlayView: Component = () => {
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
    });
  });
  onCleanup(() => {
    unlistenOverlay?.();
  });

  return (
    <div
      class="h-screen flex flex-col font-mono text-sm"
      style={{ background: "rgba(12, 12, 14, 0.85)" }}
    >
      {/* Drag bar */}
      <div
        class="flex items-center justify-between px-3 py-1.5 shrink-0"
        data-tauri-drag-region
      >
        <span class="text-poe-accent font-bold text-xs tracking-wide" data-tauri-drag-region>
          PoeScout
        </span>
        <button
          class="text-poe-muted hover:text-poe-text text-xs px-1"
          onClick={() => exitOverlay()}
          title="Exit overlay (Esc)"
        >
          ✕
        </button>
      </div>

      <div class="flex-1 overflow-y-auto px-3 pb-2">
        <BaseSearch />
      </div>
    </div>
  );
};

export default OverlayView;
