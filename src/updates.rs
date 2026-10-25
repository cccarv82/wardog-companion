//! Download only data from a fixed repository. Stage, validate, review, then activate.
use crate::catalog::Catalog;
use serde_json::{Value as V, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, io::Read, path::Path, time::Duration};
const FILES: [&str; 9] = [
    "items.json",
    "weapons.json",
    "attachments.json",
    "ammo.json",
    "damage.json",
    "equipment.json",
    "tags.json",
    "csv/weapon-compatibility.csv",
    "csv/weapon-damage.csv",
];
const REPO: &str = "https://api.github.com/repos/warpuppies/wardogs-data";
#[derive(Debug, Clone)]
pub struct Candidate {
    pub json: String,
    pub summary: String,
}
fn array<'a>(v: &'a V, key: &str) -> Result<&'a Vec<V>, String> {
    v[key]
        .as_array()
        .ok_or_else(|| format!("Unsupported source field: {key}"))
}
fn index(v: &V, key: &str) -> Result<BTreeMap<String, V>, String> {
    let mut out = BTreeMap::new();
    for row in array(v, key)? {
        let id = row["id"].as_str().ok_or("Missing ID")?.to_lowercase();
        if out.insert(id, row.clone()).is_some() {
            return Err("Duplicate source ID".into());
        }
    }
    Ok(out)
}
fn string(v: &V) -> &str {
    v.as_str().unwrap_or("")
}
fn csv_rows(bytes: &[u8]) -> Result<Vec<BTreeMap<String, String>>, String> {
    csv::Reader::from_reader(bytes)
        .deserialize()
        .map(|r| r.map_err(|e| e.to_string()))
        .collect()
}
fn num(s: Option<&String>) -> Result<V, String> {
    match s.filter(|s| !s.is_empty()) {
        None => Ok(V::Null),
        Some(s) => {
            let n = s.parse::<f64>().map_err(|_| "Invalid source number")?;
            if !n.is_finite() || n < 0. {
                return Err("Invalid source number".into());
            }
            Ok(json!(n))
        }
    }
}
fn part(item: &V, a: &V) -> V {
    json!({"id":item["id"],"name":item["name"],"slot":a["type"],"is_default":false,"unlock_free":item["unlock"].is_null()&&!item["developer"].as_bool().unwrap_or(false),"developer":item["developer"].as_bool().unwrap_or(false),"unlock":item["unlock"],"weight":item["weight"],"vendor":item["vendor"].as_array().cloned().unwrap_or_default(),"modifiers":a["modifiers"].as_array().cloned().unwrap_or_default(),"character_modifiers":a["characterModifiers"].as_array().cloned().unwrap_or_default(),"magazine_capacity":a["magazine"]["capacity"],"compatible_slots":[],"conditional":a["type"]=="Bipod"})
}
pub fn build(
    raw: &BTreeMap<String, Vec<u8>>,
    revision: &str,
    imported: &str,
) -> Result<String, String> {
    if revision.len() != 40 || !revision.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid revision".into());
    }
    let read = |name: &str| -> Result<V, String> {
        serde_json::from_slice(raw.get(name).ok_or_else(|| format!("Missing {name}"))?)
            .map_err(|e| e.to_string())
    };
    let manifest = read("MANIFEST.json")?;
    let build = &manifest["provenance"]["gameBuild"];
    if !build.is_string() {
        return Err("Missing source build".into());
    }
    let mut sources = vec![];
    let mut docs = BTreeMap::new();
    for name in FILES {
        let b = raw.get(name).ok_or("Missing download")?;
        let hash = format!("{:x}", Sha256::digest(b));
        let path = format!("data/{name}");
        let m = array(&manifest, "files")?
            .iter()
            .find(|e| e["path"] == path)
            .ok_or("File absent from manifest")?;
        if m["sha256"] != hash {
            return Err(format!("Checksum mismatch: {name}"));
        }
        sources.push(json!({"path":name,"sha256":hash}));
        if name.ends_with(".json") {
            let doc = read(name)?;
            if doc["schemaVersion"] != 1 || doc["gameBuild"] != *build {
                return Err("Mixed source builds/schema".into());
            }
            docs.insert(name, doc);
        }
    }
    let items = index(&docs["items.json"], "items")?;
    let weapons = index(&docs["weapons.json"], "weapons")?;
    let attachments = index(&docs["attachments.json"], "attachments")?;
    let ammo = array(&docs["ammo.json"], "projectiles")?;
    let compat = csv_rows(&raw["csv/weapon-compatibility.csv"])?;
    let damages = csv_rows(&raw["csv/weapon-damage.csv"])?;
    for c in &compat {
        if !weapons.contains_key(
            &c.get("weapon_id")
                .ok_or("Missing weapon ID")?
                .to_lowercase(),
        ) || !items.contains_key(&c.get("item_id").ok_or("Missing item ID")?.to_lowercase())
        {
            return Err("Broken source compatibility".into());
        }
    }
    let all = attachments
        .iter()
        .map(|(id, a)| {
            items
                .get(id)
                .map(|i| part(i, a))
                .ok_or("Attachment has no item record".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut refs = vec![];
    let mut notes = vec![];
    for (key, w) in &weapons {
        let item = items.get(key).ok_or("Weapon has no item record")?;
        let name = string(&w["name"]);
        let test =
            item["developer"].as_bool().unwrap_or(false) || name.to_lowercase().contains("test");
        let archetype = item["tags"]
            .as_array()
            .and_then(|tags| {
                tags.iter()
                    .filter_map(V::as_str)
                    .find_map(|s| s.strip_prefix("Item.Archetype."))
            })
            .unwrap_or("Other");
        let class = if test {
            "Test / vehicle"
        } else {
            match archetype {
                "AR" => "Assault rifles",
                "SMG" => "SMGs",
                "LMG" => "LMGs",
                "DMR" => "DMRs",
                "Sniper" => "Snipers",
                "Pistol" => "Pistols",
                "Shotgun" => "Shotguns",
                "Launcher" => "Launchers",
                "Bow" => "Bows",
                s => s,
            }
        };
        let mut parts = BTreeMap::<String, V>::new();
        let mut slots = vec![];
        for c in compat
            .iter()
            .filter(|r| r["weapon_id"].eq_ignore_ascii_case(key))
        {
            let slot = match c["slot"].as_str() {
                "Muzzle" => "muzzle",
                "Sight" => "optic",
                "Underbarrel" => "grip",
                "Magazine" => "magazine",
                "Barrel" => "barrel",
                "Stock" => "stock",
                "Grip" => "pistolgrip",
                "Handguard" => "handguard",
                _ => continue,
            };
            let id = c["item_id"].to_lowercase();
            let Some(a) = attachments.get(&id) else {
                continue;
            };
            let p = parts
                .entry(id.clone())
                .or_insert_with(|| part(&items[&id], a));
            p["slot"] = json!(c["slot"]);
            let fits = p["compatible_slots"].as_array_mut().unwrap();
            if !fits.contains(&json!(slot)) {
                fits.push(json!(slot));
            }
            if c["is_default"].eq_ignore_ascii_case("true") {
                p["is_default"] = json!(true)
            }
            if !slots.contains(&slot) {
                slots.push(slot);
            }
        }
        let mut rows = vec![];
        for r in damages
            .iter()
            .filter(|r| r["weapon_id"].eq_ignore_ascii_case(key))
        {
            let hit = num(r.get("damage_per_hit"))?;
            let rpm = num(r.get("rpm"))?;
            let dps = num(r.get("raw_dps"))?;
            if let (Some(h), Some(r), Some(d)) = (hit.as_f64(), rpm.as_f64(), dps.as_f64())
                && (d - h * r / 60.).abs() > 0.011
            {
                return Err("Published DPS mismatch".into());
            }
            rows.push(json!({"projectile":r["projectile"],"damage":hit,"raw_dps":dps}));
        }
        let calibers = array(w, "calibers")?;
        let base = if calibers.len() == 1 {
            format!("item.caliber.{}", string(&calibers[0]).to_lowercase())
        } else {
            String::new()
        };
        let baseline = rows
            .iter()
            .find(|r| string(&r["projectile"]).eq_ignore_ascii_case(&base));
        let projectile = ammo
            .iter()
            .find(|a| string(&a["id"]).eq_ignore_ascii_case(&base));
        let comparable = baseline.is_some()
            && projectile.is_some_and(|p| p.get("explosive").is_none())
            && !test
            && !["Shotgun", "Launcher", "Bow"].contains(&archetype);
        let pattern = w["recoil"]["pattern"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let first = pattern.iter().take(10).cloned().collect::<Vec<_>>();
        let recoil = if first.len() == 10 {
            Some(
                first
                    .iter()
                    .map(|p| {
                        p["x"]
                            .as_f64()
                            .unwrap_or(f64::NAN)
                            .hypot(p["y"].as_f64().unwrap_or(f64::NAN))
                    })
                    .sum::<f64>()
                    / 10.,
            )
        } else {
            None
        };
        let weapon = json!({"id":w["id"],"name":if key=="id.item.ak74m"{"AK-74"}else{name},"class_name":class,"test_record":test,"unlock":item["unlock"],"unlock_free":item["unlock"].is_null()&&!test,"rpm":w["rpm"],"accuracy":w["accuracy"],"ads_time":w["adsTime"],"fire_modes":w["fireModes"].as_array().cloned().unwrap_or_default(),"recoil_index":recoil,"damage_per_hit":baseline.map(|b|b["damage"].clone()),"projectile":baseline.map(|b|b["projectile"].clone()),"comparable":comparable,"damage_rows":rows,"falloff":w["damageFalloff"].as_array().cloned().unwrap_or_default(),"recoil_pattern":first});
        notes.push(json!({"weapon":w["id"],"note":"Downloaded revision: community muzzle supplement is not carried across revisions. Only source-linked mounts are used."}));
        refs.push(json!({"schema_version":1,"game_build":build,"local_build_at_import":"25937608","imported_on":imported,"source_commit":revision,"source_url":"https://github.com/warpuppies/wardogs-data","weapon":weapon,"slots":slots,"items":parts.into_values().collect::<Vec<_>>() }));
    }
    let value = json!({"schema_version":2,"game_build":build,"imported_on":imported,"source_commit":revision,"source_url":"https://github.com/warpuppies/wardogs-data","sources":sources,"weapons":refs,"all_attachments":all,"coverage_notes":notes});
    let json = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    Catalog::parse(&json)?;
    Ok(json)
}
fn download(client: &reqwest::blocking::Client, url: &str) -> Result<Vec<u8>, String> {
    let mut response = client
        .get(url)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("Download failed: {e}"))?;
    let mut b = vec![];
    response
        .by_ref()
        .take(15_000_001)
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    if b.len() > 15_000_000 {
        return Err("Download too large".into());
    }
    Ok(b)
}
pub fn check(current_json: String) -> Result<Option<Candidate>, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(25))
        .connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("WardogCompanion/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())?;
    check_with_client(current_json, &client)
}
fn check_with_client(
    current_json: String,
    client: &reqwest::blocking::Client,
) -> Result<Option<Candidate>, String> {
    let current = Catalog::parse(&current_json)?;
    let commit: V =
        serde_json::from_slice(&download(client, &format!("{REPO}/commits?per_page=1"))?)
            .map_err(|e| e.to_string())?;
    let revision = commit[0]["sha"]
        .as_str()
        .ok_or("Cannot find a source revision")?;
    if revision == current.source_commit {
        return Ok(None);
    }
    if revision.len() != 40 || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid remote revision".into());
    }
    let mut days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs()
        / 86400;
    let mut year = 1970u32;
    let leap = |year: u32| {
        year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
    };
    loop {
        let span = if leap(year) { 366 } else { 365 };
        if days < span {
            break;
        }
        days -= span;
        year += 1;
    }
    let mut month = 1;
    for span in [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ] {
        if days < span {
            break;
        }
        days -= span;
        month += 1;
    }
    let imported = format!("{year:04}-{month:02}-{:02}", days + 1);
    let mut raw = BTreeMap::new();
    let manifest = download(
        client,
        &format!(
            "https://raw.githubusercontent.com/warpuppies/wardogs-data/{revision}/data/MANIFEST.json"
        ),
    )?;
    let parsed: V = serde_json::from_slice(&manifest).map_err(|e| e.to_string())?;
    let manifest_files = array(&parsed, "files")?;
    if FILES.iter().all(|file| {
        current.sources.iter().any(|s| {
            s.path == *file
                && manifest_files
                    .iter()
                    .any(|m| m["path"] == format!("data/{file}") && m["sha256"] == s.sha256)
        })
    }) {
        return Ok(None);
    }
    raw.insert("MANIFEST.json".into(), manifest);
    for file in FILES {
        raw.insert(file.to_string(),download(client,&format!("https://raw.githubusercontent.com/warpuppies/wardogs-data/{revision}/data/{file}"))?);
    }
    let json = build(&raw, revision, &imported)?;
    let next = Catalog::parse(&json)?;
    let before: V = serde_json::from_str(&current_json).map_err(|e| e.to_string())?;
    let after: V = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    let changed = array(&after, "weapons")?
        .iter()
        .filter(|r| {
            array(&before, "weapons")
                .unwrap()
                .iter()
                .find(|w| w["weapon"]["id"] == r["weapon"]["id"])
                .is_none_or(|w| w["weapon"] != r["weapon"] || w["items"] != r["items"])
        })
        .count();
    let removed = current
        .weapons
        .iter()
        .filter(|r| next.weapon(&r.weapon.id).is_none())
        .count();
    let summary = format!(
        "Build {} → {} · {} changed/new and {} removed weapon records · {} → {} attachments. Source-only mounts: unreviewed community muzzle links are removed. Revision {}.",
        current.game_build,
        next.game_build,
        changed,
        removed,
        current.all_attachments.len(),
        next.all_attachments.len(),
        &revision[..8]
    );
    Ok(Some(Candidate { json, summary }))
}
pub fn active_json() -> Result<String, String> {
    if Catalog::path().exists() {
        std::fs::read_to_string(Catalog::path()).map_err(|e| e.to_string())
    } else {
        Ok(include_str!("../data/catalog.json").into())
    }
}
pub fn backup_path(path: &Path) -> std::path::PathBuf {
    path.with_extension("json.previous")
}
pub fn activate(path: &Path, candidate: &str, current: &str) -> Result<(), String> {
    Catalog::parse(candidate)?;
    Catalog::parse(current)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let backup = backup_path(path);
    atomic_write(&backup, current)?;
    atomic_write(path, candidate)
}
fn atomic_write(path: &Path, json: &str) -> Result<(), String> {
    let tmp = path.with_extension("json.download.tmp");
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}
pub fn rollback(path: &Path) -> Result<(), String> {
    let previous = std::fs::read_to_string(backup_path(path)).map_err(|e| e.to_string())?;
    Catalog::parse(&previous)?;
    atomic_write(path, &previous)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn connection_failure_returns_no_candidate_and_keeps_active_data() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let client = reqwest::blocking::Client::builder()
            .proxy(reqwest::Proxy::all(format!("http://{addr}")).unwrap())
            .timeout(Duration::from_millis(700))
            .connect_timeout(Duration::from_millis(300))
            .build()
            .unwrap();
        let current = include_str!("../data/catalog.json").to_string();
        let before = current.clone();
        assert!(
            check_with_client(current.clone(), &client)
                .unwrap_err()
                .contains("Download failed")
        );
        assert_eq!(current, before);
        assert!(Catalog::parse(&current).is_ok());
    }
    fn fixture() -> BTreeMap<String, Vec<u8>> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("research/wardogs-data");
        std::iter::once("MANIFEST.json")
            .chain(FILES)
            .map(|f| {
                let mut p = root.join(f.replace('/', "-"));
                if !p.exists() && f == "csv/weapon-compatibility.csv" {
                    p = root.join("weapon-compatibility.csv");
                }
                (f.into(), std::fs::read(p).unwrap())
            })
            .collect()
    }
    #[test]
    fn source_import_matches_primary_records_and_rejects_corruption() {
        let mut raw = fixture();
        let bundled = Catalog::bundled();
        let output = build(&raw, &bundled.source_commit, "test").unwrap();
        let data = Catalog::parse(&output).unwrap();
        assert_eq!(data.weapons.len(), 40);
        assert_eq!(data.all_attachments.len(), 298);
        for r in &data.weapons {
            let old = bundled.weapon(&r.weapon.id).unwrap();
            assert_eq!(r.weapon.damage_per_hit, old.weapon.damage_per_hit);
            match (r.weapon.recoil_index, old.weapon.recoil_index) {
                (Some(a), Some(b)) => assert!((a - b).abs() < 1e-12),
                (a, b) => assert_eq!(a, b),
            }
            assert_eq!(r.weapon.comparable, old.weapon.comparable);
        }
        raw.get_mut("items.json").unwrap().push(32);
        assert!(build(&raw, &bundled.source_commit, "test").is_err());
    }
    #[test]
    fn valid_checksums_do_not_bypass_schema_build_or_duplicate_checks() {
        for duplicate in [false, true] {
            let mut raw = fixture();
            let file = "items.json";
            let mut item: V = serde_json::from_slice(&raw[file]).unwrap();
            if duplicate {
                let first = item["items"][0].clone();
                item["items"].as_array_mut().unwrap().push(first);
            } else {
                item["gameBuild"] = json!("wrong");
            }
            raw.insert(file.into(), serde_json::to_vec(&item).unwrap());
            let hash = format!("{:x}", Sha256::digest(&raw[file]));
            let mut m: V = serde_json::from_slice(&raw["MANIFEST.json"]).unwrap();
            for f in m["files"].as_array_mut().unwrap() {
                if f["path"] == "data/items.json" {
                    f["sha256"] = json!(hash);
                }
            }
            raw.insert("MANIFEST.json".into(), serde_json::to_vec(&m).unwrap());
            assert!(build(&raw, &Catalog::bundled().source_commit, "test").is_err());
        }
    }
    #[test]
    fn activation_preserves_backup_and_rejection_never_overwrites_active() {
        let dir = std::env::temp_dir().join(format!("wardog-update-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("catalog.json");
        let original = include_str!("../data/catalog.json");
        std::fs::write(&p, original).unwrap();
        assert!(activate(&p, "{}", original).is_err());
        assert_eq!(std::fs::read_to_string(&p).unwrap(), original);
        let candidate = original.replace("2026-10-25", "2026-10-26");
        activate(&p, &candidate, original).unwrap();
        assert_eq!(std::fs::read_to_string(backup_path(&p)).unwrap(), original);
        std::fs::write(backup_path(&p), "{}").unwrap();
        assert!(rollback(&p).is_err());
        assert_eq!(std::fs::read_to_string(&p).unwrap(), candidate);
        std::fs::write(backup_path(&p), original).unwrap();
        rollback(&p).unwrap();
        assert_eq!(std::fs::read_to_string(&p).unwrap(), original);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
