//! Compare source versions under the same present player constraints; never silently replace parts.
use crate::{
    catalog::Catalog,
    engine::Profile,
    lab,
    reference::{ReferenceItem, Weapon},
};
#[derive(Debug)]
pub struct Report {
    pub title: String,
    pub lines: Vec<String>,
    pub recommendation: String,
    pub relevant: bool,
    pub versions: String,
}
fn value(v: Option<f64>) -> String {
    v.map(|n| format!("{n:.3}"))
        .unwrap_or_else(|| "unknown".into())
}
fn money(v: Option<f64>) -> String {
    v.map(|n| format!("${n:.2}"))
        .unwrap_or_else(|| "unknown".into())
}
fn price(p: &ReferenceItem) -> Option<f64> {
    p.vendor
        .iter()
        .filter(|v| v.tag.eq_ignore_ascii_case(&p.id))
        .map(|v| v.price)
        .min_by(f64::total_cmp)
}
fn weapon_changes(old: &Weapon, new: &Weapon, lines: &mut Vec<String>) {
    for (label, a, b) in [
        ("Damage / hit", old.damage_per_hit, new.damage_per_hit),
        ("RPM", old.rpm, new.rpm),
        ("Base aim time (s)", old.ads_time, new.ads_time),
        ("Accuracy rating", old.accuracy, new.accuracy),
        ("Base recoil index", old.recoil_index, new.recoil_index),
    ] {
        if a != b {
            lines.push(format!("{label}: {} → {}", value(a), value(b)));
        }
    }
    if old.recoil_pattern != new.recoil_pattern {
        lines.push("Weapon recoil pattern changed in the source.".into());
    }
    if old.falloff != new.falloff {
        lines.push("Weapon damage falloff changed; range results may differ.".into());
    }
    if old.unlock != new.unlock
        || old.unlock_free != new.unlock_free
        || old.test_record != new.test_record
    {
        lines.push("Weapon unlock requirements or availability changed.".into());
    }
    if old.projectile != new.projectile
        || old.comparable != new.comparable
        || old.class_name != new.class_name
        || old.fire_modes != new.fire_modes
    {
        lines.push("Weapon projectile, class, fire modes or comparison support changed.".into());
    }
}
pub fn compare(old: &Catalog, new: &Catalog, p: &Profile) -> Report {
    let mut lines = vec![];
    let mut report = Report {
        title: String::new(),
        lines: vec![],
        recommendation: String::new(),
        relevant: false,
        versions: format!(
            "Build {} → {} · {} → {}",
            old.game_build,
            new.game_build,
            &old.source_commit[..8],
            &new.source_commit[..8]
        ),
    };
    let (Some(before), Some(after)) = (old.weapon(&p.weapon_id), new.weapon(&p.weapon_id)) else {
        report.title = "Weapon coverage changed".into();
        report.lines.push(
            "Selected weapon is absent from one dataset. No complete comparison is possible."
                .into(),
        );
        report.relevant = true;
        return report;
    };
    weapon_changes(&before.weapon, &after.weapon, &mut lines);
    let mut seen = std::collections::BTreeSet::new();
    for id in p.lab_parts.values() {
        if !seen.insert(id.to_lowercase()) {
            continue;
        }
        let a = lab::item(before, id);
        let b = lab::item(after, id);
        match(a,b){
        (Some(a),Some(b))=>{
            if price(a)!=price(b){lines.push(format!("{} purchase: {} → {}",b.name,money(price(a)),money(price(b))));}
            if a.unlock!=b.unlock||a.unlock_free!=b.unlock_free{lines.push(format!("{} unlock requirements/fee changed.",b.name));}
            if a.modifiers!=b.modifiers||a.character_modifiers!=b.character_modifiers{lines.push(format!("{} attachment effects changed. Modeled effects are recalculated; other effects still need review.",b.name));}
            if a.weight!=b.weight||a.magazine_capacity!=b.magazine_capacity{lines.push(format!("{} weight or magazine capacity changed.",b.name));}
            if a.compatible_slots!=b.compatible_slots||a.conditional!=b.conditional||a.developer!=b.developer||a.is_default!=b.is_default{lines.push(format!("{} fit, default status or calculation support changed.",b.name));}
        },(Some(a),None)=>lines.push(format!("{} is no longer linked to this weapon. This may be a coverage change, not a game nerf.",a.name)),(None,Some(b))=>lines.push(format!("{} now has a source-linked fit for this weapon.",b.name)),(None,None)=>lines.push(format!("Saved part {id} is absent from both datasets; its impact is unknown.")),}
    }
    let available = after
        .items
        .iter()
        .filter(|i| {
            i.eligible_by_level(p) == Some(true)
                && (!p.owned_only || lab::owned(i, p))
                && lab::item(before, &i.id).is_none_or(|a| a.eligible_by_level(p) != Some(true))
        })
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>();
    if !available.is_empty() {
        lines.push(format!(
            "Now level/ownership-eligible in this dataset: {}{}.",
            available
                .iter()
                .take(3)
                .copied()
                .collect::<Vec<_>>()
                .join(", "),
            if available.len() > 3 {
                format!(" and {} more", available.len() - 3)
            } else {
                String::new()
            }
        ));
    }
    // Compare algorithm suggestions across datasets, not a player's saved build against an optimum.
    let old_suggestion = lab::suggest(before, p);
    let new_suggestion = lab::suggest(after, p);
    report.recommendation=match(old_suggestion,new_suggestion){
        (Ok(a),Ok(b)) if a==b=>"Recommended part choices are unchanged under your current levels, goal, budget and kept slots.".into(),
        (Ok(_),Ok(_))=>{lines.push("Recommended part choices changed for your current constraints.".into());"Review the new spending options before choosing any replacements. Your saved parts were kept.".into()},
        (_,Err(e))=>{lines.push("A complete suggestion cannot be calculated for the new data.".into());format!("Your saved parts were kept. {e}")},
        (Err(_),Ok(_))=>{lines.push("A supported suggestion is now available.".into());"Open spending options to review it. Your saved parts were kept.".into()},
    };
    if let Err(e) = lab::evaluate(after, p, &p.lab_parts) {
        lines.push(format!("Saved setup needs review: {e}"));
    }
    report.relevant = !lines.is_empty();
    report.title = if report.relevant {
        "This dataset affects your setup."
    } else {
        "No modeled change for your setup."
    }
    .into();
    report.lines = lines;
    report
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Slot;
    fn changed(edit: impl FnOnce(&mut serde_json::Value)) -> Catalog {
        let mut v: serde_json::Value =
            serde_json::from_str(include_str!("../data/catalog.json")).unwrap();
        edit(&mut v);
        Catalog::parse(&v.to_string()).unwrap()
    }
    fn profile() -> Profile {
        let mut p = Profile {
            levels: [99; 5],
            ..Profile::default()
        };
        let d = Catalog::bundled();
        p.lab_parts = lab::suggest(d.weapon(&p.weapon_id).unwrap(), &p).unwrap();
        p
    }
    #[test]
    fn irrelevant_metadata_and_other_weapons_do_not_generate_fake_alerts() {
        let d = Catalog::bundled();
        let p = profile();
        let next = changed(|v| {
            v["imported_on"] = "different".into();
            let r = v["weapons"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|r| r["weapon"]["id"] == "Id.Item.M4")
                .unwrap();
            r["weapon"]["rpm"] = serde_json::json!(800.);
        });
        let r = compare(&d, &next, &p);
        assert!(!r.relevant);
        assert!(r.lines.is_empty());
    }
    #[test]
    fn weapon_changes_and_selected_modifier_changes_are_reported_without_mutation() {
        let d = Catalog::bundled();
        let p = profile();
        let original = p.lab_parts.clone();
        let selected = p.lab_parts.values().next().unwrap().clone();
        let next = changed(|v| {
            let r = v["weapons"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|r| r["weapon"]["id"] == "Id.Item.AK74M")
                .unwrap();
            r["weapon"]["damage_per_hit"] = serde_json::json!(30.);
            let i = r["items"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|i| i["id"] == selected)
                .unwrap();
            i["character_modifiers"] = serde_json::json!([{"attribute":"Character.AimSwayScaleMultiplier","op":"multiplyCompound","value":0.5}]);
        });
        let r = compare(&d, &next, &p);
        assert!(r.relevant);
        assert!(r.lines.iter().any(|s| s.contains("Damage / hit")));
        assert!(r.lines.iter().any(|s| s.contains("effects changed")));
        assert_eq!(p.lab_parts, original);
    }
    #[test]
    fn lost_mounts_are_coverage_changes_and_never_automatic_replacements() {
        let d = Catalog::bundled();
        let mut p = profile();
        let id = d
            .weapon(&p.weapon_id)
            .unwrap()
            .items
            .iter()
            .find(|i| i.compatible_slots.contains(&Slot::Muzzle))
            .unwrap()
            .id
            .clone();
        p.lab_parts = std::collections::BTreeMap::from([(Slot::Muzzle, id.clone())]);
        let next = changed(|v| {
            for r in v["weapons"].as_array_mut().unwrap() {
                if r["weapon"]["id"] == "Id.Item.AK74M" {
                    r["items"].as_array_mut().unwrap().retain(|i| i["id"] != id);
                }
            }
        });
        let r = compare(&d, &next, &p);
        assert!(r.lines.iter().any(|s| s.contains("coverage change")));
        assert_eq!(p.lab_parts[&Slot::Muzzle], id);
    }
}
