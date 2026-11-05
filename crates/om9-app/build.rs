// SPDX-License-Identifier: LGPL-2.1-or-later
//! Embed the authored window icon as the Windows executable icon, so Explorer and the
//! taskbar show the OpenMatrix9 gem instead of the generic application icon.
//!
//! `winresource`-style crates are not vendored here; the Windows SDK resource compiler
//! is enough. Machines without the SDK keep building, just without an exe icon.
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../launcher/OpenMatrix9.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    // Debug command handlers can exceed the Windows default GUI-thread stack.
    // Reserve 8 MiB; pages are committed on demand by Windows.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg-bin=om9=/STACK:8388608");
    }
    let Some(rc) = resource_compiler() else {
        println!("cargo:warning=rc.exe not found; om9.exe keeps the default Windows icon");
        return;
    };
    let icon = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../launcher/OpenMatrix9.ico");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let script = out.join("om9.rc");
    let resource = out.join("om9.res");
    let script_text = format!("1 ICON {:?}\n", icon.display().to_string());
    if std::fs::write(&script, script_text).is_err() {
        println!("cargo:warning=could not write the icon resource script");
        return;
    }
    let status = Command::new(&rc)
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg(&script)
        .status();
    match status {
        Ok(status) if status.success() => {
            println!("cargo:rustc-link-arg-bin=om9={}", resource.display());
        }
        _ => println!("cargo:warning=rc.exe could not compile launcher/OpenMatrix9.ico"),
    }
}

/// `rc.exe` from the newest installed Windows 10/11 SDK. `ponytail:` fixed SDK layout,
/// extend the search if another toolchain ships rc.exe elsewhere.
fn resource_compiler() -> Option<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(value) = std::env::var("WindowsSdkDir") {
        roots.push(PathBuf::from(value).join("bin"));
    }
    for variable in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Ok(value) = std::env::var(variable) {
            roots.push(PathBuf::from(value).join("Windows Kits/10/bin"));
        }
    }
    let mut best: Option<(String, PathBuf)> = None;
    for root in roots {
        let Ok(versions) = std::fs::read_dir(&root) else {
            continue;
        };
        for version in versions.flatten() {
            let candidate = version.path().join("x64/rc.exe");
            if !candidate.is_file() {
                continue;
            }
            let name = version.file_name().to_string_lossy().into_owned();
            if best.as_ref().is_none_or(|(current, _)| &name > current) {
                best = Some((name, candidate));
            }
        }
    }
    best.map(|(_, path)| path)
}
