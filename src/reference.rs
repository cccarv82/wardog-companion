//! Pinned community records with explicit compatibility evidence.
use crate::engine::{Profile, Slot, Track};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Reference {
    pub schema_version: u32,
    pub game_build: String,
    pub local_build_at_import: String,
    pub imported_on: String,
    pub source_commit: String,
    pub source_url: String,
    pub weapon: Weapon,
    pub items: Vec<ReferenceItem>,
    #[serde(default)]
    pub slots: Vec<Slot>,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Weapon {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub class_name: String,
    #[serde(default)]
    pub test_record: bool,
    pub unlock: Option<Unlock>,
    pub rpm: Option<f64>,
    pub accuracy: Option<f64>,
    pub ads_time: Option<f64>,
    pub recoil_index: Option<f64>,
    pub damage_per_hit: Option<f64>,
    pub projectile: Option<String>,
    #[serde(default)]
    pub comparable: bool,
    #[serde(default)]
    pub fire_modes: Vec<String>,
    #[serde(default)]
    pub damage_rows: Vec<DamageRow>,
    #[serde(default)]
    pub unlock_free: bool,
    #[serde(default)]
    pub falloff: Vec<Falloff>,
    #[serde(default)]
    pub recoil_pattern: Vec<RecoilPoint>,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Falloff {
    pub distance: f64,
    pub multiplier: f64,
    #[serde(default = "linear")]
    pub interpolation: String,
}
fn linear() -> String {
    "Linear".into()
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RecoilPoint {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DamageRow {
    pub projectile: String,
    pub damage: Option<f64>,
    pub raw_dps: Option<f64>,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ReferenceItem {
    pub id: String,
    pub name: String,
    pub slot: String,
    pub is_default: bool,
    pub unlock: Option<Unlock>,
    pub weight: Option<f32>,
    pub vendor: Vec<Price>,
    pub modifiers: Vec<Modifier>,
    #[serde(default)]
    pub character_modifiers: Vec<Modifier>,
    pub magazine_capacity: Option<u32>,
    pub compatibility_source: Option<String>,
    #[serde(default)]
    pub compatible_slots: Vec<Slot>,
    #[serde(default)]
    pub conditional: bool,
    #[serde(default)]
    pub unlock_free: bool,
    #[serde(default)]
    pub developer: bool,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Unlock {
    pub role: String,
    pub level: u8,
    pub cost: u32,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Price {
    pub tag: String,
    pub price: f64,
}
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Modifier {
    pub attribute: String,
    pub op: String,
    pub value: f64,
}
impl Reference {
    #[cfg(test)]
    pub fn load() -> Self {
        let data: Self = serde_json::from_str(include_str!("../data/ak74-reference.json"))
            .expect("Valid built-in reference");
        assert_eq!(data.schema_version, 1);
        data
    }
}
impl ReferenceItem {
    pub fn unlock_track(&self) -> Option<Track> {
        match self.unlock.as_ref()?.role.as_str() {
            "Wardog" => Some(Track::Wardog),
            "Infantry" => Some(Track::Assault),
            "Medic" => Some(Track::Medic),
            "Recon" => Some(Track::Recon),
            "Support" => Some(Track::Support),
            _ => None,
        }
    }
    pub fn eligible_by_level(&self, p: &Profile) -> Option<bool> {
        if self.developer {
            return Some(false);
        }
        let Some(unlock) = &self.unlock else {
            return (self.is_default || self.unlock_free).then_some(true);
        };
        let track = self.unlock_track()?;
        Some(p.levels[track.index()] >= unlock.level)
    }
    pub fn slot_label(&self) -> &str {
        match self.slot.as_str() {
            "Sight" => "SIGHT",
            "Underbarrel" => "FOREGRIP / UNDERBARREL",
            "Magazine" => "MAGAZINE",
            "Muzzle" => "MUZZLE",
            _ => &self.slot,
        }
    }
    pub fn price_text(&self) -> String {
        let Some(first) = self.vendor.iter().map(|v| v.price).min_by(f64::total_cmp) else {
            return "Purchase price not published in this source".into();
        };
        let last = self
            .vendor
            .iter()
            .map(|v| v.price)
            .max_by(f64::total_cmp)
            .unwrap();
        let variants = self
            .vendor
            .iter()
            .filter(|v| !v.tag.eq_ignore_ascii_case(&self.id))
            .count();
        if first == last {
            format!("Recorded purchase price: ${first:.2}")
        } else {
            format!(
                "Recorded purchase price: ${first:.2}–${last:.2} · {variants} variants; unlock not included"
            )
        }
    }
}
impl Modifier {
    pub fn label(&self) -> &str {
        match self.attribute.as_str() {
            "Weapon.AimTime" => "Aim time",
            "Weapon.VerticalRecoilMultiplier" => "Vertical recoil",
            "Weapon.HorizontalRecoilMultiplier" => "Horizontal recoil",
            "Weapon.SpreadMultiplier" => "Spread",
            "Weapon.EquipSpeedMultiplier" => "Equip speed",
            "Character.AimSwayScaleMultiplier" => "Aim sway",
            "Weapon.AdsZoomMultiplier" => "Sight magnification",
            _ => &self.attribute,
        }
    }
    pub fn description(&self) -> String {
        match self.op.as_str() {
            "multiply" | "multiplyCompound" => format!(
                "{}: ×{:.3} ({:+.1}%) [{}]",
                self.label(),
                self.value,
                (self.value - 1.) * 100.,
                self.op
            ),
            "add" => format!("{}: adds {:+.3} [add]", self.label(), self.value),
            "override" => format!("{}: sets {:.3} [override]", self.label(), self.value),
            _ => format!("{}: unknown operation ({})", self.label(), self.op),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_is_consistent_and_has_no_duplicate_ids() {
        let data = Reference::load();
        assert_eq!(data.game_build, data.local_build_at_import);
        assert_eq!(data.weapon.id, "Id.Item.AK74M");
        assert_eq!(data.items.len(), 35);
        let ids: std::collections::BTreeSet<_> =
            data.items.iter().map(|i| i.id.to_lowercase()).collect();
        assert_eq!(ids.len(), data.items.len());
        assert!(
            data.items
                .iter()
                .flat_map(|i| &i.modifiers)
                .all(|m| m.value.is_finite()
                    && ["multiply", "multiplyCompound", "add", "override"]
                        .contains(&m.op.as_str()))
        );
    }
    #[test]
    fn missing_requirements_are_unknown_and_levels_do_not_imply_purchase() {
        let data = Reference::load();
        let mut p = Profile::default();
        let dot = data
            .items
            .iter()
            .find(|i| i.id == "Id.Item.BasicRedDot")
            .unwrap();
        assert_eq!(dot.eligible_by_level(&p), Some(false));
        p.levels[0] = 17;
        assert_eq!(dot.eligible_by_level(&p), Some(true));
        assert!(dot.price_text().contains("not published"));
        let unknown = data.items.iter().find(|i| i.name == "MMGL Sight").unwrap();
        assert_eq!(unknown.eligible_by_level(&p), None);
    }
    #[test]
    fn muzzles_have_explicit_compatibility_evidence_and_source_values() {
        let data = Reference::load();
        let muzzles: Vec<_> = data.items.iter().filter(|i| i.slot == "Muzzle").collect();
        assert_eq!(muzzles.len(), 5);
        assert!(
            muzzles
                .iter()
                .all(|part| part.compatibility_source.as_deref()
                    == Some("https://wardogsweapons.com/api/weapon/ak74/attachments/MUZZLE"))
        );
        let brake = muzzles.iter().find(|i| i.name == "Ballista Brake").unwrap();
        assert_eq!(brake.unlock.as_ref().unwrap().level, 12);
        assert!(
            brake
                .modifiers
                .iter()
                .any(|m| m.attribute == "Weapon.HorizontalRecoilMultiplier"
                    && (m.value - 0.8).abs() < 1e-8)
        );
    }
}
