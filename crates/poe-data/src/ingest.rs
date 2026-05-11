use crate::db::Database;
use anyhow::Result;
use poe_core::types::{BaseItem, BaseItemProperties, BaseItemRequirements, Mod, ModStat, SpawnWeight};
use std::collections::HashMap;
use std::path::Path;

const REPOE_BASE: &str = "https://raw.githubusercontent.com/repoe-fork/repoe-fork.github.io/master/data";

pub async fn download_repoe_data(data_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(data_dir)?;

    let files = ["mods.json", "base_items.json"];
    let client = reqwest::Client::new();

    for file in &files {
        let path = data_dir.join(file);
        if path.exists() {
            tracing::info!("Data file already exists: {}", file);
            continue;
        }

        let url = format!("{}/{}", REPOE_BASE, file);
        tracing::info!("Downloading {}...", url);
        let resp = client.get(&url).send().await?;
        let bytes = resp.bytes().await?;
        std::fs::write(&path, &bytes)?;
        tracing::info!("Downloaded {} ({} bytes)", file, bytes.len());
    }

    Ok(())
}

pub fn ingest_mods(db: &Database, data_dir: &Path) -> Result<()> {
    let path = data_dir.join("mods.json");
    let data = std::fs::read_to_string(&path)?;
    let raw: HashMap<String, RawMod> = serde_json::from_str(&data)?;

    tracing::info!("Parsing {} mods...", raw.len());

    let mods: Vec<Mod> = raw
        .into_iter()
        .map(|(id, raw)| Mod {
            id,
            name: raw.name.unwrap_or_default(),
            domain: raw.domain.unwrap_or_default(),
            generation_type: raw.generation_type.unwrap_or_default(),
            group: raw.groups.as_ref().and_then(|g| g.first().cloned()).unwrap_or_default(),
            required_level: raw.required_level.unwrap_or(0),
            stats: raw.stats.unwrap_or_default().into_iter().map(|s| ModStat {
                id: s.id.unwrap_or_default(),
                min: s.min.unwrap_or(0),
                max: s.max.unwrap_or(0),
            }).collect(),
            spawn_weights: raw.spawn_weights.unwrap_or_default().into_iter().map(|sw| SpawnWeight {
                tag: sw.tag.unwrap_or_default(),
                weight: sw.weight.unwrap_or(0),
            }).collect(),
            tags: raw.tags.unwrap_or_default(),
            is_essence_only: raw.is_essence_only.unwrap_or(false),
        })
        .collect();

    tracing::info!("Batch inserting {} mods...", mods.len());
    db.batch_insert_mods(&mods)?;
    db.rebuild_fts(&mods)?;
    tracing::info!("Mods ingestion complete");

    Ok(())
}

pub fn ingest_base_items(db: &Database, data_dir: &Path) -> Result<()> {
    let path = data_dir.join("base_items.json");
    let data = std::fs::read_to_string(&path)?;
    let raw: HashMap<String, RawBaseItem> = serde_json::from_str(&data)?;

    tracing::info!("Parsing {} base items...", raw.len());

    let items: Vec<BaseItem> = raw
        .into_iter()
        .filter(|(_, raw)| raw.name.is_some())
        .map(|(id, raw)| {
            let image_url = raw.visual_identity.as_ref()
                .and_then(|vi| vi.dds_file.as_ref())
                .map(|dds| {
                    let png_path = dds.replace(".dds", ".png");
                    format!("https://web.poecdn.com/image/{}", png_path)
                });

            BaseItem {
                id,
                name: raw.name.unwrap_or_default(),
                item_class: raw.item_class.unwrap_or_default(),
                drop_level: raw.drop_level.unwrap_or(0),
                tags: raw.tags.unwrap_or_default(),
                implicits: raw.implicits.unwrap_or_default(),
                implicit_stats: vec![],
                properties: BaseItemProperties {
                    armour_min: raw.properties.as_ref().and_then(|p| p.armour.as_ref().and_then(|v| v.min)),
                    armour_max: raw.properties.as_ref().and_then(|p| p.armour.as_ref().and_then(|v| v.max)),
                    evasion_min: raw.properties.as_ref().and_then(|p| p.evasion.as_ref().and_then(|v| v.min)),
                    evasion_max: raw.properties.as_ref().and_then(|p| p.evasion.as_ref().and_then(|v| v.max)),
                    energy_shield_min: raw.properties.as_ref().and_then(|p| p.energy_shield.as_ref().and_then(|v| v.min)),
                    energy_shield_max: raw.properties.as_ref().and_then(|p| p.energy_shield.as_ref().and_then(|v| v.max)),
                    movement_speed: raw.properties.as_ref().and_then(|p| p.movement_speed),
                    block: raw.properties.as_ref().and_then(|p| p.block),
                },
                requirements: BaseItemRequirements {
                    level: raw.requirements.as_ref().and_then(|r| r.level),
                    strength: raw.requirements.as_ref().and_then(|r| r.strength).filter(|&v| v > 0),
                    dexterity: raw.requirements.as_ref().and_then(|r| r.dexterity).filter(|&v| v > 0),
                    intelligence: raw.requirements.as_ref().and_then(|r| r.intelligence).filter(|&v| v > 0),
                },
                image_url,
                inventory_width: raw.inventory_width,
                inventory_height: raw.inventory_height,
            }
        })
        .collect();

    db.batch_insert_base_items(&items)?;
    tracing::info!("Base items ingestion complete");

    Ok(())
}

// Raw serde types for RePoE JSON parsing

#[derive(serde::Deserialize)]
struct RawMod {
    name: Option<String>,
    domain: Option<String>,
    generation_type: Option<String>,
    groups: Option<Vec<String>>,
    required_level: Option<i32>,
    stats: Option<Vec<RawModStat>>,
    spawn_weights: Option<Vec<RawSpawnWeight>>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    is_essence_only: Option<bool>,
}

#[derive(serde::Deserialize)]
struct RawModStat {
    id: Option<String>,
    min: Option<i64>,
    max: Option<i64>,
}

#[derive(serde::Deserialize)]
struct RawSpawnWeight {
    tag: Option<String>,
    weight: Option<i32>,
}

#[derive(serde::Deserialize)]
struct RawBaseItem {
    name: Option<String>,
    item_class: Option<String>,
    drop_level: Option<i32>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    implicits: Option<Vec<String>>,
    properties: Option<RawBaseItemProps>,
    requirements: Option<RawRequirements>,
    visual_identity: Option<RawVisualIdentity>,
    inventory_width: Option<i32>,
    inventory_height: Option<i32>,
}

#[derive(serde::Deserialize)]
struct RawVisualIdentity {
    dds_file: Option<String>,
}

#[derive(serde::Deserialize)]
struct RawBaseItemProps {
    armour: Option<MinMax>,
    evasion: Option<MinMax>,
    energy_shield: Option<MinMax>,
    movement_speed: Option<i32>,
    block: Option<i32>,
}

#[derive(serde::Deserialize)]
struct RawRequirements {
    level: Option<i32>,
    strength: Option<i32>,
    dexterity: Option<i32>,
    intelligence: Option<i32>,
}

#[derive(serde::Deserialize)]
struct MinMax {
    min: Option<i32>,
    max: Option<i32>,
}
