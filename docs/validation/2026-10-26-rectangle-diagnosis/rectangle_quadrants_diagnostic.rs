// SPDX-License-Identifier: LGPL-2.1-or-later
//! Diagnostic only: copy to crates/om9-app/tests to run against current source.
use om9_app::session::{Controller, InvocationOrigin};
use om9_core::{budget::Limits, geometry::Geometry, model::Document};

#[test]
fn opposite_corner_is_preserved_in_all_four_quadrants() {
    let first = [3.0, 7.0, 0.0];
    let mut failures = 0;
    let mut cases = Vec::new();
    for menu in [false, true] {
        for (dx, dy) in [(10.0, 20.0), (-10.0, 20.0), (10.0, -20.0), (-10.0, -20.0)] {
            let second = [first[0] + dx, first[1] + dy, 0.0];
            let mut c = Controller::new(Document::new("rectangle diagnostic"), Limits::default()).unwrap();
            if menu {
                c.dispatch("OM9-CURVE-004", InvocationOrigin::Menu).unwrap();
            } else {
                c.submit("Rectangle").unwrap();
            }
            c.accept_world_point(first).unwrap();
            let preview = c.rect_preview(second);
            let preview_corners: Vec<_> = preview.iter().map(|edge| edge[0]).collect();
            let preview_ok = preview.len() == 4 && preview_corners.contains(&second);
            c.accept_world_point(second).unwrap();
            assert_eq!(c.document().entities.len(), 1);
            let entity = c.document().entities.values().next().unwrap();
            let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) = &entity.geometry else {
                panic!("expected exact four-line polycurve");
            };
            let committed_corners: Vec<_> = segments.iter().map(|s| match s {
                om9_core::geometry::Curve::Line { start, .. } => *start,
                _ => panic!("expected line"),
            }).collect();
            assert_eq!(segments.len(), 4);
            let commit_ok = committed_corners.contains(&second);
            let scene = c.scene().unwrap();
            assert_eq!(scene.objects.len(), 1);
            assert!(scene.objects[0].visible);
            assert_eq!(scene.epoch, c.coordinator().epoch());
            failures += usize::from(!preview_ok || !commit_ok);
            cases.push(serde_json::json!({
                "route": if menu { "menu dispatcher / world picks" } else { "CMD invocation / world picks" },
                "first": first,
                "second": second,
                "preview_corners": preview_corners,
                "committed_corners": committed_corners,
                "preview_ok": preview_ok,
                "commit_ok": commit_ok,
                "document_objects": 1,
                "scene_objects": 1,
                "scene_visible": true,
                "scene_epoch_matches_document": true,
            }));
        }
    }
    println!("RECTANGLE_DIAGNOSTIC={}", serde_json::json!({"cases": cases, "failed_cases": failures, "total_cases": 8}));
    assert_eq!(failures, 0, "opposite-corner quadrant regressions");
}

#[test]
fn typed_first_corner_is_accepted() {
    let mut c = Controller::new(Document::new("rectangle typed point"), Limits::default()).unwrap();
    c.submit("Rectangle").unwrap();
    let result = c.submit("3,7,0");
    println!("RECTANGLE_TYPED_POINT={}", serde_json::json!({
        "input": "3,7,0",
        "actual": format!("{result:?}"),
        "document_objects": c.document().entities.len(),
        "accepted": result.is_ok(),
    }));
    assert!(result.is_ok(), "typed first corner was rejected: {result:?}");
}
