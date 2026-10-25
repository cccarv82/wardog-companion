//! Opt-in application releases, separate from the game-data update pipeline.
use reqwest::blocking::Client;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

const REPO: &str = "https://api.github.com/repos/cccarv82/wardog-companion";
const DOWNLOADS: &str = "https://github.com/cccarv82/wardog-companion/releases/download/";
const MAX_INSTALLER: u64 = 128 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Release {
    pub version: String,
    pub notes: String,
    url: String,
    digest: String,
    size: u64,
}

fn version(raw: &str) -> Result<[u32; 3], String> {
    let components: Vec<_> = raw.split('.').collect();
    if components.len() != 3
        || components.iter().any(|c| {
            c.is_empty()
                || !c.bytes().all(|b| b.is_ascii_digit())
                || (c.len() > 1 && c.starts_with('0'))
        })
    {
        return Err("Unsupported release version.".into());
    }
    Ok([
        components[0].parse().map_err(|_| "Invalid version")?,
        components[1].parse().map_err(|_| "Invalid version")?,
        components[2].parse().map_err(|_| "Invalid version")?,
    ])
}

fn parse(value: &Value, current: &str) -> Result<Option<Release>, String> {
    if value["draft"].as_bool() != Some(false) || value["prerelease"].as_bool() != Some(false) {
        return Err("Only published stable releases can be installed.".into());
    }
    let tag = value["tag_name"].as_str().ok_or("Missing release tag")?;
    let next = tag.strip_prefix('v').ok_or("Unsupported release tag")?;
    if version(next)? <= version(current)? {
        return Ok(None);
    }
    let name = format!("WARDOG-Companion-{next}-Setup.exe");
    let assets = value["assets"].as_array().ok_or("Missing release assets")?;
    let matches: Vec<_> = assets
        .iter()
        .filter(|a| a["name"].as_str() == Some(&name))
        .collect();
    if matches.len() != 1 {
        return Err("This release does not contain one supported Windows installer.".into());
    }
    let asset = matches[0];
    let url = asset["browser_download_url"]
        .as_str()
        .ok_or("Missing installer URL")?;
    if url != format!("{DOWNLOADS}{tag}/{name}") {
        return Err("Installer is outside the official release.".into());
    }
    let digest = asset["digest"]
        .as_str()
        .and_then(|d| d.strip_prefix("sha256:"))
        .ok_or("GitHub did not provide an installer SHA-256 digest.")?;
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid installer digest.".into());
    }
    let size = asset["size"]
        .as_u64()
        .filter(|n| *n > 0 && *n <= MAX_INSTALLER)
        .ok_or("Installer size is outside the supported limit.")?;
    Ok(Some(Release {
        version: next.into(),
        notes: value["body"]
            .as_str()
            .unwrap_or("See the release notes on GitHub.")
            .chars()
            .take(3000)
            .collect(),
        url: url.into(),
        digest: digest.to_lowercase(),
        size,
    }))
}

fn client() -> Result<Client, String> {
    Client::builder()
        .https_only(true)
        .user_agent(concat!("WARDOG-Companion/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|e| e.to_string())
}

pub fn check() -> Result<Option<Release>, String> {
    let response = client()?
        .get(format!("{REPO}/releases/latest"))
        .send()
        .map_err(|e| format!("Could not reach GitHub. {e}"))?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err("No stable release is published yet. Your app is unchanged.".into());
    }
    let mut bytes = Vec::new();
    response
        .error_for_status()
        .map_err(|e| format!("Release check failed. {e}"))?
        .take(256_001)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 256_000 {
        return Err("Release metadata exceeds the size limit.".into());
    }
    parse(
        &serde_json::from_slice(&bytes).map_err(|e| e.to_string())?,
        env!("CARGO_PKG_VERSION"),
    )
}

fn valid_file(path: &Path, release: &Release) -> Result<(), String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() != release.size {
        return Err("Installer size mismatch.".into());
    }
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
    if format!("{:x}", hasher.finalize()) != release.digest {
        return Err("Installer checksum mismatch. Nothing was installed.".into());
    }
    Ok(())
}

pub fn download(release: Release) -> Result<PathBuf, String> {
    let root =
        std::env::var_os("LOCALAPPDATA").ok_or("Windows local application folder unavailable")?;
    let folder = PathBuf::from(root)
        .join("WardogCompanion/Updates")
        .join(&release.version);
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    download_to(&client()?, &release, &folder)
}

