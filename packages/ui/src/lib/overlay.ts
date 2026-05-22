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

interface SavedLayout {
  x: number;
  y: number;
  width: number;
  height: number;
}

function getOverlayWindow(): Window {
  // @ts-expect-error `skip` is internal API — prevents creating a new window
  return new Window("overlay", { skip: true });
}

function getSavedLayout(): SavedLayout | null {
  try {
    const raw = localStorage.getItem("overlay-layout");
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (parsed.width > 0 && parsed.height > 0) return parsed;
  } catch {}
  return null;
}

function saveLayout(layout: SavedLayout) {
  localStorage.setItem("overlay-layout", JSON.stringify(layout));
}

export async function enterOverlay() {
  const win = getOverlayWindow();

  const isVisible = await win.isVisible();
  if (isVisible) return;

  // PoE must be running
  let poeRect: PoeRect;
  try {
    poeRect = await invoke("get_poe_window_rect");
  } catch {
    return;
  }

  const scaleFactor = await win.scaleFactor();

  // Use saved position/size if available
  const saved = getSavedLayout();
  if (saved) {
    await win.setSize(new LogicalSize(saved.width, saved.height));
    await win.setPosition(new LogicalPosition(saved.x, saved.y));
    await win.show();
    await win.setFocus();
    return;
  }

  // First time — center on PoE window
  const overlayWidth = Math.max(480, Math.round((poeRect.width / scaleFactor) * 0.4));
  const overlayHeight = Math.max(600, Math.round((poeRect.height / scaleFactor) * 0.6));
  const overlayX = Math.round(poeRect.x / scaleFactor) + Math.round(((poeRect.width / scaleFactor) - overlayWidth) / 2);
  const overlayY = Math.round(poeRect.y / scaleFactor) + Math.round(((poeRect.height / scaleFactor) - overlayHeight) / 2);

  await win.setSize(new LogicalSize(overlayWidth, overlayHeight));
  await win.setPosition(new LogicalPosition(overlayX, overlayY));
  await win.show();
  await win.setFocus();
}

export async function exitOverlay() {
  const win = getOverlayWindow();

  const isVisible = await win.isVisible();
  if (!isVisible) return;

  // Save current position and size before hiding
  try {
    const pos = await win.outerPosition();
    const size = await win.outerSize();
    const scaleFactor = await win.scaleFactor();
    saveLayout({
      x: Math.round(pos.x / scaleFactor),
      y: Math.round(pos.y / scaleFactor),
      width: Math.round(size.width / scaleFactor),
      height: Math.round(size.height / scaleFactor),
    });
  } catch {}

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

export async function resetOverlayLayout() {
  const win = getOverlayWindow();
  const isVisible = await win.isVisible();
  if (isVisible) {
    await exitOverlay();
    localStorage.removeItem("overlay-layout");
    await enterOverlay();
  } else {
    localStorage.removeItem("overlay-layout");
  }
}

export async function cleanupOverlayShortcut() {
  await unregister("F2");
}
