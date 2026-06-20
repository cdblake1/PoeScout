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

// Map timer types + commands

export interface TrackerState {
  kind: "Stopped" | "Idle" | "InMap";
  since?: string;
  zone_name?: string;
  map_name?: string;
  area_level?: number | null;
  map_tier?: number | null;
  started_at?: string;
  deaths?: number;
}

export interface MapEncounter {
  category: string;
  detail: string | null;
  timestamp: string;
}

export interface MapRun {
  id: number | null;
  map_name: string;
  area_id: string | null;
  area_level: number | null;
  area_type: string | null;
  map_tier: number | null;
  instance_id: string | null;
  league: string | null;
  session_id: number | null;
  started_at: string;
  ended_at: string;
  duration_secs: number;
  hideout_secs: number;
  deaths: number;
  level_ups: number[];
  encounters: MapEncounter[];
  loot_chaos: number | null;
}

export interface LootItem {
  name: string;
  type_line: string;
  stack_size: number;
  unit_chaos: number | null;
  total_chaos: number | null;
  frame_type: number | null;
}

export interface MapStats {
  total_runs: number;
  avg_duration_secs: number;
  maps_per_hour: number;
  total_deaths: number;
}

export interface MapTypeStat {
  map_name: string;
  area_id: string | null;
  run_count: number;
  avg_duration_secs: number;
  avg_loot_chaos: number | null;
  total_deaths: number;
}

export interface MechanicStat {
  category: string;
  encounter_count: number;
  maps_with: number;
  pct_of_maps: number;
  avg_duration_secs: number;
  avg_loot_chaos: number | null;
  total_deaths: number;
}

export interface PortfolioSnapshot {
  id: number | null;
  timestamp: string;
  total_chaos: number;
  total_divine: number;
}

export interface MapSession {
  id: number | null;
  label: string | null;
  league: string | null;
  started_at: string;
  ended_at: string | null;
  start_chaos: number | null;
  end_chaos: number | null;
  profit_chaos: number | null;
  active_secs: number;
  notes: string | null;
  run_count: number;
  chaos_per_hour: number | null;
}

export interface SessionDetail {
  session: MapSession;
  runs: MapRun[];
}

export async function getTrackerState(): Promise<TrackerState> {
  return invoke("get_tracker_state");
}

export async function getMapHistory(
  limit: number,
  offset: number
): Promise<MapRun[]> {
  return invoke("get_map_history", { limit, offset });
}

export async function getMapStats(): Promise<MapStats> {
  return invoke("get_map_stats");
}

export async function getMapTypeStats(): Promise<MapTypeStat[]> {
  return invoke("get_map_type_stats");
}

// Per-mechanic stats (6.8)

export async function getMechanicStats(): Promise<MechanicStat[]> {
  return invoke("get_mechanic_stats");
}

export async function getMapHistoryByMechanic(
  category: string,
  limit: number,
  offset: number
): Promise<MapRun[]> {
  return invoke("get_map_history_by_mechanic", { category, limit, offset });
}

// Items per hour (6.7a)

/** Discriminated union mirroring `ItemRateScope` in Rust (`#[serde(tag="kind")]`). */
export type ItemRateScope =
  | { kind: "current_session" }
  | { kind: "session"; id: number }
  | { kind: "last_sessions"; n: number }
  | { kind: "all_time" }
  | { kind: "date_range"; start: string; end: string };

export interface ItemRate {
  name: string;
  /** "inventory" for 6.7a; later: "stash:bestiary", "ocr:<key>". */
  source: string;
  stacks: number;
  drops: number;
  total_chaos: number;
  active_secs: number;
  items_per_hour: number;
  chaos_per_hour: number;
}

export async function getItemsPerHour(scope: ItemRateScope): Promise<ItemRate[]> {
  return invoke("get_items_per_hour", { scope });
}

export async function getNetWorthHistory(limit: number): Promise<PortfolioSnapshot[]> {
  return invoke("get_net_worth_history", { limit });
}

export async function getMapSessions(
  limit: number,
  offset: number
): Promise<MapSession[]> {
  return invoke("get_map_sessions", { limit, offset });
}

export async function getSessionDetail(sessionId: number): Promise<SessionDetail> {
  return invoke("get_session_detail", { sessionId });
}