fn download_to(client: &Client, release: &Release, folder: &Path) -> Result<PathBuf, String> {
    let target = folder.join(format!("WARDOG-Companion-{}-Setup.exe", release.version));
    if target.exists() {
        valid_file(&target, release)?;
        return Ok(target);
    }
    let temporary = folder.join(format!("download-{}.part", std::process::id()));
    let mut output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        let response = client
            .get(&release.url)
            .send()
            .and_then(|r| r.error_for_status())
            .map_err(|e| format!("Download failed. {e}"))?;
        if response.content_length().is_some_and(|n| n != release.size) {
            return Err("Installer size mismatch.".into());
        }
        let count = std::io::copy(&mut response.take(release.size + 1), &mut output)
            .map_err(|e| e.to_string())?;
        if count != release.size {
            return Err("Incomplete or oversized installer.".into());
        }
        output.flush().map_err(|e| e.to_string())?;
        output.sync_all().map_err(|e| e.to_string())?;
        drop(output);
        valid_file(&temporary, release)?;
        fs::rename(&temporary, &target).map_err(|e| e.to_string())?;
        Ok(target)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub fn install(path: &Path, release: &Release) -> Result<(), String> {
    valid_file(path, release)?;
    #[cfg(windows)]
    {
        std::process::Command::new(path)
            .arg("/NORESTART")
            .spawn()
            .map_err(|e| format!("Could not start Setup. {e}"))?;
        Ok(())
    }
    #[cfg(not(windows))]
    Err("Application installation is supported on Windows only.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn fixture() -> Value {
        json!({"tag_name":"v1.1.0","draft":false,"prerelease":false,"body":"Release notes", "assets":[{"name":"WARDOG-Companion-1.1.0-Setup.exe","browser_download_url":format!("{DOWNLOADS}v1.1.0/WARDOG-Companion-1.1.0-Setup.exe"),"digest":format!("sha256:{:x}",Sha256::digest(b"fixture")),"size":7}]})
    }
    #[test]
    fn stable_versions_are_compared_numerically() {
        assert!(parse(&fixture(), "1.0.0").unwrap().is_some());
        assert!(parse(&fixture(), "1.1.0").unwrap().is_none());
        assert!(parse(&fixture(), "1.10.0").unwrap().is_none());
        for bad in ["1.0", "01.0.0", "1.0.0-beta", "../1.0.0"] {
            assert!(version(bad).is_err());
        }
    }
    #[test]
    fn rejects_untrusted_or_ambiguous_release_assets() {
        for key in ["draft", "prerelease"] {
            let mut v = fixture();
            v[key] = json!(true);
            assert!(parse(&v, "1.0.0").is_err());
        }
        let mut v = fixture();
        v["assets"][0]["browser_download_url"] = json!("https://example.com/setup.exe");
        assert!(parse(&v, "1.0.0").is_err());
        let mut v = fixture();
        v["assets"][0]["digest"] = Value::Null;
        assert!(parse(&v, "1.0.0").is_err());
        let mut v = fixture();
        v["assets"][0]["size"] = json!(MAX_INSTALLER + 1);
        assert!(parse(&v, "1.0.0").is_err());
        let mut v = fixture();
        let duplicate = v["assets"][0].clone();
        v["assets"].as_array_mut().unwrap().push(duplicate);
        assert!(parse(&v, "1.0.0").is_err());
    }
    #[test]
    fn cached_installer_is_rechecked_before_use() {
        let folder =
            std::env::temp_dir().join(format!("wardog-installer-test-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        let path = folder.join("fixture.bin");
        let release = parse(&fixture(), "1.0.0").unwrap().unwrap();
        fs::write(&path, b"fixture").unwrap();
        assert!(valid_file(&path, &release).is_ok());
        fs::write(&path, b"corrupt").unwrap();
        assert!(valid_file(&path, &release).is_err());
        fs::write(&path, b"short").unwrap();
        assert!(valid_file(&path, &release).is_err());
        fs::remove_file(path).unwrap();
        fs::remove_dir(folder).unwrap();
    }
    #[test]
    fn downloads_are_atomic_and_failures_leave_no_executable() {
        use std::net::TcpListener;
        let folder =
            std::env::temp_dir().join(format!("wardog-download-test-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        let client = Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();
        for (index, response) in [
            "HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\ncorrupt",
            "HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\nshort",
            "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            "HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\nfixture",
        ]
        .into_iter()
        .enumerate()
        {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let server = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut request = [0; 4096];
                assert!(stream.read(&mut request).unwrap() > 0);
                stream.write_all(response.as_bytes()).unwrap();
            });
            let mut release = parse(&fixture(), "1.0.0").unwrap().unwrap();
            release.url = format!("http://{address}/installer");
            let result = download_to(&client, &release, &folder);
            server.join().unwrap();
            if index < 3 {
                assert!(result.is_err());
                assert_eq!(fs::read_dir(&folder).unwrap().count(), 0);
            } else {
                let path = result.unwrap();
                assert_eq!(fs::read(&path).unwrap(), b"fixture");
                assert!(download_to(&client, &release, &folder).is_ok());
                fs::write(&path, b"corrupt").unwrap();
                assert!(download_to(&client, &release, &folder).is_err());
                fs::remove_file(path).unwrap();
            }
        }
        fs::remove_dir(folder).unwrap();
    }
}
