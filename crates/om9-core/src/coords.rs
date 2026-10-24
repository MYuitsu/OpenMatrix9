// SPDX-License-Identifier: LGPL-2.1-or-later
//! World/local coordinate input (OM9-L3-01-08-007).
//!
//! A local point is not a world point: `resolve` is the only door, and it refuses a
//! local point without a frame. The parser keeps exactly the accepted grammar and the
//! error labels of the session it replaces, so command prompts and messages are
//! unchanged.
use crate::{
    Error, Result,
    geometry::{Point3, Unit, finite},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldPoint {
    xyz: Point3,
}
impl WorldPoint {
    pub fn try_new(xyz: Point3) -> Result<Self> {
        finite(xyz)?;
        Ok(Self { xyz })
    }
    pub fn xyz(self) -> Point3 {
        self.xyz
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalPoint {
    xyz: Point3,
}
impl LocalPoint {
    pub fn try_new(xyz: Point3) -> Result<Self> {
        finite(xyz)?;
        Ok(Self { xyz })
    }
    pub fn xyz(self) -> Point3 {
        self.xyz
    }
}

pub trait LocalFrame {
    fn to_world(&self, point: LocalPoint) -> Result<WorldPoint>;
    fn to_local(&self, point: WorldPoint) -> Result<LocalPoint>;
    /// A local displacement (not a position) in world axes. The default implementation
    /// subtracts the frame origin, so a shifted frame cannot leak its origin into a
    /// relative entry.
    fn delta_to_world(&self, delta: LocalPoint) -> Result<Point3> {
        let origin = self.to_world(LocalPoint::try_new([0.; 3])?)?.xyz();
        let point = self.to_world(delta)?.xyz();
        Ok([
            point[0] - origin[0],
            point[1] - origin[1],
            point[2] - origin[2],
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateInput {
    World(WorldPoint),
    Local(LocalPoint),
    /// `@x,y[,z]` is a displacement from the previous input point in the active frame.
    Relative(LocalPoint),
}

/// The only conversion door: a local point needs an explicit frame.
pub fn resolve(input: CoordinateInput, frame: Option<&dyn LocalFrame>) -> Result<WorldPoint> {
    match input {
        CoordinateInput::World(point) => Ok(point),
        CoordinateInput::Local(point) => {
            let frame = frame.ok_or(Error::Missing("coordinate frame"))?;
            frame.to_world(point)
        }
        CoordinateInput::Relative(_) => Err(Error::Missing("relative base point")),
    }
}

/// Resolve world/local/relative input. Relative input additionally needs the previous
/// point; it is a displacement in the active frame, never an absolute position.
pub fn resolve_with_base(
    input: CoordinateInput,
    frame: Option<&dyn LocalFrame>,
    base: Option<WorldPoint>,
) -> Result<WorldPoint> {
    match input {
        CoordinateInput::World(point) => Ok(point),
        CoordinateInput::Local(point) => {
            let frame = frame.ok_or(Error::Missing("coordinate frame"))?;
            frame.to_world(point)
        }
        CoordinateInput::Relative(delta) => {
            let frame = frame.ok_or(Error::Missing("coordinate frame"))?;
            let base = base.ok_or(Error::Missing("relative base point"))?;
            let delta = frame.delta_to_world(delta)?;
            let point = base.xyz();
            WorldPoint::try_new([
                point[0] + delta[0],
                point[1] + delta[1],
                point[2] + delta[2],
            ])
        }
    }
}

/// Parse `Wx,y[,z]` (world), `@x,y[,z]` (relative) or bare `x,y[,z]` (local).
pub fn parse_input(text: &str, unit: Unit) -> Result<CoordinateInput> {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix('W').or_else(|| text.strip_prefix('w')) {
        return Ok(CoordinateInput::World(WorldPoint::try_new(
            parse_xyz(rest, unit)?.xyz(),
        )?));
    }
    if let Some(rest) = text.strip_prefix('@') {
        return Ok(CoordinateInput::Relative(parse_local(rest, unit)?));
    }
    Ok(CoordinateInput::Local(parse_local(text, unit)?))
}

fn parse_xyz(text: &str, unit: Unit) -> Result<LocalPoint> {
    let parts: Vec<&str> = text.split(',').map(str::trim).collect();
    if !(2..=3).contains(&parts.len()) {
        return Err(Error::Invalid("x,y[,z] coordinate"));
    }
    let mut xyz = [0.; 3];
    for (index, part) in parts.iter().enumerate() {
        let value: f64 = part
            .parse()
            .map_err(|_| Error::Invalid("coordinate number"))?;
        xyz[index] = unit.length_to_mm(value)?;
    }
    LocalPoint::try_new(xyz)
}

/// `x,y` or `x,y,z` in `unit`; z defaults to 0. The labels are the session's own.
pub fn parse_local(text: &str, unit: Unit) -> Result<LocalPoint> {
    parse_xyz(text, unit)
}
