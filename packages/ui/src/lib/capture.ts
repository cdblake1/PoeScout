import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { register, unregister } from "@tauri-apps/plugin-global-shortcut";
import { parsePoEItem } from "./poe-item-parser";
import { searchBases, listBasesByClass, type BaseItem } from "./tauri";
import { setActiveTab, setNavigateToBase, setCapturedItemLevel } from "./navigation";
import { enterOverlay } from "./overlay";

async function handleCapture() {
  try {
    // Capture FIRST while PoE still has focus (Rust simulates Ctrl+C)
    const raw: string = await invoke("capture_item_text");

    if (!raw.includes("Rarity:")) return;

    const parsed = parsePoEItem(raw);
    console.log("[capture] raw clipboard text:", JSON.stringify(raw));
    console.log("[capture] parsed itemLevel:", parsed.itemLevel);
    if (!parsed.baseName) return;

    let matched: BaseItem | null = null;

    if (parsed.rarity !== "Magic") {
      const res = await searchBases({ text: parsed.baseName, limit: 5 });
      matched = res.items.find((item) => item.name === parsed.baseName) ?? null;
    }

    // Magic fallback or exact match failed: substring match against class bases
    if (!matched && parsed.itemClass) {
      const classItems = await listBasesByClass(parsed.itemClass);
      const sorted = [...classItems].sort(
        (a, b) => b.name.length - a.name.length
      );
      matched =
        sorted.find((item) => parsed.baseName!.includes(item.name)) ?? null;
    }

    if (matched) {
      setCapturedItemLevel(parsed.itemLevel);
      setActiveTab("bases");
      setNavigateToBase(matched);
      // Send to overlay window before showing it
      await emit("overlay-show-base", { item: matched, itemLevel: parsed.itemLevel });
      await enterOverlay();
    }
  } catch (err) {
    console.error("[capture] Failed to capture item:", err);
  }
}

export async function initCaptureShortcut() {
  try {
    await register("CommandOrControl+Q", (e) => {
      if (e.state === "Pressed") {
        handleCapture();
      }
    });
    console.log("[capture] Ctrl+Q shortcut registered");
  } catch (err) {
    console.error("[capture] Failed to register Ctrl+Q shortcut:", err);
  }
}

export async function cleanupCaptureShortcut() {
  try {
    await unregister("CommandOrControl+Q");
  } catch {
    // Ignore cleanup errors
  }
}
