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
    let sdk = PathBuf::from(env::var("OM9_OCCT_SDK").unwrap_or_else(|_| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../build/standalone-deps/occt-8.0.1-sdk"
        )
        .into()
    }));
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
        .std("c++20")
        .include(&include)
        .file("src/bridge.cpp")
        .file("src/brep_bridge.cpp")
        .file("src/creation_bridge.cpp")
        .file("src/extrusion_bridge.cpp")
        .file("src/editing_bridge.cpp")
        .file("src/transform_bridge.cpp")
        .file("src/intersect_bridge.cpp")
        // OCCT hides M_PI under strict C++17 and its headers use std::min/std::max.
        .define("_USE_MATH_DEFINES", None)
        .define("NOMINMAX", None)
        .define("WIN32_LEAN_AND_MEAN", None)
        .warnings(false);
    let brep_3dm = env::var_os("CARGO_FEATURE_3DM_BREP").is_some();
    let mut opennurbs_library = PathBuf::new();
    let mut opennurbs_zlib = PathBuf::new();
    if brep_3dm {
        let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
        let root = manifest
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root");
        let opennurbs_include = env::var("OM9_OPENNURBS_INCLUDE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| root.join("build/standalone-deps/opennurbs"));
        opennurbs_library = env::var("OM9_OPENNURBS_LIB")
            .map(PathBuf::from)
            .unwrap_or_else(|_| root.join("build/native-brep/out/opennurbs/Release"));
        opennurbs_zlib = env::var("OM9_OPENNURBS_ZLIB")
            .map(PathBuf::from)
            .unwrap_or_else(|_| opennurbs_library.join("../zlib/Release"));
        assert!(
            opennurbs_library.join("opennurbsStatic.lib").exists(),
            "opennurbsStatic.lib not found at {}; build build/native-brep first or set OM9_OPENNURBS_LIB",
            opennurbs_library.display()
        );
        assert!(
            opennurbs_zlib.join("zlib.lib").exists(),
            "zlib.lib not found at {}; set OM9_OPENNURBS_ZLIB",
            opennurbs_zlib.display()
        );
        let legacy = manifest.join("src/legacy3dm");
        for file in [
            "ThreeDmBrep.cpp",
            "ThreeDmSurfaces.cpp",
            "ThreeDmSolidShells.cpp",
            "legacy_validation.cpp",
            "brep_curve_adapter.cpp",
            "brep_3dm_bridge.cpp",
        ] {
            build.file(legacy.join(file));
        }
        build
            .include(&opennurbs_include)
            .include(&legacy)
            .define("ON_CMAKE_BUILD", None);
        println!("cargo:rerun-if-changed={}", legacy.display());
        println!("cargo:rerun-if-env-changed=OM9_OPENNURBS_INCLUDE");
        println!("cargo:rerun-if-env-changed=OM9_OPENNURBS_LIB");
        println!("cargo:rerun-if-env-changed=OM9_OPENNURBS_ZLIB");
    }
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        build.flag("/EHa");
    }
    build.compile("om9_occt_bridge");

    println!("cargo:rustc-link-search=native={}", library.display());
    // Modeling, topology, mesh and boolean toolkits the bridge exercises.
    for toolkit in [
        "TKernel",
        "TKMath",
        "TKG2d",
        "TKG3d",
        "TKGeomBase",
        "TKBRep",
        "TKTopAlgo",
        "TKGeomAlgo",
        "TKPrim",
        "TKBO",
        "TKBool",
        "TKMesh",
        "TKFillet",
        "TKOffset",
        "TKShHealing",
    ] {
        println!("cargo:rustc-link-lib=dylib={toolkit}");
    }
    if brep_3dm {
        println!("cargo:rustc-link-lib=dylib=TKShHealing");
        println!(
            "cargo:rustc-link-search=native={}",
            opennurbs_library.display()
        );
        println!(
            "cargo:rustc-link-search=native={}",
            opennurbs_zlib.display()
        );
        println!("cargo:rustc-link-lib=static=opennurbsStatic");
        println!("cargo:rustc-link-lib=static=zlib");
        for library in ["Shlwapi", "Usp10", "shell32", "advapi32", "Gdi32", "User32"] {
            println!("cargo:rustc-link-lib=dylib={library}");
        }
    }
    // GetProcessMemoryInfo, used by the lifetime probe.
    println!("cargo:rustc-link-lib=dylib=psapi");
    println!("cargo:rerun-if-env-changed=OCCT_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=OCCT_LIB_DIR");
    println!("cargo:rerun-if-changed=src/bridge.cpp");
    println!("cargo:rerun-if-changed=src/brep_bridge.cpp");
    println!("cargo:rerun-if-changed=src/brep_snapshot.hxx");
    println!("cargo:rerun-if-changed=src/creation_bridge.cpp");
    println!("cargo:rerun-if-changed=src/extrusion_bridge.cpp");
    println!("cargo:rerun-if-changed=src/editing_bridge.cpp");
    println!("cargo:rerun-if-changed=src/transform_bridge.cpp");
    println!("cargo:rerun-if-changed=src/intersect_bridge.cpp");
}
