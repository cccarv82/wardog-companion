//! Explicit recommendation policies, not measured optical or combat performance.
use crate::{
    engine::{Engagement, Profile, Slot},
    lab,
    reference::{Reference, ReferenceItem},
};
use std::collections::BTreeMap;

pub const MODEL: u8 = 2;

pub fn engagement(data: &Reference, p: &Profile) -> Engagement {
    if p.engagement != Engagement::Automatic {
        return p.engagement;
    }
    match data.weapon.class_name.as_str() {
        "Snipers" | "DMRs" => Engagement::Long,
        "Assault rifles" | "LMGs" => Engagement::Medium,
        _ => Engagement::Close,
    }
}

pub fn zoom_band(data: &Reference, p: &Profile) -> (f64, f64) {
    match engagement(data, p) {
        Engagement::Long if data.weapon.class_name == "Snipers" => (6., 12.),
        Engagement::Long => (3., 12.),
        Engagement::Medium => (1.5, 4.),
        _ => (1., 1.5),
    }
}
pub fn zoom(part: &ReferenceItem) -> Option<f64> {
    let mods: Vec<_> = part
        .modifiers
        .iter()
        .filter(|m| m.attribute == "Weapon.AdsZoomMultiplier")
        .collect();
    (mods.len() == 1)
        .then(|| mods[0])
        .filter(|m| m.op == "override" && m.value.is_finite() && m.value > 0.)
        .map(|m| m.value)
}
pub fn suitable_sight(data: &Reference, p: &Profile, part: &ReferenceItem) -> bool {
    let (min, max) = zoom_band(data, p);
    zoom(part).is_some_and(|z| z >= min && z <= max)
}
pub fn magazine_floor(data: &Reference, p: &Profile) -> Option<u32> {
    if p.minimum_capacity > 0 {
        return Some(p.minimum_capacity);
    }
    let defaults: Vec<_> = data
        .items
        .iter()
        .filter(|i| lab::fits(i, Slot::Magazine) && i.is_default)
        .collect();
    if defaults.len() == 1
        && let Some(n) = defaults[0].magazine_capacity
    {
        return Some(n);
    }
    match data.weapon.class_name.as_str() {
        "Assault rifles" | "SMGs" => Some(30),
        "LMGs" => Some(50),
        "DMRs" => Some(10),
        "Snipers" => Some(5),
        "Pistols" => Some(7),
        "Shotguns" => Some(2),
        _ => None,
    }
}
pub fn automatic_magazine(data: &Reference, p: &Profile, part: &ReferenceItem) -> bool {
    magazine_floor(data, p).is_none_or(|floor| part.magazine_capacity.is_some_and(|n| n >= floor))
}

#[derive(Debug, Clone, Copy)]
pub struct Context {
    precision: u8,
    axes: [f64; 2],
    default_range: Engagement,
}
impl Context {
    pub fn new(data: &Reference) -> Self {
        let axes = if data.weapon.recoil_pattern.len() >= 10 {
            let (x, y) = data
                .weapon
                .recoil_pattern
                .iter()
                .take(10)
                .fold((0., 0.), |(x, y), pt| (x + pt.x.abs(), y + pt.y.abs()));
            if x + y > 0. {
                [
                    (2. * y / (x + y)).clamp(0.5, 1.5),
                    (2. * x / (x + y)).clamp(0.5, 1.5),
                ]
            } else {
                [1.; 2]
            }
        } else {
            [1.; 2]
        };
        Self {
            precision: match data.weapon.class_name.as_str() {
                "Snipers" => 2,
                "DMRs" => 1,
                _ => 0,
            },
            axes,
            default_range: engagement(data, &Profile::default()),
        }
    }
    pub fn weights(self, p: &Profile) -> [f64; 6] {
        let [v, h, a] = p.goal.weights().map(f64::from);
        let mut w = [
            v * 0.8 * self.axes[0],
            h * 0.8 * self.axes[1],
            a * 0.8,
            0.1,
            0.05,
            0.05,
        ];
        if self.precision > 0 {
            let scale = if self.precision == 2 { 0.25 } else { 0.6 };
            w[0] *= scale;
            w[1] *= scale;
            w[2] += 0.1;
            w[3] += 0.1;
            w[5] += if self.precision == 2 { 0.25 } else { 0.1 };
        }
        match if p.engagement == Engagement::Automatic {
            self.default_range
        } else {
            p.engagement
        } {
            Engagement::Long => {
                w[2] *= 0.75;
                w[3] *= 1.75;
                w[5] *= 2.;
            }
            Engagement::Close => {
                w[2] *= 1.3;
                w[4] *= 2.;
                w[5] *= 0.5;
            }
            _ => {}
        }
        let sum: f64 = w.iter().sum();
        w.map(|n| n / sum)
    }
}

