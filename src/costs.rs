use crate::{engine::Profile, lab, reference::Reference};
use std::collections::BTreeSet;

pub struct PartCost {
    pub id: String,
    pub name: String,
    pub owned: bool,
    pub unlocked: bool,
    pub purchase: Option<f64>,
    pub fee: Option<u32>,
}
pub struct Cost {
    pub parts: Vec<PartCost>,
    pub purchase: f64,
    pub fees: u32,
    pub complete: bool,
}
pub fn summary(data: &Reference, p: &Profile) -> Cost {
    let mut result = Cost {
        parts: vec![],
        purchase: 0.,
        fees: 0,
        complete: true,
    };
    let mut seen = BTreeSet::new();
    for id in p.lab_parts.values() {
        if !seen.insert(id.to_lowercase()) {
            continue;
        }
        let Some(part) = lab::item(data, id) else {
            result.complete = false;
            continue;
        };
        let owned = lab::owned(part, p);
        let unlocked = owned || p.unlocked.iter().any(|s| s.eq_ignore_ascii_case(id));
        // Only the exact item price is valid; variant prices do not establish this item's price.
        let purchase = if owned {
            Some(0.)
        } else {
            part.vendor
                .iter()
                .filter(|v| v.tag.eq_ignore_ascii_case(id))
                .map(|v| v.price)
                .min_by(f64::total_cmp)
        };
        let fee = if unlocked || part.unlock_free || part.is_default {
            Some(0)
        } else {
            part.unlock.as_ref().map(|u| u.cost)
        };
        result.purchase += purchase.unwrap_or(0.);
        result.fees = result.fees.saturating_add(fee.unwrap_or(0));
        result.complete &= purchase.is_some() && fee.is_some();
        result.parts.push(PartCost {
            id: id.clone(),
            name: part.name.clone(),
            owned,
            unlocked,
            purchase,
            fee,
        });
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ownership_and_paid_unlocks_are_separate_and_missing_prices_stay_unknown() {
        let d = Reference::load();
        let mut p = Profile::default();
        let part = d
            .items
            .iter()
            .find(|i| {
                i.unlock.as_ref().is_some_and(|u| u.cost > 0)
                    && i.vendor.iter().any(|v| v.tag == i.id)
            })
            .unwrap();
        p.lab_parts
            .insert(crate::engine::Slot::Optic, part.id.clone());
        let a = summary(&d, &p);
        assert!(a.purchase > 0. && a.fees > 0);
        p.unlocked.insert(part.id.to_uppercase());
        let b = summary(&d, &p);
        assert_eq!(b.fees, 0);
        assert_eq!(a.purchase, b.purchase);
        p.owned.insert(part.id.to_lowercase());
        let c = summary(&d, &p);
        assert_eq!(c.purchase, 0.);
        assert_eq!(c.fees, 0);
        p.owned.clear();
        p.lab_parts.clear();
        p.lab_parts
            .insert(crate::engine::Slot::Optic, "Id.Item.BasicRedDot".into());
        assert!(!summary(&d, &p).complete);
    }
}
