// SPDX-License-Identifier: LGPL-2.1-or-later
//! Matrix menu macros whose complete option sequence is supported by a command owner.
use om9_addon_geom::subd::selection::{Direction, Operation as CageSelection};
use super::{Controller, DispatchOutcome, Error, InvocationOrigin};
use om9_core::{Result, gumball::Axis, top11::Top11Action};

#[derive(Clone, Copy)]
pub enum ContextPreset {
    Group,
    Ungroup,
    Center,
    Hide,
    Show,
    Lock,
    Unlock,
    PointsOff,
    PointsOffSelected,
    SetGripAxis(Axis),
    ProngDrop(f64),
    Bezel(&'static str),
    MeshBoolean(super::mesh_boolean::MeshBoolean),
    TsplineFaces(om9_core::top11::MeshFaceSide),
    TsplineSelection(CageSelection),
    TsplinePaint,
    TsplinePivot,
    TsplineExtract { offset: bool },
    TsplinePickEdges,
    TsplineSplitCurves,
    TsplineFromLines,
    TsplineExtrudeLines,
    TsplineManipulator(om9_scene::gumball_view::Mode, om9_scene::scene::PickKind),
    TsPlane,
    Delete,
    ReduceMesh,
    ViewCapture(bool),
    CurveDirection,
    ExtrudeCrvToPoint,
    ExtrudeCrvTapered,
    MeshOutline,
    ShowSelected,
    Offset {
        command: &'static str,
        both: bool,
        through: bool,
    },
    Builder(om9_core::builder::BuilderKind),
    Cutter(&'static str),
    Split(&'static str),
    GemRow,
}

impl ContextPreset {
    pub fn disabled_reason(self, controller: &Controller) -> Option<&'static str> {
        let count = controller.selected().len();
        match self {
            Self::ExtrudeCrvTapered if !cfg!(feature="occt-brep") => Some("Extrude Tapered requires the exact BRep kernel"),
            Self::ExtrudeCrvToPoint if !cfg!(feature="occt-brep") => Some("Extrude to Point requires the exact BRep kernel"),
            Self::CurveDirection => controller.curve_direction_unavailable(),
            Self::Offset { command, .. } if controller.addon_code(command).is_none() => {
                Some("Enable the Matrix curve tools for this offset preset")
            }
            Self::Bezel(_) if !controller.selected().iter().any(|id| {
                om9_core::gem::gem_record_in(controller.document(), *id).is_some()
            }) => Some("Select a gemstone for the bezel"),
            Self::TsplinePaint if controller.cage_paint_mode().is_none() && controller.tspline_component_kind().is_none() => Some("Select T-Spline components to paint"),
            Self::TsplineSelection(op) => controller.cage_selection_reason(op),
            Self::TsplinePivot | Self::TsplineManipulator(_, _) if !controller.cage_object_manipulator_ready() => Some("Select a T-Spline cage or its components"),
            Self::TsplineExtract { .. } if controller.tspline_component_kind() != Some(om9_scene::scene::PickKind::Edge) => Some("Select T-Spline edges"),
            Self::TsplineFaces(_) if !controller.tspline_faces_selected() => Some("Select T-Spline faces"),
            Self::SetGripAxis(_) if !controller.grips_selected() => {
                Some("Select curve edit or control points")
            }
            Self::PointsOffSelected if controller.top11_grip_mode().is_none() => Some("No objects have points turned on"),
            Self::ShowSelected if !controller.document().entities.values().any(|e| e.attributes.hidden) => Some("No hidden objects to show"),
            Self::Group if count < 2 => Some("Select at least two objects to group"),
            Self::Ungroup
                if !controller
                    .selected()
                    .iter()
                    .any(|id| !controller.document().groups_of(*id).is_empty()) =>
            {
                Some("Select a grouped object")
            }
            Self::Center | Self::Hide | Self::Lock if count == 0 => Some("Select objects first"),
            Self::Split("Point") if count > 1 => Some("Select one curve to split"),
            Self::Split("Isocurve") if count != 1 => Some("Select one object to split"),
            _ => None,
        }
    }
    pub fn parse(script: &str) -> Option<Self> {
        let mut words = script
            .split_whitespace()
            .map(|w| w.trim_start_matches('_').to_ascii_lowercase());
        let command = words.next()?;
        if let Some(op) = super::mesh_boolean::MeshBoolean::from_name(&command) {
            return words.next().is_none().then_some(Self::MeshBoolean(op));
        }
        if matches!(command.as_str(), "offset" | "offsetcrvonsrf") {
            let (mut both, mut through) = (false, false);
            for word in words {
                match word.as_str() {
                    "b" if !both && command == "offset" => both = true,
                    "t" if !through => through = true,
                    _ => return None,
                }
            }
            return (both || through).then_some(Self::Offset {
                command: if command == "offset" {
                    "CurveOffsetCurve"
                } else {
                    "CurveOffsetOffsetCrvOnSrf"
                },
                both,
                through,
            });
        }
        if matches!(command.as_str(), "-_tsextractedges" | "tsextractedges" | "om9-tspline-067") {
            return match words.next().as_deref() {
                None => Some(Self::TsplinePickEdges),
                Some("offset") if words.next().is_none() => Some(Self::TsplineExtract { offset: true }),
                Some("enter") if words.eq(["selnone", "sellast"]) => Some(Self::TsplineExtract { offset: false }),
                _ => None,
            };
        }
        if command == "-_tsmanip" {
            use om9_scene::gumball_view::Mode;
            if words.next()?.as_str() != "enabled=_on" { return None; }
            let style = words.next()?;
            if style == "setpivot" {
                return words.next().is_none().then_some(Self::TsplinePivot);
            }
            let mode = match style.as_str() {
                "manipulator=_translate" => Mode::Move,
                "manipulator=_rotate" => Mode::Rotate,
                "manipulator=_scale" => Mode::Scale,
                _ => return None,
            };
            use om9_scene::scene::PickKind;
            let kind = match words.next()?.as_str() {
                "selectionmode=_object" => PickKind::Object,
                "selectionmode=_vertex" => PickKind::Vertex,
                "selectionmode=_edge" => PickKind::Edge,
                "selectionmode=_face" => PickKind::Face,
                _ => return None,
            };
            return (words.next()?.as_str() == "enter" && words.next().is_none())
                .then_some(Self::TsplineManipulator(mode, kind));
        }
        if command == "-_setpt" {
            let mut flags = [None; 3];
            for _ in 0..3 {
                let word = words.next()?;
                let (key, value) = word.split_once('=')?;
                let axis = ["xset", "yset", "zset"].iter().position(|s| *s == key)?;
                if flags[axis].is_some() {
                    return None;
                }
                flags[axis] = Some(match value.trim_start_matches('_') {
                    "yes" => true,
                    "no" => false,
                    _ => return None,
                });
            }
            if words.next()?.as_str() != "0"
                || words.next().is_some()
                || flags.iter().filter(|v| **v == Some(true)).count() != 1
            {
                return None;
            }
            return [Axis::X, Axis::Y, Axis::Z]
                .into_iter()
                .find(|axis| flags[axis.index()] == Some(true))
                .map(Self::SetGripAxis);
        }
        let Some(option) = words.next() else {
            return match command.as_str() {
                "tsextrudelines" | "om9-tspline-073" => Some(Self::TsplineExtrudeLines),
                "tsfromlines" | "om9-tspline-072" => Some(Self::TsplineFromLines),
                "tssplitcurves" | "om9-tspline-069" => Some(Self::TsplineSplitCurves),
                "tsplane" | "om9-tspline-048" => Some(Self::TsPlane),
                "deletetsplinefaces" => Some(Self::TsplineFaces(om9_core::top11::MeshFaceSide::Remaining)),
                "tsduplicatefaces" | "om9-tspline-055" => Some(Self::TsplineFaces(om9_core::top11::MeshFaceSide::Selected)),
                "delete" => Some(Self::Delete),
                "reducemesh" => Some(Self::ReduceMesh),
                "showselected" => Some(Self::ShowSelected),
                "extrudecrvtapered" => Some(Self::ExtrudeCrvTapered),
                "extrudecrvtopoint" => Some(Self::ExtrudeCrvToPoint),
                "dir" | "om9-util-001" => Some(Self::CurveDirection),
                "viewcapturetofile" | "om9-view-005" => Some(Self::ViewCapture(false)),
                "-viewcapturetofile" => Some(Self::ViewCapture(true)),
                "meshoutline" => Some(Self::MeshOutline),
                "group" => Some(Self::Group),
                "ungroup" => Some(Self::Ungroup),
                "gvcenterobjects" => Some(Self::Center),
                "hide" => Some(Self::Hide),
                "show" => Some(Self::Show),
                "lock" => Some(Self::Lock),
                "unlock" => Some(Self::Unlock),
                "ptoffselected" => Some(Self::PointsOffSelected),
                "pointsoff" => Some(Self::PointsOff),
                "gveternity" | "gvsignet" | "gvbuilderraisedband" => {
                    super::builder::builder_command(&command).map(Self::Builder)
                }
                _ => None,
            };
        };
        let last = words.next();
        if words.next().is_some() {
            return None;
        }
        match (command.as_str(), option.as_str(), last.as_deref()) {
            ("-_tsselection" | "-tsselection", "nextu", None) => Some(Self::TsplineSelection(CageSelection::Step(Direction::NextU))),
            ("-_tsselection" | "-tsselection", "prevu", None) => Some(Self::TsplineSelection(CageSelection::Step(Direction::PrevU))),
            ("-_tsselection" | "-tsselection", "nextv", None) => Some(Self::TsplineSelection(CageSelection::Step(Direction::NextV))),
            ("-_tsselection" | "-tsselection", "prevv", None) => Some(Self::TsplineSelection(CageSelection::Step(Direction::PrevV))),
            ("-_tsselection", "selectring", None) => Some(Self::TsplineSelection(CageSelection::Ring)),
            ("-_tsselection", "togglepaintmode", None) => Some(Self::TsplinePaint),
            ("-_tsselection", "selectloop", None) => Some(Self::TsplineSelection(CageSelection::Loop)),
            ("-_tsselection", "growselection", None) => Some(Self::TsplineSelection(CageSelection::Grow)),
            ("-_tsselection", "shrinkselection", None) => Some(Self::TsplineSelection(CageSelection::Shrink)),
            ("gvbezel", "bezeltype=_bezel", None) => Some(Self::Bezel("Mode=Bezel")),
            ("gvbezel", "bezeltype=_galleryrail", None) => Some(Self::Bezel("Mode=GalleryRail")),
            ("gvbezel", "bezeltype=_underbezel", None) => Some(Self::Bezel("Mode=UnderBezel")),
            ("gvprongadder", value, None) => value
                .strip_prefix("drop=")?
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .map(Self::ProngDrop),
            ("gvazurecutter", "cuttermode=_gem", None) => Some(Self::Cutter("Mode=Gem")),
            ("gvazurecutter", "cuttermode=_azure", None) => Some(Self::Cutter("Mode=Azure")),
            ("split", "point" | "p", None) => Some(Self::Split("Point")),
            ("split", "i", None) => Some(Self::Split("Isocurve")),
            (
                "gvgemoncrv",
                "culetdir=_down" | "culetdir=_object" | "culetdir=_f4",
                Some("spacingmethod=_fixed"),
            ) => Some(Self::GemRow),
            _ => None,
        }
    }

    pub fn start(
        self,
        controller: &mut Controller,
        script: &str,
        origin: InvocationOrigin,
    ) -> Result<DispatchOutcome> {
        if controller.is_running() {
            return Err(Error::Busy);
        }
        match self {
            Self::MeshBoolean(op) => controller.start_mesh_boolean(op)?,
            Self::TsPlane => controller.start_tspline_plane()?,
            Self::TsplineSelection(op) => controller.select_cage_components(op)?,
            Self::TsplinePaint => controller.toggle_cage_paint()?,
            Self::TsplineFromLines => controller.start_from_lines()?,
            Self::TsplineExtrudeLines => controller.start_extrude_lines()?,
            Self::TsplineSplitCurves => controller.start_split_curves()?,
            Self::TsplinePickEdges => controller.start_edge_selection()?,
            Self::TsplineExtract { offset } => controller.extract_cage_edges(offset)?,
            Self::TsplinePivot => controller.start_cage_pivot()?,
            Self::TsplineManipulator(mode, kind) => controller.start_cage_manipulator(mode, kind)?,
            Self::TsplineFaces(side) => controller.edit_tspline_faces(side)?,
            Self::Delete => controller.delete_current_selection()?,
            Self::ReduceMesh => controller.start_reduce_mesh()?,
            Self::ShowSelected => controller.start_isolated_selection(super::show_selected::PickOwners::Hidden)?,
            Self::PointsOffSelected => controller.start_isolated_selection(super::show_selected::PickOwners::PointsOn)?,
            Self::ExtrudeCrvToPoint => controller.start_extrude_point()?,
            Self::ExtrudeCrvTapered => controller.start_tapered()?,
            Self::CurveDirection => controller.start_curve_direction()?,
            Self::ViewCapture(scripted) => controller.start_view_capture(scripted)?,
            Self::MeshOutline => controller.start_mesh_outline()?,
            Self::Group => controller.group_selection()?,
            Self::Offset {
                command,
                both,
                through,
            } => {
                controller.run_addon_command(command)?;
                for (apply, option) in [(both, "BothSides=Yes"), (through, "ThroughPoint=Yes")] {
                    if apply && let Err(error) = controller.submit(option) {
                        controller.cancel_addon_session();
                        return Err(error);
                    }
                }
            }
            Self::Bezel(option) => {
                controller.dispatch_head_tool(super::head_commands::BEZEL_BUILDER)?;
                if let Err(error) = controller.submit(option) {
                    controller.submit_head_text("Cancel");
                    return Err(error);
                }
            }
            Self::ProngDrop(drop) => {
                controller.dispatch_settings_tool(super::settings_commands::PRONG_BUILDER)?;
                if let Err(error) = controller
                    .submit_settings_text(&format!("Drop={drop}"))
                    .unwrap_or(Err(Error::Invalid("prong drop")))
                {
                    controller.submit_settings_text("Cancel");
                    return Err(error);
                }
            }
            Self::Ungroup => controller.ungroup_selection()?,
            Self::Center => controller.center_selected()?,
            Self::Hide => {
                if controller.selected().is_empty() {
                    return Err(Error::Missing("objects to hide"));
                }
                let stamps =
                    controller.collect_object_sources(controller.selected().iter().copied())?;
                controller.set_objects_hidden(stamps, true)?;
                controller.clear_selection();
            }
            Self::Show => {
                let hidden = controller
                    .document()
                    .entities
                    .values()
                    .filter(|e| e.attributes.hidden)
                    .map(|e| e.id);
                let stamps = controller.collect_object_sources(hidden)?;
                if !stamps.is_empty() {
                    controller.set_objects_hidden(stamps, false)?;
                }
            }
            Self::PointsOff => controller.clear_top11(),
            Self::SetGripAxis(axis) => {
                let moves = controller
                    .selected_grips()
                    .into_iter()
                    .map(|grip| {
                        let mut target = grip.point;
                        target[axis.index()] = 0.;
                        (grip, target)
                    })
                    .collect::<Vec<_>>();
                return controller
                    .top11_move_grips(&moves)
                    .map(DispatchOutcome::Top11);
            }
            Self::Lock => {
                if controller.selected().is_empty() {
                    return Err(Error::Missing("objects to lock"));
                }
                let stamps =
                    controller.collect_object_sources(controller.selected().iter().copied())?;
                controller.edit_object_property(stamps, super::PropertyEdit::Locked(true))?;
                controller.clear_selection();
            }
            Self::Unlock => {
                let doc = controller.document();
                let locked = doc
                    .entities
                    .values()
                    .filter(|e| {
                        e.attributes.locked
                            && doc
                                .permissions_for_entity(e, false)
                                .is_ok_and(|p| p.visible)
                            && doc
                                .layer_permissions(e.attributes.layer, false)
                                .is_ok_and(|p| p.editable)
                    })
                    .map(|e| e.id);
                let stamps = controller.collect_object_sources(locked)?;
                controller.edit_object_property(stamps, super::PropertyEdit::Locked(false))?;
            }
            Self::Builder(kind) => controller.begin_builder(kind)?,
            Self::GemRow => {
                controller
                    .dispatch_shared_gem(script)
                    .ok_or(Error::Invalid("gem row preset"))??;
                return Ok(DispatchOutcome::Command);
            }
            Self::Cutter(option) => {
                controller.dispatch(super::gem_cutters::GEM_CUTTER, origin)?;
                if let Err(error) = controller
                    .gem_cutter_text(option)
                    .unwrap_or(Err(Error::Invalid("cutter preset")))
                {
                    controller.gem_cutter_cancel();
                    return Err(error);
                }
            }
            Self::Split("Point") => {
                return controller.top11_submit("Split Point")
                    .ok_or(Error::Missing("Split Point command"))?
                    .map(DispatchOutcome::Top11);
            }
            Self::Split(option) => {
                if controller.selected().len() != 1 {
                    return Err(Error::Invalid("split preset requires one target"));
                }
                controller.dispatch(Top11Action::Split.feature_id(), origin)?;
                match controller.top11_submit(option) {
                    Some(Ok(result)) => return Ok(DispatchOutcome::Top11(result)),
                    error => {
                        controller.cancel();
                        return Err(error
                            .and_then(Result::err)
                            .unwrap_or(Error::Invalid("split preset selection")));
                    }
                }
            }
        }
        Ok(DispatchOutcome::Command)
    }
}

pub(super) fn toggle(current: bool, value: Option<&str>) -> Result<bool> {
    match value.map(str::to_ascii_lowercase).as_deref() {
        None => Ok(!current),
        Some("yes" | "on" | "true") => Ok(true),
        Some("no" | "off" | "false") => Ok(false),
        _ => Err(Error::Invalid("expected Yes or No")),
    }
}
