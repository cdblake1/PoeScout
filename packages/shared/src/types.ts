// Mirrors Rust types from poe-core for frontend use
// These are re-exported from packages/ui/src/lib/tauri.ts

export interface Mod {
  id: string;
  name: string;
  domain: string;
  generation_type: string;
  group: string;
  required_level: number;
  stats: ModStat[];
  spawn_weights: SpawnWeight[];
  tags: string[];
  is_essence_only: boolean;
}

export interface ModStat {
  id: string;
  min: number;
  max: number;
}

export interface SpawnWeight {
  tag: string;
  weight: number;
}

export interface BaseItem {
  id: string;
  name: string;
  item_class: string;
  drop_level: number;
  tags: string[];
  implicits: string[];
}
