// SPDX-License-Identifier: LGPL-2.1-or-later
use super::Controller;
use om9_addon_abi::session::{Event, Geo, Input, OptionToken, Reply};
use om9_core::{
    Error, Result,
    budget::{Limits, Usage},
    cage_isocurve::{CagePatches, EdgeSpan},
    curve_surface_ops::closest_point_on_placed_curve,
    geometry::{Curve, Geometry, NurbsCurve, Point3, Transform},
    identity::{DocumentId, ObjectId},
    model::Entity,
    transaction::{Mutation, TransactionRequest},
};
use om9_scene::scene::{PickKind, Subelement};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const PICK: &str = "tsExtractEdges";
const LABEL: &str = "Extract Edge Curves";
pub(super) const OFFSET: &str = "tsExtractEdgesOffset";

#[derive(Debug)]
pub(super) struct ExtractEdges {
    stamp: (DocumentId, u64, u64, u64),
    spans: Vec<(Transform, EdgeSpan)>,
}

#[derive(Default, Deserialize, Serialize)]
struct Settings {
    fraction: Option<f64>,
    both: bool,
}

impl ExtractEdges {
    fn stamp(c: &Controller) -> (DocumentId, u64, u64, u64) {
        (
            c.document,
            c.coordinator.epoch(),
            c.selection_revision(),
            c.sub_revision(),
        )
    }

    pub(super) fn current(&self, c: &Controller) -> bool {
        self.stamp == Self::stamp(c)
    }

    pub(super) fn capture(c: &Controller) -> Result<Self> {
        if c.tspline_component_kind() != Some(PickKind::Edge) {
            return Err(Error::Invalid("Select T-Spline edges"));
        }
        let limits = c.coordinator.limits();
        let count = c.subselection().len();
        // Each retained edge owns at most two 4x4 patches, their knots and weights.
        Usage::array::<[u8; 3072]>(count)?.check(limits)?;
        if count as u64 > limits.max_outputs {
            return Err(Error::Budget("extracted curves"));
        }
        let scene = c.scene().ok_or(Error::Missing("T-Spline scene"))?;
        let mut groups = BTreeMap::<ObjectId, BTreeSet<[u32; 2]>>::new();
        for pick in c.subselection() {
            if !groups.contains_key(&pick.root) {
                pick.validate(scene, c.coordinator.epoch())?;
            } else if pick.source != scene.document.stamp(pick.root)?
                || pick.world_epoch != scene.epoch
                || !pick.path.is_empty()
            {
                return Err(Error::Stale("T-Spline edge"));
            }
            let Subelement::MeshEdge { a, b } = pick.subset else {
                return Err(Error::Invalid("cage edge"));
            };
            groups
                .entry(pick.root)
                .or_default()
                .insert([a.min(b), a.max(b)]);
        }
        let mut spans = Vec::with_capacity(count);
        for (root, edges) in groups {
            let entity = &c.document().entities[&root];
            let Geometry::Mesh(mesh) = &entity.geometry else {
                return Err(Error::Invalid("cage mesh"));
            };
            let patches = CagePatches::new(mesh, limits)?;
            for edge in edges {
                spans.push((entity.placement, patches.edge_span(edge)?));
            }
        }
        Ok(Self {
            stamp: Self::stamp(c),
            spans,
        })
    }

    fn curves(&self, settings: &Settings, limits: &Limits) -> Result<Vec<(Transform, NurbsCurve)>> {
        let f = settings
            .fraction
            .ok_or(Error::Missing("Pick or enter an offset"))?;
        let opposite = settings.both && f != 0.;
        let count = self
            .spans
            .len()
            .checked_mul(if opposite { 2 } else { 1 })
            .ok_or(Error::Overflow("curve count"))?;
        if count as u64 > limits.max_outputs {
            return Err(Error::Budget("extracted curves"));
        }
        let mut curves = Vec::with_capacity(count);
        for (placement, span) in &self.spans {
            curves.push((*placement, span.curve(f, limits)?));
            if opposite {
                curves.push((*placement, span.curve(-f, limits)?));
            }
        }
        Ok(curves)
    }

    fn pick_fraction(&self, point: Point3, limits: &Limits) -> Result<f64> {
        let mut best = None;
        for (placement, span) in &self.spans {
            for (sign, [start, end]) in span.perpendicular_edges() {
                let guide = Curve::Line {
                    start,
                    end,
                    domain: [0., 1.],
                };
                if start == end {
                    continue;
                }
                let (fraction, _, dist) =
                    closest_point_on_placed_curve(&guide, *placement, point, limits)?;
                if best.is_none_or(|(d, _)| dist < d) {
                    best = Some((dist, sign * fraction));
                }
            }
        }
        best.map(|(_, f)| f)
            .ok_or(Error::Invalid("pick on a perpendicular cage edge"))
    }

