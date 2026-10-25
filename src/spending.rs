//! Price/performance trade-offs, never measured accuracy or percentages of actual combat power.
use crate::{engine::Profile, lab, reference::Reference};
#[derive(Debug)]
pub struct Option {
    pub label: &'static str,
    pub build: lab::Build,
    pub gain_share: f64,
}
#[derive(Debug)]
pub struct Report {
    pub has_gain: bool,
    pub options: Vec<Option>,
    pub extra_cost: f64,
    pub remaining_gain: f64,
}
pub fn analyze(data: &Reference, p: &Profile) -> Result<Report, String> {
    let choices = lab::frontier(data, p)?;
    let low = choices.first().unwrap().estimate.score(p);
    let high = choices.last().unwrap().estimate.score(p);
    let span = high - low;
    let mut options = vec![];
    let mut efficient = 0;
    for (label, target) in [("Economy", 0.70), ("Sweet spot", 0.95), ("Maximum", 1.)] {
        let index = choices
            .iter()
            .position(|b| b.estimate.score(p) >= low + target * span - 1e-9)
            .unwrap_or(choices.len() - 1);
        if label == "Sweet spot" {
            efficient = index;
        }
        if options
            .iter()
            .any(|o: &Option| o.build.parts == choices[index].parts)
        {
            continue;
        }
        options.push(Option {
            label,
            build: choices[index].clone(),
            gain_share: if span > 1e-9 {
                (choices[index].estimate.score(p) - low) / span
            } else {
                1.
            },
        });
    }
    // If thresholds collapse to the same setup, show one honest choice, not duplicate tiers.
    if options.len() == 1 {
        options[0].label = "Best value";
    }
    let extra_cost = choices.last().unwrap().estimate.cost - choices[efficient].estimate.cost;
    let remaining_gain = if span > 1e-9 {
        (high - choices[efficient].estimate.score(p)) / span * 100.
    } else {
        0.
    };
    Ok(Report {
        has_gain: span > 1e-9,
        options,
        extra_cost,
        remaining_gain,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Goal, Slot};
    #[test]
    fn choices_are_nondominated_and_maximum_keeps_the_existing_optimum() {
        let d = Reference::load();
        let p = Profile {
            levels: [99; 5],
            goal: Goal::Control,
            ..Profile::default()
        };
        let f = lab::frontier(&d, &p).unwrap();
        assert!(f.len() > 1);
        for pair in f.windows(2) {
            assert!(pair[0].estimate.cost < pair[1].estimate.cost);
            assert!(pair[0].estimate.score(&p) < pair[1].estimate.score(&p));
        }
        let r = analyze(&d, &p).unwrap();
        assert_eq!(
            r.options.last().unwrap().build.parts,
            lab::suggest(&d, &p).unwrap()
        );
        for o in &r.options {
            assert!(o.build.estimate.cost <= p.budget as f64);
            assert!(o.gain_share >= 0.70 - 1e-9);
        }
        assert!(r.remaining_gain <= 5. + 1e-8);
    }
    #[test]
    fn zero_budget_and_pins_do_not_create_fake_tiers_or_relax_constraints() {
        let d = Reference::load();
        let mut p = Profile {
            levels: [99; 5],
            budget: 0,
            ..Profile::default()
        };
        let r = analyze(&d, &p).unwrap();
        assert_eq!(r.options.len(), 1);
        assert!(!r.has_gain);
        assert_eq!(r.options[0].build.estimate.cost, 0.);
        let id = d
            .items
            .iter()
            .find(|i| {
                lab::fits(i, Slot::Muzzle)
                    && i.vendor
                        .iter()
                        .any(|v| v.tag.eq_ignore_ascii_case(&i.id) && v.price > 0.)
            })
            .unwrap()
            .id
            .clone();
        p.lab_parts.insert(Slot::Muzzle, id);
        p.lab_pins.insert(Slot::Muzzle);
        assert!(analyze(&d, &p).is_err());
    }
}
