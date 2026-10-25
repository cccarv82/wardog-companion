//! Shared domain and backward-compatible profile data.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Track {
    Wardog,
    Assault,
    Medic,
    Recon,
    Support,
}
impl Track {
    pub const ALL: [Self; 5] = [
        Self::Wardog,
        Self::Assault,
        Self::Medic,
        Self::Recon,
        Self::Support,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Wardog => "Overall",
            Self::Assault => "Assault",
            Self::Medic => "Medic",
            Self::Recon => "Recon",
            Self::Support => "Support",
        }
    }
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|track| *track == self).unwrap()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Slot {
    Muzzle,
    Grip,
    Optic,
    Magazine,
    Barrel,
    Stock,
    PistolGrip,
    Handguard,
}
impl Slot {
    pub const ALL: [Self; 8] = [
        Self::Muzzle,
        Self::Optic,
        Self::Grip,
        Self::Magazine,
        Self::Barrel,
        Self::Stock,
        Self::PistolGrip,
        Self::Handguard,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Muzzle => "Muzzle",
            Self::Grip => "Foregrip",
            Self::Optic => "Sight",
            Self::Magazine => "Magazine",
            Self::Barrel => "Barrel",
            Self::Stock => "Stock",
            Self::PistolGrip => "Pistol grip",
            Self::Handguard => "Handguard",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Goal {
    Control,
    Agility,
    #[default]
    Balanced,
}
impl Goal {
    pub const ALL: [Self; 3] = [Self::Control, Self::Balanced, Self::Agility];
    pub fn label(self) -> &'static str {
        match self {
            Self::Control => "Control",
            Self::Agility => "Agility",
            Self::Balanced => "Balanced",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Control => "Prioritizes recoil control.",
            Self::Agility => "Prioritizes shorter aim time.",
            Self::Balanced => "Balances recoil control and aim time.",
        }
    }
    pub fn weights(self) -> [f32; 3] {
        match self {
            Self::Control => [0.55, 0.35, 0.10],
            Self::Agility => [0.15, 0.15, 0.70],
            Self::Balanced => [0.35, 0.30, 0.35],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub levels: [u8; 5],
    pub owned: BTreeSet<String>,
    pub owned_only: bool,
    pub budget: u32,
    pub goal: Goal,
    // Legacy demo IDs are retained when saving an older profile, but do not affect the live model.
    pub locks: BTreeMap<Slot, String>,
    pub lab_parts: BTreeMap<Slot, String>,
    pub lab_pins: BTreeSet<Slot>,
    pub levels_confirmed: bool,
    pub preferred_sight: Option<String>,
    pub minimum_capacity: u32,
    pub weapon_id: String,
    pub class_checks: BTreeMap<String, ClassChecks>,
    pub unlocked: BTreeSet<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ClassChecks {
    pub fits: bool,
    pub requirements: bool,
    pub panel: bool,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            levels: [0; 5],
            owned: BTreeSet::new(),
            owned_only: false,
            budget: 2000,
            goal: Goal::Balanced,
            locks: BTreeMap::new(),
            lab_parts: BTreeMap::new(),
            lab_pins: BTreeSet::new(),
            levels_confirmed: false,
            preferred_sight: None,
            minimum_capacity: 0,
            weapon_id: "Id.Item.AK74M".into(),
            class_checks: BTreeMap::new(),
            unlocked: BTreeSet::new(),
        }
    }
}
