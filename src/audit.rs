use crate::{
    catalog::Catalog,
    engine::{Engagement, Goal, Profile},
    lab, purpose,
    ranking::{self, Mode, Scenario},
};
pub fn export(data: &Catalog, output: &std::path::Path) -> Result<(), String> {
    let mut cases = Vec::new();
    for levels in [[49, 11, 12, 5, 14], [100; 5]] {
        let p = Profile {
            levels,
            ..Profile::default()
        };
        for class in data.classes().into_iter().filter(|s| s != "Test / vehicle") {
            for distance in [0, 100, 200, 300, 500] {
                for hp in [100, 150, 200] {
                    for builds in [false, true] {
                        for mode in Mode::ALL {
                            let report = ranking::compare(
                                data,
                                &class,
                                mode,
                                Scenario {
                                    hp,
                                    distance,
                                    builds,
                                },
                                &p,
                                true,
                            );
                            if report.entries.windows(2).any(|r| r[0].score > r[1].score)
                                || report.entries.iter().any(|r| {
                                    r.weapon.test_record
                                        || r.weapon.class_name != class
                                        || r.ttk.is_some_and(|v| !v.is_finite() || v < 0.)
                                        || r.build_cost.is_some_and(|cost| cost > p.budget as f64)
                                        || (builds
                                            && ranking::eligible(&r.weapon, &p) != Some(true))
                                })
                            {
                                return Err("Ranking invariant failed; report not written".into());
                            }
                            cases.push(serde_json::json!({"levels":levels,"class":class,"distance_m":distance,"hp":hp,"builds":builds,"mode":mode.to_string(),
                    "ranked":report.entries.iter().map(|r|serde_json::json!({"id":r.weapon.id,"name":r.weapon.name,"position":r.position,
                        "projectile":r.weapon.projectile,"damage_at_distance":r.damage,"ttk_s":r.ttk,"dps":r.dps,"recoil_index":r.recoil,"parts":r.parts,"parts_cost":r.build_cost})).collect::<Vec<_>>(),"excluded":report.excluded}));
                        }
                    }
                }
            }
        }
    }
    let mut setups = vec![];
    for levels in [[49, 11, 12, 5, 14], [100; 5]] {
        for reference in data.weapons.iter().filter(|r| !r.weapon.test_record) {
            for engagement in Engagement::ALL {
                for goal in Goal::ALL {
                    for budget in [0, 2000] {
                        let p = Profile {
                            levels,
                            engagement,
                            goal,
                            budget,
                            ..Profile::default()
                        };
                        let parts = lab::suggest(reference, &p)?;
                        let estimate = lab::evaluate(reference, &p, &parts)?;
                        if estimate.cost > budget as f64
                            || !estimate.unknown_prices.is_empty()
                            || parts
                                .get(&crate::engine::Slot::Optic)
                                .and_then(|id| lab::item(reference, id))
                                .is_some_and(|part| !purpose::suitable_sight(reference, &p, part))
                            || parts
                                .get(&crate::engine::Slot::Magazine)
                                .and_then(|id| lab::item(reference, id))
                                .is_some_and(|part| {
                                    !purpose::automatic_magazine(reference, &p, part)
                                })
                        {
                            return Err(format!(
                                "Context invariant failed for {}",
                                reference.weapon.name
                            ));
                        }
                        setups.push(serde_json::json!({"weapon":reference.weapon.name,"class":reference.weapon.class_name,"levels":levels,"budget":budget,"requested_range":engagement,"effective_range":purpose::engagement(reference,&p),"goal":goal,"parts":parts.iter().map(|(slot,id)|serde_json::json!({"slot":slot,"id":id,"name":lab::item(reference,id).unwrap().name})).collect::<Vec<_>>(),"cost":estimate.cost,"policy":purpose::summary(reference,&p),"notes":purpose::notes(reference,&p,&parts)}));
                    }
                }
            }
        }
    }
    let report = serde_json::json!({"game_build":data.game_build,"revision":data.source_commit,"app_version":env!("CARGO_PKG_VERSION"),"recommendation_model":purpose::MODEL,
        "kind":"Automated model audit, not an in-game validation","cases":cases.len(),"results":cases,"setup_cases":setups.len(),"setup_results":setups});
    let bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    std::fs::write(output, bytes).map_err(|e| e.to_string())
}
