// SPDX-License-Identifier: LGPL-2.1-or-later
//! Menu-binding acceptance for `OM9-L2-03-16`: one stable command ID per documented
//! Curve Tools command, the authored icon of that row, and parity between the Curve menu
//! label and the documented Matrix command script. A row may never be reachable from only
//! one entry point, and the icon palette may never map a row to a different command.
use om9_app::{
    curve_menu::{self, BindingState},
    icons, menu_catalog,
    session::{Controller, InvocationOrigin},
};
use om9_core::{Error, budget::Limits, model::Document};

#[cfg(feature = "occt-brep")]
#[test]
fn running_curve_menu_commands_retain_their_actual_feature() {
    use om9_core::{geometry::Geometry, model::Entity};
    let mut doc = Document::new("curve completion routing");
    let mut ids = Vec::new();
    for y in [0., 5., 10.] {
        let curve = om9_core::curve_build::polyline(&[[0., y, 0.], [10., y, 0.]]).unwrap();
        let entity = Entity::new("curve", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    for point in [[0., 0., 1.], [5., 5., 1.], [10., 0., 1.], [15., 5., 1.]] {
        let entity = Entity::new("point", Geometry::Point(point), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let surface = om9_core::surface_kernel::plane(
        [-20., -20., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
        40.,
        40.,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("surface", Geometry::Surface(surface), None);
    ids.push(entity.id);
    doc.entities.insert(entity.id, entity);
    for (start, end) in [([0., 0., 0.], [5., 0., 0.]), ([5., 0., 0.], [10., 0., 0.])] {
        let entity = Entity::new(
            "connected",
            Geometry::Curve(om9_core::geometry::Curve::Line {
                start,
                end,
                domain: [0., 1.],
            }),
            None,
        );
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let cylinder = om9_core::surface_kernel::cylinder(
        [0., 0., 0.],
        [0., 1., 0.],
        [0., 0., 1.],
        10.,
        20.,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("sideways cylinder", Geometry::Surface(cylinder), None);
    ids.push(entity.id);
    doc.entities.insert(entity.id, entity);
    let brep = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box {
            x_mm: 10.,
            y_mm: 20.,
            z_mm: 30.,
        },
        om9_kernel::creation::DEFAULT_DEFLECTION_MM,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("box", Geometry::Brep(brep), None);
    ids.push(entity.id);
    doc.entities.insert(entity.id, entity);
    let mesh = om9_core::geometry::Mesh {
        vertices: vec![
            [-20., -20., 0.],
            [20., -20., 0.],
            [20., 20., 0.],
            [-20., 20., 0.],
        ],
        faces: vec![vec![0, 1, 2], vec![0, 2, 3]],
        normals: vec![[0., 0., 1.]; 4],
        colors: vec![],
        corner_uvs: vec![],
    };
    let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    ids.push(entity.id);
    doc.entities.insert(entity.id, entity);
    for x in [2., 7.] {
        let curve = om9_core::curve_build::polyline(&[[x, -5., 0.], [x, 15., 0.]]).unwrap();
        let entity = Entity::new("crossing", Geometry::Curve(curve), None);
        ids.push(entity.id);
        doc.entities.insert(entity.id, entity);
    }
    let cylinder = om9_core::surface_kernel::cylinder(
        [0., 0., -10.],
        [1., 0., 0.],
        [0., 1., 0.],
        10.,
        20.,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("cylinder", Geometry::Surface(cylinder), None);
    ids.push(entity.id);
    doc.entities.insert(entity.id, entity);
    let selections: &[&[usize]] = &[
        &[],
        &[0],
        &[0, 1],
        &[0, 1, 2],
        &[3, 4],
        &[3, 4, 5, 6],
        &[7],
        &[0, 7],
        &[0, 1, 7],
        &[7, 3, 4, 5, 6],
        &[12],
        &[12, 3, 4, 5],
        &[0, 1, 13, 14],
        &[15],
        &[8, 9],
        &[10],
        &[11],
    ];
    let mut running = Vec::new();
    let mut immediate = Vec::new();
    let mut missing_fixture = Vec::new();
    for binding in curve_menu::BINDINGS {
        let mut started = false;
        for selection in selections {
            let mut controller = Controller::new(doc.clone(), Limits::default()).unwrap();
            controller
                .select_objects(&selection.iter().map(|&i| ids[i]).collect::<Vec<_>>())
                .unwrap();
            if controller
                .dispatch(binding.feature, InvocationOrigin::Menu)
                .is_ok()
            {
                started = true;
                if controller.is_running() {
                    assert_eq!(
                        controller.running_feature(),
                        Some(binding.feature),
                        "{}",
                        binding.label
                    );
                    running.push(binding.label);
                } else {
                    immediate.push(binding.label);
                }
                break;
            }
        }
        if !started {
            missing_fixture.push(binding.label);
        }
    }
    eprintln!("running={running:?} immediate={immediate:?} missing_fixture={missing_fixture:?}");
    assert!(
        missing_fixture.is_empty(),
        "missing usable fixtures: {missing_fixture:?}"
    );
    assert!(
        running.len() >= 30,
        "unexpected loss of running Curve sessions"
    );
}

#[test]
fn every_documented_curve_menu_row_has_one_stable_id_and_icon() {
    assert_eq!(curve_menu::BINDINGS.len(), 58);
    assert_eq!(menu_catalog::CURVE.len(), 58);
    let mut l3 = std::collections::BTreeSet::new();
    let mut features = std::collections::BTreeSet::new();
    let mut labels = std::collections::BTreeSet::new();
    let mut icons_seen = std::collections::BTreeSet::new();
    for (index, binding) in curve_menu::BINDINGS.iter().enumerate() {
        assert!(l3.insert(binding.l3_id), "duplicate L3 {}", binding.l3_id);
        assert!(
            features.insert(binding.feature),
            "duplicate feature {}",
            binding.feature
        );
        assert!(
            labels.insert(binding.label.to_ascii_lowercase()),
            "duplicate label {}",
            binding.label
        );
        assert!(
            icons_seen.insert(binding.icon),
            "duplicate icon {}",
            binding.icon
        );
        assert_eq!(
            binding.l3_id,
            format!("OM9-L3-03-16-{:03}", index + 1),
            "L3 order"
        );
        // The Curve top menu is the authored MainMenu.ini list, in that exact order.
        assert_eq!(menu_catalog::CURVE[index], binding.icon, "menu order");
        assert!(
            menu_catalog::CURVE.contains(&binding.icon),
            "{} is not in the authored Curve palette",
            binding.icon
        );
        // The icon palette and the menu row must name the same command.
        assert_eq!(
            icons::feature_for(binding.icon),
            Some(binding.feature),
            "{} palette feature",
            binding.icon
        );
        assert_eq!(
            curve_menu::by_l3(binding.l3_id).map(|row| row.feature),
            Some(binding.feature)
        );
        assert_eq!(
            curve_menu::by_icon(binding.icon).map(|row| row.feature),
            Some(binding.feature)
        );
        assert_eq!(
            curve_menu::by_feature(binding.feature).map(|row| row.l3_id),
            Some(binding.l3_id)
        );
        assert_eq!(
            curve_menu::by_label(binding.label).map(|row| row.feature),
            Some(binding.feature)
        );
        assert!(!binding.label.is_empty());
        assert!(!binding.source_command.is_empty());
        assert!(!binding.owner.is_empty());
    }
    assert_eq!(l3.len(), 58);
    assert_eq!(features.len(), 58);
}

#[test]
fn every_row_dispatches_to_its_session() {
    for binding in curve_menu::BINDINGS {
        assert_eq!(
            binding.state,
            BindingState::Implemented,
            "{}",
            binding.l3_id
        );
        assert!(binding.enabled(), "{}", binding.l3_id);
        assert_eq!(binding.disabled_reason, "", "{}", binding.l3_id);
        assert!(
            Controller::availability(binding.feature).is_ok(),
            "{}",
            binding.feature
        );
        assert!(curve_menu::enabled(binding.feature));
        assert_eq!(curve_menu::disabled_reason(binding.feature), None);
    }
}

/// The menu button and the command line both enter through `dispatch`, so a feature that
/// `availability` accepts must also have a real dispatch arm. A fresh controller per row
/// keeps one row's pending session from masking the next row's wiring.
#[test]
fn every_row_reaches_the_dispatch_entry_from_the_menu() {
    for binding in curve_menu::BINDINGS {
        let mut controller =
            Controller::new(Document::new("curve menu dispatch"), Limits::default()).unwrap();
        if let Err(Error::Unsupported(reason)) =
            controller.dispatch(binding.feature, InvocationOrigin::Menu)
        {
            panic!("{}: {reason}", binding.feature);
        }
    }
}

/// `(menu label, stable feature id, documented Matrix script, script feature)`.
///
/// The label is what the Curve menu shows and what a user can type; the script is the
/// Matrix8 command the row is documented from. The two builder rows share a documented
/// script with their plain row (`_Crv2View`, `_Sketch _OnMesh`), so their script resolves
/// to the plain feature while the visible label resolves to the builder row.
type DocumentedRow = (&'static str, &'static str, &'static str, &'static str);
const DOCUMENTED: [DocumentedRow; 58] = [
    ("PolyLine", "OM9-CURVE-001", "Polyline", "OM9-CURVE-001"),
    ("Line", "OM9-CURVE-002", "Line", "OM9-CURVE-002"),
    (
        "Interp Curve",
        "OM9-CURVE-003",
        "InterpCrv",
        "OM9-CURVE-003",
    ),
    ("Rectangle", "OM9-CURVE-004", "Rectangle", "OM9-CURVE-004"),
    ("Circle", "OM9-CURVE-005", "Circle", "OM9-CURVE-005"),
    ("Ellipse", "OM9-CURVE-006", "Ellipse", "OM9-CURVE-006"),
    ("Arc", "OM9-CURVE-007", "Arc", "OM9-CURVE-007"),
    (
        "Arc Direction",
        "OM9-CURVE-008",
        "Arc_StartPoint",
        "OM9-CURVE-008",
    ),
    ("Rebuild", "OM9-CURVE-009", "Rebuild", "OM9-CURVE-009"),
    (
        "Refit to Tolerance",
        "OM9-CURVE-010",
        "FitCrv",
        "OM9-CURVE-010",
    ),
    (
        "Quick Curve Blend",
        "OM9-CURVE-011",
        "Blend",
        "OM9-CURVE-011",
    ),
    (
        "Adjustable Curve Blend",
        "OM9-CURVE-012",
        "BlendCrv",
        "OM9-CURVE-012",
    ),
    (
        "Curve from 2 Views",
        "OM9-CURVE-013",
        "Crv2View",
        "OM9-CURVE-013",
    ),
    (
        "Curve 2 Views with History",
        "OM9-CURVE-014",
        "Crv2View",
        "OM9-CURVE-013",
    ),
    ("Pullback", "OM9-CURVE-015", "Pull", "OM9-CURVE-015"),
    ("Project", "OM9-CURVE-016", "Project", "OM9-CURVE-016"),
    (
        "Intersection",
        "OM9-CURVE-017",
        "Intersect",
        "OM9-CURVE-017",
    ),
    ("Offset Curve", "OM9-CURVE-018", "Offset", "OM9-CURVE-018"),
    (
        "Offset Curve on Surface",
        "OM9-CURVE-020",
        "OffsetCrvOnSrf",
        "OM9-CURVE-020",
    ),
    (
        "Extract Isocurve",
        "OM9-CURVE-019",
        "ExtractIsocurve",
        "OM9-CURVE-019",
    ),
    ("Fillet Curves", "OM9-CURVE-021", "Fillet", "OM9-CURVE-021"),
    (
        "Fillet Corners",
        "OM9-CURVE-023",
        "FilletCorners",
        "OM9-CURVE-023",
    ),
    (
        "Chamfer Curves",
        "OM9-CURVE-022",
        "Chamfer",
        "OM9-CURVE-022",
    ),
    (
        "Create UV Curves",
        "OM9-CURVE-025",
        "CreateUVCrv",
        "OM9-CURVE-025",
    ),
    (
        "Apply UV Curves",
        "OM9-CURVE-024",
        "ApplyCrv",
        "OM9-CURVE-024",
    ),
    (
        "Through Points",
        "OM9-CURVE-026",
        "CurveThroughPt",
        "OM9-CURVE-026",
    ),
    ("Extend Curve", "OM9-CURVE-027", "Extend", "OM9-CURVE-027"),
    ("Polygon", "OM9-CURVE-028", "Polygon", "OM9-CURVE-028"),
    ("Spiral", "OM9-CURVE-029", "Spiral", "OM9-CURVE-029"),
    ("Helix", "OM9-CURVE-030", "Helix", "OM9-CURVE-030"),
    ("Point", "OM9-CURVE-034", "Point", "OM9-CURVE-034"),
    (
        "Mark Curve Start",
        "OM9-CURVE-033",
        "CrvStart",
        "OM9-CURVE-033",
    ),
    ("Mark Curve End", "OM9-CURVE-032", "CrvEnd", "OM9-CURVE-032"),
    (
        "Adjust Closed Curve Seam",
        "OM9-CURVE-031",
        "CrvSeam",
        "OM9-CURVE-031",
    ),
    (
        "Continue Interpolated Curve",
        "OM9-CURVE-035",
        "ContinueInterpCrv",
        "OM9-CURVE-035",
    ),
    ("Divide Curve", "OM9-CURVE-036", "Divide", "OM9-CURVE-036"),
    (
        "Curve Boolean",
        "OM9-CURVE-037",
        "CurveBoolean",
        "OM9-CURVE-037",
    ),
    (
        "Tween Curves",
        "OM9-CURVE-038",
        "TweenCurves",
        "OM9-CURVE-038",
    ),
    (
        "Duplicate Edge",
        "OM9-CURVE-040",
        "DupEdge",
        "OM9-CURVE-040",
    ),
    (
        "Duplicate Border",
        "OM9-CURVE-039",
        "DupBorder",
        "OM9-CURVE-039",
    ),
    ("Curve", "OM9-CURVE-042", "Curve", "OM9-CURVE-042"),
    (
        "Continue Control Point Curve",
        "OM9-CURVE-041",
        "ContinueCurve",
        "OM9-CURVE-041",
    ),
    (
        "Extract Angle Isocurve",
        "OM9-CURVE-044",
        "gvExtractIsocurve",
        "OM9-CURVE-044",
    ),
    (
        "Cross Section Profiles",
        "OM9-CURVE-043",
        "CSec",
        "OM9-CURVE-043",
    ),
    ("Sketch", "OM9-CURVE-045", "Sketch", "OM9-CURVE-045"),
    (
        "Sketch on Surface",
        "OM9-CURVE-046",
        "SketchOnSurface",
        "OM9-CURVE-046",
    ),
    (
        "Sketch on Mesh",
        "OM9-CURVE-049",
        "SketchOnMesh",
        "OM9-CURVE-049",
    ),
    (
        "Polyline on Surface",
        "OM9-CURVE-048",
        "PolylineOnSurface",
        "OM9-CURVE-048",
    ),
    (
        "Interp Curve on Surface",
        "OM9-CURVE-047",
        "InterpCrvOnSrf",
        "OM9-CURVE-047",
    ),
    (
        "Curve to Lines",
        "OM9-CURVE-050",
        "Convert",
        "OM9-CURVE-050",
    ),
    ("Section", "OM9-CURVE-051", "Section", "OM9-CURVE-051"),
    ("Match", "OM9-CURVE-052", "Match", "OM9-CURVE-052"),
    ("Silhouette", "OM9-CURVE-053", "Silhouette", "OM9-CURVE-053"),
    (
        "Extract Wire Frame",
        "OM9-CURVE-054",
        "ExtractWireframe",
        "OM9-CURVE-054",
    ),
    ("Soft Edit", "OM9-CURVE-055", "SoftEditCrv", "OM9-CURVE-055"),
    (
        "Offset Normal to Surface",
        "OM9-CURVE-057",
        "OffsetNormal",
        "OM9-CURVE-057",
    ),
    ("Arc Blend", "OM9-CURVE-056", "ArcBlend", "OM9-CURVE-056"),
    (
        "Intersect Two Sets",
        "OM9-CURVE-058",
        "IntersectTwoSets",
        "OM9-CURVE-058",
    ),
];

#[test]
fn menu_label_and_documented_script_resolve_to_the_same_command() {
    for (label, feature, script, script_feature) in DOCUMENTED {
        let binding = curve_menu::by_label(label).unwrap_or_else(|| panic!("no row for {label}"));
        assert_eq!(binding.feature, feature, "row feature for {label}");
        assert_eq!(
            Controller::resolve_alias(label),
            Some(feature),
            "typing the menu label {label}"
        );
        assert_eq!(
            Controller::resolve_alias(script),
            Some(script_feature),
            "documented script {script}"
        );
        assert!(
            Controller::availability(binding.feature).is_ok(),
            "{}",
            binding.feature
        );
    }
}

#[test]
fn line_and_point_commands_still_create_geometry_through_the_same_ids() {
    let mut controller = Controller::new(Document::new("curve menu"), Limits::default()).unwrap();
    controller.submit("Line").unwrap();
    controller.submit("0,0,0").unwrap();
    controller.submit("3,4,0").unwrap();
    assert_eq!(controller.document().entities.len(), 1);
    let undo = controller.coordinator().undo_len();
    assert_eq!(undo, 1);
    controller.submit("Point").unwrap();
    controller.submit("1,2,3").unwrap();
    assert_eq!(controller.document().entities.len(), 2);
    assert_eq!(controller.coordinator().undo_len(), 2);
}
