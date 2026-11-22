// SPDX-License-Identifier: LGPL-2.1-or-later
//! Parametric Round Diamond facets in the gem's local frame.

use super::{GemCut, GemShape, GemSpec, Point3};

const OCTANTS: usize = 8;
const RIM_STEPS: usize = 8;
const TURN: f64 = std::f64::consts::TAU / OCTANTS as f64;
pub(super) const RIM: usize = OCTANTS * RIM_STEPS;
const TABLE: usize = 2 * RIM;
const STAR: usize = TABLE + OCTANTS;
const LOWER: usize = STAR + OCTANTS;
const CULET: usize = LOWER + OCTANTS;
pub(super) const CROWN_START: usize = 1 + RIM;
pub(super) const PAVILION_START: usize = CROWN_START + 4 * OCTANTS;
const FACE_COUNT: usize = PAVILION_START + 3 * OCTANTS;

pub(super) fn applies(spec: &GemSpec) -> bool {
    spec.shape == GemShape::Round && spec.cut == GemCut::Diamond
        && spec.facet_count == GemCut::Diamond.default_facet_count() && spec.profile.is_none()
}

fn point(spec: &GemSpec, angle: f64, radius: f64, height: f64) -> Point3 {
    let (sin, cos) = (angle + TURN * 0.5).sin_cos();
    [spec.length_mm * 0.5 * radius * cos, spec.width_mm * 0.5 * radius * sin, height]
}

fn rim(spec: &GemSpec, middle_radius: f64, heights: [f64; 2]) -> impl Iterator<Item = Point3> + '_ {
    (0..RIM).map(move |index| {
        let quarter = TURN / 4.0;
        let angle = TURN * index as f64 / RIM_STEPS as f64;
        let local = ((index % (RIM_STEPS / 2)) as f64 * TURN / RIM_STEPS as f64 - quarter).cos();
        let fraction = (local - quarter.cos()) / (1.0 - quarter.cos());
        let radius = (quarter.cos() + fraction * (middle_radius - quarter.cos())) / local;
        point(spec, angle, radius, heights[0] + fraction * (heights[1] - heights[0]))
    })
}

pub(super) fn girdle(spec: &GemSpec) -> impl Iterator<Item = Point3> + '_ {
    rim(spec, Profile::new(spec).rim_middle_radius, [0.0; 2])
}

pub(super) fn table(spec: &GemSpec) -> impl Iterator<Item = Point3> + '_ {
    let height = spec.girdle_mm * 0.5 + spec.crown_height_mm;
    let radius = spec.table_ratio();
    (0..OCTANTS).map(move |index| point(spec, TURN * index as f64, radius, height))
}

const LOWER_RADIUS: f64 = 0.2002;

struct Profile {
    star_radius: f64,
    star_height: f64,
    lower_height: f64,
    culet_height: f64,
    rim_middle_radius: f64,
    rim_top: [f64; 2],
    rim_bottom: [f64; 2],
}

impl Profile {
    fn new(spec: &GemSpec) -> Self {
        let band = spec.girdle_mm * 0.5;
        let table_radius = spec.table_ratio();
        let star_radius = table_radius + (1.0 - table_radius) * 0.457;
        let half = TURN * 0.5;
        // The two adjacent main facets share each star or lower-half junction.
        let star_height = band + spec.crown_height_mm
            * (1.0 - star_radius * half.cos()) / (1.0 - table_radius);
        let culet_height = -band - spec.pavilion_depth_mm;
        let lower_height = culet_height + spec.pavilion_depth_mm * LOWER_RADIUS * half.cos();

        // Project the intermediate girdle vertices onto the neighbouring facet
        // planes. Limit scalloping for thin bands so the two surfaces stay apart.
        let chord = (half * 0.5).cos();
        let upper_slope = (star_height - band) / (chord * (1.0 - star_radius));
        let lower_slope = (-band - lower_height) / (chord * (1.0 - LOWER_RADIUS));
        let rise = (1.0 - chord).min(spec.girdle_mm * 0.75 / (upper_slope + lower_slope));
        Self {
            star_radius, star_height, lower_height, culet_height,
            rim_middle_radius: chord + rise,
            rim_top: [band, band - upper_slope * rise],
            rim_bottom: [-band, -band + lower_slope * rise],
        }
    }
}

