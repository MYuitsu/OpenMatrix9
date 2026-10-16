//! A CLONE IS THE SAME PART ONCE MORE, NOT A COPY OF IT.
//!
//! A copy is a part of its own from the moment it is made: an edit of the one it came from does not reach it. A
//! clone shares its geometry with the original - an edit of the original's sketch or features rebuilds every clone,
//! including a feature added after the clone was made - while it stands, moves and is mated on its own.
use qymcad_core::model::{CompPatternKind, Project};

/// An assembly with a single 20x20x10 box part at the origin. Returns (project, part component).
fn assembly_with_part() -> (Project, u64) {
    let mut p = Project::default();
    let root = p.ensure_root();
    p.set_active_component(Some(root));
    let part = p.add_part("Bolt");
    p.set_active_component(Some(part));
    p.add_box(20.0, 20.0, 10.0);
    let _ = qymcad_testkit::regenerate(&mut p);
    (p, part)
}

/// The volume of a component's body as it stands now, its last one (0 means there is no body).
fn body_volume(p: &Project, c: u64) -> f64 {
    p.active_body(c).and_then(|b| p.bodies.iter().find(|x| x.id == b)).map(|b| b.mesh.volume()).unwrap_or(0.0)
}

/// A feature added to `part` the way the box tool adds one: a new box united with the part's body, 20 x 20 x 30 in all.
fn a_later_feature(p: &mut Project, part: u64) {
    p.set_active_component(Some(part));
    let b = p.add_box(20.0, 20.0, 30.0);
    p.finish_base_body(b, 1);
    let _ = qymcad_testkit::regenerate(p);
}

/// The source box made twice as tall, the way an edit of its dimension does.
fn taller(p: &mut Project, part: u64) {
    let body = p.active_body(part).expect("the source body");
    let n = p.timeline.iter_mut().find(|n| n.kind.bodies().contains(&body)).expect("its node");
    if let qymcad_core::feature::FeatureKind::Box3 { dz, .. } = &mut n.kind {
        *dz = 20.0;
    }
    n.dirty = true;
    let _ = qymcad_testkit::regenerate(p);
}

#[test]
fn a_clone_follows_an_edit_of_its_original() {
    let (mut p, part) = assembly_with_part();
    let clone = p.clone_part(part, p.root).expect("a clone is made");
    let _ = qymcad_testkit::regenerate(&mut p);
    assert!((body_volume(&p, clone) - 4000.0).abs() < 1.0, "the clone has the original's body, not {}", body_volume(&p, clone));
    taller(&mut p, part);
    assert!((body_volume(&p, clone) - 8000.0).abs() < 1.0, "the original grew and the clone stayed at {}", body_volume(&p, clone));
}

/// A FEATURE ADDED TO THE ORIGINAL AFTER THE CLONE WAS MADE reaches the clone too: a second box on the part.
#[test]
fn a_clone_follows_a_feature_added_later() {
    let (mut p, part) = assembly_with_part();
    let clone = p.clone_part(part, p.root).expect("a clone is made");
    let _ = qymcad_testkit::regenerate(&mut p);
    a_later_feature(&mut p, part);
    let (orig, got) = (body_volume(&p, part), body_volume(&p, clone));
    assert!((orig - 12000.0).abs() < 1.0, "setup: the original is now {orig}");
    assert!((got - orig).abs() < 1.0, "the original got a feature after the clone was made, and the clone stayed at {got}");
}

#[test]
fn a_copy_does_not_follow() {
    let (mut p, part) = assembly_with_part();
    let copy = p.clone_component(part, p.root).expect("a copy is made");
    let _ = qymcad_testkit::regenerate(&mut p);
    taller(&mut p, part);
    assert!((body_volume(&p, copy) - 4000.0).abs() < 1.0, "a copy is a part of its own, and it followed to {}", body_volume(&p, copy));
}

#[test]
fn a_clone_stands_and_moves_on_its_own() {
    let (mut p, part) = assembly_with_part();
    let clone = p.clone_part(part, p.root).expect("a clone is made");
    let at = p.component_transform(clone)[3];
    p.move_component(clone, [40.0, 0.0, 0.0]);
    let _ = qymcad_testkit::regenerate(&mut p);
    assert_eq!(p.component_transform(part)[3], 0.0, "moving the clone moved the original");
    assert_eq!(p.component_transform(clone)[3], at + 40.0);
    assert!((body_volume(&p, clone) - 4000.0).abs() < 1.0, "a moved clone keeps its body");
}

