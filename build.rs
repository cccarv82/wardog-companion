use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=assets/wardog.ico");
    println!("cargo:rerun-if-env-changed=RC");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    let icon = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("assets/wardog.ico");
    let script = out.join("wardog.rc");
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let numeric = format!("{},0", version.replace('.', ","));
    fs::write(
        &script,
        format!(
            "1 ICON \"{}\"\n1 VERSIONINFO\nFILEVERSION {numeric}\nPRODUCTVERSION {numeric}\nFILEOS 0x40004\nFILETYPE 1\nBEGIN\n BLOCK \"StringFileInfo\"\n BEGIN\n BLOCK \"040904B0\"\n BEGIN\n VALUE \"CompanyName\", \"Carlos Carvalho and contributors\\0\"\n VALUE \"FileDescription\", \"WARDOGS Companion\\0\"\n VALUE \"FileVersion\", \"{version}\\0\"\n VALUE \"ProductName\", \"WARDOGS Companion\\0\"\n VALUE \"ProductVersion\", \"{version}\\0\"\n VALUE \"OriginalFilename\", \"wardog-companion.exe\\0\"\n END\n END\n BLOCK \"VarFileInfo\"\n BEGIN\n VALUE \"Translation\", 0x409, 1200\n END\nEND\n",
            icon.display().to_string().replace('\\', "/"),
        ),
    )
    .expect("Write icon resource script");
    let compiler = env::var_os("RC")
        .map(PathBuf::from)
        .or_else(|| {
            let kits = PathBuf::from(env::var_os("ProgramFiles(x86)")?).join("Windows Kits/10/bin");
            let mut versions: Vec<_> = fs::read_dir(kits)
                .ok()?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .collect();
            versions.sort();
            versions
                .into_iter()
                .rev()
                .map(|p| p.join("x64/rc.exe"))
                .find(|p| p.is_file())
        })
        .unwrap_or_else(|| PathBuf::from("rc.exe"));
    let resource = out.join("wardog.res");
    let status = Command::new(compiler)
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg(script)
        .status()
        .expect("Windows SDK resource compiler is required; alternatively set RC");
    assert!(status.success(), "Icon resource compilation failed");
    println!("cargo:rustc-link-arg={}", resource.display());
}