pub(super) fn body(spec: &GemSpec) -> (Vec<Point3>, Vec<Vec<u32>>) {
    let profile = Profile::new(spec);
    let half = TURN * 0.5;

    let mut vertices = Vec::with_capacity(CULET + 1);
    vertices.extend(rim(spec, profile.rim_middle_radius, profile.rim_top));
    vertices.extend(rim(spec, profile.rim_middle_radius, profile.rim_bottom));
    vertices.extend(table(spec));
    vertices.extend((0..OCTANTS).map(|index| {
        point(spec, TURN * index as f64 + half, profile.star_radius, profile.star_height)
    }));
    vertices.extend((0..OCTANTS).map(|index| {
        point(spec, TURN * index as f64 + half, LOWER_RADIUS, profile.lower_height)
    }));
    vertices.push([0.0, 0.0, profile.culet_height]);

    let mut faces = Vec::with_capacity(FACE_COUNT);
    faces.push((TABLE..STAR).map(|corner| corner as u32).collect());
    let mut face = |indices: &[usize]| {
        faces.push(indices.iter().map(|&index| index as u32).collect());
    };
    for rim in 0..RIM {
        let next = (rim + 1) % RIM;
        face(&[rim, rim + RIM, next + RIM, next]);
    }
    for sector in 0..OCTANTS {
        let previous = (sector + OCTANTS - 1) % OCTANTS;
        let next = (sector + 1) % OCTANTS;
        let rim = sector * RIM_STEPS;
        face(&[TABLE + sector, STAR + previous, rim, STAR + sector]);
        face(&[TABLE + sector, STAR + sector, TABLE + next]);
        for start in [rim, rim + RIM_STEPS / 2] {
            let mut wing = vec![STAR + sector];
            wing.extend((start..=start + RIM_STEPS / 2).map(|index| index % RIM));
            face(&wing);
        }
    }
    for sector in 0..OCTANTS {
        let previous = (sector + OCTANTS - 1) % OCTANTS;
        let rim = sector * RIM_STEPS;
        face(&[RIM + rim, LOWER + previous, CULET, LOWER + sector]);
        for start in [rim, rim + RIM_STEPS / 2] {
            let mut wing = vec![LOWER + sector];
            wing.extend((start..=start + RIM_STEPS / 2).rev().map(|index| RIM + index % RIM));
            face(&wing);
        }
    }
    (vertices, faces)
}

/// Hide fan diagonals inside the octagonal table and clipped wing facets.
pub(super) fn mesh_table_diagonal(mesh: &super::Mesh, a: u32, b: u32) -> bool {
    let a = a as usize;
    let b = b as usize;
    // Existing documents can still contain the earlier 32-station cut.
    let rim = match (mesh.vertices.len(),mesh.faces.len()) {
        (89,94) => 32,
        (n,f) if n == CULET + 1 && f == FACE_COUNT + OCTANTS - 3 + 32 * 3 => RIM,
        _ => return false,
    };
    let table = rim * 2;
    let star = table + OCTANTS;
    if !mesh.vertices[table..star].iter().all(|point| point[2] == mesh.vertices[table][2])
        || !mesh.faces[..OCTANTS - 2].iter().all(|face| {
            face.len() == 3 && face.iter().all(|&index| (table..star).contains(&(index as usize)))
        }) { return false; }
    if rim == RIM {
        let (a,b) = (a.min(b),a.max(b));
        if a < table && b < table && a / rim == b / rim {
            return b-a != 1 && b-a != rim-1;
        }
        if a < table && (star..CULET).contains(&b) { return a % (RIM_STEPS / 2) != 0; }
    }
    (table..star).contains(&a) && (table..star).contains(&b)
        && a.abs_diff(b) != 1 && a.abs_diff(b) != OCTANTS - 1
}
