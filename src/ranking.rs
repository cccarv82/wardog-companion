//! Class comparisons of unmodified source records under one explicit raw-damage scenario.
use crate::engine::Slot;
use crate::{
    catalog::Catalog,
    engine::{Profile, Track},
    lab,
    reference::Weapon,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Fastest,
    Steadiest,
    Balanced,
}
impl Mode {
    pub const ALL: [Self; 3] = [Self::Fastest, Self::Steadiest, Self::Balanced];
}
impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Fastest => "Fastest raw TTK",
            Self::Steadiest => "Lowest recoil",
            Self::Balanced => "Balanced",
        })
    }
}
pub fn eligible(w: &Weapon, p: &Profile) -> Option<bool> {
    if w.test_record {
        return Some(false);
    }
    if w.unlock_free {
        return Some(true);
    }
    let u = w.unlock.as_ref()?;
    let track = match u.role.as_str() {
        "Wardog" => Track::Wardog,
        "Infantry" => Track::Assault,
        "Medic" => Track::Medic,
        "Recon" => Track::Recon,
        "Support" => Track::Support,
        _ => return None,
    };
    Some(p.levels[track.index()] >= u.level)
}
pub fn availability(w: &Weapon, p: &Profile) -> String {
    if w.test_record {
        return "Development / test record".into();
    }
    if w.unlock_free {
        return "No level unlock required".into();
    }
    match (&w.unlock, eligible(w, p)) {
        (Some(u), Some(yes)) => format!(
            "{} {} · {}",
            if u.role == "Infantry" {
                "Assault"
            } else {
                &u.role
            },
            u.level,
            if yes { "Level met" } else { "Locked" }
        ),
        _ => "Unlock not published".into(),
    }
}
pub fn ttk(damage: f64, rpm: f64, hp: u32) -> Option<f64> {
    if !damage.is_finite() || !rpm.is_finite() || damage <= 0. || rpm <= 0. || hp == 0 {
        return None;
    }
    let shots = ((hp as f64 / damage) - 1e-12).ceil().max(1.);
    Some((shots - 1.) * 60. / rpm)
}
pub struct Entry {
    pub weapon: Weapon,
    pub ttk: Option<f64>,
    pub dps: Option<f64>,
    pub score: f64,
    pub damage: Option<f64>,
    pub recoil: Option<f64>,
    pub position: usize,
    pub parts: Option<BTreeMap<Slot, String>>,
    pub build_cost: Option<f64>,
    pub note: Option<String>,
}
pub struct Report {
    pub entries: Vec<Entry>,
    pub excluded: Vec<(String, String)>,
}
#[derive(Clone, Copy)]
pub struct Scenario {
    pub hp: u32,
    pub distance: u32,
    pub builds: bool,
}

