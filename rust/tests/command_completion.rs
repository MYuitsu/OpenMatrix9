use openmatrix9_rust::command_completion::{names, rank};
#[test]
fn original_names_and_match_order() {
    assert!(names().contains(&"Polygon"));
    assert!(names().contains(&"Polyline"));
    for name in ["ShowGrid", "CommandHistory", "F6", "Osnap E"] {
        assert!(names().contains(&name));
    }
    assert_eq!(rank("Polyline", "polyline"), 1);
    assert_eq!(rank("Polygon", "Pol"), 2);
    assert_eq!(rank("gvPolylineOnSurface", "pol"), 3);
    assert_eq!(rank("Polyline", "_pol"), 2);
    assert_eq!(rank("Line", "poly"), 0);
    for input in ["", "3,4,0", "Length=5", "1", "print()", "中文"] {
        assert_eq!(rank("Polyline", input), 0);
    }
}
