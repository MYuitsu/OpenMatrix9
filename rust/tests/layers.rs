use openmatrix9_rust::layers::{self, LayerError, LayerSnapshot, Selection};

fn snapshot(index: i32) -> LayerSnapshot {
    LayerSnapshot {
        index,
        exists: true,
        locked: false,
        visible: true,
        color: layers::default_color(index).unwrap(),
    }
}
#[test]
fn stable_layer_indexes_cover_all_sidebar_slots() {
    let mut names = std::collections::HashSet::new();
    for index in 1..=32 {
        assert!(names.insert(layers::name(index).unwrap()));
        assert!(layers::valid_color(layers::default_color(index).unwrap()));
        assert_eq!(snapshot(index).output_plan().unwrap().index, index);
    }
    assert_eq!(layers::name(5), Some("Gem 01"));
    assert_eq!(layers::name(21), Some("User 25"));
    for index in [i32::MIN, -1, 0, 33, i32::MAX] {
        assert!(layers::default_color(index).is_none());
        assert!(layers::name(index).is_none());
    }
}
#[test]
fn typed_selection_is_bounded_explicit_and_unambiguous() {
    for (text, index) in [
        ("Layer=Gem 01", 5),
        ("layer 32", 32),
        ("Layer = \"Metal 01\"", 1),
        ("Layer=None", 0),
        ("Layer=0", 0),
    ] {
        assert_eq!(layers::parse_selection(text), Selection::Index(index));
    }
    for text in [
        "Layer=Lights",
        "Layer=33",
        "Layer=-1",
        "Layer",
        "Layer=",
        "Layer=0x5",
        "Layer=5 Circle",
        "Layer=Gem 01\0",
    ] {
        assert_eq!(layers::parse_selection(text), Selection::Invalid);
    }
    assert_eq!(
        layers::parse_selection("LayerVisibility"),
        Selection::Unhandled
    );
    assert_eq!(layers::parse_selection("Circle"), Selection::Unhandled);
    assert_eq!(
        layers::parse_selection(&"Circle".repeat(100)),
        Selection::Unhandled
    );
    assert_eq!(
        layers::parse_selection(&"Layer=".repeat(100)),
        Selection::Invalid
    );
}
#[test]
fn locked_missing_invalid_layers_reject_before_commit() {
    let mut s = snapshot(5);
    s.locked = true;
    assert_eq!(s.output_plan(), Err(LayerError::Locked));
    s.exists = false;
    assert_eq!(s.output_plan(), Err(LayerError::MissingLayer));
    s.index = 33;
    assert_eq!(s.output_plan(), Err(LayerError::InvalidIndex));
    s = snapshot(5);
    s.color[1] = f64::NAN;
    assert_eq!(s.output_plan(), Err(LayerError::InvalidColor));
    for invalid in [f64::INFINITY, -0.01, 1.01] {
        s.color[1] = invalid;
        assert!(s.output_plan().is_err());
    }
}
#[test]
fn hidden_output_keeps_style_and_visibility_root_keeps_legacy_green() {
    let mut s = snapshot(3);
    s.visible = false;
    s.color = [0.2, 0.3, 0.4];
    assert_eq!(s.output_plan(), Ok(s));
    s.index = 0;
    s.locked = true;
    s.exists = false;
    s.color = [f64::NAN; 3];
    let root = s.output_plan().unwrap();
    assert!(root.visible && !root.locked && !root.exists);
    assert_eq!(root.color, layers::ROOT_COLOR);
}
#[test]
fn document_snapshots_do_not_share_selection_or_style() {
    let mut first = snapshot(5);
    let second = snapshot(8);
    first.locked = true;
    first.color = [1., 0., 0.];
    assert_eq!(first.output_plan(), Err(LayerError::Locked));
    assert_eq!(second.output_plan().unwrap().index, 8);
    assert_eq!(second.color, layers::default_color(8).unwrap());
}
#[test]
fn ffi_rejects_null_oversize_and_invalid_utf8_and_agrees_with_policy() {
    unsafe {
        assert_eq!(layers::om9_layer_parse_selection(std::ptr::null(), 0), -1);
        assert_eq!(
            layers::om9_layer_parse_selection(b"Layer=5".as_ptr(), usize::MAX),
            -1
        );
        assert_eq!(
            layers::om9_layer_parse_selection(b"Circle".as_ptr(), usize::MAX),
            -2
        );
        assert_eq!(layers::om9_layer_parse_selection([255u8].as_ptr(), 1), -1);
        assert_eq!(layers::om9_layer_parse_selection(b"Layer=5".as_ptr(), 7), 5);
    }
    assert_eq!(
        layers::om9_layer_output_status(5, true, true, false, 0., 0., 0.),
        LayerError::Locked as i32
    );
    assert_eq!(
        layers::om9_layer_output_status(5, true, false, false, 0., 0., 0.),
        0
    );
    assert!(layers::om9_layer_name(0).is_null());
    assert!(layers::om9_layer_default_color(32, 3).is_nan());
    assert_eq!(layers::om9_layer_root_color(1), 130. / 255.);
}
