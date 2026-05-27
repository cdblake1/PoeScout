use poe_core::types::{PricedItem, StashItem};
use poe_pricing::PricingEngine;

pub async fn price_item(item: &StashItem, pricing: &PricingEngine) -> PricedItem {
    let lookup_name = match item.frame_type {
        Some(5) | Some(6) => &item.type_line,
        Some(3) => {
            if !item.name.is_empty() {
                &item.name
            } else {
                &item.type_line
            }
        }
        _ => {
            if !item.name.is_empty() {
                &item.name
            } else {
                &item.type_line
            }
        }
    };

    let price = pricing.get_price(lookup_name).await;
    let stack = item.stack_size.unwrap_or(1) as f64;

    PricedItem {
        item: item.clone(),
        unit_price: price.as_ref().map(|p| p.chaos_value),
        total_price: price.as_ref().map(|p| p.chaos_value * stack),
        listing_count: price.as_ref().and_then(|p| p.count),
        price_source: price.map(|p| p.category),
    }
}

pub fn price_item_sync(
    item: &StashItem,
    get_price: &dyn Fn(&str) -> Option<f64>,
) -> PricedItem {
    let lookup_name = match item.frame_type {
        Some(5) | Some(6) => &item.type_line,
        Some(3) => {
            if !item.name.is_empty() {
                &item.name
            } else {
                &item.type_line
            }
        }
        _ => {
            if !item.name.is_empty() {
                &item.name
            } else {
                &item.type_line
            }
        }
    };

    let unit_price = get_price(lookup_name);
    let stack = item.stack_size.unwrap_or(1) as f64;

    PricedItem {
        item: item.clone(),
        unit_price,
        total_price: unit_price.map(|p| p * stack),
        listing_count: None,
        price_source: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(name: &str, type_line: &str, frame_type: u32, stack: u32) -> StashItem {
        StashItem {
            name: name.to_string(),
            type_line: type_line.to_string(),
            base_type: None,
            stack_size: Some(stack),
            max_stack_size: None,
            icon: String::new(),
            ilvl: None,
            identified: None,
            frame_type: Some(frame_type),
        }
    }

    #[test]
    fn currency_matches_by_type_line() {
        let item = make_item("", "Chaos Orb", 5, 10);
        let pricer = |name: &str| -> Option<f64> {
            if name == "Chaos Orb" { Some(1.0) } else { None }
        };
        let priced = price_item_sync(&item, &pricer);
        assert_eq!(priced.unit_price, Some(1.0));
        assert_eq!(priced.total_price, Some(10.0));
    }

    #[test]
    fn unique_matches_by_name() {
        let item = make_item("Headhunter", "Leather Belt", 3, 1);
        let pricer = |name: &str| -> Option<f64> {
            if name == "Headhunter" { Some(5000.0) } else { None }
        };
        let priced = price_item_sync(&item, &pricer);
        assert_eq!(priced.unit_price, Some(5000.0));
    }

    #[test]
    fn unmatched_item_gets_none() {
        let item = make_item("", "Some Random Rare", 2, 1);
        let pricer = |_: &str| -> Option<f64> { None };
        let priced = price_item_sync(&item, &pricer);
        assert_eq!(priced.unit_price, None);
        assert_eq!(priced.total_price, None);
    }
}
