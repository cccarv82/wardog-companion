//! Conservative preview of source-reported attachment multipliers.
//! Only multiplyCompound is implemented for the six modeled attributes.
use crate::{
    engine::{Profile, Slot},
    reference::{Reference, ReferenceItem},
};
use std::collections::BTreeMap;

pub const SLOTS: [Slot; 4] = [Slot::Muzzle, Slot::Optic, Slot::Grip, Slot::Magazine];
pub const ATTRIBUTES: [(&str, &str); 6] = [
    ("Weapon.VerticalRecoilMultiplier", "Vertical recoil"),
    ("Weapon.HorizontalRecoilMultiplier", "Horizontal recoil"),
    ("Weapon.AimTime", "Aim time"),
    ("Weapon.SpreadMultiplier", "Spread"),
    ("Weapon.EquipSpeedMultiplier", "Equip speed"),
    ("Character.AimSwayScaleMultiplier", "Aim sway"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    pub id: Option<String>,
    pub label: String,
}
impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}
impl Choice {
    pub fn unchanged() -> Self {
        Self {
            id: None,
            label: "Unchanged".into(),
        }
    }
    pub fn item(item: &ReferenceItem) -> Self {
        Self {
            id: Some(item.id.clone()),
            label: item.name.clone(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Estimate {
    pub factors: [f64; 6],
    pub cost: f64,
    pub unknown_prices: Vec<String>,
    pub ignored_effects: usize,
    pub selected: Vec<String>,
}
impl Estimate {
    pub fn budget_status(&self, budget: u32) -> String {
        if !self.unknown_prices.is_empty() {
            format!(
                "Known prices: ${:.2} · total unknown; budget cannot be verified",
                self.cost
            )
        } else if self.cost > budget as f64 {
            format!(
                "Selected parts: ${:.2} · over your ${budget} budget",
                self.cost
            )
        } else {
            format!(
                "Selected parts: ${:.2} / ${budget} budget · weapon and unchanged parts excluded",
                self.cost
            )
        }
    }
    pub fn score(&self, profile: &Profile) -> f64 {
        // Heuristic preference, deliberately separate from the physical multiplier composition.
        [self.factors[0], self.factors[1], self.factors[2]]
            .into_iter()
            .zip(profile.goal.weights())
            .map(|(factor, weight)| (1. - factor) * 100. * weight as f64)
            .sum()
    }
}
pub fn fits(item: &ReferenceItem, slot: Slot) -> bool {
    if !item.compatible_slots.is_empty() {
        return item.compatible_slots.contains(&slot);
    }
    matches!(
        (slot, item.slot.as_str()),
        (Slot::Muzzle, "Muzzle")
            | (Slot::Optic, "Sight")
            | (Slot::Grip, "Underbarrel")
            | (Slot::Magazine, "Magazine")
            | (Slot::Barrel, "Barrel")
            | (Slot::Stock, "Stock")
            | (Slot::PistolGrip, "Grip")
            | (Slot::Handguard, "Handguard")
    )
}
pub fn slots(data: &Reference) -> Vec<Slot> {
    if data.weapon.class_name.is_empty() {
        SLOTS.to_vec()
    } else {
        Slot::ALL
            .into_iter()
            .filter(|slot| data.slots.contains(slot))
            .collect()
    }
}
pub fn item<'a>(data: &'a Reference, id: &str) -> Option<&'a ReferenceItem> {
    data.items.iter().find(|i| i.id.eq_ignore_ascii_case(id))
}
pub fn owned(item: &ReferenceItem, p: &Profile) -> bool {
    item.is_default || p.owned.iter().any(|id| id.eq_ignore_ascii_case(&item.id))
}
pub fn candidates(data: &Reference, p: &Profile, slot: Slot) -> Vec<Choice> {
    let mut choices = vec![Choice::unchanged()];
    choices.extend(
        data.items
            .iter()
            .filter(|i| {
                fits(i, slot)
                    && i.eligible_by_level(p) == Some(true)
                    && (!p.owned_only || owned(i, p))
            })
            .map(Choice::item),
    );
    choices
}
pub fn evaluate(
    data: &Reference,
    p: &Profile,
    parts: &BTreeMap<Slot, String>,
) -> Result<Estimate, String> {
    let mut result = Estimate {
        factors: [1.; 6],
        cost: 0.,
        unknown_prices: vec![],
        ignored_effects: 0,
        selected: vec![],
    };
    for (slot, id) in parts {
        let part = item(data, id).ok_or_else(|| {
            format!("Saved item {id} is no longer available. Clear or replace it.")
        })?;
        if !fits(part, *slot) {
            return Err(format!(
                "{} does not match the {} slot in the imported data.",
                part.name,
                slot.label()
            ));
        }
        if part.conditional {
            return Err(format!(
                "{} has state-dependent effects (for example, deployed bipod). Inspect its source record in All attachments; this estimate does not assume deployment.",
                part.name
            ));
        }
        if part.eligible_by_level(p) != Some(true) {
            return Err(format!(
                "{}: unlock requirements are unknown or your level is too low.",
                part.name
            ));
        }
        if p.owned_only && !owned(part, p) {
            return Err(format!(
                "{} is not marked as owned. Open Customize to mark it, or turn off the owned-only filter in Settings.",
                part.name
            ));
        }
        result.selected.push(part.name.clone());
        let exact_price = part
            .vendor
            .iter()
            .find(|price| price.tag.eq_ignore_ascii_case(&part.id))
            .map(|price| price.price);
        match exact_price {
            Some(price) if price.is_finite() && price >= 0. => result.cost += price,
            _ => result.unknown_prices.push(part.name.clone()),
        }
        for modifier in part.modifiers.iter().chain(&part.character_modifiers) {
            let Some(index) = ATTRIBUTES
                .iter()
                .position(|(attribute, _)| *attribute == modifier.attribute)
            else {
                result.ignored_effects += 1;
                continue;
            };
            if modifier.op != "multiplyCompound" {
                return Err(format!(
                    "{} uses unsupported {} stacking for {}. This preview will not guess the result.",
                    part.name,
                    modifier.op,
                    modifier.label()
                ));
            }
            let value = modifier.value;
            if !value.is_finite() || value <= 0. {
                return Err(format!("Invalid multiplier in {}.", part.name));
            }
            result.factors[index] *= value;
            if !result.factors[index].is_finite() {
                return Err("Multiplier overflow; no estimate produced.".into());
            }
        }
    }
    Ok(result)
}
pub fn suggest(data: &Reference, p: &Profile) -> Result<BTreeMap<Slot, String>, String> {
    frontier(data, p)?
        .pop()
        .map(|b| b.parts)
        .ok_or("No eligible setup.".into())
}
#[derive(Debug, Clone)]
pub struct Build {
    pub parts: BTreeMap<Slot, String>,
    pub estimate: Estimate,
}
pub fn frontier(data: &Reference, p: &Profile) -> Result<Vec<Build>, String> {
    if let Some(id) = &p.preferred_sight {
        let sight = item(data, id)
            .filter(|part| fits(part, Slot::Optic))
            .ok_or("Your preferred sight is unavailable. Clear it in Customize.")?;
        evaluate(data, p, &BTreeMap::from([(Slot::Optic, sight.id.clone())]))?;
        if !sight.vendor.iter().any(|price| {
            price.tag.eq_ignore_ascii_case(&sight.id)
                && price.price.is_finite()
                && price.price >= 0.
        }) {
            return Err("Your preferred sight has no published exact purchase price. Choose another sight or Automatic in Customize.".into());
        }
        if p.lab_pins.contains(&Slot::Optic)
            && p.lab_parts
                .get(&Slot::Optic)
                .is_none_or(|kept| !kept.eq_ignore_ascii_case(id))
        {
            return Err("Your kept sight differs from your preferred sight. Release Keep or clear the preference in Customize.".into());
        }
    }
    if p.minimum_capacity > 0
        && !data.items.iter().any(|part| {
            fits(part, Slot::Magazine)
                && part
                    .magazine_capacity
                    .is_some_and(|n| n >= p.minimum_capacity)
                && part.eligible_by_level(p) == Some(true)
                && (!p.owned_only || owned(part, p))
        })
    {
        let future = data
            .items
            .iter()
            .filter(|part| {
                fits(part, Slot::Magazine)
                    && part
                        .magazine_capacity
                        .is_some_and(|n| n >= p.minimum_capacity)
            })
            .filter_map(|part| Some((part, part.unlock_track()?, part.unlock.as_ref()?)))
            .filter(|(_, track, unlock)| unlock.level > p.levels[track.index()])
            .min_by_key(|(_, track, unlock)| unlock.level - p.levels[track.index()]);
        return Err(
            if !p.owned_only
                && let Some((part, track, unlock)) = future
            {
                format!(
                    "{} requires {} level {}. Lower your {}-round minimum in Customize or update your levels.",
                    part.name,
                    track.label(),
                    unlock.level,
                    p.minimum_capacity
                )
            } else {
                format!(
                    "No eligible magazine meets your {}-round minimum and ownership filter. Review Customize or Settings.",
                    p.minimum_capacity
                )
            },
        );
    }
    let mut options = Vec::new();
    let active_slots = slots(data);
    for slot in &active_slots {
        let slot = *slot;
        if slot == Slot::Optic
            && let Some(id) = &p.preferred_sight
        {
            options.push(vec![Some(id.clone())]);
        } else if p.lab_pins.contains(&slot) {
            options.push(vec![p.lab_parts.get(&slot).cloned()]);
        } else {
            options.push(
                candidates(data, p, slot)
                    .into_iter()
                    .filter(|choice| {
                        if slot == Slot::Magazine
                            && p.minimum_capacity > 0
                            && choice
                                .id
                                .as_ref()
                                .and_then(|id| item(data, id))
                                .and_then(|part| part.magazine_capacity)
                                .is_none_or(|capacity| capacity < p.minimum_capacity)
                        {
                            return false;
                        }
                        choice.id.as_ref().is_none_or(|id| {
                            item(data, id).is_some_and(|part| {
                                part.vendor
                                    .iter()
                                    .any(|price| price.tag.eq_ignore_ascii_case(&part.id))
                            })
                        })
                    })
                    .map(|choice| choice.id)
                    .collect(),
            );
        }
    }
    // Reject incompatible, unavailable or unknown-price pinned selections before searching.
    let pinned = p
        .lab_parts
        .iter()
        .filter(|(slot, _)| p.lab_pins.contains(slot))
        .map(|(s, id)| (*s, id.clone()))
        .collect();
    let pinned_estimate = evaluate(data, p, &pinned)?;
    if !pinned_estimate.unknown_prices.is_empty() {
        return Err("A pinned item has no exact purchase price. Unpin it before requesting a budgeted suggestion.".into());
    }
    struct Search<'a> {
        data: &'a Reference,
        profile: &'a Profile,
        slots: &'a [Slot],
        options: &'a [Vec<Option<String>>],
        frontier: Vec<Build>,
        visited: usize,
    }
    impl Search<'_> {
        fn visit(
            &mut self,
            index: usize,
            parts: &mut BTreeMap<Slot, String>,
        ) -> Result<(), String> {
            self.visited += 1;
            if self.visited > 500_000 {
                return Err(
                    "This search exceeds the preview limit. Keep a few slots to narrow it down."
                        .into(),
                );
            }
            let Ok(estimate) = evaluate(self.data, self.profile, parts) else {
                return Ok(());
            };
            if !estimate.unknown_prices.is_empty() || estimate.cost > self.profile.budget as f64 {
                return Ok(());
            }
            if index == self.slots.len() {
                if preference_warning(self.data, self.profile, parts).is_some() {
                    return Ok(());
                }
                let score = estimate.score(self.profile);
                if self.frontier.iter().any(|b| {
                    b.estimate.cost <= estimate.cost && b.estimate.score(self.profile) >= score
                }) {
                    return Ok(());
                }
                self.frontier.retain(|b| {
                    !(estimate.cost <= b.estimate.cost && score >= b.estimate.score(self.profile))
                });
                self.frontier.push(Build {
                    parts: parts.clone(),
                    estimate,
                });
                return Ok(());
            }
            let slot = self.slots[index];
            let choices = self.options[index].clone();
            for choice in &choices {
                if let Some(id) = choice {
                    parts.insert(slot, id.clone());
                } else {
                    parts.remove(&slot);
                }
                self.visit(index + 1, parts)?;
            }
            parts.remove(&slot);
            Ok(())
        }
    }
    let mut search = Search {
        data,
        profile: p,
        slots: &active_slots,
        options: &options,
        frontier: vec![],
        visited: 0,
    };
    search.visit(0, &mut BTreeMap::new())?;
    if search.frontier.is_empty() {
        return Err("No suggestion fits your levels, budget, owned parts, kept slots and preferences. Review Customize or Settings.".into());
    }
    search
        .frontier
        .sort_by(|a, b| a.estimate.cost.total_cmp(&b.estimate.cost));
    Ok(search.frontier)
}

