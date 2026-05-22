import { Component, createSignal, onMount, onCleanup, Show } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import { Window } from "@tauri-apps/api/window";
import { getTrackerState, isPoeForegound, type TrackerState } from "../../lib/tauri";

function formatTimer(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

function getTimerWindow(): Window {
  // @ts-expect-error `skip` is internal API
  return new Window("timer", { skip: true });
}

function getOverlayWindow(): Window {
  // @ts-expect-error `skip` is internal API
  return new Window("overlay", { skip: true });
}

const TimerOverlay: Component = () => {
  const [state, setState] = createSignal<TrackerState>({ kind: "Stopped" });
  const [elapsed, setElapsed] = createSignal(0);
  const [draggable, setDraggable] = createSignal(false);

  let tickInterval: number | undefined;
  let focusInterval: number | undefined;
  let unlistenState: (() => void) | undefined;
  let showDebounce: number | undefined;

  const win = getTimerWindow();
  const overlayWin = getOverlayWindow();

  onMount(async () => {
    await win.setIgnoreCursorEvents(true);

    try {
      const s = await getTrackerState();
      setState(s);
    } catch {}

    unlistenState = await listen<TrackerState>("map-tracker:state-change", (e) => {
      setState(e.payload);
    });

    tickInterval = window.setInterval(() => {
      const s = state();
      if (s.kind === "InMap" && s.started_at) {
        setElapsed(Math.max(0, (Date.now() - new Date(s.started_at).getTime()) / 1000));
      } else if (s.kind === "Idle" && s.since) {
        setElapsed(Math.max(0, (Date.now() - new Date(s.since).getTime()) / 1000));
      } else {
        setElapsed(0);
      }
    }, 1000);

    focusInterval = window.setInterval(async () => {
      try {
        const s = state();
        const inValidZone = s.kind === "InMap" || s.kind === "Idle";

        const overlayVisible = await overlayWin.isVisible();

        if (overlayVisible && inValidZone) {
          if (!draggable()) {
            setDraggable(true);
            await win.setIgnoreCursorEvents(false);
          }
          if (showDebounce !== undefined) {
            clearTimeout(showDebounce);
            showDebounce = undefined;
          }
          await win.show();
          return;
        }

        if (draggable()) {
          setDraggable(false);
          await win.setIgnoreCursorEvents(true);
        }

        const poeFocused = await isPoeForegound();
        const shouldShow = poeFocused && inValidZone;
        const timerVisible = await win.isVisible();

        if (shouldShow && !timerVisible) {
          if (showDebounce === undefined) {
            showDebounce = window.setTimeout(async () => {
              showDebounce = undefined;
              try { await win.show(); } catch {}
            }, 750);
          }
        } else if (!shouldShow) {
          if (showDebounce !== undefined) {
            clearTimeout(showDebounce);
            showDebounce = undefined;
          }
          if (timerVisible) {
            await win.hide();
          }
        }
      } catch {}
    }, 500);
  });

  onCleanup(() => {
    if (tickInterval !== undefined) clearInterval(tickInterval);
    if (focusInterval !== undefined) clearInterval(focusInterval);
    if (showDebounce !== undefined) clearTimeout(showDebounce);
    unlistenState?.();
  });

  const onMouseDown = (e: MouseEvent) => {
    if (draggable()) {
      e.preventDefault();
      win.startDragging();
    }
  };

  return (
    <div
      class="h-screen flex items-center px-3 gap-2 font-mono text-xs select-none"
      classList={{ "cursor-move": draggable() }}
      style={{ background: "rgba(12, 12, 14, 0.80)" }}
      onMouseDown={onMouseDown}
    >
      <span
        class={`w-2 h-2 rounded-full shrink-0 ${
          state().kind === "InMap" ? "bg-green-400" : "bg-yellow-400"
        }`}
      />
      <span class="text-poe-accent truncate max-w-[140px]">
        {state().kind === "InMap" ? state().map_name : (state().zone_name || "Hideout")}
      </span>
      <span class="tabular-nums font-bold text-white ml-auto">
        {formatTimer(elapsed())}
      </span>
      <Show when={state().kind === "InMap" && (state().deaths || 0) > 0}>
        <span class="text-red-400">
          {state().deaths}d
        </span>
      </Show>
    </div>
  );
};

export default TimerOverlay;
