// SPDX-License-Identifier: LGPL-2.1-or-later
//! The ordered gem row shared by every setting command (`OM9-L3-04-15-001` input contract).
//!
//! The parent module exclusion is explicit: *"Phải dùng chung input gems và output geometry."*
//! Prong Builder, Metal from Gems, Channel Border, Pull to Rail and the mass report therefore
//! consume one [`GemRow`] and one [`GemStone`] type instead of defining their own stone data.
//!
//! A row is **ordered**: stone `i` is followed by stone `i + 1`, and a closed row wraps from the
//! last stone back to the first. Stored coordinates are millimetres in the document space; the
//! axis is the girdle plane normal and must be unit length.
use crate::{
    Error, Result,
    geometry::{Point3, finite},
    settings::{MAX_INSTANCES, add, cross, dot, mul, norm, sub, unit},
};

/// Whether the stone row is an open line or a closed loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RowShape {
    Open,
    Closed,
}

/// One round gem of a row.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GemStone {
    /// Stable identity from the gem layout owner (`OM9-L2-04-10`); never re-derived here.
    pub id: String,
    /// Girdle centre, in mm.
    pub center: Point3,
    pub diameter_mm: f64,
    /// Girdle plane normal (unit), pointing to the table side.
    pub axis: Point3,
}

impl GemStone {
    pub fn round(
        id: impl Into<String>,
        center: Point3,
        diameter_mm: f64,
        axis: Point3,
    ) -> Result<Self> {
        let stone = Self {
            id: id.into(),
            center,
            diameter_mm,
            axis,
        };
        stone.validate()?;
        Ok(stone)
    }

    pub fn radius_mm(&self) -> f64 {
        self.diameter_mm / 2.0
    }

    fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(Error::Missing("gem id"));
        }
        finite(self.center)?;
        finite(self.axis)?;
        if !self.diameter_mm.is_finite() || self.diameter_mm <= 0.0 {
            return Err(Error::Invalid("gem diameter"));
        }
        let length = norm(self.axis);
        if !length.is_finite() || (length - 1.0).abs() > 1.0e-6 {
            return Err(Error::Invalid("gem axis"));
        }
        Ok(())
    }
}

/// The touching gap between two adjacent stones.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Junction {
    /// Index of the first stone; for a closed row the last junction wraps to stone `0`.
    pub from: usize,
    pub to: usize,
    /// Midpoint between the two girdle centres.
    pub center: Point3,
    pub tangent: Point3,
    pub normal: Point3,
    /// Direction to the left of travel inside the girdle plane.
    pub side: Point3,
    /// Clearance between the two stones: centre distance minus both radii.
    pub gap_mm: f64,
    /// True for the wrap junction of a closed row.
    pub wraps: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GemRow {
    pub stones: Vec<GemStone>,
    pub shape: RowShape,
}

impl GemRow {
    pub fn new(stones: Vec<GemStone>, shape: RowShape) -> Result<Self> {
        let row = Self { stones, shape };
        row.validate()?;
        Ok(row)
    }

    pub fn open(stones: Vec<GemStone>) -> Result<Self> {
        Self::new(stones, RowShape::Open)
    }

    pub fn closed(stones: Vec<GemStone>) -> Result<Self> {
        Self::new(stones, RowShape::Closed)
    }

    pub fn len(&self) -> usize {
        self.stones.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stones.is_empty()
    }

    pub fn is_closed(&self) -> bool {
        self.shape == RowShape::Closed
    }

    /// Refuse a row the settings commands cannot consume: an empty row, a closed "loop" with
    /// fewer than three stones, coincident or grossly overlapping stones, mismatched axes and
    /// a non-finite coordinate.
    pub fn validate(&self) -> Result<()> {
        if self.stones.is_empty() {
            return Err(Error::Missing("gem row"));
        }
        if self.stones.len() > MAX_INSTANCES {
            return Err(Error::Budget("gem row"));
        }
        if self.shape == RowShape::Closed && self.stones.len() < 3 {
            return Err(Error::Invalid("closed gem row"));
        }
        let mut ids = std::collections::BTreeSet::new();
        for stone in &self.stones {
            stone.validate()?;
            if !ids.insert(stone.id.trim().to_ascii_lowercase()) {
                return Err(Error::Invalid("duplicate gem id"));
            }
        }
        for pair in self.pairs() {
            let (a, b) = pair;
            let distance = norm(sub(self.stones[b].center, self.stones[a].center));
            if !distance.is_finite() || distance <= 1.0e-9 {
                return Err(Error::Invalid("gem row order"));
            }
            let radii = self.stones[a].radius_mm() + self.stones[b].radius_mm();
            // Stones may touch or sit a little apart; two stones pushed more than half into
            // each other are a row error, not a layout the builder should silently accept.
            if distance + 1.0e-9 < radii * 0.5 {
                return Err(Error::Invalid("gem row overlap"));
            }
            let axis_dot = dot(self.stones[a].axis, self.stones[b].axis);
            if axis_dot < 0.0 {
                return Err(Error::Invalid("gem row axis"));
            }
        }
        Ok(())
    }

