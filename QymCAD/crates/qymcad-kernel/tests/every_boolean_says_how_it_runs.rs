//! EVERY BOOLEAN IN THE KERNEL GOES THROUGH THE ONE PLACE THAT SETS THE FLAG.
//!
//! `BRepAlgoAPI_*` and `BOPAlgo_*` run on one core unless told otherwise, and the flag has to be set BEFORE
//! the work: written as an expression - `BRepAlgoAPI_Cut(a, b).Shape()` - a boolean cannot take it at all,
//! because its constructor has already finished. Measured on the reference documents: with the flag the
//! scenario document rebuilds in 17.2 s against 20.3 s.
//!
//! A ninth boolean written the old way would be slower and nothing would say so - the picture is right, the
//! numbers are right, only the time is worse. Hence a check that reads the source.

/// EVERY C++ FILE OF THE WHOLE TREE, as text - not the three the bridge happens to have today.
///
/// A guard rooted at the crate it lives in reports full coverage of a shrinking part of the tree and never
/// goes red about it: a boolean written in a new file, or in another crate, would simply not be looked at.
/// The walk starts at the crates directory and asserts it found the files it knows about.
fn sources() -> Vec<(String, String)> {
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("the crates directory");
    let mut stack: Vec<std::path::PathBuf> = vec![crates.to_path_buf()];
    let mut out: Vec<(String, String)> = Vec::new();
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(&dir).expect("the tree reads").flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().is_none_or(|x| x != "cpp" && x != "hpp") {
                continue;
            }
            let name = p.strip_prefix(crates).unwrap_or(&p).to_string_lossy().replace('\\', "/");
            out.push((name, std::fs::read_to_string(&p).unwrap_or_default()));
        }
    }
    assert!(out.len() >= 4, "the walk found {} C++ files, which is fewer than the tree has - it is looking in the wrong place", out.len());
    assert!(out.iter().any(|(n, _)| n.ends_with("occt_bridge.cpp")), "the bridge itself was not found by the walk");
    out
}

/// Is this line a comment (the rules below are about code, and the comments talk about booleans a great deal).
fn is_comment(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with("*") || t.starts_with("/*")
}

#[test]
fn no_boolean_is_built_by_the_constructor_alone() {
    let mut caught: Vec<String> = Vec::new();
    for (name, text) in sources() {
        for (n, line) in text.lines().enumerate() {
            if is_comment(line) {
                continue;
            }
            // `BRepAlgoAPI_Cut algo(a, b);` and `BRepAlgoAPI_Cut(a, b).Shape()`: in both the work is done by
            // the constructor, with no chance to ask for several cores.
            let Some(at) = line.find("BRepAlgoAPI_") else { continue };
            let rest = &line[at..];
            let Some(open) = rest.find('(') else { continue };
            let head = &rest[..open];
            // a declaration with no arguments (`BRepAlgoAPI_Cut algo;`) has no bracket right after the name
            if head.contains(' ') && !head.trim_end().ends_with(char::is_alphanumeric) {
                continue;
            }
            // the one exception: a section is told NOT to run in its constructor, and takes the flag after
            if rest.contains("Standard_False") {
                continue;
            }
            caught.push(format!("{name}:{}: {}", n + 1, line.trim()));
        }
    }
    assert!(caught.is_empty(), "a boolean is performed by its constructor, so it cannot be given the parallel flag:\n{}", caught.join("\n"));
}

#[test]
fn every_boolean_object_is_configured() {
    let mut caught: Vec<String> = Vec::new();
    for (name, text) in sources() {
        let lines: Vec<&str> = text.lines().collect();
        for (n, line) in lines.iter().enumerate() {
            if is_comment(line) {
                continue;
            }
            // a declaration: `BRepAlgoAPI_Fuse algo;` or `BOPAlgo_Builder gf(heap);`
            let Some(at) = line.find("BRepAlgoAPI_").or_else(|| line.find("BOPAlgo_")) else { continue };
            let rest = &line[at..];
            let Some(var) = rest.split_whitespace().nth(1).map(|v| v.split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("")) else { continue };
            if var.is_empty() || !var.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
                continue; // not a variable name: a type in a signature, a template argument, a mention
            }
            // within the next dozen lines the algorithm must be handed to the place that sets the flag
            let window = lines[n..(n + 14).min(lines.len())].join("\n");
            let configured = [format!("qym_configure({var})"), format!("qym_boolean({var}"), format!("qym_boolean_many({var}")].iter().any(|w| window.contains(w.as_str()));
            if !configured {
                caught.push(format!("{name}:{}: {}", n + 1, line.trim()));
            }
        }
    }
    assert!(caught.is_empty(), "a boolean is built without going through `qym_configure` or `qym_boolean`, so it runs on one core:\n{}", caught.join("\n"));
}

/// AND THE FLAG IS SET IN ONE PLACE ONLY: scattered `SetRunParallel` calls are how the eight booleans came to
/// be without it in the first place - each site deciding for itself, and nobody deciding for the ninth.
#[test]
fn the_flag_is_set_in_one_place() {
    // over the whole tree: the count is of the WORKING lines, and it has to come to exactly one
    let mut places: Vec<String> = Vec::new();
    for (name, text) in sources() {
        for (n, line) in text.lines().enumerate() {
            if !is_comment(line) && line.contains("SetRunParallel") {
                places.push(format!("{name}:{}: {}", n + 1, line.trim()));
            }
        }
    }
    assert_eq!(places.len(), 1, "the parallel flag is set in {} places instead of one:\n{}", places.len(), places.join("\n"));
    assert!(places[0].contains("occt_common.hpp"), "the one place that sets the flag is no longer the shared helper: {}", places[0]);
}
