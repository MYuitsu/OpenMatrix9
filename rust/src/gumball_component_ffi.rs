// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bounded, value-only Mesh component ABI. No native pointer is retained.
use crate::gumball_component::{mesh_component_frame, Component, MAX_POINTS, MAX_TRIANGLES};
use std::panic::catch_unwind;

/// 0 success; 1 invalid ABI admission or caught panic; 2 unresolved component.
/// Every failure clears the twelve output doubles when output is nonnull.
///
/// # Safety
/// Nonnull inputs must be aligned and readable for their admitted counts;
/// output must be aligned and writable for twelve doubles. These regions
/// must not overlap. A null output cannot be cleared and returns status 1.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_gumball_component_frame(
    points: *const [f64; 3],
    point_count: usize,
    triangles: *const [u32; 3],
    triangle_count: usize,
    kind: u32,
    first: u32,
    second: u32,
    output: *mut f64,
) -> i32 {
    if output.is_null() {
        return 1;
    }
    // Clear before any admission/resolve failure. The only later write is one
    // complete successfully resolved frame, after the unwind boundary.
    unsafe { std::ptr::write_bytes(output, 0, 12) };
    let resolved = catch_unwind(|| {
        if point_count == 0
            || point_count > MAX_POINTS
            || triangle_count == 0
            || triangle_count > MAX_TRIANGLES
            || points.is_null()
            || triangles.is_null()
            || kind > 1
        {
            return Err(1);
        }
        // Borrow without copying or allocating a snapshot. The portable
        // resolver validates all finite values/topology before fan allocation.
        let points = unsafe { std::slice::from_raw_parts(points, point_count) };
        let triangles = unsafe { std::slice::from_raw_parts(triangles, triangle_count) };
        let component = if kind == 0 {
            Component::Vertex(first)
        } else {
            Component::Edge(first, second)
        };
        mesh_component_frame(points, triangles, component).map_err(|_| 2)
    });
    let frame = match resolved {
        Ok(Ok(frame)) => frame,
        Ok(Err(status)) => return status,
        Err(_) => return 1,
    };
    let packed = [
        frame.origin[0],
        frame.origin[1],
        frame.origin[2],
        frame.axes[0][0],
        frame.axes[0][1],
        frame.axes[0][2],
        frame.axes[1][0],
        frame.axes[1][1],
        frame.axes[1][2],
        frame.axes[2][0],
        frame.axes[2][1],
        frame.axes[2][2],
    ];
    unsafe { std::ptr::copy_nonoverlapping(packed.as_ptr(), output, packed.len()) };
    0
}
