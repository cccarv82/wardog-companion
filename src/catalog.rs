use crate::{
    engine::Slot,
    reference::{Reference, ReferenceItem},
};
use serde::Deserialize;
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Debug, PartialEq, Deserialize)]
pub struct Catalog {
    pub schema_version: u32,
    pub game_build: String,
    pub imported_on: String,
    pub source_commit: String,
    pub source_url: String,
    pub sources: Vec<Source>,
    pub weapons: Vec<Reference>,
    pub all_attachments: Vec<ReferenceItem>,
    pub coverage_notes: Vec<Note>,
}
#[derive(Debug, PartialEq, Deserialize)]
pub struct Source {
    pub path: String,
    pub sha256: String,
}
#[derive(Debug, PartialEq, Deserialize)]
pub struct Note {
    pub weapon: String,
    pub note: String,
}
impl Catalog {
    pub fn bundled() -> Self {
        Self::parse(include_str!("../data/catalog.json")).expect("Validated built-in catalogue")
    }
    pub fn parse(json: &str) -> Result<Self, String> {
        let data: Self = serde_json::from_str(json).map_err(|e| format!("Invalid dataset: {e}"))?;
        if data.schema_version != 2
            || data.weapons.is_empty()
            || data.weapons.len() > 1000
            || data.all_attachments.len() > 10000
            || data.game_build.is_empty()
            || data.source_commit.len() != 40
            || data.sources.is_empty()
        {
            return Err("Unsupported or empty dataset.".into());
        }
        let mut ids = BTreeSet::new();
        let mut all_parts = BTreeSet::new();
        let validate_part = |part: &ReferenceItem| -> Result<(), String> {
            if part.id.is_empty()
                || part.weight.is_some_and(|v| !v.is_finite() || v < 0.)
                || part
                    .vendor
                    .iter()
                    .any(|p| !p.price.is_finite() || p.price < 0.)
                || part
                    .modifiers
                    .iter()
                    .chain(&part.character_modifiers)
                    .any(|m| {
                        !m.value.is_finite()
                            || !["multiplyCompound", "multiply", "add", "override"]
                                .contains(&m.op.as_str())
                    })
            {
                return Err("Invalid attachment value or operation.".into());
            }
            Ok(())
        };
        for part in &data.all_attachments {
            validate_part(part)?;
            if !all_parts.insert(part.id.to_lowercase()) {
                return Err("Duplicate attachment ID.".into());
            }
        }
        for reference in &data.weapons {
            if reference.schema_version != 1
                || reference.game_build != data.game_build
                || !ids.insert(reference.weapon.id.to_lowercase())
                || reference.source_commit != data.source_commit
            {
                return Err("Mixed builds or duplicate weapon IDs.".into());
            }
            let mut parts = BTreeSet::new();
            for item in &reference.items {
                validate_part(item)?;
                if !all_parts.contains(&item.id.to_lowercase())
                    || !parts.insert(item.id.to_lowercase())
                    || item.compatible_slots.is_empty()
                    || item
                        .compatible_slots
                        .iter()
                        .any(|s| !reference.slots.contains(s))
                {
                    return Err("Broken attachment compatibility link.".into());
                }
            }
            if reference.slots.iter().any(|s| !Slot::ALL.contains(s)) {
                return Err("Unknown slot.".into());
            }
            let w = &reference.weapon;
            if w.unlock_free && (w.unlock.is_some() || w.test_record) {
                return Err("Contradictory weapon unlock metadata.".into());
            }
            if w.falloff.iter().any(|p| {
                !p.distance.is_finite()
                    || p.distance < 0.
                    || !p.multiplier.is_finite()
                    || p.multiplier < 0.
            }) || w.falloff.windows(2).any(|p| p[0].distance >= p[1].distance)
                || w.recoil_pattern
                    .iter()
                    .any(|p| !p.x.is_finite() || !p.y.is_finite())
            {
                return Err("Invalid falloff/recoil curve.".into());
            }
            if w.damage_rows.iter().any(|r| {
                [r.damage, r.raw_dps]
                    .into_iter()
                    .flatten()
                    .any(|v| !v.is_finite() || v < 0.)
            }) {
                return Err("Invalid damage row.".into());
            }
            for number in [
                w.rpm,
                w.damage_per_hit,
                w.recoil_index,
                w.accuracy,
                w.ads_time,
            ]
            .into_iter()
            .flatten()
            {
                if !number.is_finite() || number < 0. {
                    return Err("Invalid weapon number.".into());
                }
            }
        }
        for source in &data.sources {
            if source.path.is_empty()
                || source.sha256.len() != 64
                || !source.sha256.bytes().all(|c| c.is_ascii_hexdigit())
            {
                return Err("Invalid source manifest.".into());
            }
        }
        Ok(data)
    }
    pub fn path() -> PathBuf {
        std::env::var_os("WARDOG_DATA_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::current_exe()
                    .unwrap_or_default()
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("Data/catalog.json")
            })
    }
    pub fn external() -> Result<Self, String> {
        let path = Self::path();
        let metadata = std::fs::metadata(&path).map_err(|e| format!("Dataset unavailable: {e}"))?;
        if metadata.len() > 15_000_000 {
            return Err("Dataset exceeds size limit.".into());
        }
        Self::parse(&std::fs::read_to_string(path).map_err(|e| e.to_string())?)
    }
    pub fn weapon(&self, id: &str) -> Option<&Reference> {
        self.weapons
            .iter()
            .find(|r| r.weapon.id.eq_ignore_ascii_case(id))
    }
    pub fn classes(&self) -> Vec<String> {
        self.weapons
            .iter()
            .map(|r| r.weapon.class_name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_source_catalog_preserves_counts_and_has_closed_links() {
        let data = Catalog::bundled();
        assert_eq!(data.weapons.len(), 40);
        assert_eq!(data.all_attachments.len(), 298);
        assert!(data.weapon("id.item.m4").is_some());
        assert!(
            data.weapons
                .iter()
                .filter(|r| !r.weapon.test_record)
                .count()
                > 30
        );
    }
    #[test]
    fn invalid_reload_is_rejected_before_replacing_active_data() {
        let mut value: serde_json::Value =
            serde_json::from_str(include_str!("../data/catalog.json")).unwrap();
        value["weapons"][0]["game_build"] = "wrong".into();
        assert!(Catalog::parse(&value.to_string()).is_err());
        assert!(Catalog::parse("{}").is_err());
    }
}
