// SPDX-License-Identifier: LGPL-2.1-or-later
//! OCCT multi-profile pipe-shell adapter used by Sweep 1/2 History.
#![cfg(feature = "3dm-brep")]

use om9_core::budget::Limits;
use om9_kernel::creation::{self, DEFAULT_DEFLECTION_MM};

fn circle_at(x: f64, radius: f64) -> Vec<[f64; 3]> {
    (0..24)
        .map(|index| {
            let angle = std::f64::consts::TAU * index as f64 / 24.0;
            [x, radius * angle.cos(), radius * angle.sin()]
        })
        .collect()
}

#[test]
fn multi_profile_pipe_shell_builds_one_closed_brep() {
    let limits = Limits::default();
    let spine = [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]];
    let sections = vec![circle_at(0.0, 2.0), circle_at(10.0, 2.0)];
    let shell = creation::pipe_shell(
        &spine,
        None,
        &sections,
        false,
        DEFAULT_DEFLECTION_MM,
        &limits,
    )
    .expect("pipe shell");
    let measured = creation::measure(&shell, &limits).expect("measure");
    assert!(measured.area_mm2 > 100.0, "area {}", measured.area_mm2);
}

#[test]
fn two_rail_pipe_shell_accepts_an_auxiliary_spine() {
    let limits = Limits::default();
    let spine = [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]];
    let aux = [[0.0, 5.0, 1.0], [10.0, 5.0, 1.0]];
    let sections = vec![circle_at(0.0, 1.0), circle_at(10.0, 1.0)];
    let shell = creation::pipe_shell(
        &spine,
        Some(&aux),
        &sections,
        false,
        DEFAULT_DEFLECTION_MM,
        &limits,
    )
    .expect("two-rail pipe shell");
    let measured = creation::measure(&shell, &limits).expect("measure");
    assert!(measured.area_mm2 > 10.0);
}

#[test]
fn four_rail_sweep_sews_four_patches_into_one_polysurface() {
    let limits = Limits::default();
    let rails = [
        vec![[0.0, -2.0, -2.0], [10.0, -2.0, -2.0]],
        vec![[0.0, 2.0, -2.0], [10.0, 2.0, -2.0]],
        vec![[0.0, 2.0, 2.0], [10.0, 2.0, 2.0]],
        vec![[0.0, -2.0, 2.0], [10.0, -2.0, 2.0]],
    ];
    let sections = [
        vec![
            vec![[0.0, -2.0, -2.0], [0.0, 2.0, -2.0]],
            vec![[10.0, -2.0, -2.0], [10.0, 2.0, -2.0]],
        ],
        vec![
            vec![[0.0, 2.0, -2.0], [0.0, 2.0, 2.0]],
            vec![[10.0, 2.0, -2.0], [10.0, 2.0, 2.0]],
        ],
        vec![
            vec![[0.0, 2.0, 2.0], [0.0, -2.0, 2.0]],
            vec![[10.0, 2.0, 2.0], [10.0, -2.0, 2.0]],
        ],
        vec![
            vec![[0.0, -2.0, 2.0], [0.0, -2.0, -2.0]],
            vec![[10.0, -2.0, 2.0], [10.0, -2.0, -2.0]],
        ],
    ];
    let sewed =
        creation::four_rail_sweep(&rails, &sections, 1.0e-3, DEFAULT_DEFLECTION_MM, &limits)
            .expect("four rail sweep");
    let measured = creation::measure(&sewed, &limits).expect("measure");
    assert!(measured.area_mm2 > 100.0, "area {}", measured.area_mm2);
    assert!(measured.faces >= 4, "faces {}", measured.faces);
}
