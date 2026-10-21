//! Numeric gumball edits: typing an exact distance, angle or factor instead of dragging.
//!
//! Each entry is one transaction on the current selection, about the gumball pivot, so it is one Undo
//! entry exactly like a drag. The commands are the ones a jewellery modeller uses most: a millimetre
//! along an axis, a degree about an axis, a scale factor.
use super::Controller;
use om9_core::{
    Error, Result,
    geometry::Transform,
    gumball::{self, Axis},
    transaction::{Mutation, TransactionRequest},
};

impl Controller {
    /// Applies `delta` to every selected object in one transaction and returns how many moved.
    ///
    /// `delta` is already expressed in world space and about the pivot the caller chose; the placement of
    /// each object is `delta.compose(placement)`, the same composition the drag path uses.
    pub fn numeric_transform(&mut self, delta: Transform, name: &str) -> Result<usize> {
        let targets = self.gumball_targets();
        if targets.is_empty() {
            return Err(Error::Missing("selection"));
        }
        if self.scene.is_none() {
            return Err(Error::Missing("gumball scene"));
        }
        let mut mutations = Vec::with_capacity(targets.len());
        for id in targets {
            let Some(entity) = self.document().entities.get(&id) else {
                return Err(Error::Missing("object"));
            };
            let placement = delta.compose(entity.placement)?;
            if placement == entity.placement {
                continue;
            }
            mutations.push(Mutation::Placement {
                source: self.document().stamp(id)?,
                placement,
            });
        }
        if mutations.is_empty() {
            return Ok(0);
        }
        let held = self.scene_cache_usage()?;
        let prepared = self
            .coordinator
            .prepare_with_held_usage(TransactionRequest::new(name, mutations), held)?;
        let receipt = self.coordinator.commit(prepared)?;
        if receipt.published {
            self.refresh_scene()?;
            self.sync_published(om9_core::changes::ChangeCause::Edit);
        }
        Ok(self.gumball_targets().len())
    }

    /// Moves the selection by a world delta.
    pub fn numeric_move(&mut self, delta: [f64; 3]) -> Result<usize> {
        self.numeric_transform(Transform::translation(delta), "Move")
    }

    /// Turns the selection about its pivot, `degrees` counter-clockwise around `axis`.
    pub fn numeric_rotate(&mut self, axis: Axis, degrees: f64) -> Result<usize> {
        if !degrees.is_finite() {
            return Err(Error::Invalid("rotation angle"));
        }
        let pivot = self.gumball_pivot().ok_or(Error::Missing("selection"))?;
        let direction = pivot.axis(axis);
        let transform = gumball::rotation_about(pivot.origin, direction, degrees.to_radians())?;
        self.numeric_transform(transform, "Rotate")
    }

    /// Scales the selection about its pivot: one axis, or uniformly when `axis` is `None`.
    pub fn numeric_scale(&mut self, axis: Option<Axis>, factor: f64) -> Result<usize> {
        if !factor.is_finite() || factor <= 0.0 {
            return Err(Error::Invalid("scale factor"));
        }
        let pivot = self.gumball_pivot().ok_or(Error::Missing("selection"))?;
        let mut transform = Transform::identity();
        match axis {
            Some(axis) => {
                transform = transform.compose(gumball::scaling_along(
                    pivot.origin,
                    pivot.axis(axis),
                    factor,
                )?)?
            }
            None => {
                // One matrix about the pivot: composing three axis scalings would multiply the pivot
                // offset three times.
                let [x, y, z] = pivot.origin;
                // Row-major with the translation in the last column, matching `Transform::translation`.
                transform = Transform([
                    [factor, 0.0, 0.0, x * (1.0 - factor)],
                    [0.0, factor, 0.0, y * (1.0 - factor)],
                    [0.0, 0.0, factor, z * (1.0 - factor)],
                    [0.0, 0.0, 0.0, 1.0],
                ]);
            }
        }
        self.numeric_transform(transform, "Scale")
    }
}

/// Parses a numeric edit typed in the command line: `MoveX 5`, `Move 1,2,3`, `RotateZ 30`, `Scale 2`,
/// `ScaleY 1.5`.
pub(crate) enum NumericEdit {
    Move([f64; 3]),
    Rotate(Axis, f64),
    Scale(Option<Axis>, f64),
}

pub(crate) fn parse(text: &str) -> Option<NumericEdit> {
    let mut words = text.split_whitespace();
    let command = words.next()?;
    let argument = words.next()?;
    if words.next().is_some() {
        return None;
    }
    let axis = |suffix: &str| match suffix.to_ascii_uppercase().as_str() {
        "X" => Some(Axis::X),
        "Y" => Some(Axis::Y),
        "Z" => Some(Axis::Z),
        _ => None,
    };
    match command.to_ascii_lowercase().as_str() {
        "move" => {
            let mut parts = argument.split(',');
            let x = parts.next()?.parse().ok()?;
            let y = parts.next()?.parse().ok()?;
            let z = parts.next()?.parse().ok()?;
            if parts.next().is_some() {
                return None;
            }
            Some(NumericEdit::Move([x, y, z]))
        }
        "movex" | "movey" | "movez" => {
            let distance = argument.parse().ok()?;
            let axis = axis(&command[4..])?;
            Some(NumericEdit::Move(match axis {
                Axis::X => [distance, 0.0, 0.0],
                Axis::Y => [0.0, distance, 0.0],
                Axis::Z => [0.0, 0.0, distance],
            }))
        }
        "rotate" | "rotatex" | "rotatey" | "rotatez" => {
            let degrees = argument.parse().ok()?;
            let axis = if command.eq_ignore_ascii_case("rotate") {
                Axis::Z
            } else {
                axis(&command[6..])?
            };
            Some(NumericEdit::Rotate(axis, degrees))
        }
        "scale" => {
            let factor = argument.parse().ok()?;
            Some(NumericEdit::Scale(None, factor))
        }
        "scalex" | "scaley" | "scalez" => {
            let factor = argument.parse().ok()?;
            Some(NumericEdit::Scale(axis(&command[5..]), factor))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_parser_reads_each_form() {
        assert!(matches!(
            parse("Move 1,2,3"),
            Some(NumericEdit::Move([1.0, 2.0, 3.0]))
        ));
        assert!(matches!(
            parse("MoveX 5"),
            Some(NumericEdit::Move([5.0, 0.0, 0.0]))
        ));
        assert!(matches!(
            parse("MoveZ -2.5"),
            Some(NumericEdit::Move([0.0, 0.0, -2.5]))
        ));
        assert!(matches!(
            parse("RotateZ 30"),
            Some(NumericEdit::Rotate(Axis::Z, 30.0))
        ));
        assert!(matches!(
            parse("RotateX -90"),
            Some(NumericEdit::Rotate(Axis::X, -90.0))
        ));
        assert!(matches!(
            parse("Scale 2"),
            Some(NumericEdit::Scale(None, 2.0))
        ));
        assert!(matches!(
            parse("ScaleY 1.5"),
            Some(NumericEdit::Scale(Some(Axis::Y), 1.5))
        ));
        assert!(parse("Move 1,2").is_none());
        assert!(parse("MoveX").is_none());
        assert!(parse("Scale abc").is_none());
        assert!(parse("RotateW 3").is_none());
    }
}
