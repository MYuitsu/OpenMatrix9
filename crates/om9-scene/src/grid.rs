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
        let mut lines = Vec::with_capacity(82);
        for i in -20..=20 {
            let major = i % 5 == 0;
            let gray = if major { 97 } else { 48 };
            lines.push(GridLine {
                start: plane.point([i as f64, -20., 0.])?,
                end: plane.point([i as f64, 20., 0.])?,
                color: if i == 0 {
                    [0, 200, 220, 255]
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
                    [210, 35, 35, 255]
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
