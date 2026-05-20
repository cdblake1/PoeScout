import { invoke } from "@tauri-apps/api/core";

export interface ModStat {
  id: string;
  min: number;
  max: number;
}

export interface SpawnWeight {
  tag: string;
  weight: number;
}

export interface Mod {
  id: string;
  name: string;
  domain: string;
  generation_type: string;
  group: string;
  required_level: number;
  text: string;
  stats: ModStat[];
  spawn_weights: SpawnWeight[];
  tags: string[];
  is_essence_only: boolean;
  mod_type: string;
}

export interface SearchQuery {
  text: string;
  domain?: string;
  generation_type?: string;
  tags?: string[];
  min_level?: number;
  max_level?: number;
  min_weight?: number;
  limit?: number;
}

export interface SearchResult {
  mods: Mod[];
  total: number;
  query_ms: number;
}

export interface BaseItemProperties {
  armour_min: number | null;
  armour_max: number | null;
  evasion_min: number | null;
  evasion_max: number | null;
  energy_shield_min: number | null;
  energy_shield_max: number | null;
  movement_speed: number | null;
  block: number | null;
}

export interface BaseItemRequirements {
  level: number | null;
  strength: number | null;
  dexterity: number | null;
  intelligence: number | null;
}

export interface ImplicitStat {
  stat_id: string;
  min: number;
  max: number;
}

export interface BaseItem {
  id: string;
  name: string;
  item_class: string;
  drop_level: number;
  tags: string[];
  implicits: string[];
  implicit_stats: ImplicitStat[];
  implicit_text: string[];
  properties: BaseItemProperties;
  requirements: BaseItemRequirements;
  image_url: string | null;
  inventory_width: number | null;
  inventory_height: number | null;
}

export interface BaseSearchQuery {
  text: string;
  item_class?: string;
  min_level?: number;
  max_level?: number;
  limit?: number;
}

export interface BaseSearchResult {
  items: BaseItem[];
  total: number;
  query_ms: number;
}

export interface AffixEntry {
  mod_data: Mod;
  effective_weight: number;
}

export interface AffixesForBaseResult {
  affixes: AffixEntry[];
  query_ms: number;
}

export async function getAffixesForBase(
  baseTags: string[]
): Promise<AffixesForBaseResult> {
  return invoke("get_affixes_for_base", { baseTags });
}

export async function searchMods(query: SearchQuery): Promise<SearchResult> {
  return invoke("search_mods", { query });
}

export async function searchBases(
  query: BaseSearchQuery
): Promise<BaseSearchResult> {
  return invoke("search_bases", { query });
}

export async function getModById(id: string): Promise<Mod | null> {
  return invoke("get_mod_by_id", { id });
}

export async function listItemClasses(): Promise<[string, number][]> {
  return invoke("list_item_classes");
}

export async function listBasesByClass(
  itemClass: string
): Promise<BaseItem[]> {
  return invoke("list_bases_by_class", { itemClass });
}

// PoB types + commands

export interface BuildStats {
  life: string | null;
  energy_shield: string | null;
  mana: string | null;
  str_val: string | null;
  dex_val: string | null;
  int_val: string | null;
}

export interface BuildSummary {
  class_name: string;
  ascendancy: string;
  level: number;
  main_skill: string | null;
  total_stats: BuildStats;
  xml_raw: string;
}

export async function decodePobCode(input: string): Promise<BuildSummary> {
  return invoke("decode_pob_code", { input });
}

export async function detectPob(): Promise<string | null> {
  return invoke("detect_pob");
}

export async function launchPobApp(
  pobPath: string,
  buildCode?: string
): Promise<void> {
  return invoke("launch_pob_app", {
    pobPath,
    buildCode: buildCode ?? null,
  });
}
