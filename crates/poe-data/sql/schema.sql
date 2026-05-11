CREATE TABLE IF NOT EXISTS mods (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    domain TEXT NOT NULL,
    generation_type TEXT NOT NULL,
    grp TEXT NOT NULL DEFAULT '',
    required_level INTEGER NOT NULL DEFAULT 0,
    is_essence_only INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS mod_stats (
    mod_id TEXT NOT NULL,
    stat_id TEXT NOT NULL,
    min_val INTEGER NOT NULL DEFAULT 0,
    max_val INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (mod_id, stat_id),
    FOREIGN KEY (mod_id) REFERENCES mods(id)
);

CREATE TABLE IF NOT EXISTS mod_spawn_weights (
    mod_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    weight INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (mod_id, tag),
    FOREIGN KEY (mod_id) REFERENCES mods(id)
);

CREATE TABLE IF NOT EXISTS base_items (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    item_class TEXT NOT NULL,
    drop_level INTEGER NOT NULL DEFAULT 0,
    tags TEXT NOT NULL DEFAULT '[]',
    implicits TEXT NOT NULL DEFAULT '[]',
    image_url TEXT,
    inventory_width INTEGER,
    inventory_height INTEGER,
    armour_min INTEGER,
    armour_max INTEGER,
    evasion_min INTEGER,
    evasion_max INTEGER,
    energy_shield_min INTEGER,
    energy_shield_max INTEGER,
    movement_speed INTEGER,
    block_chance INTEGER,
    req_level INTEGER,
    req_str INTEGER,
    req_dex INTEGER,
    req_int INTEGER
);

CREATE INDEX IF NOT EXISTS idx_mods_domain_gen ON mods(domain, generation_type);
CREATE INDEX IF NOT EXISTS idx_spawn_weights ON mod_spawn_weights(tag, mod_id, weight);
CREATE INDEX IF NOT EXISTS idx_bases_class ON base_items(item_class);

CREATE VIRTUAL TABLE IF NOT EXISTS mods_fts USING fts5(name, stat_text, tags);
