// SPDX-License-Identifier: LGPL-2.1-or-later
use crate::camera::CPlane;
use om9_core::{Result, geometry::Point3};
#[derive(Debug, Clone, PartialEq)]
pub struct GridLine {
    pub start: Point3,
    pub end: Point3,
    pub color: [u8; 4],
    pub major: bool,
    pub axis: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    pub lines: Vec<GridLine>,
}
impl Grid {
    pub fn new(plane: CPlane) -> Result<Self> {
        plane.validate()?;
        // Side View's local X/Y are world Y/Z. Keep the existing local colours
        // for other planes, while matching the world-axis colours in this view.
        let side = plane.x == [0., 1., 0.] && plane.y == [0., 0., 1.];
        let x_axis = if side { [0, 200, 220, 255] } else { [210, 35, 35, 255] };
        let y_axis = if side { [235, 180, 50, 255] } else { [0, 200, 220, 255] };
        let mut lines = Vec::with_capacity(82);
        for i in -20..=20 {
            let major = i % 5 == 0;
            let gray = if major { 97 } else { 48 };
            lines.push(GridLine {
                start: plane.point([i as f64, -20., 0.])?,
                end: plane.point([i as f64, 20., 0.])?,
                color: if i == 0 {
                    y_axis
                } else {
                    [gray, gray, gray, 255]
                },
                major,
                axis: i == 0,
            });
            lines.push(GridLine {
                start: plane.point([-20., i as f64, 0.])?,
                end: plane.point([20., i as f64, 0.])?,
                color: if i == 0 {
                    x_axis
                } else {
                    [gray, gray, gray, 255]
                },
                major,
                axis: i == 0,
            });
        }
        Ok(Self { lines })
    }
}
