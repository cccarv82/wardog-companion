//! On-demand Windows OCR. Always confirm matches; never infer the active gun from inventory order.
use crate::catalog::Catalog;
#[derive(Debug, Clone)]
pub struct Match {
    pub id: String,
    pub name: String,
}
fn normalize(s: &str) -> String {
    s.chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}
pub fn matches(data: &Catalog, text: &str) -> Vec<Match> {
    let phrases = text
        .lines()
        .flat_map(|line| {
            let words = line.split_whitespace().collect::<Vec<_>>();
            let mut out = vec![];
            for i in 0..words.len() {
                for n in 1..=4 {
                    if i + n <= words.len() {
                        out.push(normalize(&words[i..i + n].join("")));
                    }
                }
            }
            out
        })
        .collect::<std::collections::BTreeSet<_>>();
    data.weapons
        .iter()
        .filter(|r| !r.weapon.test_record)
        .filter(|r| {
            let name = normalize(&r.weapon.name);
            if phrases.contains(&name) {
                return true;
            }
            let aliases: &[&str] = match r.weapon.name.as_str() {
                "AK-74" => &["ak74", "ak74m"],
                "M249 SAW" => &["m249"],
                "Glock 17" => &["glock17"],
                _ => &[],
            };
            aliases.iter().any(|s| phrases.contains(*s))
        })
        .map(|r| Match {
            id: r.weapon.id.clone(),
            name: r.weapon.name.clone(),
        })
        .collect()
}
pub fn read(image: Option<String>) -> Result<String, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        struct Temporary(Option<std::path::PathBuf>);
        impl Drop for Temporary {
            fn drop(&mut self) {
                if let Some(path) = &self.0 {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
        let capture = if image.is_none() {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos();
            let path =
                std::env::temp_dir().join(format!("wardog-ocr-{}-{stamp}.png", std::process::id()));
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|e| e.to_string())?;
            Temporary(Some(path))
        } else {
            Temporary(None)
        };
        let mut command = std::process::Command::new("powershell.exe");
        command
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                include_str!("../tools/read_weapon.ps1"),
            ])
            .creation_flags(0x08000000);
        // This is a test-image path, never interpolated into executable script text.
        if let Some(path) = image {
            command.env("WARDOG_OCR_TEST_IMAGE", path);
        } else {
            command.env_remove("WARDOG_OCR_TEST_IMAGE");
            command.env("WARDOG_OCR_CAPTURE_PATH", capture.0.as_ref().unwrap());
        }
        let mut child = command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        use std::io::Read;
        let stdout = child.stdout.take().ok_or("OCR output unavailable")?;
        let stderr = child.stderr.take().ok_or("OCR error output unavailable")?;
        let reader = std::thread::spawn(move || {
            let mut bytes = vec![];
            let _ = stdout.take(131072).read_to_end(&mut bytes);
            bytes
        });
        let errors = std::thread::spawn(move || {
            let mut bytes = vec![];
            let _ = stderr.take(16384).read_to_end(&mut bytes);
            bytes
        });
        let start = std::time::Instant::now();
        loop {
            if child.try_wait().map_err(|e| e.to_string())?.is_some() {
                break;
            }
            if start.elapsed() > std::time::Duration::from_secs(35) {
                let _ = child.kill();
                let _ = child.wait();
                return Err("OCR timed out. Your selection is unchanged.".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(80));
        }
        let status = child.wait().map_err(|e| e.to_string())?;
        let stdout = reader.join().map_err(|_| "OCR output stopped")?;
        let stderr = errors.join().map_err(|_| "OCR output stopped")?;
        if !status.success() {
            let err = String::from_utf8_lossy(&stderr);
            return Err(err
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("OCR unavailable. Use manual weapon selection.")
                .chars()
                .take(240)
                .collect());
        }
        let v: serde_json::Value =
            serde_json::from_slice(&stdout).map_err(|_| "OCR returned an unreadable result")?;
        v["text"]
            .as_str()
            .map(str::to_owned)
            .ok_or("OCR did not return text".into())
    }
    #[cfg(not(windows))]
    {
        let _ = image;
        Err("Windows OCR is only available on Windows.".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_names_with_punctuation_but_never_substrings_or_test_weapons() {
        let d = Catalog::bundled();
        let m = matches(&d, "AK74\nASSAULT RIFLE\nM4A1\nSKS\nRemove from Loadout");
        assert!(m.iter().any(|r| r.name == "AK-74"));
        assert!(m.iter().any(|r| r.name == "SKS"));
        assert!(!m.iter().any(|r| r.name == "M4"));
        assert!(matches(&d, "no gun found").is_empty());
    }
}