pub fn notes(data: &Reference, p: &Profile, parts: &BTreeMap<Slot, String>) -> Vec<String> {
    let mut notes = vec![];
    if lab::slots(data).contains(&Slot::Optic) {
        match parts.get(&Slot::Optic).and_then(|id|lab::item(data,id)) {
            Some(part) if !suitable_sight(data,p,part)=>notes.push(format!("{} is outside the automatic {} sight band, or its source zoom is unknown. Your saved choice is preserved.",part.name,engagement(data,p))),
            None if p.lab_pins.contains(&Slot::Optic)=>notes.push("Sight kept unchanged; its range suitability is not detected.".into()),
            None=>notes.push(format!("No confirmed {} sight fits your levels, budget and kept parts. Sight left unchanged.",engagement(data,p))),
            _=>{},
        }
    }
    if let Some(part) = parts
        .get(&Slot::Magazine)
        .and_then(|id| lab::item(data, id))
        && let Some(floor) = magazine_floor(data, p)
        && part.magazine_capacity.is_none_or(|n| n < floor)
    {
        notes.push(format!("{} is below the automatic/selected {floor}-round minimum, or has unknown capacity. Your saved choice is preserved.",part.name));
    } else if lab::slots(data).contains(&Slot::Magazine)
        && !parts.contains_key(&Slot::Magazine)
        && magazine_floor(data, p).is_some()
    {
        notes.push(if p.lab_pins.contains(&Slot::Magazine) {"Magazine kept unchanged; its capacity is not detected."}else{"Magazine unchanged; current capacity is not detected. No supported magazine meeting the floor fits the remaining constraints."}.into());
    }
    notes
}

pub fn summary(data: &Reference, p: &Profile) -> String {
    let (min, max) = zoom_band(data, p);
    let floor = magazine_floor(data, p)
        .map(|n| format!("{n}+ rounds"))
        .unwrap_or_else(|| "capacity unknown".into());
    format!(
        "{} · {}. Automatic sights use source zoom {min}–{max}; magazines: {floor}. These are recommendation policies, not measured effective ranges.",
        data.weapon.class_name,
        engagement(data, p)
    )
}
pub fn display_metrics(data: &Reference, p: &Profile) -> [usize; 2] {
    use crate::engine::Goal;
    match (data.weapon.class_name.as_str(), p.goal) {
        ("Snipers" | "DMRs", Goal::Control) => [5, 3],
        ("Snipers" | "DMRs", Goal::Balanced) => [5, 2],
        (_, Goal::Control) => [0, 1],
        (_, Goal::Balanced) => [1, 2],
        (_, Goal::Agility) => [2, 4],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policies_distinguish_precision_close_range_and_actual_recoil_axes() {
        let mut d = crate::catalog::Catalog::bundled()
            .weapons
            .iter()
            .find(|d| d.weapon.name == "M4")
            .unwrap()
            .clone();
        let p = Profile {
            goal: crate::engine::Goal::Control,
            ..Profile::default()
        };
        assert_eq!(engagement(&d, &p), Engagement::Medium);
        assert_eq!(magazine_floor(&d, &p), Some(30));
        let mut close = p.clone();
        close.engagement = Engagement::Close;
        assert_eq!(zoom_band(&d, &close), (1., 1.5));
        d.weapon.recoil_pattern = vec![crate::reference::RecoilPoint { x: 10., y: 1. }; 10];
        let horizontal = Context::new(&d).weights(&p);
        d.weapon.recoil_pattern = vec![crate::reference::RecoilPoint { x: 1., y: 10. }; 10];
        let vertical = Context::new(&d).weights(&p);
        assert!(horizontal[1] > horizontal[0]);
        assert!(vertical[0] > vertical[1]);
        assert!((horizontal.iter().sum::<f64>() - 1.).abs() < 1e-9);
        d.weapon.class_name = "Snipers".into();
        let precision = Context::new(&d).weights(&p);
        assert_eq!(zoom_band(&d, &p), (6., 12.));
        assert!(precision[5] > vertical[5]);
        let kobra = d
            .items
            .iter_mut()
            .find(|i| i.name == "Kobra Reflex")
            .unwrap();
        kobra.modifiers.push(kobra.modifiers[0].clone());
        assert!(zoom(kobra).is_none());
    }
}
