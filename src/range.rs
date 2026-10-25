use crate::{
    catalog::Catalog,
    engine::{Goal, Profile},
    ranking,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    Close,
    Medium,
    Long,
}
impl Reach {
    pub const ALL: [Self; 3] = [Self::Close, Self::Medium, Self::Long];
    pub fn distance(self) -> u32 {
        match self {
            Self::Close => 25,
            Self::Medium => 100,
            Self::Long => 300,
        }
    }
    pub fn goal(self) -> Goal {
        match self {
            Self::Close => Goal::Agility,
            Self::Medium => Goal::Balanced,
            Self::Long => Goal::Control,
        }
    }
}
impl std::fmt::Display for Reach {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Close => "Close",
            Self::Medium => "Medium",
            Self::Long => "Long",
        })
    }
}
pub fn recommend(data: &Catalog, p: &Profile, class: &str, reach: Reach) -> ranking::Report {
    let mut p = p.clone();
    p.goal = reach.goal();
    ranking::compare(
        data,
        class,
        ranking::Mode::Balanced,
        ranking::Scenario {
            hp: 100,
            distance: reach.distance(),
            builds: true,
        },
        &p,
        false,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presets_only_recommend_eligible_builds_and_do_not_change_the_profile() {
        let d = Catalog::bundled();
        let p = Profile {
            levels: [49, 11, 12, 5, 14],
            ..Profile::default()
        };
        for reach in Reach::ALL {
            let r = recommend(&d, &p, "Assault rifles", reach);
            assert!(!r.entries.is_empty());
            for e in r.entries {
                assert_eq!(ranking::eligible(&e.weapon, &p), Some(true));
                assert!(e.parts.is_some());
                assert!(e.build_cost.unwrap() <= p.budget as f64);
            }
        }
        assert_eq!(p.goal, Goal::Balanced);
    }
}