    /// Adjacent index pairs, including the wrap pair of a closed row.
    pub fn pairs(&self) -> Vec<(usize, usize)> {
        let count = self.stones.len();
        if count < 2 {
            return Vec::new();
        }
        let mut pairs: Vec<(usize, usize)> = (0..count - 1).map(|i| (i, i + 1)).collect();
        if self.is_closed() {
            pairs.push((count - 1, 0));
        }
        pairs
    }

    pub fn stone(&self, index: usize) -> Result<&GemStone> {
        self.stones
            .get(index)
            .ok_or(Error::Missing("gem row stone"))
    }

    /// Row direction at a stone: the chord between its neighbours. An end stone of an open row
    /// uses the chord to its only neighbour.
    pub fn tangent_at(&self, index: usize) -> Result<Point3> {
        let count = self.stones.len();
        if index >= count {
            return Err(Error::Missing("gem row stone"));
        }
        let previous = if index == 0 {
            if self.is_closed() {
                Some(count - 1)
            } else {
                None
            }
        } else {
            Some(index - 1)
        };
        let next = if index + 1 == count {
            if self.is_closed() {
                Some(0)
            } else {
                None
            }
        } else {
            Some(index + 1)
        };
        let direction = match (previous, next) {
            (Some(a), Some(b)) => sub(self.stones[b].center, self.stones[a].center),
            (None, Some(b)) => sub(self.stones[b].center, self.stones[index].center),
            (Some(a), None) => sub(self.stones[index].center, self.stones[a].center),
            (None, None) => return Err(Error::Invalid("gem row length")),
        };
        unit(direction)
    }

    /// Girdle plane normal at a stone (the stone axis; a single row has no averaged axis).
    pub fn normal_at(&self, index: usize) -> Result<Point3> {
        unit(self.stone(index)?.axis)
    }

    /// In-plane "north" direction at a stone: `normal x tangent`, i.e. to the left of travel.
    pub fn side_at(&self, index: usize) -> Result<Point3> {
        unit(cross(self.normal_at(index)?, self.tangent_at(index)?))
    }

    /// Junctions between consecutive stones, in row order.
    pub fn junctions(&self) -> Result<Vec<Junction>> {
        let mut junctions = Vec::new();
        for (from, to) in self.pairs() {
            let a = self.stone(from)?;
            let b = self.stone(to)?;
            let chord = sub(b.center, a.center);
            let distance = norm(chord);
            let tangent = unit(chord)?;
            let normal = unit(add(a.axis, b.axis))?;
            let side = unit(cross(normal, tangent))?;
            junctions.push(Junction {
                from,
                to,
                center: lerp_mid(a.center, b.center),
                tangent,
                normal,
                side,
                gap_mm: distance - a.radius_mm() - b.radius_mm(),
                wraps: from + 1 != to,
            });
        }
        Ok(junctions)
    }

    /// Indices of the two end stones of an open row; empty for a closed loop.
    pub fn ends(&self) -> Vec<usize> {
        if self.is_closed() || self.stones.is_empty() {
            return Vec::new();
        }
        if self.stones.len() == 1 {
            vec![0]
        } else {
            vec![0, self.stones.len() - 1]
        }
    }

    /// Girdle rails: the left and right envelope polylines through the stone girdles. This is
    /// the first-order rail the channel/metal owner offsets; a tangent-continuous envelope is a
    /// later refinement, so it is not claimed here.
    pub fn girdle_rails(&self) -> Result<[Vec<Point3>; 2]> {
        let mut left = Vec::with_capacity(self.stones.len());
        let mut right = Vec::with_capacity(self.stones.len());
        for index in 0..self.stones.len() {
            let stone = self.stone(index)?;
            let side = self.side_at(index)?;
            left.push(add(stone.center, mul(side, stone.radius_mm())));
            right.push(add(stone.center, mul(side, -stone.radius_mm())));
        }
        Ok([left, right])
    }

    /// Polyline through the stone centres, used as the metal/bead sweep path.
    pub fn centerline(&self) -> Vec<Point3> {
        self.stones.iter().map(|stone| stone.center).collect()
    }

    /// Total chord length through the centres; zero for fewer than two stones.
    pub fn chord_length_mm(&self) -> f64 {
        self.pairs()
            .into_iter()
            .map(|(a, b)| norm(sub(self.stones[b].center, self.stones[a].center)))
            .sum()
    }

    /// The row reversed. A reversed row must produce the same prong set (the end modes are the
    /// only asymmetric part, and they are re-derived from the new order, not mirrored data).
    pub fn reversed(&self) -> Result<Self> {
        let mut stones = self.stones.clone();
        stones.reverse();
        Self::new(stones, self.shape)
    }
}