export async function setTrackedCharacter(character: string | null): Promise<void> {
  return invoke("set_tracked_character", { character });
}

export async function clearMapHistory(): Promise<void> {
  return invoke("clear_map_history");
}

export async function isPoeForegound(): Promise<boolean> {
  return invoke("is_poe_foreground");
}

export interface CaptureTestResult {
  width: number;
  height: number;
  /** 0.0 = all-black frame (DX-refused capture); 1.0 = fully painted. */
  non_black_fraction: number;
}

/** Phase 6.6 spike: tries `PrintWindow` w/ `PW_RENDERFULLCONTENT` against PoE. */
export async function capturePoeTest(): Promise<CaptureTestResult> {
  return invoke("capture_poe_test");
}

// Stash & pricing types + commands

export interface PriceRecord {
  name: string;
  category: string;
  chaos_value: number;
  divine_value: number | null;
  icon: string | null;
  count: number | null;
}

export interface StashTab {
  id: string;
  index: number;
  tab_type: string;
  color: { r: number; g: number; b: number } | null;
}

export interface StashItem {
  name: string;
  type_line: string;
  base_type: string | null;
  stack_size: number | null;
  max_stack_size: number | null;
  icon: string;
  ilvl: number | null;
  identified: boolean | null;
  frame_type: number | null;
}

export interface PricedItem {
  item: StashItem;
  unit_price: number | null;
  total_price: number | null;
  price_source: string | null;
  listing_count: number | null;
}

export interface TabSummary {
  tab_name: string;
  tab_index: number;
  chaos_value: number;
  item_count: number;
}

export interface PortfolioSummary {
  total_chaos: number;
  total_divine: number;
  tab_summaries: TabSummary[];
  items: PricedItem[];
  chaos_per_hour: number | null;
  snapshot_count: number;
  rate_limited: boolean;
}

export interface Credentials {
  poesessid: string;
  account_name: string;
}

export async function setSessionId(poesessid: string, accountName: string): Promise<void> {
  return invoke("set_session_id", { poesessid, accountName });
}

export async function getStashTabs(league: string): Promise<StashTab[]> {
  return invoke("get_stash_tabs", { league });
}

export async function takeStashSnapshot(league: string): Promise<PortfolioSummary> {
  return invoke("take_stash_snapshot", { league });
}

export async function refreshPrices(league: string): Promise<void> {
  return invoke("refresh_prices", { league });
}

export async function getPrice(itemName: string, league: string): Promise<PriceRecord | null> {
  return invoke("get_price", { itemName, league });
}

export async function saveCredentials(poesessid: string, accountName: string): Promise<void> {
  return invoke("save_credentials", { poesessid, accountName });
}

export async function loadCredentials(): Promise<Credentials | null> {
  return invoke("load_credentials");
}

export async function getCurrentLeague(): Promise<string> {
  return invoke("get_current_league");
}

export async function getAllLeagues(): Promise<string[]> {
  return invoke("get_all_leagues");
}

export interface AppSettings {
  league?: string;
  selected_tabs?: number[];
  min_chaos?: number;
  character?: string;
  session_idle_timeout_secs?: number;
  /** Per-stack chaos threshold for stash snapshot totals (6.5b noise filter). */
  min_stack_chaos?: number;
  /** poe.ninja listing-count threshold for snapshot totals (6.5c noise filter). */
  min_listing_count?: number;
  /** Optional override for the league prices are fetched from (6.5c). */
  price_league?: string;
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function loadSettings(): Promise<AppSettings | null> {
  return invoke("load_settings");
}

export async function takeSelectiveSnapshot(
  league: string,
  tabIndices: number[],
): Promise<PortfolioSummary> {
  return invoke("take_selective_snapshot", { league, tabIndices });
}

export async function deleteCredentials(): Promise<void> {
  return invoke("delete_credentials");
}

export async function validateCredentials(): Promise<boolean> {
  return invoke("validate_credentials");
}

export interface SavedPortfolio {
  portfolio: PortfolioSummary;
  last_updated: string;
}

export async function savePortfolio(portfolio: PortfolioSummary): Promise<void> {
  return invoke("save_portfolio", { portfolio });
}

export async function loadPortfolio(): Promise<SavedPortfolio | null> {
  return invoke("load_portfolio");
}
