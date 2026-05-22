static HIDEOUTS: &[&str] = &[
    "Hideout",
    "Coastal Hideout",
    "Overgrown Hideout",
    "Lush Hideout",
    "Desert Hideout",
    "Glacial Hideout",
    "Backstreet Hideout",
    "Immaculate Hideout",
    "Celestial Hideout",
    "Unearthed Hideout",
    "Stately Hideout",
    "Brutal Hideout",
    "Divided Hideout",
    "Baleful Hideout",
    "Sunspire Hideout",
    "Luxurious Hideout",
    "Skeletal Hideout",
    "Enlightened Hideout",
];

static TOWNS: &[&str] = &[
    "Lioneye's Watch",
    "The Forest Encampment",
    "The Sarn Encampment",
    "Highgate",
    "Overseer's Tower",
    "The Bridge Encampment",
    "The Harbour Bridge",
    "Oriath Docks",
    "Oriath",
    "Karui Shores",
];

pub fn is_hideout(area: &str) -> bool {
    // Exact match or suffix match for custom hideouts
    HIDEOUTS.iter().any(|h| area == *h)
        || area.ends_with("Hideout")
        || area.ends_with("hideout")
}

pub fn is_town(area: &str) -> bool {
    TOWNS.iter().any(|t| area == *t)
}

pub fn is_idle_zone(area: &str) -> bool {
    is_hideout(area) || is_town(area)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hideout_detection() {
        assert!(is_hideout("Hideout"));
        assert!(is_hideout("Celestial Hideout"));
        assert!(is_hideout("My Custom Hideout"));
        assert!(!is_hideout("Strand"));
    }

    #[test]
    fn town_detection() {
        assert!(is_town("Oriath"));
        assert!(is_town("Highgate"));
        assert!(!is_town("Strand"));
    }

    #[test]
    fn idle_zone_detection() {
        assert!(is_idle_zone("Hideout"));
        assert!(is_idle_zone("Oriath"));
        assert!(!is_idle_zone("Strand"));
    }
}
