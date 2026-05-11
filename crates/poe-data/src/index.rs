use poe_core::types::Mod;
use std::collections::HashMap;

pub struct MemIndex {
    pub mods_by_id: HashMap<String, Mod>,
    pub mods_by_tag: HashMap<String, Vec<String>>,
}

impl MemIndex {
    pub fn new() -> Self {
        Self {
            mods_by_id: HashMap::new(),
            mods_by_tag: HashMap::new(),
        }
    }

    pub fn build_from_mods(&mut self, mods: &[Mod]) {
        self.mods_by_id.clear();
        self.mods_by_tag.clear();

        self.mods_by_id.reserve(mods.len());

        for m in mods {
            self.mods_by_id.insert(m.id.clone(), m.clone());

            for sw in &m.spawn_weights {
                if sw.weight > 0 {
                    self.mods_by_tag
                        .entry(sw.tag.clone())
                        .or_default()
                        .push(m.id.clone());
                }
            }
        }
    }

    pub fn get_mod(&self, id: &str) -> Option<&Mod> {
        self.mods_by_id.get(id)
    }

    pub fn get_mods_for_tag(&self, tag: &str) -> &[String] {
        self.mods_by_tag.get(tag).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn get_mods_for_tags(&self, tags: &[String]) -> Vec<&Mod> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();

        for tag in tags {
            for mod_id in self.get_mods_for_tag(tag) {
                if seen.insert(mod_id) {
                    if let Some(m) = self.mods_by_id.get(mod_id) {
                        result.push(m);
                    }
                }
            }
        }

        result
    }
}