/// A CLONE OF A CLONE IS ONE MORE CLONE OF THE ORIGINAL, and says whose it is.
#[test]
fn a_clone_of_a_clone_repeats_the_original() {
    let (mut p, part) = assembly_with_part();
    let first = p.clone_part(part, p.root).expect("a clone is made");
    let second = p.clone_part(first, p.root).expect("a clone of a clone is made");
    assert_eq!((p.instance_origin(first), p.instance_origin(second), p.instance_origin(part)), (part, part, part));
    let _ = qymcad_testkit::regenerate(&mut p);
    taller(&mut p, part);
    assert!((body_volume(&p, second) - 8000.0).abs() < 1.0, "the clone of a clone did not follow: {}", body_volume(&p, second));
}

/// THE SAME GAP IN A PATTERN: its copies are instances too, and a feature added to the source later must reach them.
#[test]
fn a_pattern_copy_follows_a_feature_added_later() {
    let (mut p, part) = assembly_with_part();
    p.add_comp_pattern(part, CompPatternKind::linear([1.0, 0.0, 0.0], 30.0, 2));
    let _ = qymcad_testkit::regenerate(&mut p);
    let copy = p.comp_pattern_of(part).expect("the pattern").copies[0];
    a_later_feature(&mut p, part);
    assert!((body_volume(&p, part) - 12000.0).abs() < 1.0, "setup: the source is now {}", body_volume(&p, part));
    assert!((body_volume(&p, copy) - body_volume(&p, part)).abs() < 1.0, "the source got a feature later, and the pattern's copy stayed at {}", body_volume(&p, copy));
}

/// WHAT READS A CLONE MOVES WITH IT: a mirror of the clone stands between the clone and the original's later feature,
/// and follows the clone down the timeline - otherwise it would stand above its own source and have none.
#[test]
fn what_reads_a_clone_moves_with_it() {
    let (mut p, part) = assembly_with_part();
    let clone = p.clone_part(part, p.root).expect("a clone is made");
    let mirror = p.add_mirror_part(clone, [0.0; 3], [1.0, 0.0, 0.0]);
    let mirror = p.body_owner(mirror).expect("the mirror's part"); // the call answers with the mirror's body
    let _ = qymcad_testkit::regenerate(&mut p);
    assert!((body_volume(&p, mirror) - 4000.0).abs() < 1.0, "setup: the mirror of the clone is {}", body_volume(&p, mirror));
    a_later_feature(&mut p, part);
    assert!(p.regen_errors.is_empty(), "the original got a feature, and the timeline broke: {:?}", p.regen_errors);
    assert!((body_volume(&p, mirror) - 12000.0).abs() < 1.0, "the mirror of the clone stayed at {}", body_volume(&p, mirror));
}

/// The box of a part's body in the assembly, as (min, max).
fn world_box(p: &Project, c: u64) -> ([f64; 3], [f64; 3]) {
    let b = p.active_body(c).expect("a body");
    let bb = p.bodies.iter().find(|x| x.id == b).and_then(|x| x.mesh.bounds()).expect("a built body");
    let m = p.world_transform(c);
    let lo = qymcad_core::feature::apply12(&m, [bb.min.x, bb.min.y, bb.min.z]);
    let hi = qymcad_core::feature::apply12(&m, [bb.max.x, bb.max.y, bb.max.z]);
    ([lo[0].min(hi[0]), lo[1].min(hi[1]), lo[2].min(hi[2])], [lo[0].max(hi[0]), lo[1].max(hi[1]), lo[2].max(hi[2])])
}

/// Whether two boxes share a volume, not only a face.
fn overlap(a: ([f64; 3], [f64; 3]), b: ([f64; 3], [f64; 3])) -> bool {
    (0..3).all(|i| a.0[i] < b.1[i] - 1e-6 && b.0[i] < a.1[i] - 1e-6)
}

/// A CLONE STANDS BESIDE ITS ORIGINAL, IN FREE SPACE: not on top of the original, where a click takes the original,
/// and not inside a neighbour standing where the first free place would be. Reported behaviour: a clone stood
/// exactly where its original stood.
#[test]
fn a_clone_stands_beside_its_original_in_free_space() {
    let (mut p, part) = assembly_with_part();
    // a neighbour 20 x 20 x 10 put 30 mm along X, where a clone of the 20 mm box with its gap would first go
    p.set_active_component(Some(p.root));
    let neighbour = p.add_part("Nut");
    p.set_active_component(Some(neighbour));
    p.add_box(20.0, 20.0, 10.0);
    p.move_component(neighbour, [30.0, 0.0, 0.0]);
    let _ = qymcad_testkit::regenerate(&mut p);
    let clone = p.clone_part(part, p.root).expect("a clone is made");
    let _ = qymcad_testkit::regenerate(&mut p);
    let (o, n, c) = (world_box(&p, part), world_box(&p, neighbour), world_box(&p, clone));
    assert!(!overlap(c, o), "the clone stands inside its original: {c:?} and {o:?}");
    assert!(!overlap(c, n), "the clone stands inside the neighbour: {c:?} and {n:?}");
    assert!(c.0[0] - o.1[0] < 100.0, "the clone went far off, not beside: {c:?} and the original {o:?}");
}
