// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-L2-04-01 Transform menu: one transaction per command, the same body for the menu
//! row and the typed name, and a visible refusal for every row without an owner.
//!
//! The golden cases use a box whose display mesh carries the exact corners, so a
//! transform can be checked on real coordinates instead of on a claimed success.
use om9_app::session::{Controller, InvocationOrigin, transform};
use om9_app::transform_menu;
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Mesh, NativeBrep, NurbsSurface, Point3, Transform},
    model::{Document, Entity},
};

fn build(entities: Vec<Entity>) -> Controller {
    let mut document = Document::new("transform");
    for entity in entities {
        document.entities.insert(entity.id, entity);
    }
    Controller::new(document, Limits::default()).expect("controller")
}

/// A closed box as a native solid: the display mesh carries the exact corners.
fn box_solid(origin: Point3, size: Point3) -> Geometry {
    let vertex = |index: usize| -> Point3 {
        std::array::from_fn(|axis| {
            origin[axis]
                + if index & (1 << axis) == 0 {
                    0.0
                } else {
                    size[axis]
                }
        })
    };
    let vertices: Vec<Point3> = (0..8).map(vertex).collect();
    let faces = vec![
        vec![0, 1, 3],
        vec![0, 3, 2],
        vec![4, 6, 7],
        vec![4, 7, 5],
        vec![0, 4, 5],
        vec![0, 5, 1],
        vec![2, 3, 7],
        vec![2, 7, 6],
        vec![0, 2, 6],
        vec![0, 6, 4],
        vec![1, 5, 7],
        vec![1, 7, 3],
    ];
    Geometry::NativeBrep(NativeBrep {
        display: Mesh {
            vertices,
            faces,
            normals: Vec::new(),
            colors: Vec::new(),
            corner_uvs: Vec::new(),
        },
        payload: vec![1, 2, 3, 4],
        crc: 0,
    })
}

fn boxed(origin: Point3, size: Point3) -> Entity {
    Entity::new("box", box_solid(origin, size), None)
}

/// A straight path curve from `start` to `end`, the authored shape the curve rows are
/// tested on because its arc length and tangent are known exactly.
fn line(start: Point3, end: Point3) -> om9_core::geometry::Curve {
    let length =
        ((end[0] - start[0]).powi(2) + (end[1] - start[1]).powi(2) + (end[2] - start[2]).powi(2))
            .sqrt();
    om9_core::geometry::Curve::Line {
        start,
        end,
        domain: [0.0, length],
    }
}

fn path_entity(start: Point3, end: Point3) -> Entity {
    Entity::new("path", Geometry::Curve(line(start, end)), None)
}

/// A box as a plain mesh: the deform rows refuse a solid, so their geometry is measured on a
/// mesh whose vertices are the same eight corners.
fn mesh_box(origin: Point3, size: Point3) -> Entity {
    let Geometry::NativeBrep(native) = box_solid(origin, size) else {
        panic!("box_solid builds a native solid");
    };
    Entity::new("mesh", Geometry::Mesh(native.display), None)
}

/// Every display vertex of a mesh object, in world space, after a row rewrote it.
fn mesh_vertices(controller: &Controller, id: om9_core::identity::ObjectId) -> Vec<Point3> {
    let entity = controller.document().entities.get(&id).expect("entity");
    let Geometry::Mesh(mesh) = &entity.geometry else {
        panic!("expected a mesh, found {:?}", entity.geometry.kind());
    };
    mesh.vertices
        .iter()
        .map(|vertex| entity.placement.point(*vertex).expect("placement"))
        .collect()
}

/// The world-space start and end of a curve after a row has rewritten its geometry.
fn curve_ends(controller: &Controller, id: om9_core::identity::ObjectId) -> (Point3, Point3) {
    let entity = controller.document().entities.get(&id).expect("entity");
    let Geometry::Curve(curve) = &entity.geometry else {
        panic!("expected a curve, found {:?}", entity.geometry.kind());
    };
    let limits = Limits::default();
    let start = om9_core::curve_ops::point_at(curve, 0.0, &limits).expect("start");
    let end = om9_core::curve_ops::point_at(curve, 1.0, &limits).expect("end");
    (
        entity.placement.point(start).expect("placement"),
        entity.placement.point(end).expect("placement"),
    )
}

fn boxed_at(origin: Point3, size: Point3, placement: Point3) -> Entity {
    let mut entity = boxed(origin, size);
    entity.placement = om9_core::geometry::Transform::translation(placement);
    entity
}

fn origins(controller: &Controller) -> Vec<Point3> {
    let mut found: Vec<Point3> = controller
        .document()
        .entities
        .values()
        .filter(|entity| matches!(entity.geometry, Geometry::NativeBrep(_)))
        .map(|entity| {
            entity
                .placement
                .point([0.0, 0.0, 0.0])
                .expect("placement origin")
        })
        .collect();
    found.sort_by(|a, b| a[0].partial_cmp(&b[0]).expect("coordinate"));
    found
}

fn world(controller: &Controller, id: om9_core::identity::ObjectId, p: Point3) -> Point3 {
    controller
        .document()
        .entities
        .get(&id)
        .expect("entity")
        .placement
        .point(p)
        .expect("placement")
}

/// The flat reference surface the two array rows are checked on: a degree one plane over
/// 10 x 10 mm whose parameter `(u, v)` evaluates to `(10u, 10v, 0)`.
fn plane() -> NurbsSurface {
    let mut poles = Vec::new();
    for u in 0..2 {
        for v in 0..2 {
            poles.push([10.0 * u as f64, 10.0 * v as f64, 0.0]);
        }
    }
    NurbsSurface {
        degree: [1, 1],
        size: [2, 2],
        poles,
        weights: vec![1.0; 4],
        knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    }
}

/// A plane tilted along U, so its normal is `(-1, 0, 1) / sqrt 2` instead of the CPlane
/// normal: a row that only translated would pass a flat-plane test while never turning.
fn tilted() -> NurbsSurface {
    let mut poles = Vec::new();
    for u in 0..2 {
        for v in 0..2 {
            poles.push([10.0 * u as f64, 10.0 * v as f64, 10.0 * u as f64]);
        }
    }
    NurbsSurface {
        degree: [1, 1],
        size: [2, 2],
        poles,
        weights: vec![1.0; 4],
        knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    }
}

fn surface_entity(surface: NurbsSurface) -> Entity {
    Entity::new("surface", Geometry::Surface(surface), None)
}

/// The centre of the display mesh in world space, so a row that bakes the mirror into the
/// geometry and a row that only moves the placement are both measured on real corners.
fn mesh_centre(controller: &Controller, id: om9_core::identity::ObjectId) -> Point3 {
    let entity = controller.document().entities.get(&id).expect("entity");
    let vertices: Vec<Point3> = match &entity.geometry {
        Geometry::NativeBrep(native) => native
            .display
            .vertices
            .iter()
            .map(|vertex| entity.placement.point(*vertex).expect("placement"))
            .collect(),
        other => panic!("expected a native solid, found {other:?}"),
    };
    let count = vertices.len() as f64;
    vertices
        .iter()
        .fold([0.0; 3], |sum, vertex| {
            [sum[0] + vertex[0], sum[1] + vertex[1], sum[2] + vertex[2]]
        })
        .map(|sum| sum / count)
}

fn centres(controller: &Controller) -> Vec<Point3> {
    let mut found: Vec<Point3> = controller
        .document()
        .entities
        .values()
        .filter(|entity| matches!(entity.geometry, Geometry::NativeBrep(_)))
        .map(|entity| mesh_centre(controller, entity.id))
        .collect();
    found.sort_by(|a, b| {
        a[0].partial_cmp(&b[0])
            .expect("x")
            .then_with(|| a[1].partial_cmp(&b[1]).expect("y"))
    });
    found
}

fn close(actual: Point3, expected: Point3) -> bool {
    actual
        .iter()
        .zip(expected)
        .all(|(a, b)| (a - b).abs() <= 1.0e-6)
}

fn undo_len(controller: &Controller) -> usize {
    controller.coordinator().undo_len()
}

fn placement(
    controller: &Controller,
    id: om9_core::identity::ObjectId,
) -> om9_core::geometry::Transform {
    controller
        .document()
        .entities
        .get(&id)
        .expect("entity")
        .placement
}

