use crate::engine::Profile;
use std::{fs, io::Write, path::PathBuf};

pub fn path() -> PathBuf {
    if let Some(path) = std::env::var_os("WARDOG_PROFILE_PATH") {
        return path.into();
    }
    let root = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".local"));
    root.join("WardogCompanion").join("profile.json")
}
pub fn load() -> Result<Profile, String> {
    let path = path();
    match fs::read(&path) {
        Ok(data) => {
            let mut profile: Profile = serde_json::from_slice(&data)
                .map_err(|e| format!("Profile not loaded; file preserved: {e}"))?;
            profile.levels = profile.levels.map(|n| n.min(100));
            profile.budget = profile.budget.min(100_000);
            profile.minimum_capacity = profile.minimum_capacity.min(100);
            Ok(profile)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Profile::default()),
        Err(e) => Err(format!("Could not read profile: {e}")),
    }
}
pub fn save(profile: &Profile) -> Result<(), String> {
    save_to(&path(), profile)
}
fn save_to(path: &std::path::Path, profile: &Profile) -> Result<(), String> {
    let data = serde_json::to_vec_pretty(profile).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    // Keep a versioned backup before adding new fields to an older profile.
    if let Ok(previous) = fs::read(path)
        && let Ok(value) = serde_json::from_slice::<serde_json::Value>(&previous)
        && (value.get("locks").is_none()
            || value.get("lab_parts").is_none()
            || value.get("levels_confirmed").is_none()
            || value.get("preferred_sight").is_none()
            || value.get("minimum_capacity").is_none()
            || value.get("weapon_id").is_none()
            || value.get("class_checks").is_none()
            || value.get("unlocked").is_none())
    {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.with_extension(if value.get("locks").is_none() {
                "json.v0.1.bak"
            } else if value.get("lab_parts").is_none() {
                "json.v0.2.bak"
            } else if value.get("levels_confirmed").is_none() {
                "json.v0.3.bak"
            } else if value.get("preferred_sight").is_none()
                || value.get("minimum_capacity").is_none()
            {
                "json.v0.6.bak"
            } else if value.get("weapon_id").is_none() {
                "json.v0.7.bak"
            } else if value.get("class_checks").is_none() {
                "json.v0.8.bak"
            } else {
                "json.v0.9.bak"
            })) {
            Ok(mut file) => file.write_all(&previous).map_err(|e| e.to_string())?,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(format!("Could not back up the previous profile: {e}")),
        }
    }
    // Keep the previous valid profile if a write is interrupted.
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, data).map_err(|e| e.to_string())?;
    fs::rename(&temporary, path).map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_nine_backup_and_paid_unlocks_survive_roundtrip() {
        let dir = std::env::temp_dir().join(format!("wardog-v09-paid-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("profile.json");
        let mut value = serde_json::to_value(Profile::default()).unwrap();
        value.as_object_mut().unwrap().remove("unlocked");
        let original = value.to_string();
        fs::write(&path, &original).unwrap();
        let mut p: Profile = serde_json::from_str(&original).unwrap();
        assert!(p.unlocked.is_empty());
        p.unlocked.insert("id.item.muzl_024".into());
        save_to(&path, &p).unwrap();
        assert_eq!(
            fs::read_to_string(path.with_extension("json.v0.9.bak")).unwrap(),
            original
        );
        let loaded: Profile = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(loaded.unlocked, p.unlocked);
        assert!(loaded.owned.is_empty());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn version_eight_migration_preserves_active_setup_and_class_reports_roundtrip() {
        let dir =
            std::env::temp_dir().join(format!("wardog-v08-class-checks-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("profile.json");
        let original = r#"{"levels":[49,11,12,5,14],"locks":{},"lab_parts":{"optic":"Id.Item.SGHT_009"},"lab_pins":[],"levels_confirmed":true,"preferred_sight":null,"minimum_capacity":0,"weapon_id":"Id.Item.AK74M"}"#;
        fs::write(&path, original).unwrap();
        let mut p: Profile = serde_json::from_str(original).unwrap();
        assert!(p.class_checks.is_empty());
        p.class_checks.insert(
            "build:revision:weapon".into(),
            crate::engine::ClassChecks {
                fits: true,
                requirements: false,
                panel: true,
            },
        );
        save_to(&path, &p).unwrap();
        let backup = path.with_extension("json.v0.8.bak");
        assert_eq!(fs::read_to_string(&backup).unwrap(), original);
        let loaded: Profile = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let check = &loaded.class_checks["build:revision:weapon"];
        assert!(check.fits && check.panel && !check.requirements);
        assert_eq!(loaded.lab_parts, p.lab_parts);
        fs::remove_file(path).unwrap();
        fs::remove_file(backup).unwrap();
        fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn version_seven_backup_keeps_active_ak_preferences_and_defaults_weapon_id() {
        let dir = std::env::temp_dir().join(format!("wardog-v07-catalog-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("profile.json");
        let original = r#"{"levels":[49,11,12,5,14],"locks":{},"lab_parts":{"optic":"Id.Item.SGHT_009"},"lab_pins":["optic"],"levels_confirmed":true,"preferred_sight":"Id.Item.SGHT_009","minimum_capacity":30}"#;
        fs::write(&path, original).unwrap();
        let p: Profile = serde_json::from_str(original).unwrap();
        assert_eq!(p.weapon_id, "Id.Item.AK74M");
        save_to(&path, &p).unwrap();
        let backup = path.with_extension("json.v0.7.bak");
        assert_eq!(fs::read_to_string(&backup).unwrap(), original);
        let loaded: Profile = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(loaded.preferred_sight, p.preferred_sight);
        assert_eq!(loaded.minimum_capacity, 30);
        fs::remove_file(path).unwrap();
        fs::remove_file(backup).unwrap();
        fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn version_six_backup_and_new_preferences_roundtrip_without_losing_progress() {
        let dir =
            std::env::temp_dir().join(format!("wardog-v06-preferences-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("profile.json");
        let original = r#"{"levels":[49,11,12,5,14],"budget":2000,"goal":"Control","locks":{},"lab_parts":{"optic":"Id.Item.SGHT_009"},"lab_pins":["optic"],"levels_confirmed":true}"#;
        fs::write(&path, original).unwrap();
        let mut p: Profile = serde_json::from_str(original).unwrap();
        assert_eq!(p.minimum_capacity, 0);
        assert!(p.preferred_sight.is_none());
        p.preferred_sight = Some("Id.Item.SGHT_009".into());
        p.minimum_capacity = 60;
        save_to(&path, &p).unwrap();
        let backup = path.with_extension("json.v0.6.bak");
        assert_eq!(fs::read_to_string(&backup).unwrap(), original);
        let loaded: Profile = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(loaded.minimum_capacity, 60);
        assert_eq!(loaded.preferred_sight, p.preferred_sight);
        assert_eq!(loaded.lab_parts, p.lab_parts);
        assert_eq!(loaded.lab_pins, p.lab_pins);
        assert_eq!(loaded.levels, [49, 11, 12, 5, 14]);
        fs::remove_file(path).unwrap();
        fs::remove_file(backup).unwrap();
        fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn profile_roundtrip_and_replacement() {
        let dir = std::env::temp_dir().join(format!("wardog-test-{}", std::process::id()));
        let file = dir.join("profile.json");
        let mut p = Profile::default();
        save_to(&file, &p).unwrap();
        p.levels[0] = 49;
        p.owned.insert("grip-control".into());
        p.locks
            .insert(crate::engine::Slot::Grip, "grip-control".into());
        p.lab_parts
            .insert(crate::engine::Slot::Grip, "Id.Item.FGRP_015".into());
        p.lab_pins.insert(crate::engine::Slot::Grip);
        save_to(&file, &p).unwrap();
        let actual: Profile = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        assert_eq!(actual.levels[0], 49);
        assert!(actual.owned.contains("grip-control"));
        assert_eq!(
            actual.lab_parts.get(&crate::engine::Slot::Grip).unwrap(),
            "Id.Item.FGRP_015"
        );
        assert!(actual.lab_pins.contains(&crate::engine::Slot::Grip));
        assert_eq!(
            actual.locks.get(&crate::engine::Slot::Grip).unwrap(),
            "grip-control"
        );
        fs::remove_file(&file).unwrap();
        fs::remove_dir(&dir).unwrap();
    }
    #[test]
    fn version_one_profile_preserves_progress_without_locks() {
        let old = r#"{"levels":[49,11,12,5,14],"owned":["optic-light"],"owned_only":true,"budget":1250,"goal":"Control"}"#;
        let profile: Profile = serde_json::from_str(old).unwrap();
        assert_eq!(profile.levels, [49, 11, 12, 5, 14]);
        assert!(profile.owned.contains("optic-light"));
        assert_eq!(profile.budget, 1250);
        assert!(profile.owned_only);
        assert!(profile.locks.is_empty());
        assert!(profile.lab_parts.is_empty());
        assert!(profile.lab_pins.is_empty());
        assert_eq!(profile.goal, crate::engine::Goal::Control);
    }
    #[test]
    fn first_save_keeps_an_unchanged_version_one_backup() {
        let dir =
            std::env::temp_dir().join(format!("wardog-migration-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("profile.json");
        let original = r#"{"levels":[49,11,12,5,14],"budget":1250}"#;
        fs::write(&path, original).unwrap();
        let mut p: Profile = serde_json::from_str(original).unwrap();
        p.locks
            .insert(crate::engine::Slot::Optic, "optic-light".into());
        save_to(&path, &p).unwrap();
        let backup = path.with_extension("json.v0.1.bak");
        assert_eq!(fs::read_to_string(&backup).unwrap(), original);
        p.budget = 2000;
        save_to(&path, &p).unwrap();
        assert_eq!(fs::read_to_string(&backup).unwrap(), original);
        fs::remove_file(path).unwrap();
        fs::remove_file(backup).unwrap();
        fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn version_two_backup_keeps_demo_locks_and_owned_items() {
        let dir = std::env::temp_dir().join(format!("wardog-v02-migration-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("profile.json");
        let old =
            r#"{"levels":[49,11,12,5,14],"owned":["optic-light"],"locks":{"grip":"grip-fast"}}"#;
        fs::write(&path, old).unwrap();
        let mut p: Profile = serde_json::from_str(old).unwrap();
        p.lab_parts
            .insert(crate::engine::Slot::Grip, "Id.Item.FGRP_015".into());
        save_to(&path, &p).unwrap();
        let backup = path.with_extension("json.v0.2.bak");
        assert_eq!(fs::read_to_string(&backup).unwrap(), old);
        let saved: Profile = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved.levels, [49, 11, 12, 5, 14]);
        assert!(saved.owned.contains("optic-light"));
        assert_eq!(
            saved.locks.get(&crate::engine::Slot::Grip).unwrap(),
            "grip-fast"
        );
        fs::remove_file(path).unwrap();
        fs::remove_file(backup).unwrap();
        fs::remove_dir(dir).unwrap();
    }
    #[test]
    fn version_three_setup_and_pins_survive_simple_ui_migration() {
        let dir = std::env::temp_dir().join(format!("wardog-v03-migration-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("profile.json");
        let old = r#"{"levels":[49,11,12,5,14],"owned":["Id.Item.FGP_03"],"budget":1500,"goal":"Control","locks":{},"lab_parts":{"grip":"Id.Item.FGP_03"},"lab_pins":["grip"]}"#;
        fs::write(&file, old).unwrap();
        let mut profile: Profile = serde_json::from_str(old).unwrap();
        assert!(!profile.levels_confirmed);
        profile.levels_confirmed = true;
        save_to(&file, &profile).unwrap();
        let backup = file.with_extension("json.v0.3.bak");
        assert_eq!(fs::read_to_string(&backup).unwrap(), old);
        let loaded: Profile = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        assert_eq!(loaded.levels, [49, 11, 12, 5, 14]);
        assert_eq!(loaded.budget, 1500);
        assert!(loaded.levels_confirmed);
        assert!(loaded.lab_pins.contains(&crate::engine::Slot::Grip));
        assert_eq!(
            loaded.lab_parts.get(&crate::engine::Slot::Grip).unwrap(),
            "Id.Item.FGP_03"
        );
        assert!(loaded.owned.contains("Id.Item.FGP_03"));
        fs::remove_file(file).unwrap();
        fs::remove_file(backup).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
