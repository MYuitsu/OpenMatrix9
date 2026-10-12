use openmatrix9_rust::fit_point_input::{admit_count, om9_fit_point_count, om9_fit_point_valid};

#[test]
fn existing_capacity_including_empty_selection_is_preserved() {
    for (used, added) in [(0,0),(0,1024),(1023,1),(1024,0)] {
        assert!(admit_count(used,added), "{used}+{added}");
        assert_eq!(om9_fit_point_count(used,added),0);
    }
}
#[test]
fn cumulative_counts_and_overflow_reject_before_extraction() {
    for (used, added) in [(0,1025),(1024,1),(1025,0),(usize::MAX,1),(1,usize::MAX)] {
        assert!(!admit_count(used,added));
        assert_eq!(om9_fit_point_count(used,added),2);
    }
}
#[test]
fn both_existing_families_keep_their_coordinate_bounds() {
    for kind in [1,2] {
        assert_eq!(om9_fit_point_valid(kind,1e9,-1e9,0.),0);
        assert_eq!(om9_fit_point_valid(kind,1e9+1.,0.,0.),3);
    }
    assert_eq!(om9_fit_point_valid(3,0.,0.,0.),1);
}
#[test]
fn nonfinite_coordinates_are_distinct_from_range_errors() {
    for kind in [1,2] {
        for value in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY] {
            assert_eq!(om9_fit_point_valid(kind,value,0.,0.),4);
        }
    }
}
