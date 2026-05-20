import { Window } from "@tauri-apps/api/window";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import { register, unregister } from "@tauri-apps/plugin-global-shortcut";
import { invoke } from "@tauri-apps/api/core";

interface PoeRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

function getOverlayWindow(): Window {
  // @ts-expect-error `skip` is internal API — prevents creating a new window
  return new Window("overlay", { skip: true });
}

export async function enterOverlay() {
  const win = getOverlayWindow();

  const isVisible = await win.isVisible();
  if (isVisible) return;

  // Get scale factor for coordinate conversion
  const scaleFactor = await win.scaleFactor();

  let overlayWidth = 560;
  let overlayHeight = 720;
  let overlayX: number | null = null;
  let overlayY: number | null = null;

  try {
    const poeRect: PoeRect = await invoke("get_poe_window_rect");
    overlayWidth = Math.max(480, Math.round((poeRect.width / scaleFactor) * 0.4));
    overlayHeight = Math.max(600, Math.round((poeRect.height / scaleFactor) * 0.6));
    overlayX = Math.round(poeRect.x / scaleFactor) + Math.round(((poeRect.width / scaleFactor) - overlayWidth) / 2);
    overlayY = Math.round(poeRect.y / scaleFactor) + Math.round(((poeRect.height / scaleFactor) - overlayHeight) / 2);
  } catch {
    // PoE not found — use defaults, center on screen
  }

  await win.setSize(new LogicalSize(overlayWidth, overlayHeight));

  if (overlayX !== null && overlayY !== null) {
    await win.setPosition(new LogicalPosition(overlayX, overlayY));
  } else {
    await win.center();
  }

  await win.show();
  await win.setFocus();
}

export async function exitOverlay() {
  const win = getOverlayWindow();

  const isVisible = await win.isVisible();
  if (!isVisible) return;

  await win.hide();

  try {
    await invoke<string>("focus_poe_window");
  } catch (err) {
    console.warn("[overlay] focus_poe_window failed:", err);
  }
}

export async function toggleOverlay() {
  const win = getOverlayWindow();

  const isVisible = await win.isVisible();
  if (isVisible) {
    await exitOverlay();
  } else {
    await enterOverlay();
  }
}

export async function initOverlayShortcut() {
  try {
    await register("F2", (e) => {
      if (e.state === "Pressed") {
        toggleOverlay();
      }
    });
    console.log("[overlay] F2 shortcut registered");
  } catch (err) {
    console.error("[overlay] Failed to register F2 shortcut:", err);
  }
}

export async function cleanupOverlayShortcut() {
  await unregister("F2");
}
