import { createSignal } from "solid-js";
import type { BaseItem } from "./tauri";

export const [activeTab, setActiveTab] = createSignal<"mods" | "bases" | "pob" | "maps" | "stash" | "settings">("mods");
export const [navigateToBase, setNavigateToBase] = createSignal<BaseItem | null>(null);
export const [capturedItemLevel, setCapturedItemLevel] = createSignal<number | null>(null);
