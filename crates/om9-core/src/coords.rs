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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateInput {
    World(WorldPoint),
    Local(LocalPoint),
}

/// The only conversion door: a local point needs an explicit frame.
pub fn resolve(input: CoordinateInput, frame: Option<&dyn LocalFrame>) -> Result<WorldPoint> {
    match input {
        CoordinateInput::World(point) => Ok(point),
        CoordinateInput::Local(point) => {
            let frame = frame.ok_or(Error::Missing("coordinate frame"))?;
            frame.to_world(point)
        }
    }
}

/// `x,y` or `x,y,z` in `unit`; z defaults to 0. The labels are the session's own.
pub fn parse_local(text: &str, unit: Unit) -> Result<LocalPoint> {
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
