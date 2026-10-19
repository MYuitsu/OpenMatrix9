//! Build the OCCT C ABI bridge.
//!
//! The pin lives in `native/occt/pin.json`; `implementation/tools/fetch_occt.py` materialises the SDK
//! from the official 8.0.1 release. Override the two directories with `OCCT_INCLUDE_DIR` and
//! `OCCT_LIB_DIR` when the SDK sits elsewhere.
//!
//! MSVC is required: OCCT's release is built with the vc14 toolset and converts access violations into
//! C++ exceptions only when the caller is compiled with `/EHa`, which is what [the bridge] relies on to
//! keep a kernel fault from taking the process down.
use std::env;
use std::path::PathBuf;

fn main() {
    let sdk = PathBuf::from(
        env::var("OM9_OCCT_SDK").unwrap_or_else(|_| {
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/standalone-deps/occt-8.0.1-sdk").into()
        }),
    );
    let include = env::var("OCCT_INCLUDE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| sdk.join("include/opencascade"));
    let library = env::var("OCCT_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| sdk.join("lib"));
    assert!(
        include.join("Standard_Version.hxx").exists(),
        "OCCT headers missing at {}; run implementation/tools/fetch_occt.py",
        include.display()
    );
    assert!(
        library.join("TKernel.lib").exists(),
        "OCCT import libraries missing at {}",
        library.display()
    );

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .include(&include)
        .file("src/bridge.cpp")
        .file("src/brep_bridge.cpp")
        // OCCT hides M_PI under strict C++17 and its headers use std::min/std::max.
        .define("_USE_MATH_DEFINES", None)
        .define("NOMINMAX", None)
        .define("WIN32_LEAN_AND_MEAN", None)
        .warnings(false);
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        build.flag("/EHa");
    }
    build.compile("om9_occt_bridge");

    println!("cargo:rustc-link-search=native={}", library.display());
    // Modeling, topology, mesh and boolean toolkits the bridge exercises.
    for toolkit in [
        "TKernel", "TKMath", "TKG2d", "TKG3d", "TKGeomBase", "TKBRep", "TKTopAlgo", "TKGeomAlgo",
        "TKPrim", "TKBO", "TKBool", "TKMesh", "TKFillet", "TKOffset",
    ] {
        println!("cargo:rustc-link-lib=dylib={toolkit}");
    }
    // GetProcessMemoryInfo, used by the lifetime probe.\n    println!("cargo:rustc-link-lib=dylib=psapi");\n    println!("cargo:rerun-if-env-changed=OCCT_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=OCCT_LIB_DIR");
    println!("cargo:rerun-if-changed=src/bridge.cpp");
    println!("cargo:rerun-if-changed=src/brep_bridge.cpp");
}