pub fn preference_warning(
    data: &Reference,
    p: &Profile,
    parts: &BTreeMap<Slot, String>,
) -> Option<String> {
    if let Some(preferred) = &p.preferred_sight
        && parts
            .get(&Slot::Optic)
            .is_none_or(|id| !id.eq_ignore_ascii_case(preferred))
    {
        return Some("This selection differs from your preferred sight. Choose an objective to update the suggestion.".into());
    }
    if p.minimum_capacity > 0
        && parts
            .get(&Slot::Magazine)
            .and_then(|id| item(data, id))
            .and_then(|part| part.magazine_capacity)
            .is_none_or(|capacity| capacity < p.minimum_capacity)
    {
        return Some(format!(
            "This selection does not confirm your {}-round minimum. Choose a magazine or adjust the minimum in Customize.",
            p.minimum_capacity
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    fn profile() -> Profile {
        Profile {
            levels: [99; 5],
            ..Profile::default()
        }
    }
    #[test]
    fn source_multipliers_compound_instead_of_adding_percentages() {
        let data = Reference::load();
        let p = profile();
        let grip = data
            .items
            .iter()
            .find(|i| i.name == "AFG Angled Foregrip")
            .unwrap();
        let sight = data
            .items
            .iter()
            .find(|i| i.name == "Kobra Reflex")
            .unwrap();
        let parts = BTreeMap::from([
            (Slot::Grip, grip.id.clone()),
            (Slot::Optic, sight.id.clone()),
        ]);
        let estimate = evaluate(&data, &p, &parts).unwrap();
        assert!((estimate.factors[2] - 0.7225).abs() < 0.00001);
        assert!((estimate.factors[0] - 0.94).abs() < 0.00001);
        assert!((estimate.factors[1] - 0.75).abs() < 0.00001);
    }
    #[test]
    fn unknown_price_is_not_free_and_is_not_budgeted() {
        let data = Reference::load();
        let mut p = profile();
        p.lab_parts
            .insert(Slot::Optic, "Id.Item.BasicRedDot".into());
        p.lab_pins.insert(Slot::Optic);
        let estimate = evaluate(&data, &p, &p.lab_parts).unwrap();
        assert_eq!(estimate.unknown_prices.len(), 1);
        assert!(
            estimate
                .budget_status(p.budget)
                .contains("cannot be verified")
        );
        assert!(suggest(&data, &p).is_err());
    }
    #[test]
    fn unsupported_stacking_and_wrong_slots_fail_closed() {
        let mut data = Reference::load();
        let p = profile();
        let part = data
            .items
            .iter_mut()
            .find(|i| i.name == "AFG Angled Foregrip")
            .unwrap();
        part.modifiers[0].op = "override".into();
        let id = part.id.clone();
        assert!(
            evaluate(&data, &p, &BTreeMap::from([(Slot::Grip, id.clone())]))
                .unwrap_err()
                .contains("unsupported")
        );
        assert!(evaluate(&data, &p, &BTreeMap::from([(Slot::Magazine, id)])).is_err());
    }
    #[test]
    fn suggestion_respects_pins_levels_ownership_and_budget() {
        let data = Reference::load();
        let mut p = profile();
        p.lab_parts.insert(Slot::Grip, "Id.Item.FGRP_015".into());
        p.lab_pins.insert(Slot::Grip);
        let parts = suggest(&data, &p).unwrap();
        assert_eq!(parts.get(&Slot::Grip), p.lab_parts.get(&Slot::Grip));
        assert!(evaluate(&data, &p, &parts).unwrap().cost <= p.budget as f64);
        p.budget = 500;
        assert!(suggest(&data, &p).is_err());
        p.budget = 2000;
        p.owned_only = true;
        assert!(suggest(&data, &p).is_err());
        p.owned.insert("id.item.fgrp_015".into());
        assert!(suggest(&data, &p).is_ok());
        p.levels[1] = 10;
        assert!(suggest(&data, &p).is_err());
    }
    #[test]
    fn invalid_pins_are_not_silently_removed() {
        let data = Reference::load();
        let mut p = profile();
        p.lab_parts
            .insert(Slot::Muzzle, "Id.Item.NoSuchMuzzle".into());
        p.lab_pins.insert(Slot::Muzzle);
        assert!(suggest(&data, &p).is_err());
    }
    #[test]
    fn four_slot_suggestions_include_verified_muzzle_and_stay_within_budget() {
        let data = Reference::load();
        let mut p = Profile {
            levels: [49, 15, 12, 5, 14],
            ..Profile::default()
        };
        let parts = suggest(&data, &p).unwrap();
        assert!(parts.contains_key(&Slot::Muzzle));
        let estimate = evaluate(&data, &p, &parts).unwrap();
        assert!(estimate.cost <= p.budget as f64);
        p.lab_parts = parts;
        p.lab_pins.insert(Slot::Muzzle);
        p.budget = 0;
        assert!(suggest(&data, &p).is_err());
    }
    #[test]
    fn magazine_capacity_and_sway_are_reported_without_optimizing_capacity() {
        let data = Reference::load();
        let p = profile();
        let mag = item(&data, "Id.Item.AK74MExtendedMagazine").unwrap();
        assert_eq!(mag.magazine_capacity, Some(60));
        let estimate = evaluate(
            &data,
            &p,
            &BTreeMap::from([(Slot::Magazine, mag.id.clone())]),
        )
        .unwrap();
        assert!((estimate.factors[5] - 1.21).abs() < 1e-8);
        assert!((estimate.factors[4] - 0.93).abs() < 1e-8);
    }
    #[test]
    fn preferred_sight_survives_objectives_without_bypassing_constraints() {
        let data = Reference::load();
        let mut p = profile();
        p.preferred_sight = Some("id.item.sght_009".into());
        for goal in crate::engine::Goal::ALL {
            p.goal = goal;
            let parts = suggest(&data, &p).unwrap();
            assert!(parts[&Slot::Optic].eq_ignore_ascii_case("Id.Item.SGHT_009"));
            assert!(evaluate(&data, &p, &parts).unwrap().cost <= p.budget as f64);
        }
        p.budget = 100;
        assert!(suggest(&data, &p).is_err());
        p.budget = 2000;
        p.owned_only = true;
        assert!(suggest(&data, &p).is_err());
        p.owned.insert("Id.Item.SGHT_009".into());
        assert!(suggest(&data, &p).is_ok());
        p.levels[0] = 0;
        assert!(suggest(&data, &p).is_err());
    }
    #[test]
    fn magazine_minimum_requires_known_capacity_and_checks_kept_magazines() {
        let mut data = Reference::load();
        let mut p = profile();
        p.minimum_capacity = 60;
        let parts = suggest(&data, &p).unwrap();
        assert_eq!(
            item(&data, &parts[&Slot::Magazine])
                .unwrap()
                .magazine_capacity,
            Some(60)
        );
        p.lab_pins.insert(Slot::Magazine); // Kept Unchanged has unknown capacity.
        assert!(suggest(&data, &p).is_err());
        p.lab_parts
            .insert(Slot::Magazine, "Id.Item.AK74MMagazine".into());
        assert!(suggest(&data, &p).is_err());
        p.lab_pins.clear();
        p.levels[1] = 11;
        assert!(suggest(&data, &p).unwrap_err().contains("Assault level 25"));
        p.levels[1] = 99;
        for part in &mut data.items {
            part.magazine_capacity = None;
        }
        assert!(suggest(&data, &p).is_err());
    }
    #[test]
    fn sight_preference_rejects_conflicts_and_missing_prices() {
        let data = Reference::load();
        let mut p = profile();
        p.preferred_sight = Some("Id.Item.BasicRedDot".into());
        assert!(suggest(&data, &p).unwrap_err().contains("purchase price"));
        p.preferred_sight = Some("Id.Item.SGHT_009".into());
        p.lab_pins.insert(Slot::Optic);
        assert!(suggest(&data, &p).unwrap_err().contains("kept sight"));
        p.lab_parts.insert(Slot::Optic, "ID.ITEM.SGHT_009".into());
        assert!(suggest(&data, &p).is_ok());
    }
    #[test]
    fn complete_catalog_searches_do_not_cross_weapon_compatibility_or_budget() {
        let catalog = crate::catalog::Catalog::bundled();
        let p = Profile {
            levels: [100; 5],
            budget: 100_000,
            ..Profile::default()
        };
        for reference in &catalog.weapons {
            let parts = suggest(reference, &p)
                .unwrap_or_else(|error| panic!("{}: {error}", reference.weapon.name));
            let estimate = evaluate(reference, &p, &parts).unwrap();
            assert!(estimate.unknown_prices.is_empty());
            assert!(estimate.cost <= p.budget as f64);
            assert!(parts.keys().all(|s| reference.slots.contains(s)));
        }
    }
}
