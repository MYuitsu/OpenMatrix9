//! Generates a large **instance** stress project from an imported 3DM project.
//!
//! The file is opened once, one of its BRep meshes becomes a block definition member, and N
//! `Geometry::Instance` entities reference that definition with different placements. The document
//! therefore stores the mesh once while the viewport has N occurrences, which is the cheap way to ask
//! "how slow is 27,000 objects in the viewport?" without writing 27,000 copies of the same triangles
//! into the project file.
//!
//! Ignored by default because it writes a multi-megabyte project and needs an imported source:
//!
//! ```text
//! OM9_STRESS_SOURCE=build/stress-27000/bong.om9 \
//! OM9_STRESS_TARGET=build/stress-27000/bong-27000.om9 \
//! cargo test --release -p om9-app --test stress_instances -- --ignored --nocapture
//! ```
//!
//! `OM9_STRESS_COUNT` (default 27000), `OM9_STRESS_TRIANGLES` (pick the BRep whose tessellation is
//! closest to this triangle count, default 2160), `OM9_STRESS_SPACING` (mm, default 1.4) and
//! `OM9_STRESS_SCALE` (default 0.2) tune the scene.

use om9_archive::Archive;
use om9_core::{
    brep::Brep,
    budget::Limits,
    geometry::{Geometry, Point3, Transform},
    model::{Definition, Document, Entity, Layer},
    transaction::Cancellation,
};
use std::collections::BTreeSet;

fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(default)
}

