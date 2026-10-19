//! Links the pinned openNURBS static library and compiles the 3DM bridge.

use std::{env, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=src/bridge.cpp");
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let root = manifest.parent().and_then(|path| path.parent()).expect("workspace root");
    let include = env::var("OM9_OPENNURBS_INCLUDE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| root.join("build/standalone-deps/opennurbs"));
    let lib = env::var("OM9_OPENNURBS_LIB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| root.join("build/native-brep/out/opennurbs/Release"));
    if !lib.join("opennurbsStatic.lib").exists() {
        panic!(
            "opennurbsStatic.lib not found at {}; build build/native-brep first or set OM9_OPENNURBS_LIB",
            lib.display()
        );
    }
    cc::Build::new()
        .cpp(true)
        .file("src/bridge.cpp")
        .include(&include)
        .define("NOMINMAX", None)
        .define("ON_CMAKE_BUILD", None)
        .flag_if_supported("/EHa")
        .flag_if_supported("/utf-8")
        .compile("om9_3dm_bridge");
    println!("cargo::rustc-link-search=native={}", lib.display());
    println!("cargo::rustc-link-lib=static=opennurbsStatic");
    // openNURBS was configured with its bundled zlib and asks for these Windows libraries itself.
    let zlib = env::var("OM9_OPENNURBS_ZLIB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| lib.parent().unwrap_or(&lib).join("zlib/Release"));
    if zlib.join("zlib.lib").exists() {
        println!("cargo::rustc-link-search=native={}", zlib.display());
        println!("cargo::rustc-link-lib=static=zlib");
    }
    println!("cargo::rustc-link-lib=dylib=Shlwapi");
    println!("cargo::rustc-link-lib=dylib=Usp10");
    println!("cargo::rustc-link-lib=dylib=psapi");
    println!("cargo::rustc-link-lib=dylib=shell32");
    println!("cargo::rustc-link-lib=dylib=advapi32");
    println!("cargo::rerun-if-env-changed=OM9_OPENNURBS_LIB");
    println!("cargo::rerun-if-env-changed=OM9_OPENNURBS_INCLUDE");
}
