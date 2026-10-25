//! Comparisons and progression use the same constraints and composition as suggestions.
use crate::{
    engine::{Profile, Slot, Track},
    lab,
    reference::Reference,
};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Change {
    pub index: usize,
    pub percent: f64,
}
impl Change {
    pub fn beneficial(&self) -> bool {
        if self.index == 4 {
            self.percent > 0.
        } else {
            self.percent < 0.
        }
    }
}
pub struct Comparison {
    pub before: Option<lab::Estimate>,
    pub after: lab::Estimate,
    pub changes: Vec<Change>,
}
pub fn changes(before: &lab::Estimate, after: &lab::Estimate) -> Vec<Change> {
    before
        .factors
        .iter()
        .zip(after.factors)
        .enumerate()
        .filter_map(|(index, (old, new))| {
            let percent = (new / old - 1.) * 100.;
            (percent.abs() >= 0.005).then_some(Change { index, percent })
        })
        .collect()
}
pub fn swapped(p: &Profile, slot: Slot, choice: &lab::Choice) -> BTreeMap<Slot, String> {
    let mut parts = p.lab_parts.clone();
    if let Some(id) = &choice.id {
        parts.insert(slot, id.clone());
    } else {
        parts.remove(&slot);
    }
    parts
}
pub fn compare(
    data: &Reference,
    p: &Profile,
    slot: Slot,
    choice: &lab::Choice,
) -> Result<Comparison, String> {
    let after = lab::evaluate(data, p, &swapped(p, slot, choice))?;
    let before = lab::evaluate(data, p, &p.lab_parts).ok();
    let changes = before
        .as_ref()
        .map(|old| changes(old, &after))
        .unwrap_or_default();
    Ok(Comparison {
        before,
        after,
        changes,
    })
}

pub struct Upgrade {
    pub name: String,
    pub track: Track,
    pub level: u8,
    pub unlock_fee: u32,
    pub cost: f64,
    pub changes: Vec<Change>,
}
pub fn next_upgrade(data: &Reference, p: &Profile) -> Option<Upgrade> {
    // Compare with the best currently eligible setup, not an intentionally weaker manual build.
    // Future unowned parts cannot be recommended while the ownership filter is active.
    if p.owned_only {
        return None;
    }
    let baseline = lab::evaluate(data, p, &lab::suggest(data, p).ok()?).ok()?;
    let mut best: Option<(u8, f64, Upgrade)> = None;
    let mut thresholds = std::collections::BTreeSet::new();
    for part in &data.items {
        let Some(track) = part.unlock_track() else {
            continue;
        };
        let unlock = part.unlock.as_ref()?;
        if unlock.level > p.levels[track.index()] && unlock.level <= 100 {
            thresholds.insert((track.index(), unlock.level));
        }
    }
    for (track_index, level) in thresholds {
        let track = Track::ALL[track_index];
        let distance = level - p.levels[track_index];
        if best.as_ref().is_some_and(|(d, _, _)| distance > *d) {
            continue;
        }
        let mut future = p.clone();
        future.levels[track_index] = level;
        let Ok(parts) = lab::suggest(data, &future) else {
            continue;
        };
        let Ok(estimate) = lab::evaluate(data, &future, &parts) else {
            continue;
        };
        let gain = estimate.score(p) - baseline.score(p);
        // At least half a heuristic preference point: avoid advertising negligible upgrades.
        if gain < 0.5 {
            continue;
        }
        let Some(part) = parts
            .values()
            .filter_map(|id| lab::item(data, id))
            .find(|part| {
                part.eligible_by_level(p) == Some(false)
                    && part.unlock_track() == Some(track)
                    && part.unlock.as_ref().is_some_and(|u| u.level == level)
            })
        else {
            continue;
        };
        let upgrade = Upgrade {
            name: part.name.clone(),
            track,
            level,
            unlock_fee: part.unlock.as_ref()?.cost,
            cost: estimate.cost,
            changes: changes(&baseline, &estimate),
        };
        if best
            .as_ref()
            .is_none_or(|(d, g, _)| distance < *d || (distance == *d && gain > *g))
        {
            best = Some((distance, gain, upgrade));
        }
    }
    best.map(|(_, _, upgrade)| upgrade)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Goal;
    #[test]
    fn swap_compares_whole_compounded_setup_without_mutating_profile() {
        let data = Reference::load();
        let mut p = Profile {
            levels: [99; 5],
            ..Profile::default()
        };
        p.lab_parts.insert(Slot::Optic, "Id.Item.SGHT_009".into());
        p.lab_parts.insert(Slot::Grip, "Id.Item.FGP_03".into());
        let original = p.lab_parts.clone();
        let candidate = lab::Choice::item(lab::item(&data, "Id.Item.FGRP_015").unwrap());
        let preview = compare(&data, &p, Slot::Grip, &candidate).unwrap();
        assert_eq!(p.lab_parts, original);
        let aim = preview.changes.iter().find(|c| c.index == 2).unwrap();
        assert!((aim.percent - 20.).abs() < 1e-8);
        assert!(!aim.beneficial());
        assert!((preview.after.cost - preview.before.unwrap().cost + 260.).abs() < 1e-8);
    }
    #[test]
    fn comparisons_reject_locked_or_unowned_parts_and_keep_unknown_cost_unknown() {
        let data = Reference::load();
        let mut p = Profile::default();
        let sight = lab::Choice::item(lab::item(&data, "Id.Item.BasicRedDot").unwrap());
        assert!(compare(&data, &p, Slot::Optic, &sight).is_err());
        p.levels = [99; 5];
        assert_eq!(
            compare(&data, &p, Slot::Optic, &sight)
                .unwrap()
                .after
                .unknown_prices
                .len(),
            1
        );
        p.owned_only = true;
        assert!(compare(&data, &p, Slot::Optic, &sight).is_err());
        p.owned.insert("id.item.basicreddot".into());
        assert!(compare(&data, &p, Slot::Optic, &sight).is_ok());
    }
    #[test]
    fn progression_selects_useful_future_unlock_and_respects_constraints() {
        let data = Reference::load();
        let mut p = Profile {
            levels: [49, 15, 12, 5, 14],
            goal: Goal::Control,
            ..Profile::default()
        };
        let next = next_upgrade(&data, &p).expect("CQB brake is a useful next-level unlock");
        assert_eq!(next.track, Track::Assault);
        assert_eq!(next.level, 16);
        assert_eq!(next.name, "CQB 74 Brake");
        assert!(next.cost <= p.budget as f64);
        p.budget = 0;
        assert!(next_upgrade(&data, &p).is_none());
        p.budget = 2000;
        p.lab_pins.insert(Slot::Muzzle);
        assert!(next_upgrade(&data, &p).is_none_or(|u| u.name != "CQB 74 Brake"));
        p.owned_only = true;
        assert!(next_upgrade(&data, &p).is_none());
        p.owned_only = false;
        p.levels = [100; 5];
        assert!(next_upgrade(&data, &p).is_none());
    }
}