#[test]
fn m03_04_01_001_scale_3d_scales_the_box_about_the_base_point() {
    let entity = boxed([0.0; 3], [10.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let before = undo_len(&controller);

    controller.submit("Scale").expect("Scale by name");
    assert_eq!(controller.active_feature(), Some(transform::SCALE_3D));
    assert_eq!(controller.prompt(), "Base point for scale: ");
    controller.submit("0,0,0").expect("base point");
    assert_eq!(controller.prompt(), "Scale factor: ");
    controller.submit("2").expect("factor");

    assert!(!controller.is_running());
    assert_eq!(
        world(&controller, id, [10.0, 10.0, 10.0]),
        [20.0, 20.0, 20.0],
        "the far corner is scaled x2 about the base point"
    );
    assert_eq!(
        undo_len(&controller),
        before + 1,
        "one transform is exactly one Undo entry"
    );
}

#[test]
fn m03_04_01_003_scale1d_grows_only_along_the_cplane_x() {
    let entity = boxed([0.0; 3], [10.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("Scale1D").expect("Scale1D by name");
    assert_eq!(controller.active_feature(), Some(transform::SCALE_1D));
    controller.submit("0,0,0").expect("base point");
    controller.submit("3").expect("factor");

    assert_eq!(
        world(&controller, id, [10.0, 10.0, 10.0]),
        [30.0, 10.0, 10.0]
    );
}

#[test]
fn m03_04_01_004_scale_non_uniform_applies_three_factors() {
    let entity = boxed([0.0; 3], [10.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("ScaleNU").expect("ScaleNU by name");
    assert_eq!(controller.active_feature(), Some(transform::SCALE_NU));
    controller.submit("0,0,0").expect("base point");
    controller.submit("2").expect("x factor");
    controller.submit("3").expect("y factor");
    controller.submit("4").expect("z factor");

    assert_eq!(
        world(&controller, id, [10.0, 10.0, 10.0]),
        [20.0, 30.0, 40.0]
    );
}

#[test]
fn m03_04_01_007_rotate_3d_turns_ninety_degrees_about_the_cplane_normal() {
    let entity = boxed([0.0; 3], [10.0, 0.0, 0.0]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("Rotate3D").expect("Rotate3D by name");
    assert_eq!(controller.active_feature(), Some(transform::ROTATE_3D));
    controller.submit("0,0,0").expect("center");
    assert_eq!(controller.prompt(), "Rotation angle in degrees: ");
    controller.submit("90").expect("angle");

    let point = world(&controller, id, [10.0, 0.0, 0.0]);
    assert!(point[0].abs() < 1e-9, "{point:?}");
    assert!((point[1] - 10.0).abs() < 1e-9, "{point:?}");
}

#[test]
fn m03_04_01_009_array_polar_creates_the_requested_count() {
    let entity = boxed([1.0, 0.0, 0.0], [1.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let before = undo_len(&controller);
    let source = placement(&controller, id);

    controller.submit("ArrayPolar").expect("ArrayPolar by name");
    controller.submit("0,0,0").expect("center");
    controller.submit("4").expect("count");
    controller.submit("360").expect("total angle");

    assert_eq!(
        controller.document().entities.len(),
        4,
        "count 4 means the source plus three copies"
    );
    assert_eq!(undo_len(&controller), before + 1, "one array, one Undo");
    assert_eq!(
        placement(&controller, id),
        source,
        "an array copy leaves the source where it is"
    );
    let mut angles: Vec<f64> = controller
        .document()
        .entities
        .values()
        .filter(|entity| entity.id != id)
        .map(|entity| {
            // The geometry is baked at `[1, 0, 0]`, so rotating a copy about the world
            // origin leaves its placement translation at zero and shows up as the
            // body point moving instead. Measure the body point, not the origin.
            let body = entity.placement.point([1.0, 0.0, 0.0]).expect("body point");
            body[1].atan2(body[0]).to_degrees()
        })
        .collect();
    angles.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    assert_eq!(angles.len(), 3, "three copies: {angles:?}");
    // 360 degrees over four items is a quarter turn, so every copy sits a quarter turn
    // from the next; the direction of the turn follows the CPlane normal. Compare the
    // sorted sets, because a quarter turn can land on either side of the +X axis.
    let mut expected = vec![-90.0, -180.0, -270.0];
    let normalise = |angle: f64| ((angle + 180.0).rem_euclid(360.0)) - 180.0;
    let mut angles: Vec<f64> = angles.iter().copied().map(normalise).collect();
    angles.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    expected
        .iter_mut()
        .for_each(|angle| *angle = normalise(*angle));
    expected.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    for (actual, expected) in angles.iter().zip(expected) {
        assert!(
            (actual - expected).abs() < 1e-6,
            "polar array places copies a quarter turn apart: {angles:?}"
        );
    }
}

#[test]
fn m03_04_01_026_array_creates_the_requested_grid() {
    let entity = boxed([0.0; 3], [1.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("Array").expect("Array by name");
    controller.submit("2").expect("columns");
    controller.submit("3").expect("rows");
    controller.submit("10").expect("column spacing");
    controller.submit("20").expect("row spacing");

    assert_eq!(
        controller.document().entities.len(),
        6,
        "2 x 3 includes the source, so five copies"
    );
}

#[test]
fn m03_04_01_030_set_points_edits_the_selected_point_coordinates() {
    let entity = Entity::new("point", Geometry::Point([1.0, 2.0, 3.0]), None);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("SetPt").expect("SetPt by name");
    assert_eq!(
        controller.prompt(),
        "Target coordinates for selected points: "
    );
    controller.submit("5,6,7").expect("position");

    assert_eq!(
        controller.document().entities[&id].geometry,
        Geometry::Point([5.0, 6.0, 7.0])
    );
    assert_eq!(placement(&controller, id), Transform::identity());
}

#[test]
fn m03_04_01_010_project_to_cplane_drops_each_object_onto_the_plane() {
    let low = boxed([0.0, 0.0, 0.0], [1.0; 3]);
    let high = boxed([0.0, 0.0, 10.0], [1.0; 3]);
    let (low_id, high_id) = (low.id, high.id);
    let mut controller = build(vec![low, high]);
    controller
        .select_objects(&[low_id, high_id])
        .expect("selection");
    let before = undo_len(&controller);

    controller
        .submit("ProjectToCPlane")
        .expect("ProjectToCPlane by name");
    assert!(
        !controller.is_running(),
        "the command needs no typed input and commits immediately"
    );

    assert!(world(&controller, low_id, [0.0; 3])[2].abs() < 1e-9);
    assert!(
        world(&controller, high_id, [0.0; 3])[2].abs() < 1e-9,
        "the raised box drops to the plane"
    );
    assert_eq!(undo_len(&controller), before + 1, "both drops are one edit");
}

#[test]
fn m03_04_01_038_remap_maps_the_source_plane_onto_the_cplane() {
    let entity = boxed([0.0; 3], [1.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller
        .submit("RemapCPlane")
        .expect("RemapCPlane by name");
    controller.submit("0,0,0").expect("source origin");
    controller.submit("1,0,0").expect("source x point");

    assert_eq!(
        world(&controller, id, [0.0, 0.0, 0.0]),
        [0.0, 0.0, 0.0],
        "the source plane already is the CPlane"
    );
}

#[test]
fn m03_04_01_002_scale2d_leaves_the_normal_axis_alone() {
    let entity = boxed([0.0; 3], [10.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("Scale2D").expect("Scale2D by name");
    assert_eq!(controller.active_feature(), Some(transform::SCALE_2D));
    controller.submit("0,0,0").expect("base point");
    controller.submit("2").expect("factor");

    assert_eq!(
        world(&controller, id, [10.0, 10.0, 10.0]),
        [20.0, 20.0, 10.0],
        "the construction-plane axes scale, the normal does not"
    );
}

#[test]
fn m03_04_01_005_orient_two_points_translates_the_selection() {
    let entity = boxed([0.0; 3], [1.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("Orient").expect("Orient by name");
    assert_eq!(controller.active_feature(), Some(transform::ORIENT_2));
    assert_eq!(controller.prompt(), "Base point: ");
    controller.submit("0,0,0").expect("base point");
    controller.submit("5,5,5").expect("target point");

    assert_eq!(world(&controller, id, [0.0; 3]), [5.0, 5.0, 5.0]);
}

#[test]
fn m03_04_01_006_orient_three_points_turns_the_selection() {
    let entity = boxed([0.0; 3], [1.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("Orient3Pt").expect("Orient3Pt by name");
    assert_eq!(controller.active_feature(), Some(transform::ORIENT_3));
    controller.submit("0,0,0").expect("base point");
    controller.submit("0,0,0").expect("target point");
    controller
        .submit("1,0,0")
        .expect("reference direction point");
    controller.submit("0,1,0").expect("target direction point");

    let body = world(&controller, id, [1.0, 0.0, 0.0]);
    assert!(body[0].abs() < 1e-9, "{body:?}");
    assert!((body[1] - 1.0).abs() < 1e-9, "{body:?}");
}

#[test]
fn m03_04_01_023_array_linear_spaces_the_copies_along_the_plane_x() {
    let entity = boxed([0.0; 3], [1.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let before = undo_len(&controller);

    controller
        .submit("ArrayLinear")
        .expect("ArrayLinear by name");
    assert_eq!(controller.active_feature(), Some(transform::ARRAY_LINEAR));
    controller.submit("3").expect("count");
    controller.submit("10").expect("spacing");

    assert_eq!(
        controller.document().entities.len(),
        3,
        "count 3 includes the source, so two copies"
    );
    assert_eq!(undo_len(&controller), before + 1, "one array, one Undo");
    let mut offsets: Vec<f64> = controller
        .document()
        .entities
        .values()
        .map(|entity| entity.placement.point([0.0, 0.0, 0.0]).expect("origin")[0])
        .collect();
    offsets.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    assert_eq!(offsets, vec![0.0, 10.0, 20.0]);
}

#[test]
fn m03_04_01_031_scale_by_plane_scales_along_the_plane_normal() {
    let entity = boxed([0.0; 3], [10.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller
        .submit("ScaleByPlane")
        .expect("ScaleByPlane by name");
    assert_eq!(controller.active_feature(), Some(transform::SCALE_BY_PLANE));
    controller.submit("0,0,0").expect("base point");
    controller.submit("2").expect("factor");

    assert_eq!(
        world(&controller, id, [10.0, 10.0, 10.0]),
        [10.0, 10.0, 20.0],
        "only the normal axis scales"
    );
}

#[test]
fn m03_04_01_042_every_catalogued_row_is_enabled_or_refused_with_a_reason() {
    assert_eq!(
        transform_menu::BINDINGS.len(),
        41,
        "the catalog has 41 rows"
    );
    let mut enabled = 0;
    for index in 1..=41 {
        let feature = format!("OM9-TRANSFORM-{index:03}");
        let binding = transform_menu::by_feature(&feature).expect("catalogued row");
        match Controller::availability(&feature) {
            Ok(()) => {
                enabled += 1;
                assert!(binding.enabled(), "{feature} is catalogued as disabled");
                assert!(transform::handles(&feature), "{feature} has no body");
            }
            Err(error) => {
                assert!(!binding.enabled(), "{feature} is enabled but unavailable");
                let reason = binding.disabled_reason;
                assert!(!reason.is_empty(), "{feature} must record why it is off");
                let message = format!("{error}");
                assert!(
                    message.contains(reason),
                    "{feature}: {message} does not carry {reason}"
                );
            }
        }
    }
    assert_eq!(
        enabled,
        transform::handled_rows().len(),
        "every body is catalogued"
    );
    assert_eq!(
        enabled, 39,
        "thirty-nine rows run a body; the rest stay visible and disabled"
    );
    assert_eq!(
        transform::from_alias("Scale 3-D"),
        Some(transform::SCALE_3D),
        "the authored label reaches the same row"
    );
}

/// The authored artwork and the catalog must agree: `MainMenu.ini [Menu10]` lays out the
/// Transform menu, `icons.rs` binds each authored key to a feature id, and
/// `transform_menu::BINDINGS` is what the command layer answers. A row whose key is not in
/// the authored menu, or an authored icon bound to the wrong feature, is a defect even when
/// the row is disabled.
#[test]
fn m03_04_01_042_catalog_matches_the_authored_menu_features_and_icons() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo = manifest
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root");
    let ini =
        std::fs::read_to_string(repo.join("Resources/menu/MainMenu.ini")).expect("MainMenu.ini");
    let block = ini
        .split("[Menu10]")
        .nth(1)
        .and_then(|rest| rest.split("[Menu11]").next())
        .expect("the Transform menu block");
    let authored: Vec<&str> = block
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            // `IconCount=40` is a count, not an icon row: only `Icon<number>=` names one.
            let (index, name) = line.split_once('=')?;
            let index = index.strip_prefix("Icon")?;
            index.parse::<usize>().ok()?;
            Some(name.trim())
        })
        .collect();
    assert_eq!(
        authored.len(),
        40,
        "the authored Transform menu has 40 icons"
    );

    let features: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(manifest.join("assets/features.json")).expect("features.json"),
    )
    .expect("features.json parses");
    let names: std::collections::BTreeMap<&str, &str> = features
        .as_array()
        .expect("feature list")
        .iter()
        .filter_map(|feature| {
            let id = feature.get("id")?.as_str()?;
            let name = feature.get("name")?.as_str()?;
            id.starts_with("OM9-TRANSFORM-").then_some((id, name))
        })
        .collect();
    assert_eq!(
        names.len(),
        41,
        "features.json catalogues 41 Transform rows"
    );

    for binding in transform_menu::BINDINGS {
        assert_eq!(
            names.get(binding.feature).copied(),
            Some(binding.label),
            "{} label must be the authored feature name",
            binding.feature
        );
        assert!(
            !binding.command_key.is_empty(),
            "{} has no authored key",
            binding.feature
        );
        // Two rows are newer than the authored artwork: Matrix' own menu has no
        // Orient on Curve or ReleaseFromCage icon, and inventing one is not allowed.
        let known_without_artwork =
            matches!(binding.feature, "OM9-TRANSFORM-017" | "OM9-TRANSFORM-022");
        if known_without_artwork {
            assert!(
                !authored.contains(&binding.command_key),
                "{} must stay out of the authored menu",
                binding.feature
            );
            continue;
        }
        assert!(
            authored.contains(&binding.command_key),
            "{} key `{}` is not in the authored Transform menu",
            binding.feature,
            binding.command_key
        );
        assert_eq!(
            om9_app::icons::feature_for(binding.command_key),
            Some(binding.feature),
            "the authored icon `{}` must belong to {}",
            binding.command_key,
            binding.feature
        );
    }

    // The authored menu has one row the milestone tree does not own: the SmartPattern
    // placeholder has no L3, so it stays uncatalogued instead of getting a fake command.
    let catalogued: Vec<&str> = transform_menu::BINDINGS
        .iter()
        .map(|binding| binding.command_key)
        .collect();
    let extra: Vec<&&str> = authored
        .iter()
        .filter(|name| !catalogued.contains(name))
        .collect();
    assert_eq!(
        extra,
        vec![&"TransformSmartPattern"],
        "only the unplanned placeholder is left"
    );
}

/// A menu click and a typed command must reach one body: both start the same session and
/// produce the same geometry, which is the rule the Transform toolbar is built on.
#[test]
fn m03_04_01_menu_click_and_typed_command_run_the_same_body() {
    let mut clicked = build(vec![boxed([0.0; 3], [10.0; 3])]);
    let clicked_id = *clicked.document().entities.keys().next().expect("entity");
    clicked.select_objects(&[clicked_id]).expect("selection");
    clicked
        .dispatch(transform::SCALE_3D, InvocationOrigin::Menu)
        .expect("menu click starts the same command");
    assert_eq!(clicked.active_feature(), Some(transform::SCALE_3D));
    assert_eq!(clicked.prompt(), "Base point for scale: ");
    clicked.submit("0,0,0").expect("base point");
    clicked.submit("2").expect("factor");

    let mut typed = build(vec![boxed([0.0; 3], [10.0; 3])]);
    let typed_id = *typed.document().entities.keys().next().expect("entity");
    typed.select_objects(&[typed_id]).expect("selection");
    typed.submit("Scale").expect("typed name");
    assert_eq!(typed.active_feature(), Some(transform::SCALE_3D));
    assert_eq!(typed.prompt(), "Base point for scale: ");
    typed.submit("0,0,0").expect("base point");
    typed.submit("2").expect("factor");

    assert_eq!(
        world(&clicked, clicked_id, [10.0, 10.0, 10.0]),
        world(&typed, typed_id, [10.0, 10.0, 10.0]),
        "the menu row and the typed name apply one implementation"
    );
    assert_eq!(
        undo_len(&clicked),
        undo_len(&typed),
        "and one transaction each"
    );
}

/// `OrientOnCrv`: the object stands on the path. The picked point is snapped to the curve,
/// the object's own normal becomes the curve tangent, and the run is one Undo entry.
#[test]
fn m03_04_01_016_orient_perpendicular_to_curve_stands_the_object_on_the_path() {
    let box_entity = boxed([-1.0, -1.0, 0.0], [2.0, 2.0, 2.0]);
    let box_id = box_entity.id;
    let path = path_entity([0.0; 3], [10.0, 0.0, 0.0]);
    let path_id = path.id;
    let mut controller = build(vec![box_entity, path]);
    controller
        .select_objects(&[box_id, path_id])
        .expect("selection");
    let before = undo_len(&controller);

    controller
        .dispatch(transform::ORIENT_CURVE, InvocationOrigin::Menu)
        .expect("Orient Perpendicular to Curve");
    assert_eq!(controller.prompt(), "Point on curve to orient at: ");
    // The typed point is not on the curve: it must snap to the path rather than be taken.
    controller.submit("5,2,0").expect("point on curve");

    assert!(!controller.is_running());
    let origin = placement(&controller, box_id)
        .point([0.0; 3])
        .expect("origin");
    assert!(
        (origin[0] - 5.0).abs() < 1e-9 && origin[1].abs() < 1e-9 && origin[2].abs() < 1e-9,
        "the object stands on the curve at the snapped station, got {origin:?}"
    );
    let axis = placement(&controller, box_id)
        .point([0.0, 0.0, 1.0])
        .expect("caret");
    let normal = [
        axis[0] - origin[0],
        axis[1] - origin[1],
        axis[2] - origin[2],
    ];
    assert!(
        (normal[0] - 1.0).abs() < 1e-9 && normal[1].abs() < 1e-9 && normal[2].abs() < 1e-9,
        "the object normal follows the path tangent, got {normal:?}"
    );
    assert_eq!(undo_len(&controller), before + 1, "one Undo entry");
    assert_eq!(
        world(&controller, path_id, [0.0; 3]),
        [0.0; 3],
        "the path curve is not a target and does not move"
    );
}

/// `ArrayCrv`: the copies are spaced by measured arc length along the path, and the source
/// stays at the start of the curve.
#[test]
fn m03_04_01_022_array_along_curve_spaces_copies_by_arc_length() {
    let box_entity = boxed([0.0; 3], [1.0; 3]);
    let path = path_entity([0.0; 3], [10.0, 0.0, 0.0]);
    let mut controller = build(vec![box_entity, path]);
    let box_id = *controller.document().entities.keys().next().expect("box");
    let path_id = *controller.document().entities.keys().nth(1).expect("path");
    controller
        .select_objects(&[box_id, path_id])
        .expect("selection");
    let before = undo_len(&controller);

    controller
        .dispatch(transform::ARRAY_CURVE, InvocationOrigin::Cmd)
        .expect("Array Along Curve");
    assert_eq!(controller.prompt(), "Number of items: ");
    controller.submit("3").expect("count");

    assert!(!controller.is_running());
    let found = origins(&controller);
    assert_eq!(found.len(), 3, "the box plus two copies");
    let along: Vec<f64> = found
        .iter()
        .filter(|origin| origin[1].abs() < 1e-9)
        .map(|origin| origin[0])
        .collect();
    assert_eq!(along.len(), 3, "one source and two copies along the path");
    for (station, expected) in along.iter().zip([0.0, 5.0, 10.0]) {
        assert!(
            (station - expected).abs() < 1e-6,
            "station {station} is not the measured {expected} mm point"
        );
    }
    assert_eq!(undo_len(&controller), before + 1, "one Undo entry");
}

/// A curve row is refused before it asks anything when the selection has no path, or more
/// than one path, and the refusal leaves the document untouched.
#[test]
fn m03_04_01_curve_rows_refuse_a_selection_without_exactly_one_path() {
    for feature in [transform::ORIENT_CURVE, transform::ARRAY_CURVE] {
        let only_box = boxed([0.0; 3], [1.0; 3]);
        let id = only_box.id;
        let mut controller = build(vec![only_box]);
        controller.select_objects(&[id]).expect("selection");
        let before = undo_len(&controller);
        assert!(
            controller
                .dispatch(feature, InvocationOrigin::Menu)
                .is_err(),
            "{feature} needs a path curve"
        );
        assert!(!controller.is_running());
        assert_eq!(undo_len(&controller), before);

        let box_entity = boxed([0.0; 3], [1.0; 3]);
        let first = path_entity([0.0; 3], [10.0, 0.0, 0.0]);
        let second = path_entity([0.0; 3], [0.0, 10.0, 0.0]);
        let ids = [box_entity.id, first.id, second.id];
        let mut controller = build(vec![box_entity, first, second]);
        controller.select_objects(&ids).expect("selection");
        assert!(
            controller
                .dispatch(feature, InvocationOrigin::Menu)
                .is_err(),
            "{feature} refuses two path curves"
        );
        assert!(!controller.is_running());
        assert_eq!(undo_len(&controller), before);
    }
}

/// `ProjectToCPlane` reasons per object: two boxes at different heights both land on the
/// plane, which a single shared step cannot do.
#[test]
fn m03_04_01_010_project_to_cplane_lands_objects_from_different_heights() {
    let low = boxed_at([0.0; 3], [1.0; 3], [0.0, 0.0, 5.0]);
    let high = boxed_at([0.0; 3], [1.0; 3], [0.0, 0.0, 9.0]);
    let low_id = low.id;
    let high_id = high.id;
    let mut controller = build(vec![low, high]);
    controller
        .select_objects(&[low_id, high_id])
        .expect("selection");
    let before = undo_len(&controller);

    controller
        .dispatch(transform::PROJECT_CPLANE, InvocationOrigin::Menu)
        .expect("Project to C-Plane");

    assert!(!controller.is_running());
    assert_eq!(
        placement(&controller, low_id)
            .point([0.0; 3])
            .expect("origin")[2],
        0.0,
        "the low box lands on the plane"
    );
    assert_eq!(
        placement(&controller, high_id)
            .point([0.0; 3])
            .expect("origin")[2],
        0.0,
        "the high box lands on the plane too"
    );
    assert_eq!(undo_len(&controller), before + 1, "one Undo entry");
}

#[test]
fn m03_04_01_negatives_refuse_without_changing_the_document() {
    let entity = boxed([0.0; 3], [1.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);

    // No selection: the command never starts.
    let error = controller
        .dispatch(transform::SCALE_3D, InvocationOrigin::Cmd)
        .expect_err("no selection");
    assert!(matches!(error, om9_core::Error::Missing(_)), "{error}");

    controller.select_objects(&[id]).expect("selection");

    // A zero factor is refused and the command stays open for a new value.
    controller.submit("Scale").expect("Scale by name");
    controller.submit("0,0,0").expect("base point");
    assert!(controller.submit("0").is_err(), "a zero factor is refused");
    assert_eq!(controller.prompt(), "Scale factor: ", "the run continues");
    controller.cancel();

    // A one-item array is not an array: the count is refused as it is typed, and the
    // question is asked again instead of failing after four inputs.
    controller.submit("Array").expect("Array by name");
    assert!(controller.submit("1").is_err(), "count 1 is refused");
    assert_eq!(controller.prompt(), "Number of columns: ");
    controller.cancel();

    // A row without an owner is refused with its recorded reason.
    let error = controller
        .dispatch("OM9-TRANSFORM-019", InvocationOrigin::Cmd)
        .expect_err("Solid Points ON has no owner service");
    let reason = transform_menu::by_feature("OM9-TRANSFORM-019")
        .expect("catalogued row")
        .disabled_reason;
    assert!(
        format!("{error}").contains(reason),
        "the refusal repeats the recorded reason: {error}"
    );

    // A locked object can never enter a transform, because the selection refuses it
    // first; `begin_transform` keeps the same check as defence in depth.
    controller.cancel();
    let locked = {
        let mut entity = boxed([0.0; 3], [1.0; 3]);
        entity.attributes.locked = true;
        entity
    };
    let locked_id = locked.id;
    let mut locked_controller = build(vec![locked]);
    let undo_before = undo_len(&locked_controller);
    let error = locked_controller
        .select_objects(&[locked_id])
        .expect_err("a locked object cannot be selected");
    assert!(matches!(error, om9_core::Error::Permission(_)), "{error}");
    assert_eq!(undo_len(&locked_controller), undo_before);
}

/// Symmetry mirrors the selection across the plane the two points define, and the copy
/// switch decides whether the source survives; both outcomes are one transaction.
#[test]
fn m03_04_01_018_symmetry_mirrors_across_the_picked_plane() {
    let entity = boxed([0.0; 3], [2.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let before = undo_len(&controller);

    controller.submit("Symmetry").expect("Symmetry by name");
    assert_eq!(controller.active_feature(), Some(transform::SYMMETRY));
    assert_eq!(controller.prompt(), "Mirror plane origin: ");
    controller.submit("0,0,0").expect("plane origin");
    assert_eq!(controller.prompt(), "Mirror plane normal point: ");
    controller.submit("1,0,0").expect("plane normal");
    assert_eq!(
        controller.prompt(),
        "Copy mirrored objects (0 = no, 1 = yes): "
    );
    controller.submit("0").expect("move instead of copy");

    assert_eq!(undo_len(&controller), before + 1, "one Undo entry");
    assert!(
        close(mesh_centre(&controller, id), [-1.0, 1.0, 1.0]),
        "the box is mirrored in X about the origin plane: {:?}",
        mesh_centre(&controller, id)
    );
    assert_eq!(centres(&controller).len(), 1, "moving keeps one object");

    // A copy leaves the source in place and adds exactly one mirrored object.
    let entity = boxed([0.0; 3], [2.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    controller.submit("Symmetry").expect("Symmetry by name");
    controller.submit("0,0,0").expect("plane origin");
    controller.submit("1,0,0").expect("plane normal");
    controller.submit("1").expect("copy the mirrored object");

    let found = centres(&controller);
    assert_eq!(found.len(), 2, "the copy is added next to the source");
    assert!(close(found[0], [-1.0, 1.0, 1.0]), "{found:?}");
    assert!(close(found[1], [1.0, 1.0, 1.0]), "{found:?}");
}

/// Stretch is the one-direction scale the owner service publishes: the factor moves the
/// picked direction only, and the base point stays put.
#[test]
fn m03_04_01_032_stretch_scales_along_the_picked_direction() {
    let entity = boxed([0.0; 3], [2.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let before = undo_len(&controller);

    controller.submit("Stretch").expect("Stretch by name");
    assert_eq!(controller.active_feature(), Some(transform::STRETCH));
    assert_eq!(controller.prompt(), "Base point for stretch: ");
    controller.submit("0,0,0").expect("base point");
    assert_eq!(controller.prompt(), "Stretch direction: ");
    controller.submit("1,0,0").expect("stretch direction");
    assert_eq!(controller.prompt(), "Stretch factor: ");
    controller.submit("2").expect("factor");

    assert_eq!(undo_len(&controller), before + 1, "one Undo entry");
    assert!(
        close(mesh_centre(&controller, id), [2.0, 1.0, 1.0]),
        "only the picked direction stretches: {:?}",
        mesh_centre(&controller, id)
    );
    assert!(
        close(world(&controller, id, [2.0, 2.0, 2.0]), [4.0, 2.0, 2.0]),
        "the far corner doubles in X and keeps Y and Z"
    );
}

/// Array Along Surface fills the evaluated grid of the reference surface; the surface and
/// the objects are selected together, and the source object is one of the items.
#[test]
fn m03_04_01_024_array_along_surface_places_copies_on_the_surface_grid() {
    let surface = surface_entity(plane());
    let surface_id = surface.id;
    let entity = boxed([0.0; 3], [2.0; 3]);
    let box_id = entity.id;
    let mut controller = build(vec![surface, entity]);
    controller
        .select_objects(&[surface_id, box_id])
        .expect("selection");

    let command = transform_menu::by_feature(transform::ARRAY_SURFACE)
        .expect("catalogued row")
        .command_line;
    controller.submit(command).expect("Array Along Surface");
    assert_eq!(controller.prompt(), "Number of rows: ");
    controller.submit("2").expect("rows");
    assert_eq!(controller.prompt(), "Number of columns: ");
    controller.submit("2").expect("columns");
    assert_eq!(
        controller.prompt(),
        "Orient copies to the surface normal (0 = no, 1 = yes): "
    );
    controller.submit("0").expect("no orientation");

    let found = centres(&controller);
    assert_eq!(found.len(), 4, "a 2 x 2 grid over the source object");
    let expected = [
        [1.0, 1.0, 1.0],
        [1.0, 11.0, 1.0],
        [11.0, 1.0, 1.0],
        [11.0, 11.0, 1.0],
    ];
    for (actual, target) in found.iter().zip(expected) {
        assert!(close(*actual, target), "{found:?} misses {target:?}");
    }
}

/// Array Curve on Surface projects every station of the path curve onto the surface, so the
/// items follow the curve over the surface rather than the curve's own plane.
#[test]
fn m03_04_01_025_array_curve_on_surface_projects_the_stations_onto_the_surface() {
    let surface = surface_entity(plane());
    let surface_id = surface.id;
    let path = path_entity([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let path_id = path.id;
    let entity = boxed([0.0; 3], [2.0; 3]);
    let box_id = entity.id;
    let mut controller = build(vec![surface, path, entity]);
    controller
        .select_objects(&[surface_id, path_id, box_id])
        .expect("selection");

    let command = transform_menu::by_feature(transform::ARRAY_CURVE_SURFACE)
        .expect("catalogued row")
        .command_line;
    controller.submit(command).expect("Array Curve on Surface");
    assert_eq!(controller.prompt(), "Number of items: ");
    controller.submit("3").expect("count");
    assert_eq!(
        controller.prompt(),
        "Orient copies to the surface normal (0 = no, 1 = yes): "
    );
    controller.submit("0").expect("no orientation");

    let found = centres(&controller);
    assert_eq!(found.len(), 3, "count items on the curve");
    let expected = [[1.0, 1.0, 1.0], [6.0, 1.0, 1.0], [11.0, 1.0, 1.0]];
    for (actual, target) in found.iter().zip(expected) {
        assert!(close(*actual, target), "{found:?} misses {target:?}");
    }
}

/// Orient on Surface stands the object on the projected point and turns it onto the surface
/// normal, which is what separates it from a plain move.
#[test]
fn m03_04_01_015_orient_on_surface_stands_the_object_on_the_surface() {
    let surface = surface_entity(tilted());
    let surface_id = surface.id;
    let entity = boxed([0.0; 3], [2.0; 3]);
    let box_id = entity.id;
    let mut controller = build(vec![surface, entity]);
    controller
        .select_objects(&[surface_id, box_id])
        .expect("selection");
    let before = undo_len(&controller);

    let command = transform_menu::by_feature(transform::ORIENT_SURFACE)
        .expect("catalogued row")
        .command_line;
    controller.submit(command).expect("Orient on Surface");
    assert_eq!(controller.prompt(), "Point on surface to orient at: ");
    controller.submit("5,5,5").expect("surface point");

    assert_eq!(undo_len(&controller), before + 1, "one Undo entry");
    assert!(
        close(world(&controller, box_id, [0.0; 3]), [5.0, 5.0, 5.0]),
        "the object reference point lands on the surface point: {:?}",
        world(&controller, box_id, [0.0; 3])
    );
    let placement = placement(&controller, box_id);
    let axis = {
        let origin = placement.point([0.0, 0.0, 0.0]).expect("origin");
        let tip = placement.point([0.0, 0.0, 1.0]).expect("tip");
        [tip[0] - origin[0], tip[1] - origin[1], tip[2] - origin[2]]
    };
    let normal = [1.0 / 2.0_f64.sqrt(), 0.0, -1.0 / 2.0_f64.sqrt()];
    let dot = axis[0] * normal[0] + axis[1] * normal[1] + axis[2] * normal[2];
    assert!(
        (dot.abs() - 1.0).abs() <= 1.0e-6,
        "the object's Z axis is the surface normal: {axis:?}"
    );
}

/// The reference rows refuse before the first prompt when the selection cannot supply the
/// reference, so a run never asks a question and then fails on the answer.
#[test]
fn m03_04_01_reference_rows_refuse_a_selection_without_their_reference() {
    let entity = boxed([0.0; 3], [2.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let command = transform_menu::by_feature(transform::ORIENT_SURFACE)
        .expect("catalogued row")
        .command_line;
    let error = controller
        .submit(command)
        .expect_err("no surface is selected");
    assert!(
        format!("{error}").contains("reference surface selection"),
        "{error}"
    );
    assert_eq!(
        controller.prompt(),
        "Command: ",
        "the refused row never opened a question"
    );

    // The curve-on-surface row needs both references, and says so before prompting.
    let surfaces = surface_entity(plane());
    let surface_id = surfaces.id;
    let path = path_entity([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let path_id = path.id;
    let entity = boxed([0.0; 3], [2.0; 3]);
    let box_id = entity.id;
    let mut controller = build(vec![surfaces, path, entity]);
    controller
        .select_objects(&[surface_id, path_id, box_id])
        .expect("selection");
    let command = transform_menu::by_feature(transform::ARRAY_CURVE_SURFACE)
        .expect("catalogued row")
        .command_line;
    controller.submit(command).expect("references are present");
    assert_eq!(controller.prompt(), "Number of items: ");
    controller.cancel();
}

/// Orient on Curve turns the object's X axis onto the path tangent and lands its reference
/// point on the snapped station: the tangent frame, not the perpendicular one row 016 uses.
#[test]
fn m03_04_01_017_orient_on_curve_follows_the_tangent() {
    let path = path_entity([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let path_id = path.id;
    let entity = boxed([0.0; 3], [2.0; 3]);
    let box_id = entity.id;
    let mut controller = build(vec![path, entity]);
    controller
        .select_objects(&[path_id, box_id])
        .expect("selection");

    controller.submit("Orient on Curve").expect("row by label");
    assert_eq!(controller.prompt(), "Point on curve to orient at: ");
    controller.submit("5,2,0").expect("point on the path");

    assert!(
        close(world(&controller, box_id, [0.0; 3]), [5.0, 0.0, 0.0]),
        "the reference point snaps to the path station: {:?}",
        world(&controller, box_id, [0.0; 3])
    );
    let placement = placement(&controller, box_id);
    let origin = placement.point([0.0, 0.0, 0.0]).expect("origin");
    let tip = placement.point([1.0, 0.0, 0.0]).expect("tip");
    let axis = [tip[0] - origin[0], tip[1] - origin[1], tip[2] - origin[2]];
    assert!(
        (axis[0] - 1.0).abs() <= 1.0e-6 && axis[1].abs() <= 1.0e-6 && axis[2].abs() <= 1.0e-6,
        "the local X axis follows the +X tangent: {axis:?}"
    );
}

/// Bend turns the straight path into an arc of `span / theta`: a 90 degree bend of a 10 mm
/// curve has a radius of `10 / (pi / 2)`, and the far end sits at `(R, R, 0)`.
#[test]
fn m03_04_01_008_bend_turns_the_curve_into_an_arc() {
    let entity = path_entity([0.0; 3], [10.0, 0.0, 0.0]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let before = undo_len(&controller);

    controller.submit("Bend").expect("Bend by name");
    assert_eq!(controller.active_feature(), Some(transform::BEND));
    assert_eq!(controller.prompt(), "Base point for bend: ");
    controller.submit("0,0,0").expect("base point");
    controller.submit("10,0,0").expect("bend axis");
    controller.submit("0,10,0").expect("bend direction");
    controller.submit("90").expect("bend angle");

    assert!(!controller.is_running());
    let radius = 10.0 / std::f64::consts::FRAC_PI_2;
    let (start, end) = curve_ends(&controller, id);
    assert!(close(start, [0.0; 3]), "{start:?}");
    assert!(
        close(end, [radius, radius, 0.0]),
        "the far end lands on the quarter arc: {end:?}"
    );
    assert_eq!(undo_len(&controller), before + 1);
}

/// Taper scales the section over the span: factor 3 at the far end turns a 1 mm offset into
/// 3 mm while the base stays put.
#[test]
fn m03_04_01_033_taper_scales_the_section_over_the_span() {
    let entity = path_entity([0.0, 1.0, 0.0], [10.0, 1.0, 0.0]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("Taper").expect("Taper by name");
    assert_eq!(controller.active_feature(), Some(transform::TAPER));
    assert_eq!(controller.prompt(), "Base point for taper: ");
    controller.submit("0,0,0").expect("base point");
    controller.submit("10,0,0").expect("taper axis");
    controller.submit("3").expect("taper factor");

    let (start, end) = curve_ends(&controller, id);
    assert!(close(start, [0.0, 1.0, 0.0]), "the base keeps its offset");
    assert!(close(end, [10.0, 3.0, 0.0]), "{end:?}");
}

/// Shear is affine: a factor of 0.5 slides the far end by half of its measured distance, and
/// the curve stays a line.
#[test]
fn m03_04_01_034_shear_slides_the_section_along_the_picked_direction() {
    let entity = path_entity([0.0, 1.0, 0.0], [10.0, 1.0, 0.0]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("Shear").expect("Shear by name");
    assert_eq!(controller.active_feature(), Some(transform::SHEAR));
    assert_eq!(controller.prompt(), "Base point for shear: ");
    controller.submit("0,0,0").expect("base point");
    controller.submit("10,0,0").expect("measure direction");
    controller.submit("0,10,0").expect("shear direction");
    controller.submit("0.5").expect("shear factor");

    let (start, end) = curve_ends(&controller, id);
    assert!(close(start, [0.0, 1.0, 0.0]), "the base plane is fixed");
    assert!(close(end, [10.0, 6.0, 0.0]), "{end:?}");
}

/// Twist turns the section about the axis: the offset `(0, 1, 0)` at the far end becomes
/// `(0, 0, 1)` after a quarter turn about `+X`.
#[test]
fn m03_04_01_035_twist_turns_the_section_over_the_span() {
    let entity = path_entity([0.0, 1.0, 0.0], [10.0, 1.0, 0.0]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("Twist").expect("Twist by name");
    assert_eq!(controller.active_feature(), Some(transform::TWIST));
    assert_eq!(controller.prompt(), "Base point for twist: ");
    controller.submit("0,0,0").expect("base point");
    controller.submit("10,0,0").expect("twist axis");
    controller.submit("90").expect("twist angle");

    let (_, end) = curve_ends(&controller, id);
    assert!(close(end, [10.0, 0.0, 1.0]), "{end:?}");
}

/// Maelstrom is the twist plus a radial pull: the same quarter turn, then the section shrinks
/// to half, so the far end lands at `(10, 0, 0.5)`.
#[test]
fn m03_04_01_036_maelstrom_twists_and_pulls_the_section_in() {
    let entity = path_entity([0.0, 1.0, 0.0], [10.0, 1.0, 0.0]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");

    controller.submit("Maelstrom").expect("Maelstrom by name");
    assert_eq!(controller.active_feature(), Some(transform::MAELSTROM));
    assert_eq!(controller.prompt(), "Base point for maelstrom: ");
    controller.submit("0,0,0").expect("base point");
    controller.submit("10,0,0").expect("maelstrom axis");
    controller.submit("90").expect("maelstrom angle");
    controller.submit("0.5").expect("pull");

    let (_, end) = curve_ends(&controller, id);
    assert!(close(end, [10.0, 0.0, 0.5]), "{end:?}");
}

/// Soft Move moves the geometry inside its radius and leaves the rest alone, and a mesh is
/// deformed at its vertices.
#[test]
fn m03_04_01_040_soft_move_moves_only_inside_the_falloff() {
    let entity = mesh_box([0.0; 3], [2.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let before = undo_len(&controller);

    controller.submit("SoftMove").expect("SoftMove by name");
    assert_eq!(controller.active_feature(), Some(transform::SOFT_MOVE));
    assert_eq!(controller.prompt(), "Base point for soft move: ");
    controller.submit("0,0,0").expect("base point");
    controller.submit("0,0,5").expect("displacement point");
    controller.submit("1.5").expect("radius");

    let vertices = mesh_vertices(&controller, id);
    assert!(
        vertices
            .iter()
            .any(|vertex| close(*vertex, [0.0, 0.0, 5.0])),
        "the corner at the base moves by the whole displacement: {vertices:?}"
    );
    assert_eq!(
        vertices
            .iter()
            .filter(|vertex| vertex[2] > 2.0 + 1.0e-9)
            .count(),
        1,
        "only the vertex inside the radius moves: {vertices:?}"
    );
    assert!(
        vertices
            .iter()
            .any(|vertex| close(*vertex, [2.0, 2.0, 2.0])),
        "a corner outside the radius is untouched: {vertices:?}"
    );
    assert_eq!(undo_len(&controller), before + 1);
}

/// A solid cannot be deformed without a kernel deformer, so the row refuses it instead of
/// leaving a display mesh that only looks deformed.
#[test]
fn m03_04_01_deform_rows_refuse_a_solid() {
    let entity = boxed([0.0; 3], [2.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let before = undo_len(&controller);

    controller.submit("Twist").expect("Twist by name");
    controller.submit("0,0,0").expect("base point");
    controller.submit("0,0,10").expect("twist axis");
    let error = controller.submit("90").expect_err("a solid is refused");
    assert!(
        format!("{error}").contains("solid"),
        "the refusal names the reason: {error}"
    );
    assert_eq!(undo_len(&controller), before, "a refusal changes nothing");
    assert_eq!(
        world(&controller, id, [2.0, 2.0, 2.0]),
        [2.0, 2.0, 2.0],
        "the solid stays where it was"
    );
}

/// Flow Along Curve carries every point from its station on the source path to the matching
/// station on the target path, riding rigidly with the tangent there.
#[test]
fn m03_04_01_027_flow_along_curve_carries_geometry_onto_the_target_path() {
    let source = path_entity([0.0; 3], [10.0, 0.0, 0.0]);
    let target = path_entity([0.0; 3], [0.0, 10.0, 0.0]);
    let flowed = mesh_box([5.0, 1.0, 0.0], [1.0, 2.0, 2.0]);
    let id = flowed.id;
    let ids = [source.id, target.id, id];
    let mut controller = build(vec![source, target, flowed]);
    controller.select_objects(&ids).expect("selection");
    let before = undo_len(&controller);

    controller.submit("Flow").expect("Flow by name");
    assert_eq!(controller.prompt(), "Point on the source curve: ");
    controller.submit("9,0,0").expect("source curve pick");
    assert!(!controller.is_running());

    // The station (5,0,0) of the source path lands on (0,5,0) of the target path, and the
    // minimal rotation about +Z turns the 1 mm offset (0,1,0) into (-1,0,0).
    let vertices = mesh_vertices(&controller, id);
    assert!(
        vertices
            .iter()
            .any(|vertex| close(*vertex, [-1.0, 5.0, 0.0])),
        "the near corner rides its station: {vertices:?}"
    );
    assert!(
        vertices
            .iter()
            .any(|vertex| close(*vertex, [-1.0, 6.0, 0.0])),
        "the far corner rides its own station: {vertices:?}"
    );
    assert_eq!(undo_len(&controller), before + 1);
}

/// Flow Along Surface keeps the `(u, v)` of a point on the source surface and lands it on the
/// target surface at the same parameters.
#[test]
fn m03_04_01_028_flow_along_surface_maps_the_source_parameters_onto_the_target() {
    let source = surface_entity(plane());
    let target = surface_entity(tilted());
    let flowed = path_entity([1.0, 1.0, 0.0], [3.0, 1.0, 0.0]);
    let id = flowed.id;
    let ids = [source.id, target.id, id];
    let mut controller = build(vec![source, target, flowed]);
    controller.select_objects(&ids).expect("selection");

    controller
        .submit("FlowAlongSrf")
        .expect("FlowAlongSrf by name");
    assert_eq!(controller.prompt(), "Point on the source surface: ");
    controller.submit("9,9,0").expect("source surface pick");
    assert!(!controller.is_running());

    let (start, end) = curve_ends(&controller, id);
    assert!(close(start, [1.0, 1.0, 1.0]), "{start:?}");
    assert!(close(end, [3.0, 1.0, 3.0]), "{end:?}");
}

/// Splop wraps the selection onto the surface at the nearest point of that surface.
#[test]
fn m03_04_01_029_splop_wraps_the_selection_onto_the_surface() {
    let surface = surface_entity(plane());
    let wrapped = path_entity([1.0, 1.0, 2.0], [3.0, 1.0, 2.0]);
    let id = wrapped.id;
    let ids = [surface.id, id];
    let mut controller = build(vec![surface, wrapped]);
    controller.select_objects(&ids).expect("selection");
    let before = undo_len(&controller);

    controller.submit("Splop").expect("Splop by name");
    assert!(!controller.is_running());

    let (start, end) = curve_ends(&controller, id);
    assert!(close(start, [1.0, 1.0, 0.0]), "{start:?}");
    assert!(close(end, [3.0, 1.0, 0.0]), "{end:?}");
    assert_eq!(undo_len(&controller), before + 1);
}

/// A flow row needs two frames and at least one object before it runs, so it can never fold
/// the objects with no target.
#[test]
fn m03_04_01_flow_rows_need_two_frames_and_one_object() {
    let only_box = boxed([0.0; 3], [1.0; 3]);
    let id = only_box.id;
    let mut controller = build(vec![only_box]);
    controller.select_objects(&[id]).expect("selection");
    assert!(
        controller
            .dispatch("OM9-TRANSFORM-027", InvocationOrigin::Menu)
            .is_err(),
        "no curves"
    );

    let source = path_entity([0.0; 3], [10.0, 0.0, 0.0]);
    let target = path_entity([0.0; 3], [0.0, 10.0, 0.0]);
    let box_entity = boxed([5.0, 1.0, 0.0], [1.0; 3]);
    let ids = [source.id, target.id, box_entity.id];
    let mut controller = build(vec![source, target, box_entity]);
    controller.select_objects(&ids).expect("selection");
    let before = undo_len(&controller);
    controller.submit("Flow").expect("Flow by name");
    let error = controller
        .submit("9,0,0")
        .expect_err("a solid cannot be flowed");
    assert!(
        format!("{error}").contains("solid"),
        "the refusal names the reason: {error}"
    );
    assert_eq!(undo_len(&controller), before);
}

fn surface_poles(controller: &Controller, id: om9_core::identity::ObjectId) -> Vec<Point3> {
    match &controller
        .document()
        .entities
        .get(&id)
        .expect("entity")
        .geometry
    {
        Geometry::Surface(surface) => surface.poles.clone(),
        other => panic!("not a surface: {other:?}"),
    }
}

/// MoveUVN offsets the control net along the surface's own U, V and normal directions, one
/// solved frame per pole, and one Undo entry per command.
#[test]
fn m03_04_01_039_move_uvn_offsets_the_control_net_along_u_v_and_the_normal() {
    let entity = surface_entity(plane());
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let before = undo_len(&controller);

    controller.submit("MoveUVN").expect("MoveUVN by name");
    assert_eq!(
        controller.prompt(),
        "Move in the surface normal direction: "
    );
    controller.submit("1").expect("normal offset");
    assert_eq!(controller.prompt(), "Move along the U direction: ");
    controller.submit("0").expect("U offset");
    assert_eq!(controller.prompt(), "Move along the V direction: ");
    controller.submit("0").expect("V offset");

    let poles = surface_poles(&controller, id);
    assert_eq!(poles.len(), 4);
    assert!(
        poles.iter().all(|pole| (pole[2] - 1.0).abs() < 1.0e-9),
        "the flat surface lifts by one along +Z: {poles:?}"
    );
    assert_eq!(undo_len(&controller), before + 1);

    // The U direction of this plane is +X, so a U offset slides the net without touching Y or Z.
    let entity = surface_entity(plane());
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    controller.submit("MoveUVN").expect("MoveUVN by name");
    controller.submit("0").expect("normal offset");
    controller.submit("2").expect("U offset");
    controller.submit("0").expect("V offset");
    let poles = surface_poles(&controller, id);
    let mut moved: Vec<f64> = poles.iter().map(|pole| pole[0]).collect();
    moved.sort_by(|a, b| a.partial_cmp(b).expect("coordinate"));
    assert!(
        moved
            .iter()
            .zip([2.0, 2.0, 12.0, 12.0])
            .all(|(found, want)| (found - want).abs() < 1.0e-9),
        "every pole slides two along +X: {poles:?}"
    );
    let mut ys: Vec<f64> = poles.iter().map(|pole| pole[1]).collect();
    ys.sort_by(|a, b| a.partial_cmp(b).expect("coordinate"));
    assert!(
        ys.iter()
            .zip([0.0, 0.0, 10.0, 10.0])
            .all(|(found, want)| (found - want).abs() < 1.0e-9),
        "the U step leaves Y alone: {poles:?}"
    );
    assert!(poles.iter().all(|pole| pole[2].abs() < 1.0e-9));
}

/// MoveUVN edits surfaces, so it refuses a selection that holds anything else and never asks a
/// question.
#[test]
fn m03_04_01_039_move_uvn_refuses_a_non_surface_selection() {
    let entity = boxed([0.0; 3], [2.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let error = controller
        .submit("MoveUVN")
        .expect_err("a solid is not a surface net");
    assert!(format!("{error}").contains("surfaces only"), "{error}");
    assert_eq!(controller.prompt(), "Command: ");
}

/// The cage entity a command created: the only mesh in the document with eight vertices.
fn cage_of(controller: &Controller) -> om9_core::identity::ObjectId {
    controller
        .document()
        .entities
        .values()
        .find(|entity| {
            matches!(&entity.geometry, Geometry::Mesh(mesh) if mesh.vertices.len() == 8
                && mesh.faces.len() == 6)
                && entity.extensions.contains_key("om9.cage")
        })
        .expect("a cage object")
        .id
}

/// Create Cage wraps the selection in one mesh box and links every caged object to it.
#[test]
fn m03_04_01_041_create_cage_wraps_the_selection_in_a_box() {
    let entity = mesh_box([0.0; 3], [10.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let before = undo_len(&controller);

    controller.submit("Cage").expect("Cage by name");
    assert!(!controller.is_running());

    let cage = cage_of(&controller);
    let entity = controller
        .document()
        .entities
        .get(&id)
        .expect("caged object");
    let link = entity
        .extensions
        .get("om9.cage")
        .expect("the back pointer is recorded");
    assert!(
        link.payload.contains(&cage.to_string()),
        "the link names its cage: {}",
        link.payload
    );
    assert_eq!(undo_len(&controller), before + 1);

    // The cage hugs the world bounds of the selection, so its corners span 0..10 on every axis.
    let cage_entity = controller
        .document()
        .entities
        .get(&cage)
        .expect("cage entity");
    let Geometry::Mesh(mesh) = &cage_entity.geometry else {
        panic!("the cage is a mesh");
    };
    assert_eq!(mesh.vertices.len(), 8);
    let mut low = [f64::MAX; 3];
    let mut high = [f64::MIN; 3];
    for vertex in &mesh.vertices {
        for axis in 0..3 {
            low[axis] = low[axis].min(vertex[axis]);
            high[axis] = high[axis].max(vertex[axis]);
        }
    }
    assert!(close(low, [0.0; 3]), "{low:?}");
    assert!(close(high, [10.0; 3]), "{high:?}");
}

/// Cage Edit moves one corner and carries the attached geometry with the cage field.
#[test]
fn m03_04_01_020_cage_edit_moves_the_attached_geometry() {
    let entity = mesh_box([0.0; 3], [10.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    controller.submit("Cage").expect("Cage by name");
    let cage = cage_of(&controller);
    let before = mesh_vertices(&controller, id);
    controller.select_objects(&[cage]).expect("cage selection");

    controller.submit("CageEdit").expect("CageEdit by name");
    assert_eq!(controller.prompt(), "Cage corner to move (1-8): ");
    controller.submit("8").expect("corner");
    assert_eq!(controller.prompt(), "Corner offset: ");
    controller.submit("10,0,0").expect("offset");

    let after = mesh_vertices(&controller, id);
    assert_eq!(after.len(), before.len());
    let moved: Vec<Point3> = before
        .iter()
        .zip(&after)
        .filter(|(old, new)| !close(**old, **new))
        .map(|(_, new)| *new)
        .collect();
    assert_eq!(
        moved.len(),
        1,
        "only the corner at maximum rides the edit: {after:?}"
    );
    assert!(close(moved[0], [20.0, 10.0, 10.0]), "{:?}", moved[0]);
}

/// ReleaseFromCage drops the relation and keeps the geometry where the cage left it.
#[test]
fn m03_04_01_021_release_from_cage_keeps_the_geometry() {
    let entity = mesh_box([0.0; 3], [10.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    controller.submit("Cage").expect("Cage by name");
    let cage = cage_of(&controller);
    controller.select_objects(&[cage]).expect("cage selection");
    controller.submit("CageEdit").expect("CageEdit by name");
    controller.submit("8").expect("corner");
    controller.submit("10,0,0").expect("offset");
    let moved = mesh_vertices(&controller, id);

    controller.select_objects(&[id]).expect("object selection");
    controller
        .submit("ReleaseFromCage")
        .expect("ReleaseFromCage by name");
    let entity = controller
        .document()
        .entities
        .get(&id)
        .expect("released object");
    assert!(
        !entity.extensions.contains_key("om9.cage"),
        "the back pointer is gone"
    );
    assert_eq!(
        mesh_vertices(&controller, id),
        moved,
        "the shape is untouched"
    );
}

/// A cage row refuses a selection that has no cage relation, before it asks anything.
#[test]
fn m03_04_01_cage_rows_refuse_without_a_cage() {
    let entity = boxed([0.0; 3], [2.0; 3]);
    let id = entity.id;
    let mut controller = build(vec![entity]);
    controller.select_objects(&[id]).expect("selection");
    let error = controller
        .submit("ReleaseFromCage")
        .expect_err("nothing is attached");
    assert!(
        format!("{error}").contains("not attached to a cage"),
        "{error}"
    );
    assert_eq!(controller.prompt(), "Command: ");
}

/// The recorded flow link of one object, as the document stores it.
fn flow_link(controller: &Controller, id: om9_core::identity::ObjectId) -> String {
    controller
        .document()
        .entities
        .get(&id)
        .expect("flowed object")
        .extensions
        .get("om9.flow")
        .expect("a recorded flow link")
        .payload
        .clone()
}

/// Smart Flow maps the selection from the base surface to the target surface and records the
/// link that makes the relation live.
#[test]
fn m03_04_01_011_smart_flow_maps_from_the_base_surface() {
    let base = surface_entity(plane());
    let target = surface_entity(tilted());
    let curve = path_entity([1.0, 1.0, 0.0], [3.0, 1.0, 0.0]);
    let id = curve.id;
    let ids = [base.id, target.id, id];
    let mut controller = build(vec![base, target, curve]);
    controller.select_objects(&ids).expect("selection");
    let before = undo_len(&controller);

    controller.submit("SmartFlow").expect("SmartFlow by name");
    assert_eq!(controller.prompt(), "Point on the base surface: ");
    controller.submit("9,9,0").expect("base surface pick");
    assert!(!controller.is_running());

    let (start, end) = curve_ends(&controller, id);
    assert!(close(start, [1.0, 1.0, 1.0]), "{start:?}");
    assert!(close(end, [3.0, 1.0, 3.0]), "{end:?}");
    let link = flow_link(&controller, id);
    assert!(link.contains("ACTIVE"), "{link}");
    assert!(link.contains("\"revision\":1"), "{link}");
    assert_eq!(undo_len(&controller), before + 1);
}

/// Smart Flow Rigid moves the object frame instead of the geometry, so a gemstone keeps its
/// shape and size.
#[test]
fn m03_04_01_012_smart_flow_rigid_moves_the_frame() {
    let base = surface_entity(plane());
    let target = surface_entity(tilted());
    let solid = boxed_at([0.0; 3], [2.0; 3], [2.0, 2.0, 0.0]);
    let id = solid.id;
    let ids = [base.id, target.id, id];
    let mut controller = build(vec![base, target, solid]);
    controller.select_objects(&ids).expect("selection");
    let geometry_before = controller
        .document()
        .entities
        .get(&id)
        .expect("object")
        .geometry
        .clone();

    controller
        .submit("SmartFlowRigid")
        .expect("SmartFlowRigid by name");
    assert_eq!(controller.prompt(), "Point on the base surface: ");
    controller.submit("9,9,0").expect("base surface pick");

    // The origin sat at (2,2,0): the same (u,v) on the tilted plane is (2,2,2).
    let centre = controller
        .document()
        .entities
        .get(&id)
        .expect("object")
        .placement
        .point([0.0, 0.0, 0.0])
        .expect("placement origin");
    assert!(close(centre, [2.0, 2.0, 2.0]), "{centre:?}");
    assert_eq!(
        controller.document().entities[&id].geometry,
        geometry_before,
        "the rigid row never rewrites the geometry"
    );
    let link = flow_link(&controller, id);
    assert!(link.contains("\"rigid\":true"), "{link}");
}

/// Break keeps the geometry and marks the link broken; a second break is refused.
#[test]
fn m03_04_01_014_smart_flow_break_marks_the_link_broken() {
    let base = surface_entity(plane());
    let target = surface_entity(tilted());
    let curve = path_entity([1.0, 1.0, 0.0], [3.0, 1.0, 0.0]);
    let id = curve.id;
    let ids = [base.id, target.id, id];
    let mut controller = build(vec![base, target, curve]);
    controller.select_objects(&ids).expect("selection");
    controller.submit("SmartFlow").expect("SmartFlow by name");
    controller.submit("9,9,0").expect("base surface pick");
    let flowed = curve_ends(&controller, id);

    controller.select_objects(&[id]).expect("object selection");
    controller
        .submit("SmartFlowBreak")
        .expect("SmartFlowBreak by name");
    let link = flow_link(&controller, id);
    assert!(link.contains("BROKEN"), "{link}");
    assert!(link.contains("\"revision\":2"), "{link}");
    assert_eq!(curve_ends(&controller, id), flowed, "the shape is kept");

    let error = controller
        .submit("SmartFlowBreak")
        .expect_err("the link is already broken");
    assert!(
        format!("{error}").contains("no live Smart Flow link"),
        "{error}"
    );
}

/// Pull Down Curves brings curves that sit on the target surface back onto the base at the same
/// parameters.
#[test]
fn m03_04_01_015_smart_flow_pulldown_brings_curves_to_the_base() {
    let base = surface_entity(plane());
    let target = surface_entity(tilted());
    let curve = path_entity([1.0, 1.0, 1.0], [3.0, 1.0, 3.0]);
    let id = curve.id;
    let ids = [base.id, target.id, id];
    let mut controller = build(vec![base, target, curve]);
    controller.select_objects(&ids).expect("selection");

    controller
        .submit("SmartFlowPullDownCurves")
        .expect("SmartFlowPullDownCurves by name");
    assert_eq!(controller.prompt(), "Point on the base surface: ");
    controller.submit("9,9,0").expect("base surface pick");

    let (start, end) = curve_ends(&controller, id);
    assert!(close(start, [1.0, 1.0, 0.0]), "{start:?}");
    assert!(close(end, [3.0, 1.0, 0.0]), "{end:?}");
}