fn lerp_mid(a: Point3, b: Point3) -> Point3 {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

/// Build a straight open row of round stones with a shared girdle axis.
pub fn straight_row(
    diameters: &[f64],
    spacing_mm: f64,
    origin: Point3,
    direction: Point3,
    axis: Point3,
) -> Result<GemRow> {
    if diameters.is_empty() {
        return Err(Error::Missing("gem row"));
    }
    let direction = unit(direction)?;
    let axis = unit(axis)?;
    if dot(direction, axis).abs() > 1.0e-6 {
        return Err(Error::Invalid("gem row axis"));
    }
    let mut stones = Vec::with_capacity(diameters.len());
    let mut distance = 0.0;
    for (index, diameter) in diameters.iter().enumerate() {
        let stone_diameter = *diameter;
        if !stone_diameter.is_finite() || stone_diameter <= 0.0 {
            return Err(Error::Invalid("gem diameter"));
        }
        if index > 0 {
            let previous = diameters[index - 1];
            distance += (previous + stone_diameter) / 2.0 + spacing_mm;
        }
        stones.push(GemStone::round(
            format!("gem_{:02}", index + 1),
            add(origin, mul(direction, distance)),
            stone_diameter,
            axis,
        )?);
    }
    GemRow::open(stones)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row() -> GemRow {
        straight_row(&[1.0, 1.0, 1.0], 0.0, [0.0; 3], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0])
            .unwrap()
    }

    #[test]
    fn a_straight_row_spaces_stones_by_diameter_plus_gap() {
        let row = straight_row(
            &[2.0, 2.0, 2.0],
            0.5,
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
        )
        .unwrap();
        assert_eq!(row.len(), 3);
        assert!((row.stones[1].center[0] - 2.5).abs() < 1.0e-9);
        assert!((row.stones[2].center[0] - 5.0).abs() < 1.0e-9);
        assert_eq!(row.ends(), vec![0, 2]);
        assert_eq!(row.junctions().unwrap().len(), 2);
    }

    #[test]
    fn zero_spacing_means_touching_stones_with_a_zero_gap() {
        let row = row();
        for junction in row.junctions().unwrap() {
            assert!(junction.gap_mm.abs() < 1.0e-9);
        }
        assert!((row.chord_length_mm() - 2.0).abs() < 1.0e-9);
    }

    #[test]
    fn overlapping_stones_are_refused() {
        let mut stones = row().stones;
        stones[1].center[0] = 0.2;
        assert_eq!(
            GemRow::open(stones).err(),
            Some(Error::Invalid("gem row overlap"))
        );
    }

    #[test]
    fn duplicate_stone_ids_are_refused() {
        let mut stones = row().stones;
        stones[1].id = stones[0].id.clone();
        assert_eq!(
            GemRow::open(stones).err(),
            Some(Error::Invalid("duplicate gem id"))
        );
    }

    #[test]
    fn a_closed_row_wraps_the_last_junction() {
        let stones = (0..6)
            .map(|index| {
                let angle = std::f64::consts::TAU * index as f64 / 6.0;
                GemStone::round(
                    format!("g{index}"),
                    [10.0 * angle.cos(), 10.0 * angle.sin(), 0.0],
                    2.0,
                    [0.0, 0.0, 1.0],
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let row = GemRow::closed(stones).unwrap();
        let junctions = row.junctions().unwrap();
        assert_eq!(junctions.len(), 6);
        assert!(junctions.iter().any(|junction| junction.wraps));
        assert!(row.ends().is_empty());
    }

    #[test]
    fn a_closed_row_needs_three_stones() {
        let stones = row().stones[..2].to_vec();
        assert_eq!(
            GemRow::closed(stones).err(),
            Some(Error::Invalid("closed gem row"))
        );
        // Three stones are a legal loop.
        assert!(GemRow::closed(row().stones).is_ok());
    }

    #[test]
    fn the_side_direction_is_left_of_travel() {
        let row = row();
        let side = row.side_at(1).unwrap();
        // Travel is +X and the girdle normal is +Z, so left is +Y.
        assert!((side[1] - 1.0).abs() < 1.0e-9);
    }

    #[test]
    fn rails_sit_one_radius_each_side_of_the_centres() {
        let row = row();
        let [left, right] = row.girdle_rails().unwrap();
        assert_eq!(left.len(), 3);
        for index in 0..3 {
            assert!((left[index][1] - 0.5).abs() < 1.0e-9);
            assert!((right[index][1] + 0.5).abs() < 1.0e-9);
        }
    }

    #[test]
    fn a_reversed_row_keeps_the_same_stones() {
        let row = row();
        let reversed = row.reversed().unwrap();
        assert_eq!(reversed.len(), row.len());
        assert_eq!(reversed.stones[0].id, row.stones[2].id);
    }
}