pub fn falloff(w: &Weapon, distance: u32) -> Result<f64, String> {
    let d = distance as f64;
    if d == 0. && w.falloff.is_empty() {
        return Ok(1.);
    }
    let first = w
        .falloff
        .first()
        .ok_or("Distance falloff is not published")?;
    if d <= first.distance {
        return Ok(first.multiplier);
    }
    for point in &w.falloff {
        if (point.distance - d).abs() < 1e-9 {
            return Ok(point.multiplier);
        }
    }
    for pair in w.falloff.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        if d > a.distance && d < b.distance {
            return match a.interpolation.as_str() {
                "Linear" => Ok(a.multiplier + (b.multiplier-a.multiplier)*(d-a.distance)/(b.distance-a.distance)),
                "Constant" => Ok(a.multiplier),
                _ => Err("Cubic/unknown falloff segment lacks published tangents; choose an exact source distance".into()),
            };
        }
    }
    Ok(w.falloff.last().unwrap().multiplier)
}
fn recoil(w: &Weapon, factors: Option<[f64; 6]>) -> Option<f64> {
    let Some(f) = factors else {
        return w.recoil_index;
    };
    (w.recoil_pattern.len() >= 10).then(|| {
        w.recoil_pattern
            .iter()
            .take(10)
            .map(|p| (p.x * f[1]).hypot(p.y * f[0]))
            .sum::<f64>()
            / 10.
    })
}
#[cfg(test)]
pub fn rank(
    data: &Catalog,
    class: &str,
    mode: Mode,
    hp: u32,
    p: &Profile,
    include_locked: bool,
) -> Vec<Entry> {
    compare(
        data,
        class,
        mode,
        Scenario {
            hp,
            distance: 0,
            builds: false,
        },
        p,
        include_locked,
    )
    .entries
}
pub fn compare(
    data: &Catalog,
    class: &str,
    mode: Mode,
    scenario: Scenario,
    p: &Profile,
    include_locked: bool,
) -> Report {
    let mut rows = Vec::new();
    let mut excluded = Vec::new();
    for reference in data.weapons.iter().filter(|r| r.weapon.class_name == class) {
        let w = &reference.weapon;
        let mut reject = |reason: String| {
            excluded.push((w.name.clone(), reason));
        };
        if w.test_record {
            reject("Development/test record".into());
            continue;
        }
        if (!include_locked || scenario.builds) && eligible(w, p) != Some(true) {
            reject("Weapon level requirement is unmet or unavailable".into());
            continue;
        }
        let (parts, build_cost, factors) = if scenario.builds {
            let mut profile = p.clone();
            profile.lab_parts.clear();
            profile.lab_pins.clear();
            profile.preferred_sight = None;
            profile.minimum_capacity = 0;
            match lab::suggest(reference, &profile).and_then(|parts| {
                lab::evaluate(reference, &profile, &parts).map(|estimate| (parts, estimate))
            }) {
                Ok((parts, estimate)) => (Some(parts), Some(estimate.cost), Some(estimate.factors)),
                Err(error) => {
                    reject(format!("No supported build within constraints: {error}"));
                    continue;
                }
            }
        } else {
            (None, None, None)
        };
        let distance_result = falloff(w, scenario.distance);
        let damage = if w.comparable {
            w.damage_per_hit
                .zip(distance_result.as_ref().ok().copied())
                .map(|(d, f)| d * f)
        } else {
            None
        };
        let time = damage.zip(w.rpm).and_then(|(d, r)| ttk(d, r, scenario.hp));
        let dps = damage
            .zip(w.rpm)
            .filter(|(d, r)| *d >= 0. && *r > 0.)
            .map(|(d, r)| d * r / 60.);
        let stability = recoil(w, factors);
        let note = time.is_none().then(|| {
            distance_result.as_ref().err().cloned().unwrap_or_else(|| {
                if !w.comparable {
                    "TTK unavailable: unsupported projectile baseline".into()
                } else if w.rpm.is_none() {
                    "TTK unavailable: RPM not published".into()
                } else {
                    "No positive damage at this distance".into()
                }
            })
        });
        let reason = if mode != Mode::Steadiest && time.is_none() {
            Some(if !w.comparable {
                "Ambiguous/unsupported projectile baseline".into()
            } else if w.rpm.is_none_or(|rpm| rpm <= 0.) {
                "RPM is not published".into()
            } else if let Err(error) = distance_result {
                error
            } else {
                "No positive modeled damage at this distance".into()
            })
        } else if mode != Mode::Fastest && stability.is_none() {
            Some("Ten-shot recoil pattern is unavailable".into())
        } else {
            None
        };
        if let Some(reason) = reason {
            reject(reason);
            continue;
        }
        rows.push(Entry {
            weapon: w.clone(),
            ttk: time,
            dps,
            score: 0.,
            damage,
            recoil: stability,
            position: 0,
            parts,
            build_cost,
            note,
        });
    }
    // Average within-class ordinal positions: no mixing seconds with unknown recoil units.
    let values: Vec<_> = rows
        .iter()
        .map(|r| (r.ttk.unwrap_or(0.), r.recoil.unwrap_or(0.)))
        .collect();
    for row in &mut rows {
        row.score = match mode {
            Mode::Fastest => row.ttk.unwrap(),
            Mode::Steadiest => row.recoil.unwrap(),
            Mode::Balanced => {
                let position = |value: f64, metric: usize| {
                    values
                        .iter()
                        .map(|v| {
                            let other = if metric == 0 { v.0 } else { v.1 };
                            if other < value - 1e-9 {
                                1.
                            } else if (other - value).abs() <= 1e-9 {
                                0.5
                            } else {
                                0.
                            }
                        })
                        .sum::<f64>()
                };
                (position(row.ttk.unwrap(), 0) + position(row.recoil.unwrap(), 1)) / 2.
            }
        };
    }
    rows.sort_by(|a, b| {
        a.score
            .total_cmp(&b.score)
            .then(a.weapon.name.cmp(&b.weapon.name))
    });
    let mut position = 1;
    for index in 0..rows.len() {
        if index > 0 && (rows[index].score - rows[index - 1].score).abs() > 1e-9 {
            position = index + 1;
        }
        rows[index].position = position;
    }
    Report {
        entries: rows,
        excluded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discrete_ttk_counts_first_hit_at_zero_and_rejects_unknowns() {
        assert_eq!(ttk(25., 600., 100), Some(0.3));
        assert_eq!(ttk(100., 600., 100), Some(0.));
        assert_eq!(ttk(34., 600., 100), Some(0.2));
        assert_eq!(ttk(0., 600., 100), None);
        assert_eq!(ttk(25., 0., 100), None);
    }
    #[test]
    fn rankings_keep_class_scenario_and_unlock_rules_separate() {
        let data = Catalog::bundled();
        let p = Profile::default();
        let rows = rank(&data, "Assault rifles", Mode::Fastest, 100, &p, true);
        assert!(!rows.is_empty());
        assert!(rows.iter().all(|r| r.weapon.class_name == "Assault rifles"
            && r.weapon.comparable
            && !r.weapon.test_record));
        assert!(rows.windows(2).all(|r| r[0].ttk <= r[1].ttk));
        let starting = rank(&data, "Assault rifles", Mode::Fastest, 100, &p, false);
        assert!(!starting.is_empty());
        assert!(starting.iter().all(|r| r.weapon.unlock_free));
        assert!(starting.iter().any(|r| r.weapon.id == "Id.Item.AK74M"));
        assert!(rank(&data, "Shotguns", Mode::Fastest, 100, &p, true).is_empty());
        let ak = rows
            .iter()
            .find(|r| r.weapon.id == "Id.Item.AK74M")
            .unwrap();
        assert!((ak.dps.unwrap() - 281.6666666667).abs() < 1e-6);
        assert!((ak.ttk.unwrap() - 3. * 60. / 650.).abs() < 1e-6);
    }
    #[test]
    fn user_levels_filter_locked_smgs_and_more_hp_changes_shot_count_not_dps() {
        let data = Catalog::bundled();
        let p = Profile {
            levels: [49, 11, 12, 5, 14],
            ..Profile::default()
        };
        let rows = rank(&data, "SMGs", Mode::Fastest, 100, &p, false);
        assert_eq!(rows.len(), 2);
        assert!(
            rows.iter()
                .any(|r| r.weapon.name == "PP-19 Bizon" && r.weapon.unlock_free)
        );
        assert_eq!(rows[0].weapon.name, "MP5");
        let tougher = rank(&data, "SMGs", Mode::Fastest, 200, &p, false);
        assert!(tougher[0].ttk > rows[0].ttk);
        assert_eq!(tougher[0].dps, rows[0].dps);
        let steady = rank(&data, "SMGs", Mode::Steadiest, 100, &p, true);
        assert!(
            steady
                .windows(2)
                .all(|r| r[0].weapon.recoil_index <= r[1].weapon.recoil_index)
        );
        let balanced = rank(&data, "SMGs", Mode::Balanced, 100, &p, false);
        assert_eq!(balanced[0].weapon.name, "MP5");
    }
    #[test]
    fn distance_curves_use_linear_segments_and_exact_cubic_keys_without_guessing() {
        let data = Catalog::bundled();
        let m4 = &data.weapon("Id.Item.M4").unwrap().weapon;
        assert!((falloff(m4, 850).unwrap() - 0.6).abs() < 1e-9);
        assert_eq!(falloff(m4, 500).unwrap(), 1.);
        let amp = &data.weapon("Id.Item.MP9").unwrap().weapon;
        assert!(falloff(amp, 150).is_err());
        assert_eq!(falloff(amp, 200).unwrap(), 0.6);
        assert_eq!(falloff(amp, 50).unwrap(), 1.);
        assert_eq!(falloff(amp, 1300).unwrap(), 0.1);
        let judge = &data.weapon("Id.Item.Judge").unwrap().weapon;
        assert_eq!(falloff(judge, 0).unwrap(), 1.);
        assert!(falloff(judge, 100).is_err());
        let p = Profile::default();
        let report = compare(
            &data,
            "SMGs",
            Mode::Fastest,
            Scenario {
                hp: 100,
                distance: 500,
                builds: false,
            },
            &p,
            true,
        );
        assert!(
            report
                .excluded
                .iter()
                .any(|(name, reason)| name == "AMP-9" && reason.contains("tangents"))
        );
    }
    #[test]
    fn builds_obey_levels_budget_and_scale_each_recoil_component() {
        let data = Catalog::bundled();
        let p = Profile {
            levels: [49, 11, 12, 5, 14],
            ..Profile::default()
        };
        let report = compare(
            &data,
            "Assault rifles",
            Mode::Balanced,
            Scenario {
                hp: 100,
                distance: 100,
                builds: true,
            },
            &p,
            true,
        );
        assert!(!report.entries.is_empty());
        assert!(
            report
                .excluded
                .iter()
                .any(|(name, reason)| name == "M4" && reason.contains("level"))
        );
        for entry in &report.entries {
            let reference = data.weapon(&entry.weapon.id).unwrap();
            let estimate = lab::evaluate(reference, &p, entry.parts.as_ref().unwrap()).unwrap();
            assert!(entry.build_cost.unwrap() <= p.budget as f64);
            let expected = reference
                .weapon
                .recoil_pattern
                .iter()
                .take(10)
                .map(|v| (v.x * estimate.factors[1]).hypot(v.y * estimate.factors[0]))
                .sum::<f64>()
                / 10.;
            assert!((entry.recoil.unwrap() - expected).abs() < 1e-9);
        }
    }
    #[test]
    fn equal_scores_share_rank_and_constant_segments_hold_until_the_next_key() {
        let mut data = Catalog::bundled();
        let ids: Vec<_> = data
            .weapons
            .iter()
            .enumerate()
            .filter(|(_, r)| r.weapon.class_name == "Assault rifles")
            .map(|(i, _)| i)
            .collect();
        for i in ids {
            let w = &mut data.weapons[i].weapon;
            w.rpm = Some(600.);
            w.damage_per_hit = Some(25.);
            w.comparable = true;
        }
        let rows = rank(
            &data,
            "Assault rifles",
            Mode::Fastest,
            100,
            &Profile::default(),
            true,
        );
        assert!(rows.iter().all(|r| r.position == 1));
        let w = &mut data
            .weapons
            .iter_mut()
            .find(|r| r.weapon.id == "Id.Item.M4")
            .unwrap()
            .weapon;
        w.falloff[0].interpolation = "Constant".into();
        w.falloff[1].multiplier = 0.5;
        assert_eq!(falloff(w, 499).unwrap(), 1.);
        assert_eq!(falloff(w, 500).unwrap(), 0.5);
    }
}
