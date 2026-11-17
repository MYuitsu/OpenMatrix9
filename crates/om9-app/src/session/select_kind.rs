// SPDX-License-Identifier: LGPL-2.1-or-later
//! Rhino 5's selection commands by object type: `SelAll`, `SelNone`, `SelCrv`, `SelLine`,
//! `SelPolyline`, `SelPolygon` (closed polylines), `SelClosedCrv`, `SelOpenCrv`, `SelPt`, `SelSrf`, `SelPolysrf`, `SelMesh`.
//!
//! As in Rhino, a `Sel…` command adds the visible, selectable objects of its type to the
//! selection (it does not clear what is already selected) and reports how many it added, e.g.
//! "2 curves added to selection." `SelNone` clears the selection.
use super::Controller;
use om9_core::geometry::{Curve, Geometry};
use om9_core::identity::ObjectId;
use om9_core::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    All,
    None,
    Curve,
    Line,
    Polyline,
    Polygon,
    ClosedCurve,
    OpenCurve,
    Point,
    Surface,
    Polysurface,
    Mesh,
}

impl Kind {
    fn parse(text: &str) -> Option<Kind> {
        let name = text.trim().trim_start_matches(['_', '-', '!', ' ']);
        Some(match name.to_ascii_lowercase().as_str() {
            "selall" => Kind::All,
            "selnone" => Kind::None,
            "selcrv" => Kind::Curve,
            "selline" => Kind::Line,
            "selpolyline" => Kind::Polyline,
            // Closed polylines (triangles, squares, hexagons …).
            "selpolygon" => Kind::Polygon,
            "selclosedcrv" => Kind::ClosedCurve,
            "selopencrv" => Kind::OpenCurve,
            "selpt" => Kind::Point,
            "selsrf" => Kind::Surface,
            "selpolysrf" => Kind::Polysurface,
            "selmesh" => Kind::Mesh,
            _ => return None,
        })
    }

    /// (one, many) as Rhino words them.
    fn nouns(self) -> (&'static str, &'static str) {
        match self {
            Kind::All | Kind::None => ("object", "objects"),
            Kind::Curve | Kind::ClosedCurve | Kind::OpenCurve => ("curve", "curves"),
            Kind::Line => ("line", "lines"),
            Kind::Polyline => ("polyline", "polylines"),
            Kind::Polygon => ("polygon", "polygons"),
            Kind::Point => ("point", "points"),
            Kind::Surface => ("surface", "surfaces"),
            Kind::Polysurface => ("polysurface", "polysurfaces"),
            Kind::Mesh => ("mesh", "meshes"),
        }
    }
}

/// The start and end of a curve (for open/closed), its segments flattened.
fn ends(c: &Curve) -> Option<([f64; 3], [f64; 3])> {
    let p = |q: om9_core::geometry::Point3| q;
    match c {
        Curve::Line { start, end, .. } => Some((p(*start), p(*end))),
        Curve::Circle { domain, angle_domain, .. } => {
            let span = angle_domain.unwrap_or(*domain);
            // A full circle is closed; an arc is open.
            let full = (span[1] - span[0]).abs() >= std::f64::consts::TAU - 1e-9;
            Some(if full { ([0.0; 3], [0.0; 3]) } else { ([0.0; 3], [1.0, 0.0, 0.0]) })
        }
        Curve::Nurbs(n) => {
            if n.periodic {
                return Some(([0.0; 3], [0.0; 3]));
            }
            Some((p(*n.poles.first()?), p(*n.poles.last()?)))
        }
        Curve::Polycurve { segments, .. } => {
            let first = ends(segments.first()?)?.0;
            let last = ends(segments.last()?)?.1;
            Some((first, last))
        }
    }
}

fn closed(c: &Curve) -> bool {
    ends(c).is_some_and(|(a, b)| {
        let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() < 1e-6
    })
}

/// Straight pieces only (a line is one).
fn straight(c: &Curve) -> bool {
    match c {
        Curve::Line { .. } => true,
        Curve::Nurbs(n) => n.degree == 1,
        Curve::Polycurve { segments, .. } => segments.iter().all(straight),
        Curve::Circle { .. } => false,
    }
}

