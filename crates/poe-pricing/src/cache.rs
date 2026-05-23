use poe_core::types::PriceRecord;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct PriceCache {
    prices: HashMap<String, PriceRecord>,
    divine_price: f64,
    last_updated: Option<Instant>,
    ttl: Duration,
}

impl PriceCache {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            prices: HashMap::new(),
            divine_price: 1.0,
            last_updated: None,
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub fn is_stale(&self) -> bool {
        match self.last_updated {
            None => true,
            Some(t) => t.elapsed() > self.ttl,
        }
    }

    pub fn update(&mut self, records: Vec<PriceRecord>) {
        self.prices.clear();

        for record in &records {
            if record.name == "Divine Orb" && record.category == "Currency" {
                self.divine_price = record.chaos_value;
            }
        }

        let divine = self.divine_price;
        for mut record in records {
            if divine > 0.0 {
                record.divine_value = Some(record.chaos_value / divine);
            }
            self.prices.insert(record.name.to_lowercase(), record);
        }

        self.last_updated = Some(Instant::now());
    }

    pub fn get_price(&self, item_name: &str) -> Option<&PriceRecord> {
        self.prices.get(&item_name.to_lowercase())
    }

    pub fn divine_ratio(&self) -> f64 {
        self.divine_price
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(name: &str, category: &str, chaos: f64) -> PriceRecord {
        PriceRecord {
            name: name.to_string(),
            category: category.to_string(),
            chaos_value: chaos,
            divine_value: None,
            icon: None,
        }
    }

    #[test]
    fn cache_starts_stale() {
        let cache = PriceCache::new(300);
        assert!(cache.is_stale());
    }

    #[test]
    fn update_makes_fresh() {
        let mut cache = PriceCache::new(300);
        cache.update(vec![make_record("Chaos Orb", "Currency", 1.0)]);
        assert!(!cache.is_stale());
    }

    #[test]
    fn case_insensitive_lookup() {
        let mut cache = PriceCache::new(300);
        cache.update(vec![make_record("Divine Orb", "Currency", 200.0)]);
        assert!(cache.get_price("divine orb").is_some());
        assert!(cache.get_price("DIVINE ORB").is_some());
    }

    #[test]
    fn divine_ratio_computed() {
        let mut cache = PriceCache::new(300);
        cache.update(vec![
            make_record("Divine Orb", "Currency", 200.0),
            make_record("Exalted Orb", "Currency", 10.0),
        ]);
        assert_eq!(cache.divine_ratio(), 200.0);
        let ex = cache.get_price("exalted orb").unwrap();
        assert!((ex.divine_value.unwrap() - 0.05).abs() < 0.001);
    }
}
