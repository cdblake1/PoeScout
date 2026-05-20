use anyhow::Result;
use poe_core::types::{AffixEntry, AffixesForBaseResult, BaseItem, BaseItemProperties, BaseItemRequirements, BaseSearchQuery, BaseSearchResult, ImplicitStat, Mod, ModStat, SearchQuery, SearchResult, SpawnWeight};

const BASE_ITEM_COLS: &str = "id, name, item_class, drop_level, tags, implicits, image_url, inventory_width, inventory_height, armour_min, armour_max, evasion_min, evasion_max, energy_shield_min, energy_shield_max, movement_speed, block_chance, req_level, req_str, req_dex, req_int";

fn parse_base_item_row(row: &rusqlite::Row) -> rusqlite::Result<BaseItem> {
    let tags_str: String = row.get(4)?;
    let implicits_str: String = row.get(5)?;
    Ok(BaseItem {
        id: row.get(0)?,
        name: row.get(1)?,
        item_class: row.get(2)?,
        drop_level: row.get(3)?,
        tags: serde_json::from_str(&tags_str).unwrap_or_default(),
        implicits: serde_json::from_str(&implicits_str).unwrap_or_default(),
        implicit_stats: vec![],
        implicit_text: vec![],
        image_url: row.get(6)?,
        inventory_width: row.get(7)?,
        inventory_height: row.get(8)?,
        properties: BaseItemProperties {
            armour_min: row.get(9)?,
            armour_max: row.get(10)?,
            evasion_min: row.get(11)?,
            evasion_max: row.get(12)?,
            energy_shield_min: row.get(13)?,
            energy_shield_max: row.get(14)?,
            movement_speed: row.get(15)?,
            block: row.get(16)?,
        },
        requirements: BaseItemRequirements {
            level: row.get(17)?,
            strength: row.get(18)?,
            dexterity: row.get(19)?,
            intelligence: row.get(20)?,
        },
    })
}
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA cache_size=-64000;")?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(include_str!("../sql/schema.sql"))?;
        Ok(())
    }

    pub fn is_empty(&self) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM mods", [], |r| r.get(0))?;
        Ok(count == 0)
    }

    pub fn insert_mod(&self, m: &Mod) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO mods (id, name, domain, generation_type, grp, required_level, is_essence_only, implicit_tags, text, mod_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![m.id, m.name, m.domain, m.generation_type, m.group, m.required_level, m.is_essence_only, serde_json::to_string(&m.tags).unwrap_or_default(), m.text, m.mod_type],
        )?;

        // Stats
        for stat in &m.stats {
            conn.execute(
                "INSERT OR REPLACE INTO mod_stats (mod_id, stat_id, min_val, max_val) VALUES (?1, ?2, ?3, ?4)",
                params![m.id, stat.id, stat.min, stat.max],
            )?;
        }

        // Spawn weights
        for (i, sw) in m.spawn_weights.iter().enumerate() {
            conn.execute(
                "INSERT OR REPLACE INTO mod_spawn_weights (mod_id, tag, weight, position) VALUES (?1, ?2, ?3, ?4)",
                params![m.id, sw.tag, sw.weight, i as i32],
            )?;
        }

        // Tags
        let tags_str = m.tags.join(",");
        conn.execute(
            "INSERT OR REPLACE INTO mods_fts (rowid, name, stat_text, tags)
             VALUES ((SELECT rowid FROM mods WHERE id = ?1), ?2, ?3, ?4)",
            params![
                m.id,
                m.name,
                m.stats.iter().map(|s| s.id.as_str()).collect::<Vec<_>>().join(" "),
                tags_str
            ],
        )?;

        Ok(())
    }

    pub fn insert_base_item(&self, item: &BaseItem) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO base_items (id, name, item_class, drop_level, tags, implicits, image_url, inventory_width, inventory_height,
             armour_min, armour_max, evasion_min, evasion_max, energy_shield_min, energy_shield_max, movement_speed, block_chance,
             req_level, req_str, req_dex, req_int)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
            params![
                item.id, item.name, item.item_class, item.drop_level,
                serde_json::to_string(&item.tags)?,
                serde_json::to_string(&item.implicits)?,
                item.image_url, item.inventory_width, item.inventory_height,
                item.properties.armour_min, item.properties.armour_max,
                item.properties.evasion_min, item.properties.evasion_max,
                item.properties.energy_shield_min, item.properties.energy_shield_max,
                item.properties.movement_speed, item.properties.block,
                item.requirements.level, item.requirements.strength,
                item.requirements.dexterity, item.requirements.intelligence,
            ],
        )?;
        Ok(())
    }

    pub fn list_item_classes(&self) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT item_class, COUNT(*) as cnt FROM base_items GROUP BY item_class ORDER BY cnt DESC"
        )?;
        let classes = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(classes)
    }

    pub fn list_bases_by_class(&self, item_class: &str) -> Result<Vec<BaseItem>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!("SELECT {} FROM base_items WHERE item_class = ?1 AND name != '' ORDER BY drop_level ASC", BASE_ITEM_COLS);
        let mut stmt = conn.prepare(&sql)?;
        let mut items: Vec<BaseItem> = stmt
            .query_map(params![item_class], |row| parse_base_item_row(row))?
            .collect::<rusqlite::Result<_>>()?;
        self.resolve_implicits_inner(&conn, &mut items)?;
        Ok(items)
    }

    fn resolve_implicits_inner(&self, conn: &Connection, items: &mut [BaseItem]) -> Result<()> {
        let mut stmt = conn.prepare_cached(
            "SELECT stat_id, min_val, max_val FROM mod_stats WHERE mod_id = ?1"
        )?;
        let mut text_stmt = conn.prepare_cached(
            "SELECT text FROM mods WHERE id = ?1"
        )?;
        for item in items.iter_mut() {
            for mod_id in &item.implicits {
                let stats: Vec<ImplicitStat> = stmt
                    .query_map(params![mod_id], |row| {
                        Ok(ImplicitStat {
                            stat_id: row.get(0)?,
                            min: row.get(1)?,
                            max: row.get(2)?,
                        })
                    })?
                    .collect::<rusqlite::Result<_>>()?;
                item.implicit_stats.extend(stats);

                let text: String = text_stmt
                    .query_row(params![mod_id], |row| row.get(0))
                    .unwrap_or_default();
                if !text.is_empty() {
                    item.implicit_text.push(text);
                }
            }
        }
        Ok(())
    }

    pub fn batch_insert_mods(&self, mods: &[Mod]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        {
            let mut stmt_mod = tx.prepare_cached(
                "INSERT OR REPLACE INTO mods (id, name, domain, generation_type, grp, required_level, is_essence_only, implicit_tags, text, mod_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"
            )?;
            let mut stmt_stat = tx.prepare_cached(
                "INSERT OR REPLACE INTO mod_stats (mod_id, stat_id, min_val, max_val) VALUES (?1, ?2, ?3, ?4)"
            )?;
            let mut stmt_sw = tx.prepare_cached(
                "INSERT OR REPLACE INTO mod_spawn_weights (mod_id, tag, weight, position) VALUES (?1, ?2, ?3, ?4)"
            )?;

            for m in mods {
                stmt_mod.execute(params![m.id, m.name, m.domain, m.generation_type, m.group, m.required_level, m.is_essence_only, serde_json::to_string(&m.tags).unwrap_or_default(), m.text, m.mod_type])?;
                for stat in &m.stats {
                    stmt_stat.execute(params![m.id, stat.id, stat.min, stat.max])?;
                }
                for (i, sw) in m.spawn_weights.iter().enumerate() {
                    stmt_sw.execute(params![m.id, sw.tag, sw.weight, i as i32])?;
                }
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn batch_insert_base_items(&self, items: &[BaseItem]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO base_items (id, name, item_class, drop_level, tags, implicits, image_url, inventory_width, inventory_height,
                 armour_min, armour_max, evasion_min, evasion_max, energy_shield_min, energy_shield_max, movement_speed, block_chance,
                 req_level, req_str, req_dex, req_int)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)"
            )?;

            for item in items {
                stmt.execute(params![
                    item.id, item.name, item.item_class, item.drop_level,
                    serde_json::to_string(&item.tags)?,
                    serde_json::to_string(&item.implicits)?,
                    item.image_url, item.inventory_width, item.inventory_height,
                    item.properties.armour_min, item.properties.armour_max,
                    item.properties.evasion_min, item.properties.evasion_max,
                    item.properties.energy_shield_min, item.properties.energy_shield_max,
                    item.properties.movement_speed, item.properties.block,
                    item.requirements.level, item.requirements.strength,
                    item.requirements.dexterity, item.requirements.intelligence,
                ])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn rebuild_fts(&self, mods: &[Mod]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM mods_fts", [])?;
        let tx = conn.unchecked_transaction()?;

        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO mods_fts (rowid, name, stat_text, tags)
                 VALUES ((SELECT rowid FROM mods WHERE id = ?1), ?2, ?3, ?4)"
            )?;

            for m in mods {
                let stat_text = m.stats.iter().map(|s| s.id.as_str()).collect::<Vec<_>>().join(" ");
                let tags_str = m.tags.join(",");
                stmt.execute(params![m.id, m.name, stat_text, tags_str])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_all_mods(&self) -> Result<Vec<Mod>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, domain, generation_type, grp, required_level, is_essence_only, text, mod_type FROM mods"
        )?;

        let mod_rows: Vec<(String, String, String, String, String, i32, bool, String, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            })?
            .collect::<rusqlite::Result<_>>()?;

        let mut mods = Vec::with_capacity(mod_rows.len());
        for (id, name, domain, gen_type, group, req_level, essence_only, text, mod_type) in mod_rows {
            let stats = self.load_mod_stats_inner(&conn, &id)?;
            let spawn_weights = self.load_spawn_weights_inner(&conn, &id)?;
            let tags = self.load_mod_tags_inner(&conn, &id)?;

            mods.push(Mod {
                id,
                name,
                domain,
                generation_type: gen_type,
                group,
                required_level: req_level,
                text,
                stats,
                spawn_weights,
                tags,
                is_essence_only: essence_only,
                mod_type,
            });
        }

        Ok(mods)
    }

    fn load_mod_stats_inner(&self, conn: &Connection, mod_id: &str) -> Result<Vec<ModStat>> {
        let mut stmt = conn.prepare_cached(
            "SELECT stat_id, min_val, max_val FROM mod_stats WHERE mod_id = ?1"
        )?;
        let stats = stmt
            .query_map(params![mod_id], |row| {
                Ok(ModStat {
                    id: row.get(0)?,
                    min: row.get(1)?,
                    max: row.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(stats)
    }

    fn load_spawn_weights_inner(&self, conn: &Connection, mod_id: &str) -> Result<Vec<SpawnWeight>> {
        let mut stmt = conn.prepare_cached(
            "SELECT tag, weight FROM mod_spawn_weights WHERE mod_id = ?1"
        )?;
        let weights = stmt
            .query_map(params![mod_id], |row| {
                Ok(SpawnWeight {
                    tag: row.get(0)?,
                    weight: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(weights)
    }

    fn load_mod_tags_inner(&self, _conn: &Connection, _mod_id: &str) -> Result<Vec<String>> {
        // Tags are stored in spawn_weights as the tag field
        // For now, derive tags from spawn weights where weight > 0
        Ok(vec![])
    }

    pub fn search_mods(&self, query: &SearchQuery) -> Result<SearchResult> {
        let start = Instant::now();
        let conn = self.conn.lock().unwrap();
        let limit = query.limit.unwrap_or(50);

        let mut sql = String::from(
            "SELECT m.id, m.name, m.domain, m.generation_type, m.grp, m.required_level, m.is_essence_only, m.text, m.mod_type
             FROM mods m"
        );
        let mut conditions = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if !query.text.is_empty() {
            sql.push_str(" JOIN mods_fts fts ON fts.rowid = m.rowid");
            conditions.push(format!("mods_fts MATCH ?{}", param_values.len() + 1));
            param_values.push(Box::new(query.text.clone()));
        }

        if let Some(ref domain) = query.domain {
            conditions.push(format!("m.domain = ?{}", param_values.len() + 1));
            param_values.push(Box::new(domain.clone()));
        }

        if let Some(ref gen_type) = query.generation_type {
            conditions.push(format!("m.generation_type = ?{}", param_values.len() + 1));
            param_values.push(Box::new(gen_type.clone()));
        }

        if let Some(min_lvl) = query.min_level {
            conditions.push(format!("m.required_level >= ?{}", param_values.len() + 1));
            param_values.push(Box::new(min_lvl));
        }

        if let Some(max_lvl) = query.max_level {
            conditions.push(format!("m.required_level <= ?{}", param_values.len() + 1));
            param_values.push(Box::new(max_lvl));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(&format!(" LIMIT ?{}", param_values.len() + 1));
        param_values.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let mod_rows: Vec<(String, String, String, String, String, i32, bool, String, String)> = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            })?
            .collect::<rusqlite::Result<_>>()?;

        let total = mod_rows.len();
        let mut mods = Vec::with_capacity(total);
        for (id, name, domain, gen_type, group, req_level, essence_only, text, mod_type) in mod_rows {
            let stats = self.load_mod_stats_inner(&conn, &id)?;
            let spawn_weights = self.load_spawn_weights_inner(&conn, &id)?;
            mods.push(Mod {
                id,
                name,
                domain,
                generation_type: gen_type,
                group,
                required_level: req_level,
                text,
                stats,
                spawn_weights,
                tags: vec![],
                is_essence_only: essence_only,
                mod_type,
            });
        }

        let query_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok(SearchResult { mods, total, query_ms })
    }

    pub fn search_bases(&self, query: &BaseSearchQuery) -> Result<BaseSearchResult> {
        let start = Instant::now();
        let conn = self.conn.lock().unwrap();
        let limit = query.limit.unwrap_or(50);

        let mut sql = format!("SELECT {} FROM base_items", BASE_ITEM_COLS);
        let mut conditions = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if !query.text.is_empty() {
            conditions.push(format!("name LIKE ?{}", param_values.len() + 1));
            param_values.push(Box::new(format!("%{}%", query.text)));
        }

        if let Some(ref item_class) = query.item_class {
            conditions.push(format!("item_class = ?{}", param_values.len() + 1));
            param_values.push(Box::new(item_class.clone()));
        }

        if let Some(min_lvl) = query.min_level {
            conditions.push(format!("drop_level >= ?{}", param_values.len() + 1));
            param_values.push(Box::new(min_lvl));
        }

        if let Some(max_lvl) = query.max_level {
            conditions.push(format!("drop_level <= ?{}", param_values.len() + 1));
            param_values.push(Box::new(max_lvl));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(&format!(" LIMIT ?{}", param_values.len() + 1));
        param_values.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let mut items: Vec<BaseItem> = stmt
            .query_map(params_refs.as_slice(), |row| parse_base_item_row(row))?
            .collect::<rusqlite::Result<_>>()?;
        self.resolve_implicits_inner(&conn, &mut items)?;

        let total = items.len();
        let query_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok(BaseSearchResult { items, total, query_ms })
    }

    /// Equipment-type tags that have influence variants in spawn_weights.
    const EQUIPMENT_TAGS: &'static [&'static str] = &[
        "2h_axe", "2h_mace", "2h_sword", "amulet", "axe", "belt",
        "body_armour", "boots", "bow", "claw", "dagger", "gloves",
        "helmet", "mace", "quiver", "ring", "rune_dagger", "sceptre",
        "shield", "staff", "sword", "wand", "warstaff",
    ];

    /// Influence suffixes: (tag_suffix, source_label)
    const INFLUENCE_SUFFIXES: &'static [(&'static str, &'static str)] = &[
        ("shaper", "shaper"),
        ("elder", "elder"),
        ("crusader", "crusader"),
        ("adjudicator", "warlord"),   // Warlord internally = adjudicator
        ("basilisk", "hunter"),       // Hunter internally = basilisk
        ("eyrie", "redeemer"),        // Redeemer internally = eyrie
    ];

    /// Domains that contain equipment-craftable mods.
    const AFFIX_DOMAINS: &'static [&'static str] = &[
        "item", "crafted", "delve", "unveiled", "veiled",
    ];

    pub fn get_affixes_for_base(&self, base_tags: &[String]) -> Result<AffixesForBaseResult> {
        let start = Instant::now();
        let conn = self.conn.lock().unwrap();

        if base_tags.is_empty() {
            return Ok(AffixesForBaseResult { affixes: vec![], query_ms: 0.0 });
        }

        // Start with the base item's own tags
        let mut all_tags: Vec<String> = base_tags.to_vec();

        // Generate influence-variant tags: for each equipment tag the base has,
        // add {equipment_tag}_{influence_suffix} for all 6 influences
        for tag in base_tags {
            if Self::EQUIPMENT_TAGS.contains(&tag.as_str()) {
                for (suffix, _label) in Self::INFLUENCE_SUFFIXES {
                    all_tags.push(format!("{}_{}", tag, suffix));
                }
            }
        }

        // Build placeholders for tags
        let tag_count = all_tags.len();
        let tag_placeholders: Vec<String> = (1..=tag_count).map(|i| format!("?{}", i)).collect();
        let tag_in_clause = tag_placeholders.join(", ");

        // Build domain IN clause
        let domain_start = tag_count + 1;
        let domain_placeholders: Vec<String> = (0..Self::AFFIX_DOMAINS.len())
            .map(|i| format!("?{}", domain_start + i))
            .collect();
        let domain_in_clause = domain_placeholders.join(", ");

        let sql = format!(
            "SELECT m.id, m.name, m.domain, m.generation_type, m.grp, m.required_level, m.is_essence_only,
                    m.implicit_tags, sw.weight as effective_weight, m.text, m.mod_type
             FROM mods m
             JOIN mod_spawn_weights sw ON sw.mod_id = m.id
             WHERE sw.tag IN ({tag_in_clause})
               AND sw.position = (
                 SELECT MIN(sw2.position) FROM mod_spawn_weights sw2
                 WHERE sw2.mod_id = m.id AND sw2.tag IN ({tag_in_clause})
               )
               AND sw.weight > 0
               AND m.domain IN ({domain_in_clause})
             ORDER BY m.generation_type, m.grp, m.required_level DESC"
        );

        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        for tag in &all_tags {
            params.push(Box::new(tag.clone()));
        }
        for domain in Self::AFFIX_DOMAINS {
            params.push(Box::new(domain.to_string()));
        }

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;

        let rows: Vec<(String, String, String, String, String, i32, bool, String, i32, String, String)> = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok((
                    row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                    row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?,
                    row.get(8)?, row.get(9)?, row.get(10)?,
                ))
            })?
            .collect::<rusqlite::Result<_>>()?;

        let mut affixes = Vec::with_capacity(rows.len());
        for (id, name, domain, gen_type, group, req_level, essence_only, implicit_tags_json, eff_weight, text, mod_type) in rows {
            let stats = self.load_mod_stats_inner(&conn, &id)?;
            let spawn_weights = self.load_spawn_weights_inner(&conn, &id)?;
            let tags: Vec<String> = serde_json::from_str(&implicit_tags_json).unwrap_or_default();
            affixes.push(AffixEntry {
                mod_data: Mod {
                    id,
                    name,
                    domain,
                    generation_type: gen_type,
                    group,
                    required_level: req_level,
                    text,
                    stats,
                    spawn_weights,
                    tags,
                    is_essence_only: essence_only,
                    mod_type,
                },
                effective_weight: eff_weight,
            });
        }

        let query_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok(AffixesForBaseResult { affixes, query_ms })
    }
}
