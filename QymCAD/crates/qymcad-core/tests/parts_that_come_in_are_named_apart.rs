//! A PART THAT COMES IN IS TOLD APART FROM ONE ALREADY HERE.
//!
//! Reported behaviour: bringing a part in from a file, pasting one and importing a format that names its parts gave a
//! second "Part 1" in the tree, two rows with nothing to tell them apart.
use qymcad_core::model::Project;

/// What a person reads: the key of the catalogue this document names its first part by reads "Part 1".
fn shown(stored: &str) -> String {
    stored.replace("name-part-n#1", "Part 1")
}

#[test]
fn a_part_read_as_one_already_here_is_numbered() {
    let mut p = Project::default();
    p.new_document();
    let own = p.components.iter().find(|c| c.name.starts_with("name-part-n")).map(|c| c.name.clone()).expect("the first part");
    assert_eq!(shown(&own), "Part 1", "setup: the first part reads Part 1");
    let root = p.root;
    p.set_active_component(Some(root));
    let came = p.add_part("Part 1"); // the words of a file, beside the key of the document
    p.name_apart(&[came], &shown);
    let name = p.components.iter().find(|c| c.id == came).map(|c| c.name.clone()).unwrap_or_default();
    assert_eq!(name, "Part 1 (2)", "the part from the file reads as the one already here");
}

#[test]
fn a_copy_is_told_from_its_original() {
    let mut p = Project::default();
    let part = p.new_document();
    let copy = p.clone_component(part, p.root).expect("the copy");
    let (a, b) = (p.components.iter().find(|c| c.id == part).map(|c| c.name.clone()), p.components.iter().find(|c| c.id == copy).map(|c| c.name.clone()));
    assert_ne!(a, b, "the copy carries the name of its original");
}
