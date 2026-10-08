use openmatrix9_rust::edit::{Kind, Session};
#[test]
fn om9_solid_001_two_sets_and_duplicates() {
    let mut s = Session::new(Kind::Difference);
    assert!(!s.finish());
    assert!(s.add("A"));
    assert!(s.finish());
    assert_eq!(s.phase, 2);
    assert!(!s.add("A"));
    assert!(!s.finish());
    assert!(s.add("B"));
    assert!(s.finish());
    assert_eq!(s.phase, 3);
    assert!(!s.add("C"));
    assert!(!s.delete_input);
}
#[test]
fn om9_solid_004_exactly_two_and_five_modes() {
    let mut s = Session::new(Kind::TwoObjects);
    s.add("A");
    assert!(!s.finish());
    s.add("B");
    assert!(!s.add("C"));
    assert!(s.finish());
    for mode in [1, 2, 3, 4, 0] {
        assert!(s.cycle());
        assert_eq!(s.cycle, mode);
    }
}
#[test]
fn om9_top11_selection_undo() {
    for k in [Kind::Join, Kind::Trim, Kind::Union] {
        let mut s = Session::new(k);
        assert!(s.add("A"));
        assert!(s.undo_selection());
        assert!(!s.finish());
        s.add("A");
        assert!(!s.finish());
        s.add("B");
        assert!(s.finish());
        assert_eq!(s.phase, if k == Kind::Trim { 4 } else { 3 });
    }
}
#[test]
fn om9_top11_005_single_explode_input() {
    let mut s = Session::new(Kind::Explode);
    assert!(s.add("Wire"));
    assert!(s.finish());
    assert!(s.delete_input);
}