/// Row-major spin around Z, then a tilt around X, a uniform scale, then translation.
fn placement(translation: Point3, spin: f64, tilt: f64, scale: f64) -> Transform {
    let (s, c) = spin.sin_cos();
    let (st, ct) = tilt.sin_cos();
    Transform([
        [c * scale, -s * ct * scale, s * st * scale, translation[0]],
        [s * scale, c * ct * scale, -c * st * scale, translation[1]],
        [0.0, st * scale, ct * scale, translation[2]],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

#[test]
#[ignore = "writes a large stress project; run explicitly with OM9_STRESS_SOURCE/OM9_STRESS_TARGET"]
fn generate_instance_stress_project() {
    let source = std::env::var_os("OM9_STRESS_SOURCE")
        .expect("OM9_STRESS_SOURCE must name an imported .om9 project");
    let target = std::env::var_os("OM9_STRESS_TARGET")
        .expect("OM9_STRESS_TARGET must name the output .om9 project");
    let count = env_usize("OM9_STRESS_COUNT", 27_000);
    let wanted_triangles = env_usize("OM9_STRESS_TRIANGLES", 2_160);
    let whole = std::env::var("OM9_STRESS_WHOLE").is_ok_and(|value| value == "1");
    let identical = std::env::var("OM9_STRESS_IDENTICAL").is_ok_and(|value| value == "1");
    let spacing = env_f64("OM9_STRESS_SPACING", if whole { 2.0 } else { 1.4 });
    let scale = env_f64("OM9_STRESS_SCALE", if whole { 0.12 } else { 0.2 });

    let restored = Archive::open_restored(
        std::path::Path::new(&source),
        &Limits::default(),
        &BTreeSet::new(),
    )
    .expect("read the source project");
    let source_document = restored.document;

    if std::env::var("OM9_STRESS_INSPECT").is_ok_and(|value| value == "1") {
        for entity in source_document.entities.values() {
            let kind = match &entity.geometry {
                Geometry::Brep(brep) => format!("Brep {} tris", brep.display.triangles.len()),
                Geometry::Instance(id) => format!("Instance {id:?}"),
                other => format!("{other:?}"),
            };
            println!(
                "entity {:?} label {:?} layer {:?} {kind}",
                entity.id, entity.label, entity.attributes.layer
            );
        }
        for definition in source_document.definitions.values() {
            println!(
                "definition {:?} label {:?} members {}",
                definition.id,
                definition.label,
                definition.members.len()
            );
            for member in &definition.members {
                let kind = match &member.geometry {
                    Geometry::Brep(brep) => format!("Brep {} tris", brep.display.triangles.len()),
                    Geometry::Instance(id) => format!("Instance {id:?}"),
                    other => format!("{other:?}"),
                };
                println!("  member {:?} label {:?} {kind}", member.id, member.label);
            }
        }
        return;
    }

    // `OM9_STRESS_WHOLE=1` uses every non-instance object of the source as one definition, so one
    // occurrence is the whole earring; otherwise one BRep of the requested triangle count is used.
    let mut members: Vec<Entity> = Vec::new();
    let mut chosen = None;
    if whole {
        for entity in source_document.entities.values() {
            let Geometry::Instance(definition_id) = entity.geometry else {
                members.push(entity.clone());
                continue;
            };
            let Some(definition) = source_document.definitions.get(&definition_id) else {
                continue;
            };
            for member in &definition.members {
                let Geometry::Brep(brep) = &member.geometry else {
                    continue;
                };
                let placement = entity
                    .placement
                    .compose(member.placement)
                    .expect("instance member transform");
                let mut display = brep.display.clone();
                for point in &mut display.points {
                    *point = placement.point(*point).expect("instance member point");
                }
                let brep =
                    Brep::from_native(brep.source().to_string(), display, &Limits::default())
                        .expect("transformed display binding");
                let mut member = member.clone();
                member.geometry = Geometry::Brep(brep);
                member.placement = Transform::identity();
                members.push(member);
            }
        }
        assert!(!members.is_empty(), "the source project has no geometry");
    } else {
        for entity in source_document.entities.values() {
            if let Geometry::Brep(brep) = &entity.geometry {
                let score = brep.display.triangles.len().abs_diff(wanted_triangles);
                if chosen.as_ref().is_none_or(|(best, _)| score < *best) {
                    chosen = Some((score, entity));
                }
            }
        }
        let (_, entity) = chosen.expect("the source project has no BRep");
        members.push(entity.clone());
    }

    let triangles: usize = members
        .iter()
        .map(|entity| match &entity.geometry {
            Geometry::Brep(brep) => brep.display.triangles.len(),
            _ => 0,
        })
        .sum();
    assert!(
        triangles > 0,
        "the chosen geometry has no display tessellation"
    );

    // Recentre the definition at its own bounds centre, so every occurrence spins around itself.
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for entity in &members {
        if let Geometry::Brep(brep) = &entity.geometry {
            for point in &brep.display.points {
                for axis in 0..3 {
                    min[axis] = min[axis].min(point[axis]);
                    max[axis] = max[axis].max(point[axis]);
                }
            }
        }
    }
    let centre = [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ];
    let extent = (max[0] - min[0]).max(max[1] - min[1]).max(max[2] - min[2]);
    for member in &mut members {
        member.placement = Transform::translation([-centre[0], -centre[1], -centre[2]]);
    }

    let colour = members
        .iter()
        .find_map(|entity| {
            entity
                .attributes
                .layer
                .and_then(|id| source_document.layers.get(&id))
                .map(|layer| layer.color)
        })
        .unwrap_or([180, 200, 210, 255]);

    let mut document = Document::new(format!("bong 5X7 x{count} instances"));
    let layer = Layer::new(format!("Stress {count}"), colour);
    document.layer_order.push(layer.id);
    document.current_layer = Some(layer.id);
    document.layers.insert(layer.id, layer.clone());

    // The source layer ids do not exist in the new document; the members take the stress layer instead.
    for member in &mut members {
        member.attributes.layer = Some(layer.id);
    }

    let definition = Definition::new("bong 5X7", members);
    let definition_id = definition.id;
    let member_count = definition.members.len();
    document.definitions.insert(definition_id, definition);

    // A centred cubic lattice, small variations so no two occurrences are identical.
    let side = (count as f64).cbrt().ceil().max(1.0) as i64;
    let half = (side - 1) as f64 * 0.5;
    for index in 0..count {
        let index = index as i64;
        let x = (index % side) as f64 - half;
        let y = ((index / side) % side) as f64 - half;
        let z = (index / (side * side)) as f64 - half;
        let spin = if identical {
            0.0
        } else {
            index as f64 * 2.399_963_229_728_653
        };
        let tilt = if identical {
            0.0
        } else {
            (index as f64 * 0.7).sin() * 0.35
        };
        let scale = if identical {
            scale
        } else {
            scale * (0.9 + 0.2 * ((index as f64 * 0.37).sin() * 0.5 + 0.5))
        };
        let jitter = |axis: usize| {
            if identical {
                0.0
            } else {
                ((index * (7 + axis as i64 * 13)) % 100) as f64 / 100.0 - 0.5
            }
        };

        let mut entity = Entity::new("bong 5X7", Geometry::Instance(definition_id), layer.id);
        entity.placement = placement(
            [
                x * spacing + jitter(0) * spacing * 0.2,
                y * spacing + jitter(1) * spacing * 0.2,
                z * spacing + jitter(2) * spacing * 0.2,
            ],
            spin,
            tilt,
            scale,
        );
        document.entities.insert(entity.id, entity);
    }

    Archive::save(
        std::path::Path::new(&target),
        &document,
        &om9_app::document_limits(),
        &Cancellation::default(),
    )
    .expect("write the stress project");

    let bytes = std::fs::metadata(std::path::Path::new(&target))
        .expect("stress project metadata")
        .len();
    eprintln!(
        "wrote {count} instances of {member_count} member(s), {triangles} triangles per occurrence \
         (extent {extent:.2} mm, whole={whole}) to {:?}: {bytes} bytes",
        std::path::Path::new(&target)
    );
}