/// How many straight segments a straight curve has.
fn pieces(c: &Curve) -> usize {
    match c {
        Curve::Line { .. } => 1,
        Curve::Nurbs(n) => n.poles.len().saturating_sub(1),
        Curve::Polycurve { segments, .. } => segments.iter().map(pieces).sum(),
        Curve::Circle { .. } => 0,
    }
}

fn matches(kind: Kind, g: &Geometry) -> bool {
    match (kind, g) {
        (Kind::All, _) => true,
        (Kind::None, _) => false,
        (Kind::Curve, Geometry::Curve(_)) => true,
        (Kind::Line, Geometry::Curve(c)) => straight(c) && pieces(c) == 1,
        (Kind::Polyline, Geometry::Curve(c)) => straight(c) && pieces(c) >= 2,
        (Kind::Polygon, Geometry::Curve(c)) => straight(c) && pieces(c) >= 3 && closed(c),
        (Kind::ClosedCurve, Geometry::Curve(c)) => closed(c),
        (Kind::OpenCurve, Geometry::Curve(c)) => !closed(c),
        (Kind::Point, Geometry::Point(_)) => true,
        (Kind::Mesh, Geometry::Mesh(_)) => true,
        // One face: a surface (trimmed or not); more: a polysurface (solids too).
        (Kind::Surface, Geometry::Surface(_)) => true,
        (Kind::Surface, Geometry::Brep(b)) => b.topology().faces.len() == 1,
        (Kind::Polysurface, Geometry::Brep(b)) => b.topology().faces.len() > 1,
        (Kind::Polysurface, Geometry::NativeBrep(_)) => true,
        _ => false,
    }
}

impl Controller {
    /// Runs `text` when it is one of the selection commands above; `None` otherwise.
    pub(super) fn select_kind_text(&mut self, text: &str) -> Option<Result<usize>> {
        let kind = Kind::parse(text)?;
        Some(self.select_kind(kind))
    }

    fn select_kind(&mut self, kind: Kind) -> Result<usize> {
        if kind == Kind::None {
            let had = self.selection.len();
            self.select_objects(&[])?;
            self.addon_messages.push(format!("{had} {} deselected.", if had == 1 { "object" } else { "objects" }));
            return Ok(had);
        }
        let scene = self.scene.as_ref().ok_or(Error::Missing("selection scene"))?;
        let document = self.document();
        let mut ids: Vec<ObjectId> = self.selection.iter().copied().collect();
        let mut added = 0usize;
        let mut seen = std::collections::BTreeSet::new();
        for object in &scene.objects {
            if !object.visible || !object.selectable || !object.has_pick_geometry() || !seen.insert(object.root) {
                continue;
            }
            if ids.contains(&object.root) {
                continue;
            }
            let Some(entity) = document.entities.get(&object.root) else { continue };
            if matches(kind, &entity.geometry) {
                ids.push(object.root);
                added += 1;
            }
        }
        if added > 0 {
            self.select_objects(&ids)?;
        }
        let (one, many) = kind.nouns();
        self.addon_messages.push(match added {
            0 => format!("No {many} added to selection."),
            1 => format!("1 {one} added to selection."),
            n => format!("{n} {many} added to selection."),
        });
        Ok(added)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_parse_as_rhino_types_them() {
        assert_eq!(Kind::parse("SelCrv"), Some(Kind::Curve));
        assert_eq!(Kind::parse("_selsrf"), Some(Kind::Surface));
        assert_eq!(Kind::parse(" SelPolysrf "), Some(Kind::Polysurface));
        assert_eq!(Kind::parse("SelPt"), Some(Kind::Point));
        assert_eq!(Kind::parse("SelPolyline"), Some(Kind::Polyline));
        assert_eq!(Kind::parse("SelPolygon"), Some(Kind::Polygon));
        assert_eq!(Kind::parse("Sel"), None);
    }
}
