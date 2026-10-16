//! A FILE THAT IS NOT THERE IS CALLED NOT THERE, not unreadable.
//!
//! Reported behaviour: a STEP import of a path with no file behind it said "STEP: the geometry could not be read or
//! handed over", the words of a file that was there and broken, while the DXF import of a missing path said "No such
//! file or directory". A person told the geometry could not be read goes looking for a fault in the file.
use qymcad_kernel::{read_exact, read_exact_tree, ExactFormat};

#[test]
fn a_missing_step_or_iges_says_the_file_is_not_found() {
    let path = format!("{}/qymcad-no-such-file-{}.step", std::env::temp_dir().display(), std::process::id());
    assert!(!std::path::Path::new(&path).exists(), "setup: {path} exists");
    for format in [ExactFormat::Step, ExactFormat::Iges] {
        let once = read_exact(format, &path, 0.5).err().expect("a missing file reads as nothing");
        let tree = read_exact_tree(format, &path, 0.5).err().expect("a missing file reads as nothing");
        for said in [once, tree] {
            assert!(said.starts_with("cad-file-not-found#"), "{format:?}: a missing file is reported as {said:?}, not as a file not found");
            assert!(said.ends_with(&path), "{format:?}: the report {said:?} does not name the path {path}");
        }
    }
}