    pub(super) fn step(&self, c: &Controller, e: &Event) -> Result<Reply> {
        if !self.current(c) {
            return Err(Error::Stale("T-Spline edge selection changed"));
        }
        let mut settings: Settings = if e.input == Input::Start {
            Settings::default()
        } else {
            serde_json::from_value(e.state.clone())
                .map_err(|_| Error::Invalid("edge offset state"))?
        };
        let limits = c.coordinator.limits();
        let k = c.document().page_unit.factor();
        match &e.input {
            Input::Text { text } => {
                settings.fraction = Some(
                    text.trim()
                        .parse()
                        .map_err(|_| Error::Invalid("enter an offset between -1 and 1"))?,
                )
            }
            Input::Hover { point } | Input::Point { point } => {
                settings.fraction = Some(self.pick_fraction(point.map(|v| v / k), limits)?)
            }
            Input::Option { name, value } if name == "BothSides" => {
                settings.both = match value.as_deref().map(str::to_ascii_lowercase).as_deref() {
                    None => !settings.both,
                    Some("yes" | "on" | "true") => true,
                    Some("no" | "off" | "false") => false,
                    _ => return Err(Error::Invalid("BothSides expects Yes or No")),
                };
            }
            Input::Start | Input::Selection | Input::Enter => {}
            _ => return Err(Error::Invalid("pick or enter an offset between -1 and 1")),
        }
        let mut reply = Reply {
            prompt:
                "Pick a point on a perpendicular edge or type an offset (-1 to 1), Enter to finish"
                    .into(),
            options: vec![OptionToken::toggle("BothSides", settings.both)],
            done: e.input == Input::Enter,
            state: serde_json::to_value(&settings)
                .map_err(|_| Error::Invalid("edge offset state"))?,
            ..Default::default()
        };
        if reply.done && settings.fraction.is_none() {
            return Err(Error::Missing("Pick or enter an offset"));
        }
        if settings.fraction.is_some() && !reply.done {
            let curves = self.curves(&settings, limits)?;
            let samples = curves
                .len()
                .checked_mul(super::addon_session::PREVIEW_SAMPLES)
                .ok_or(Error::Overflow("edge preview samples"))?;
            let mut preview = Usage::array::<[Point3; 2]>(samples)?;
            preview.points = samples as u64;
            preview.add(Usage::array::<[u8; 3072]>(self.spans.len())?, limits)?;
            for (placement, n) in curves {
                let points = n
                    .poles
                    .into_iter()
                    .map(|p| Ok(placement.point(p)?.map(|v| v * k)))
                    .collect::<Result<_>>()?;
                reply.preview.push(Geo::Nurbs {
                    degree: n.degree,
                    points,
                    weights: n.weights,
                    knots: n.knots,
                });
            }
        }
        Ok(reply)
    }

    fn publish(&self, c: &mut Controller, settings: Settings, select_last: bool) -> Result<usize> {
        if self.stamp != Self::stamp(c) {
            return Err(Error::Stale("T-Spline edge selection changed"));
        }
        let curves = self.curves(&settings, c.coordinator.limits())?;
        let mut ids = Vec::with_capacity(curves.len());
        let mut mutations = Vec::with_capacity(curves.len());
        for (placement, curve) in curves {
            let mut entity = Entity::new(
                LABEL,
                Geometry::Curve(Curve::Nurbs(curve)),
                c.document().current_layer,
            );
            entity.placement = placement;
            if entity.attributes.layer.is_some() {
                entity.attributes.color = None;
            }
            ids.push(entity.id);
            mutations.push(Mutation::Insert {
                document: c.document,
                entity,
            });
        }
        c.publish_request_with_selection(
            TransactionRequest::new(LABEL, mutations),
            select_last.then_some(ids.as_slice()),
        )?;
        Ok(ids.len())
    }

    pub(super) fn publish_center(&self, c: &mut Controller, select_last: bool) -> Result<usize> {
        self.publish(
            c,
            Settings {
                fraction: Some(0.),
                both: false,
            },
            select_last,
        )
    }

    pub(super) fn finish_offset(&self, c: &mut Controller, reply: &Reply) -> Result<usize> {
        let settings = serde_json::from_value(reply.state.clone())
            .map_err(|_| Error::Invalid("edge offset state"))?;
        self.publish(c, settings, false)
    }
}

impl Controller {
    pub(crate) fn collecting_cage_edges(&self) -> bool {
        self.addon_session()
            .is_some_and(|s| s.owner == "om9" && s.code == PICK)
    }

    pub(super) fn start_edge_selection(&mut self) -> Result<()> {
        if self.tspline_component_kind() != Some(PickKind::Edge) {
            self.clear_subselection();
        }
        self.start_interactive_session(PICK, LABEL, "om9".into(), false, false)
    }

    pub(super) fn extract_cage_edges(&mut self, offset: bool) -> Result<()> {
        if offset {
            return self.start_interactive_session(
                OFFSET,
                "Extract Edges Offset",
                "om9".into(),
                false,
                false,
            );
        }
        ExtractEdges::capture(self)?.publish_center(self, true)?;
        self.clear_subselection();
        Ok(())
    }
}
