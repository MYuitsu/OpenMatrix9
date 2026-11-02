// SPDX-License-Identifier: LGPL-2.1-or-later
use super::Controller;
use crate::command_options::{
    Activation, CommandOption, CommandOptionEffect, CommandOptionId, CommandOptionInterface,
    CommandOptionKind as Kind, assign_shortcuts,
};
use crate::polyline_options::{PolylineOptionInterface, PolylineOptionKind};
use om9_core::{Error, Result};

// Every existing token provider is registered here, once. A new owner only
// declares its available options and values; rendering and activation stay shared.
type TokenProvider = fn(&Controller) -> Vec<(&'static str, &'static str)>;
pub(super) const PROVIDERS: &[(&str, TokenProvider)] = &[
    ("annotation", Controller::annotation_option_tokens),
    ("measure", Controller::measure_option_tokens),
    ("transform", Controller::transform_option_tokens),
    ("surface_distance", Controller::surface_option_tokens),
    ("curve_pair", Controller::curve_pair_option_tokens),
    ("tween_surface", Controller::tween_surface_option_tokens),
    ("seam", Controller::seam_option_tokens),
    ("extrude_all", Controller::extrude_all_option_tokens),
    ("extend_surface", Controller::extend_option_tokens),
    ("split_surface", Controller::split_option_tokens),
    ("surface_pair", Controller::surface_pair_option_tokens),
    ("surface_tangent", Controller::tangent_option_tokens),
    ("surface_chamfer", Controller::chamfer_option_tokens),
    ("surface_fillet", Controller::fillet_option_tokens),
    (
        "surface_variable_chamfer",
        Controller::variable_chamfer_option_tokens,
    ),
    (
        "surface_variable_blend",
        Controller::variable_blend_option_tokens,
    ),
    (
        "surface_variable_offset",
        Controller::variable_offset_option_tokens,
    ),
    ("surface_shrink", Controller::shrink_option_tokens),
    ("hist_options", Controller::history_options_tokens),
    ("top11", Controller::top11_option_tokens),
    ("settings", Controller::settings_option_tokens),
    ("slash", Controller::slash_option_tokens),
    ("surface_soft_edit", Controller::soft_edit_option_tokens),
    ("solid", Controller::solid_option_tokens),
    ("circle", Controller::circle_option_tokens),
    ("rect", Controller::rect_option_tokens),
    ("interp", Controller::interp_option_tokens),
    ("ellipse", Controller::ellipse_option_tokens),
    ("arc", Controller::arc_option_tokens),
    ("rebuild", Controller::rebuild_option_tokens),
    ("fit", Controller::fit_option_tokens),
    ("refit_srf", Controller::refit_srf_option_tokens),
    ("offset_curve", Controller::offset_curve_option_tokens),
    ("project", Controller::project_option_tokens),
    ("blend", Controller::blend_option_tokens),
    ("isocurve", Controller::isocurve_option_tokens),
    ("pull", Controller::pull_option_tokens),
    ("intersect", Controller::intersect_option_tokens),
    ("crv2view", Controller::crv2view_option_tokens),
    ("through_points", Controller::through_points_option_tokens),
    ("polygon", Controller::polygon_option_tokens),
    ("helix", Controller::helix_option_tokens),
    ("spiral", Controller::spiral_option_tokens),
    ("fillet_corners", Controller::fillet_corners_option_tokens),
    ("divide", Controller::divide_option_tokens),
    ("to_lines", Controller::to_lines_option_tokens),
    ("crv_seam", Controller::crv_seam_option_tokens),
    ("control_curve", Controller::control_curve_option_tokens),
    ("continue_curve", Controller::continue_curve_option_tokens),
    (
        "offset_on_surface",
        Controller::offset_on_surface_option_tokens,
    ),
    ("offset_normal", Controller::offset_normal_option_tokens),
    ("corner", Controller::corner_option_tokens),
    ("cross_section", Controller::cross_section_option_tokens),
    ("curve_soft_edit", Controller::curve_soft_edit_option_tokens),
    ("angle_isocurve", Controller::angle_isocurve_option_tokens),
    ("extrude_surface", Controller::extrude_surface_option_tokens),
    ("heightfield", Controller::heightfield_option_tokens),
    ("extend_curve", Controller::extend_curve_option_tokens),
    ("tween_curves", Controller::tween_curves_option_tokens),
    ("continue_interp", Controller::continue_interp_option_tokens),
    ("arc_blend", Controller::arc_blend_option_tokens),
];

impl CommandOptionInterface for Controller {
    fn command_options(&self) -> Vec<CommandOption> {
        if self.polyline_pending.is_some() {
            return self
                .polyline_command_options()
                .into_iter()
                .map(|option| CommandOption {
                    id: CommandOptionId {
                        owner: "polyline",
                        name: option.id.name(),
                        run: self.command_run,
                    },
                    kind: match option.id.kind() {
                        PolylineOptionKind::Toggle => Kind::Toggle,
                        PolylineOptionKind::Choice => Kind::Choice,
                        PolylineOptionKind::Number => Kind::Number,
                        PolylineOptionKind::Point => Kind::Point,
                        PolylineOptionKind::Action => Kind::Action,
                    },
                    label: option.label(),
                    value: match option.value {
                        crate::polyline_options::PolylineOptionValue::Flag(value) => {
                            crate::command_options::CommandValue::Flag(value)
                        }
                        crate::polyline_options::PolylineOptionValue::Choice(value) => {
                            crate::command_options::CommandValue::Choice(value)
                        }
                        crate::polyline_options::PolylineOptionValue::Number(value) => {
                            crate::command_options::CommandValue::Number(value)
                        }
                        crate::polyline_options::PolylineOptionValue::Point(value) => {
                            crate::command_options::CommandValue::Point(value)
                        }
                        crate::polyline_options::PolylineOptionValue::None => {
                            crate::command_options::CommandValue::None
                        }
                    },
                    shortcut: Some(option.id.shortcut()),
                    activation: Activation::Polyline(option.id),
                })
                .collect();
        }
        if let Some(session) = &self.session
            && session.feature == super::LINE
        {
            let mut options = vec![
                descriptor(
                    "line",
                    "BothSides",
                    if session.both_sides {
                        "BothSides=No"
                    } else {
                        "BothSides=Yes"
                    },
                ),
                descriptor("line", "Length=", "Length="),
            ];
            options[0].kind = Kind::Toggle;
            options[0].label = format!("BothSides={}", yes_no(session.both_sides));
            options[0].value = crate::command_options::CommandValue::Flag(session.both_sides);
            options[1].value = crate::command_options::CommandValue::Number(session.length_mm);
            options[1].label = numeric_label(
                "Length",
                session.length_mm.map(|value| value / session.unit.factor()),
            );
            assign_shortcuts(&mut options);
            for option in &mut options {
                option.id.run = self.command_run;
            }
            return options;
        }
        for &(owner, provider) in PROVIDERS {
            let tokens = provider(self);
            if tokens.is_empty() {
                continue;
            }
            let mut options: Vec<_> = tokens
                .into_iter()
                .map(|(label, command)| {
                    let mut option = descriptor(owner, label, command);
                    self.describe_command_option(owner, &mut option);
                    option.value = self.typed_value(owner, option.id.name);
                    option
                })
                .collect();
            assign_shortcuts(&mut options);
            for option in &mut options {
                option.id.run = self.command_run;
            }
            return options;
        }
        Vec::new()
    }

    fn activate_command_option(&mut self, id: CommandOptionId) -> Result<CommandOptionEffect> {
        let option = self
            .command_options()
            .into_iter()
            .find(|option| option.id == id)
            .ok_or(Error::Invalid(
                "command option unavailable in this input stage",
            ))?;
        match option.activation {
            Activation::Draft(prefix) => Ok(CommandOptionEffect::Draft(prefix.into())),
            Activation::Submit(command) => {
                self.submit(command)?;
                Ok(CommandOptionEffect::Applied)
            }
            Activation::Polyline(id) => {
                self.activate_polyline_option(id)?;
                Ok(CommandOptionEffect::Applied)
            }
        }
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}
fn numeric_label(name: &str, value: Option<f64>) -> String {
    value.map_or_else(|| format!("{name}="), |value| format!("{name}={value}"))
}
fn descriptor(owner: &'static str, label: &'static str, command: &'static str) -> CommandOption {
    let entry = command.ends_with('=');
    let name = if entry {
        label.trim_end_matches('=')
    } else {
        label
    };
    CommandOption {
        id: CommandOptionId {
            owner,
            name,
            run: 0,
        },
        kind: if entry {
            match name {
                "File" => Kind::Text,
                "Axis" | "Point" | "Direction" => Kind::Point,
                _ => Kind::Number,
            }
        } else if command.contains('=') {
            Kind::Choice
        } else {
            Kind::Action
        },
        label: label.into(),
        value: crate::command_options::CommandValue::None,
        shortcut: None,
        activation: if entry {
            Activation::Draft(command)
        } else {
            Activation::Submit(command)
        },
    }
}

impl Controller {
    /// Normalize only declared assignment names/fixed choices. Values and owner
    /// validation are preserved; arbitrary geometry input is never rewritten.
    pub(super) fn canonical_option_assignment(&self, text: &str) -> Option<String> {
        let text = text.trim();
        let (name, value) = text.split_once('=')?;
        for option in self.command_options() {
            match option.activation {
                Activation::Draft(prefix)
                    if prefix.trim_end_matches('=').eq_ignore_ascii_case(name) =>
                {
                    return Some(format!("{prefix}{value}"));
                }
                Activation::Submit(command) if command.eq_ignore_ascii_case(text) => {
                    return Some(command.into());
                }
                _ => {}
            }
        }
        None
    }

    /// Typed current values for owners that already expose their state directly.
    /// Other providers keep `None` until their bridge is replaced.
    fn typed_value(&self, owner: &str, name: &str) -> crate::command_options::CommandValue {
        use crate::command_options::CommandValue;
        match owner {
            "solid" if name == "MarkFoci" => self.solid_pending.as_ref()
                .filter(|p| p.feature == "OM9-SOLID-013")
                .map_or(CommandValue::None, |p| CommandValue::Flag(p.ellipsoid_mark_foci)),
            "solid" if name == "Diameter" => self.solid_pending.as_ref()
                .map_or(CommandValue::None, |p| match p.feature {
                    "OM9-SOLID-011" => CommandValue::Flag(p.pipe_diameter),
                    "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017" => CommandValue::Flag(p.cone_diameter),
                    _ => CommandValue::None,
                }),
            "solid" if name == "Area" => self.solid_pending.as_ref()
                .filter(|p|matches!(p.feature,"OM9-SOLID-020" | "OM9-SOLID-017" | "OM9-SOLID-019"))
                .map_or(CommandValue::None,|p|CommandValue::Flag(p.cone_area)),
            "solid" if name == "Solid" => self.solid_pending.as_ref()
                .filter(|p|matches!(p.feature,"OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-018"))
                .map_or(CommandValue::None,|p|CommandValue::Flag(p.cone_flags.solid())),
            "solid" if name == "BothSides" => self.solid_pending.as_ref()
                .filter(|p|p.feature=="OM9-SOLID-017")
                .map_or(CommandValue::None,|p|CommandValue::Flag(p.cone_flags.both_sides())),
            "solid" if name == "Thick" => self.solid_pending.as_ref()
                .filter(|p| p.feature == "OM9-SOLID-011")
                .map_or(CommandValue::None, |p| CommandValue::Flag(p.pipe_thick)),
            "solid" if name == "Cap" => self.solid_pending.as_ref()
                .filter(|p| p.feature == "OM9-SOLID-011")
                .map_or(CommandValue::None, |p| CommandValue::Choice(if p.pipe_round { "Round" } else if p.pipe_capped { "Flat" } else { "None" })),
            "solid" if name == "ShapeBlending=Global" => self.solid_pending.as_ref()
                .filter(|p| p.feature == "OM9-SOLID-011")
                .map_or(CommandValue::None, |p| CommandValue::Flag(p.pipe_global)),
            "solid" if name == "DeleteInput" => self.solid_pending.as_ref()
                .filter(|p| matches!(p.feature, "OM9-SOLID-001" | "OM9-SOLID-002" | "OM9-SOLID-003" | "OM9-SOLID-004" | "SolidBooleanSplit" | "SolidCreateSolid"))
                .map_or(CommandValue::None, |p| CommandValue::Flag(p.difference_delete_input)),
            "solid" if name == "Copy" => self.solid_pending.as_ref()
                .filter(|p| p.feature == "OM9-SOLID-006")
                .map_or(CommandValue::None, |p| CommandValue::Flag(p.extract_copy)),
            "solid" if name == "NextRadius" => self.solid_pending.as_ref()
                .filter(|p| p.feature == "OM9-SOLID-007")
                .map_or(CommandValue::None, |p| CommandValue::Number(p.fillet_radius)),
            "solid" if name == "CurrentRadius" => CommandValue::Number(self.solid_fillet_current_radius()),
            "solid" if name == "Thickness" => self.solid_pending.as_ref()
                .filter(|p| p.feature == "OM9-SOLID-029")
                .map_or(CommandValue::None, |p| CommandValue::Number(p.shell_thickness)),
            "solid" if name == "ShowRadius" => self.solid_pending.as_ref()
                .filter(|p| p.feature == "OM9-SOLID-007")
                .map_or(CommandValue::None, |p| CommandValue::Flag(p.fillet_show_radius)),
            "solid" if name == "LinkHandles" => self.solid_pending.as_ref()
                .filter(|p| p.feature == "OM9-SOLID-007")
                .map_or(CommandValue::None, |p| CommandValue::Flag(p.fillet_link_handles)),
            "annotation" => {
                let Some(p) = &self.annotation_pending else {
                    return CommandValue::None;
                };
                match name {
                    "Height" => CommandValue::Number(Some(p.record.style.height_mm)),
                    "Arrow" => CommandValue::Number(Some(p.record.style.arrow_mm)),
                    "Precision" => CommandValue::Integer(Some(p.record.style.precision)),
                    "Scale" => CommandValue::Number(Some(p.record.style.scale)),
                    "Angle" => CommandValue::Number(Some(p.record.rotation_degrees)),
                    "Text" => CommandValue::Text(p.record.text.clone()),
                    "Position" => CommandValue::Point(Some(p.record.text_point)),
                    _ => CommandValue::None,
                }
            }
            "measure" => {
                let Some(p) = &self.measure_pending else {
                    return CommandValue::None;
                };
                match name {
                    "Continuous" => CommandValue::Flag(p.options.continuous),
                    "Ortho" => CommandValue::Flag(p.options.ortho),
                    "U" | "V" => CommandValue::Number(
                        p.options
                            .iso
                            .filter(|(u, _)| *u == (name == "U"))
                            .map(|(_, v)| v),
                    ),
                    _ => CommandValue::None,
                }
            }
            "transform" => {
                let Some(p) = &self.transform_pending else {
                    return CommandValue::None;
                };
                match name {
                    "Spacing" => CommandValue::Number(p.options.spacing),
                    "Count" => CommandValue::Integer(p.numbers.first().map(|n| *n as usize)),
                    "Basepoint" => CommandValue::Point(p.options.basepoint),
                    "Preview" => CommandValue::Flag(p.options.preview),
                    "SetX" => CommandValue::Flag(p.options.axes[0]),
                    "SetY" => CommandValue::Flag(p.options.axes[1]),
                    "SetZ" => CommandValue::Flag(p.options.axes[2]),
                    "World" | "CPlane" => CommandValue::Choice(if p.options.align_cplane {
                        "CPlane"
                    } else {
                        "World"
                    }),
                    "NoRotation" | "Freeform" | "Roadlike" => {
                        CommandValue::Choice(p.options.orientation.name())
                    }
                    "Indices" => CommandValue::Text(Some(
                        p.options
                            .indices
                            .iter()
                            .map(usize::to_string)
                            .collect::<Vec<_>>()
                            .join(","),
                    )),
                    _ => CommandValue::None,
                }
            }
            "helix" => {
                let Some(pending) = &self.helix_pending else {
                    return CommandValue::None;
                };
                match name {
                    "Radius" => CommandValue::Number(pending.radius),
                    "Height" => CommandValue::Number(Some(pending.height)),
                    "Turns" => CommandValue::Number(Some(pending.turns)),
                    "Axis" => CommandValue::Point(pending.axis),
                    _ => CommandValue::None,
                }
            }
            "spiral" => {
                let Some(pending) = &self.spiral_pending else {
                    return CommandValue::None;
                };
                match name {
                    "StartRadius" => CommandValue::Number(pending.start_radius),
                    "EndRadius" => CommandValue::Number(pending.end_radius),
                    "Turns" => CommandValue::Number(Some(pending.turns)),
                    "Height" => CommandValue::Number(Some(pending.height)),
                    "Axis" => CommandValue::Point(pending.axis),
                    _ => CommandValue::None,
                }
            }
            "angle_isocurve" => {
                let Some(pending) = &self.angle_isocurve_pending else {
                    return CommandValue::None;
                };
                match name {
                    "Angle" => CommandValue::Number(Some(pending.angle)),
                    "Direction" => CommandValue::Choice(if pending.axis == 0 { "U" } else { "V" }),
                    _ => CommandValue::None,
                }
            }
            "offset_curve" => {
                let Some(pending) = &self.offset_curve_pending else {
                    return CommandValue::None;
                };
                match name {
                    "Distance" => CommandValue::Number(Some(pending.distance)),
                    _ => CommandValue::None,
                }
            }
            "fit" => {
                let Some(pending) = &self.fit_pending else {
                    return CommandValue::None;
                };
                match name {
                    "Tolerance" => CommandValue::Number(Some(pending.tolerance)),
                    _ => CommandValue::None,
                }
            }
            "refit_srf" => {
                let Some(pending) = &self.refit_srf_pending else {
                    return CommandValue::None;
                };
                match name {
                    "Tolerance" => CommandValue::Number(Some(pending.tolerance)),
                    _ => CommandValue::None,
                }
            }
            "cross_section" => {
                let Some(pending) = &self.cross_section_pending else {
                    return CommandValue::None;
                };
                match name {
                    "Count" => CommandValue::Integer(Some(pending.count)),
                    "Spacing" => CommandValue::Number(Some(pending.spacing)),
                    _ => CommandValue::None,
                }
            }
            "extrude_surface" => {
                let Some(pending) = &self.extrude_surface_pending else {
                    return CommandValue::None;
                };
                match name {
                    "Height" => CommandValue::Number(Some(pending.height)),
                    _ => CommandValue::None,
                }
            }
            "divide" => {
                let Some(pending) = &self.divide_pending else {
                    return CommandValue::None;
                };
                match name {
                    "Count" => CommandValue::Integer(Some(pending.count)),
                    _ => CommandValue::None,
                }
            }
            "rebuild" => match &self.rebuild_pending {
                Some(pending) if name == "Degree" => CommandValue::Integer(Some(pending.degree)),
                Some(pending) if name == "Points" => CommandValue::Integer(Some(pending.points)),
                _ => CommandValue::None,
            },
            "project" => match &self.project_pending {
                Some(pending) if name == "Direction" => CommandValue::Point(pending.direction),
                _ => CommandValue::None,
            },
            "blend" => match &self.blend_pending {
                Some(pending) if name == "Scale1" => CommandValue::Number(Some(pending.scale1)),
                Some(pending) if name == "Scale2" => CommandValue::Number(Some(pending.scale2)),
                _ => CommandValue::None,
            },
            "pull" => match &self.pull_pending {
                Some(pending) if name == "Tolerance" => {
                    CommandValue::Number(Some(pending.tolerance))
                }
                _ => CommandValue::None,
            },
            "intersect" => match &self.intersect_pending {
                Some(pending) if name == "Tolerance" => {
                    CommandValue::Number(Some(pending.tolerance))
                }
                _ => CommandValue::None,
            },
            "polygon" => match &self.polygon_pending {
                Some(pending) if name == "Sides" => CommandValue::Integer(Some(pending.sides)),
                Some(pending) if name == "Radius" => CommandValue::Number(pending.radius),
                Some(pending) if name == "Circumscribed" => {
                    CommandValue::Flag(pending.circumscribed)
                }
                _ => CommandValue::None,
            },
            "through_points" => match &self.through_points_pending {
                Some(pending) if name == "Degree" => CommandValue::Integer(Some(pending.degree)),
                Some(pending) if name == "Closed" => CommandValue::Flag(pending.closed),
                _ => CommandValue::None,
            },
            "crv_seam" => match &self.crv_seam_pending {
                Some(pending) if name == "Seam" => CommandValue::Number(Some(pending.fraction)),
                _ => CommandValue::None,
            },
            "extend_curve" => match &self.extend_curve_pending {
                Some(pending) if name == "Length" => CommandValue::Number(Some(pending.length)),
                Some(pending) if name == "End" => CommandValue::Flag(!pending.at_start),
                Some(pending) if name == "Start" => CommandValue::Flag(pending.at_start),
                _ => CommandValue::None,
            },
            "tween_curves" => match &self.tween_curves_pending {
                Some(pending) if name == "Count" => CommandValue::Integer(Some(pending.count)),
                _ => CommandValue::None,
            },
            "fillet_corners" => match &self.fillet_corners_pending {
                Some(pending) if name == "Radius" => CommandValue::Number(Some(pending.radius)),
                _ => CommandValue::None,
            },
            "to_lines" => match &self.to_lines_pending {
                Some(pending) if name == "Count" => CommandValue::Integer(Some(pending.count)),
                _ => CommandValue::None,
            },
            "offset_on_surface" => match &self.offset_on_surface_pending {
                Some(pending) if name == "Distance" => CommandValue::Number(Some(pending.distance)),
                _ => CommandValue::None,
            },
            "offset_normal" => match &self.offset_normal_pending {
                Some(pending) if name == "Distance" => CommandValue::Number(Some(pending.distance)),
                _ => CommandValue::None,
            },
            "arc" => match &self.arc_pending {
                Some(pending) if name == "Radius" => CommandValue::Number(pending.radius),
                Some(pending) if name == "3Point" => {
                    CommandValue::Flag(pending.mode == "ThreePoint")
                }
                Some(pending) if name == "Center" => CommandValue::Flag(pending.mode == "Center"),
                Some(pending) if name == "StartPoint" => {
                    CommandValue::Flag(pending.mode == "StartPoint")
                }
                _ => CommandValue::None,
            },
            "rect" => match &self.rect_pending {
                Some(pending) if name == "Length" => CommandValue::Number(pending.width),
                Some(pending) if name == "Width" => CommandValue::Number(pending.width),
                Some(pending) if name == "Height" => CommandValue::Number(pending.height),
                Some(pending) if name == "Square" => CommandValue::Flag(pending.square),
                Some(pending) if name == "Center" => CommandValue::Flag(pending.mode == "Center"),
                Some(pending) if name == "3Point" => {
                    CommandValue::Flag(pending.mode == "ThreePoint")
                }
                Some(pending) if name == "Vertical" => {
                    CommandValue::Flag(pending.mode == "Vertical")
                }
                _ => CommandValue::None,
            },
            "circle" => match &self.circle_pending {
                Some(pending) if name == "Radius" => CommandValue::Number(pending.radius),
                Some(pending) if name == "Degree" => CommandValue::Integer(Some(pending.degree)),
                Some(pending) if name == "PointCount" => {
                    CommandValue::Integer(Some(pending.point_count))
                }
                Some(pending) if name == "Deformable" => CommandValue::Flag(pending.deformable),
                Some(pending) if name.starts_with("Size=") => {
                    CommandValue::Flag(pending.size == &name[5..])
                }
                Some(pending) if name == "Solution=1" => CommandValue::Flag(pending.solution == 1),
                Some(pending) if name == "Solution=2" => CommandValue::Flag(pending.solution == 2),
                _ => CommandValue::None,
            },
            "ellipse" => match &self.ellipse_pending {
                Some(pending) if name == "RadiusX" => CommandValue::Number(pending.radius_x),
                Some(pending) if name == "RadiusY" => CommandValue::Number(pending.radius_y),
                Some(pending) if name == "Degree" => CommandValue::Integer(Some(pending.degree)),
                Some(pending) if name == "PointCount" => {
                    CommandValue::Integer(Some(pending.point_count))
                }
                Some(pending) if name == "Deformable" => CommandValue::Flag(pending.deformable),
                Some(pending) if name == "MarkFoci" => CommandValue::Flag(pending.mark_foci),
                Some(pending) if name == "Center" => CommandValue::Flag(pending.mode == "Center"),
                Some(pending) if name == "Corner" => CommandValue::Flag(pending.mode == "Corner"),
                Some(pending) if name == "Vertical" => {
                    CommandValue::Flag(pending.mode == "Vertical")
                }
                Some(pending) if name == "Diameter" => {
                    CommandValue::Flag(pending.mode == "Diameter")
                }
                Some(pending) if name == "FromFoci" => {
                    CommandValue::Flag(pending.mode == "FromFoci")
                }
                Some(pending) if name == "AroundCurve" => {
                    CommandValue::Flag(pending.mode == "AroundCurve")
                }
                _ => CommandValue::None,
            },
            "interp" => match &self.interp_pending {
                Some(pending) if name == "Degree" => CommandValue::Integer(Some(pending.degree)),
                Some(pending) if name == "Close" => CommandValue::Flag(pending.close),
                _ => CommandValue::None,
            },
            "control_curve" => match &self.control_curve_pending {
                Some(pending) if name == "Degree" => CommandValue::Integer(Some(pending.degree)),
                _ => CommandValue::None,
            },
            "continue_curve" => match &self.continue_curve_pending {
                Some(pending) if name == "Degree" => CommandValue::Integer(Some(pending.degree)),
                _ => CommandValue::None,
            },
            "continue_interp" => match &self.continue_interp_pending {
                Some(pending) if name == "Degree" => CommandValue::Integer(Some(pending.degree)),
                _ => CommandValue::None,
            },
            "heightfield" => match &self.heightfield_pending {
                Some(pending) if name == "Height" => CommandValue::Number(Some(pending.height)),
                Some(pending) if name == "Size" => CommandValue::Number(Some(pending.size)),
                Some(pending) if name == "File" => CommandValue::Text(pending.file.clone()),
                _ => CommandValue::None,
            },
            "corner" => match &self.corner_pending {
                Some(pending) if name == "Radius" => CommandValue::Number(Some(pending.radius)),
                Some(pending) if name == "Distance" => CommandValue::Number(Some(pending.distance)),
                _ => CommandValue::None,
            },
            "hist_options" => match &self.history_options_pending {
                Some(pending) if name == "Closed" => CommandValue::Flag(pending.closed),
                Some(pending) if name == "Flip" => CommandValue::Flag(pending.flip),
                _ => CommandValue::None,
            },
            "isocurve" => match &self.isocurve_pending {
                Some(pending) if name == "U" => {
                    if pending.iso == om9_core::surface_kernel::SurfaceIso::U {
                        CommandValue::Number(Some(pending.value))
                    } else {
                        CommandValue::None
                    }
                }
                Some(pending) if name == "V" => {
                    if pending.iso == om9_core::surface_kernel::SurfaceIso::V {
                        CommandValue::Number(Some(pending.value))
                    } else {
                        CommandValue::None
                    }
                }
                _ => CommandValue::None,
            },
            "surface_tangent" => match &self.tangent_pending {
                Some(pending) if name == "Direction" => CommandValue::Point(pending.direction),
                _ => CommandValue::None,
            },
            "surface_pair" => match &self.surface_pair_pending {
                Some(pending) if name == "Source" => {
                    CommandValue::Choice(match pending.source_edge {
                        Some(om9_core::surface_ops::SurfaceEdge::U0) => "U0",
                        Some(om9_core::surface_ops::SurfaceEdge::U1) => "U1",
                        Some(om9_core::surface_ops::SurfaceEdge::V0) => "V0",
                        Some(om9_core::surface_ops::SurfaceEdge::V1) => "V1",
                        None => return CommandValue::None,
                    })
                }
                Some(pending) if name == "Target" => {
                    CommandValue::Choice(match pending.target_edge {
                        Some(om9_core::surface_ops::SurfaceEdge::U0) => "U0",
                        Some(om9_core::surface_ops::SurfaceEdge::U1) => "U1",
                        Some(om9_core::surface_ops::SurfaceEdge::V0) => "V0",
                        Some(om9_core::surface_ops::SurfaceEdge::V1) => "V1",
                        None => return CommandValue::None,
                    })
                }
                Some(pending) if name == "Continuity=" => {
                    CommandValue::Choice(super::surface_edit::continuity_label(pending.continuity))
                }
                _ => CommandValue::None,
            },
            "slash" => match &self.slash_pending {
                Some(pending) if name == "First" => {
                    CommandValue::Point(pending.points.first().copied())
                }
                Some(pending) if name == "Second" => {
                    CommandValue::Point(pending.points.get(1).copied())
                }
                _ => CommandValue::None,
            },
            "top11" => match &self.top11_pending {
                Some(super::top11::Top11Pending::Move { vertical, .. }) if name == "Vertical" => {
                    CommandValue::Flag(*vertical)
                }
                Some(super::top11::Top11Pending::Duplicate {
                    vertical,
                    from_last,
                    use_last_distance,
                    use_last_direction,
                    ..
                }) => match name {
                    "Vertical" => CommandValue::Flag(*vertical),
                    "FromLastPoint" => CommandValue::Flag(*from_last),
                    "UseLastDistance" => CommandValue::Flag(*use_last_distance),
                    "UseLastDirection" => CommandValue::Flag(*use_last_direction),
                    _ => CommandValue::None,
                },
                Some(super::top11::Top11Pending::Mirror { copy, .. }) if name == "Copy" => {
                    CommandValue::Flag(*copy)
                }
                Some(super::top11::Top11Pending::Rotate { copy, .. }) if name == "Copy" => {
                    CommandValue::Flag(*copy)
                }
                _ => CommandValue::None,
            },
            "seam" => match &self.seam_pending {
                Some(pending) if name == "Axis" => CommandValue::Choice(match pending.axis {
                    Some(om9_core::surface_ops::SurfaceIso::U) => "U",
                    Some(om9_core::surface_ops::SurfaceIso::V) => "V",
                    None => return CommandValue::None,
                }),
                Some(pending) if name == "Tolerance" => CommandValue::Number(pending.tolerance_mm),
                _ => CommandValue::None,
            },
            "split_surface" => match &self.split_pending {
                Some(pending) if name == "Parameter" => CommandValue::Number(pending.parameter),
                _ => CommandValue::None,
            },
            "surface_fillet" => match &self.fillet_pending {
                Some(pending) if name == "Radius" => CommandValue::Number(pending.start_radius),
                Some(pending) if name == "Start" => CommandValue::Number(pending.start_radius),
                Some(pending) if name == "End" => CommandValue::Number(pending.end_radius),
                _ => CommandValue::None,
            },
            "surface_variable_chamfer" => match &self.variable_chamfer_pending {
                Some(pending) if name == "Start" => CommandValue::Number(pending.start_setback),
                Some(pending) if name == "End" => CommandValue::Number(pending.end_setback),
                _ => CommandValue::None,
            },
            "surface_variable_blend" => match &self.variable_blend_pending {
                Some(pending) if name == "Start" => CommandValue::Number(pending.start_scale),
                Some(pending) if name == "End" => CommandValue::Number(pending.end_scale),
                _ => CommandValue::None,
            },
            "surface_variable_offset" => match &self.variable_offset_pending {
                Some(pending) if name == "Start" => CommandValue::Number(pending.start),
                Some(pending) if name == "End" => CommandValue::Number(pending.end),
                _ => CommandValue::None,
            },
            "surface_shrink" => match &self.shrink_pending {
                Some(pending) if name == "U0" => CommandValue::Number(pending.u0),
                Some(pending) if name == "U1" => CommandValue::Number(pending.u1),
                Some(pending) if name == "V0" => CommandValue::Number(pending.v0),
                Some(pending) if name == "V1" => CommandValue::Number(pending.v1),
                _ => CommandValue::None,
            },
            "surface_soft_edit" => {
                let Some(pending) = &self.soft_edit_pending else {
                    return CommandValue::None;
                };
                match name {
                    "U" => CommandValue::Number(pending.base[0]),
                    "V" => CommandValue::Number(pending.base[1]),
                    "Dx" => CommandValue::Number(pending.delta[0]),
                    "Dy" => CommandValue::Number(pending.delta[1]),
                    "Dz" => CommandValue::Number(pending.delta[2]),
                    "UDistance" => CommandValue::Number(Some(pending.u_distance)),
                    "VDistance" => CommandValue::Number(Some(pending.v_distance)),
                    "Copy" => CommandValue::Flag(pending.copy),
                    "FixEdges" => CommandValue::Flag(pending.fix_edges),
                    _ => CommandValue::None,
                }
            }
            _ => CommandValue::None,
        }
    }
    fn describe_command_option(&self, owner: &str, option: &mut CommandOption) {
        let name = option.id.name;
        macro_rules! numbers {
            ($pending:ident, $($key:literal => $field:ident),+ $(,)?) => {
                if let Some(p) = &self.$pending {
                    match name { $($key => option.label = format!("{name}={}", p.$field),)+ _ => {} }
                }
            };
        }
        macro_rules! optional_numbers {
            ($pending:ident, $($key:literal => $field:ident),+ $(,)?) => {
                if let Some(p) = &self.$pending {
                    match name { $($key => option.label = numeric_label(name, p.$field),)+ _ => {} }
                }
            };
        }
        macro_rules! flag {
            ($value:expr) => {{
                option.kind = Kind::Toggle;
                option.label = format!("{name}={}", yes_no($value));
            }};
        }
        macro_rules! mode {
            ($value:expr) => {{
                let target = match name {
                    "2Point" => "TwoPoint",
                    "3Point" => "ThreePoint",
                    "4Point" => "FourPoint",
                    _ => name,
                };
                option.kind = Kind::Choice;
                option.label = format!("{name}={}", yes_no($value == target));
            }};
        }
        match owner {
            "annotation" => {
                if let Some(p) = &self.annotation_pending {
                    match name {
                        "Text" => option.kind = Kind::Text,
                        "Position" | "Point1" | "Point2" | "Point3" => option.kind = Kind::Point,
                        "Height" => {
                            option.label = numeric_label(name, Some(p.record.style.height_mm))
                        }
                        "Arrow" => {
                            option.label = numeric_label(name, Some(p.record.style.arrow_mm))
                        }
                        "Scale" => option.label = numeric_label(name, Some(p.record.style.scale)),
                        "Precision" => {
                            option.label = format!("Precision={}", p.record.style.precision)
                        }
                        "Angle" => {
                            option.label = numeric_label(name, Some(p.record.rotation_degrees))
                        }
                        _ => {}
                    }
                }
            }
            "measure" => {
                if let Some(p) = &self.measure_pending {
                    match name {
                        "Continuous" => flag!(p.options.continuous),
                        "Ortho" => flag!(p.options.ortho),
                        _ => {}
                    }
                }
            }
            "transform" => {
                if let Some(p) = &self.transform_pending {
                    match name {
                        "Preview" => flag!(p.options.preview),
                        "SetX" => flag!(p.options.axes[0]),
                        "SetY" => flag!(p.options.axes[1]),
                        "SetZ" => flag!(p.options.axes[2]),
                        "NoRotation" | "Freeform" | "Roadlike" => {
                            mode!(p.options.orientation.name())
                        }
                        "World" | "CPlane" => mode!(if p.options.align_cplane {
                            "CPlane"
                        } else {
                            "World"
                        }),
                        "Indices" => {
                            option.kind = Kind::Text;
                        }
                        "Spacing" => option.label = numeric_label(name, p.options.spacing),
                        _ => {}
                    }
                }
            }
            "angle_isocurve" => {
                numbers!(angle_isocurve_pending, "Angle" => angle);
                if name == "Direction" {
                    option.kind = Kind::Choice;
                    option.label = format!(
                        "Direction={}",
                        if self.angle_isocurve_pending.as_ref().unwrap().axis == 0 {
                            "U"
                        } else {
                            "V"
                        }
                    );
                }
            }
            "arc" => {
                optional_numbers!(arc_pending, "Radius" => radius);
                if let Some(p) = &self.arc_pending {
                    if name == "Diameter" {
                        option.label = numeric_label(name, p.radius.map(|r| r * 2.));
                    }
                    if matches!(name, "3Point" | "Center" | "StartPoint") {
                        mode!(p.mode);
                    }
                }
            }
            "blend" => {
                numbers!(blend_pending, "Scale1" => scale1, "Scale2" => scale2);
            }
            "circle" => {
                numbers!(circle_pending, "Degree" => degree, "PointCount" => point_count, "Solution" => solution);
                optional_numbers!(circle_pending, "Radius" => radius);
                if let Some(p) = &self.circle_pending {
                    match name {
                        "Deformable" => flag!(p.deformable),
                        "Approx" => flag!(p.deformable),
                        "Center" | "2Point" | "3Point" | "Vertical" | "FitPoints"
                        | "AroundCurve" | "Tangent" => mode!(p.mode),
                        "Diameter" => option.label = numeric_label(name, p.radius.map(|r| r * 2.)),
                        "Circumference" => {
                            option.label =
                                numeric_label(name, p.radius.map(|r| r * std::f64::consts::TAU))
                        }
                        "Area" => {
                            option.label =
                                numeric_label(name, p.radius.map(|r| r * r * std::f64::consts::PI))
                        }
                        "Point" => {
                            if let Some([x, y, z]) = p.tangent_point {
                                option.label = format!("Point={x},{y},{z}");
                            }
                        }
                        _ if name.starts_with("Size=") => {
                            let size = name.trim_start_matches("Size=");
                            option.label = format!("{name} ({})", yes_no(p.size == size));
                        }
                        _ => {}
                    }
                }
            }
            "continue_curve" => {
                numbers!(continue_curve_pending, "Degree" => degree);
            }
            "continue_interp" => {
                numbers!(continue_interp_pending, "Degree" => degree);
            }
            "control_curve" => {
                numbers!(control_curve_pending, "Degree" => degree);
            }
            "corner" => {
                numbers!(corner_pending, "Radius" => radius, "Distance" => distance);
            }
            "cross_section" => {
                numbers!(cross_section_pending, "Count" => count, "Spacing" => spacing);
            }
            "crv_seam" => {
                numbers!(crv_seam_pending, "Seam" => fraction);
            }
            "surface_soft_edit" => {
                let Some(pending) = &self.soft_edit_pending else {
                    return;
                };
                match name {
                    "U" => option.label = numeric_label("U", pending.base[0]),
                    "V" => option.label = numeric_label("V", pending.base[1]),
                    "Dx" => option.label = numeric_label("Dx", pending.delta[0]),
                    "Dy" => option.label = numeric_label("Dy", pending.delta[1]),
                    "Dz" => option.label = numeric_label("Dz", pending.delta[2]),
                    "UDistance" => option.label = format!("UDistance={}", pending.u_distance),
                    "VDistance" => option.label = format!("VDistance={}", pending.v_distance),
                    "Copy" => flag!(pending.copy),
                    "FixEdges" => flag!(pending.fix_edges),
                    "DirConstraint" => {
                        option.kind = Kind::Choice;
                        option.label = format!("DirConstraint={}", pending.constraint.label());
                    }
                    _ => {}
                }
            }
            "curve_soft_edit" => {
                numbers!(curve_soft_edit_pending, "Index" => index);
                if name == "Point"
                    && let Some(p) = &self.curve_soft_edit_pending
                    && p.has_target
                {
                    let [x, y, z] = p.target;
                    option.label = format!("Point={x},{y},{z}");
                }
            }
            "divide" => {
                numbers!(divide_pending, "Count" => count);
            }
            "ellipse" => {
                numbers!(ellipse_pending, "Degree" => degree, "PointCount" => point_count);
                optional_numbers!(ellipse_pending, "RadiusX" => radius_x, "RadiusY" => radius_y);
                if let Some(p) = &self.ellipse_pending {
                    match name {
                        "MarkFoci" => flag!(p.mark_foci),
                        "Deformable" => flag!(p.deformable),
                        "Approx" => flag!(p.deformable),
                        "Center" | "Corner" | "Diameter" | "Vertical" | "FromFoci"
                        | "AroundCurve" => mode!(p.mode),
                        _ => {}
                    }
                }
            }
            "extend_curve" => {
                numbers!(extend_curve_pending, "Length" => length);
            }
            "extrude_surface" => {
                numbers!(extrude_surface_pending, "Height" => height);
            }
            "fillet_corners" => {
                numbers!(fillet_corners_pending, "Radius" => radius);
            }
            "fit" => {
                numbers!(fit_pending, "Tolerance" => tolerance);
            }
            "refit_srf" => {
                numbers!(refit_srf_pending, "Tolerance" => tolerance);
            }
            "heightfield" => {
                numbers!(heightfield_pending, "Height" => height, "Size" => size);
                if name == "File"
                    && let Some(p) = &self.heightfield_pending
                {
                    option.label = format!("File={}", p.file.as_deref().unwrap_or(""));
                }
            }
            "helix" => {
                numbers!(helix_pending, "Height" => height, "Turns" => turns);
                optional_numbers!(helix_pending, "Radius" => radius);
                if name == "Axis"
                    && let Some([x, y, z]) = self.helix_pending.as_ref().and_then(|p| p.axis)
                {
                    option.label = format!("Axis={x},{y},{z}");
                }
            }
            "interp" => {
                numbers!(interp_pending, "Degree" => degree);
            }
            "intersect" => {
                numbers!(intersect_pending, "Tolerance" => tolerance);
            }
            "isocurve" => {
                if let Some(p) = &self.isocurve_pending {
                    let active = match p.iso {
                        om9_core::surface_kernel::SurfaceIso::U => "U",
                        om9_core::surface_kernel::SurfaceIso::V => "V",
                    };
                    option.label = if name == active {
                        format!("{name}={}", p.value)
                    } else {
                        option.label.clone()
                    };
                }
            }
            "offset_curve" => {
                numbers!(offset_curve_pending, "Distance" => distance);
            }
            "offset_normal" => {
                numbers!(offset_normal_pending, "Distance" => distance);
            }
            "offset_on_surface" => {
                numbers!(offset_on_surface_pending, "Distance" => distance);
            }
            "polygon" => {
                numbers!(polygon_pending, "Sides" => sides);
                if let Some(p) = &self.polygon_pending {
                    match name {
                        "Circumscribed" => flag!(p.circumscribed),
                        "Inscribed" => flag!(!p.circumscribed),
                        _ => {}
                    }
                }
            }
            "project" => {
                if name == "Direction"
                    && let Some([x, y, z]) = self.project_pending.as_ref().and_then(|p| p.direction)
                {
                    option.label = format!("Direction={x},{y},{z}");
                }
            }
            "pull" => {
                numbers!(pull_pending, "Tolerance" => tolerance);
            }
            "rebuild" => {
                numbers!(rebuild_pending, "Degree" => degree, "Points" => points);
            }
            "rect" => {
                if let Some(p) = &self.rect_pending {
                    match name {
                        "Square" => flag!(p.square),
                        "Vertical" | "3Point" | "Center" => mode!(p.mode),
                        "Length" => {
                            option.label = numeric_label(
                                name,
                                if matches!(p.mode, "ThreePoint" | "Vertical") {
                                    p.height
                                } else {
                                    p.width
                                },
                            )
                        }
                        _ => {}
                    }
                }
            }
            "solid" => {
                if let Some(p) = &self.solid_pending {
                    if matches!(option.activation, Activation::Submit(command) if command.starts_with("Mode="))
                    {
                        mode!(if p.box_mode {
                            p.box_kind
                        } else {
                            p.sphere_kind
                        });
                    }
                    match name {
                        "MarkFoci" if p.feature == "OM9-SOLID-013" => flag!(p.ellipsoid_mark_foci),
                        "Diameter" if p.feature == "OM9-SOLID-011" => flag!(p.pipe_diameter),
                        "Diameter" if matches!(p.feature, "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017") => flag!(p.cone_diameter),
                        "Area" if matches!(p.feature, "OM9-SOLID-020" | "OM9-SOLID-017" | "OM9-SOLID-019") => flag!(p.cone_area),
                        "Solid" if matches!(p.feature, "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-018") => flag!(p.cone_flags.solid()),
                        "BothSides" if p.feature == "OM9-SOLID-017" => flag!(p.cone_flags.both_sides()),
                        "Thick" if p.feature == "OM9-SOLID-011" => flag!(p.pipe_thick),
                        "ShapeBlending=Global" if p.feature == "OM9-SOLID-011" => {
                            option.label = format!("ShapeBlending=Global ({})", yes_no(p.pipe_global));
                        }
                        "Cap" if p.feature == "OM9-SOLID-011" => {
                            option.kind = Kind::Choice;
                            option.label = format!("Cap={}", if p.pipe_round { "Round" } else if p.pipe_capped { "Flat" } else { "None" });
                        }
                        "DeleteInput" if matches!(p.feature, "OM9-SOLID-001" | "OM9-SOLID-002" | "OM9-SOLID-003" | "OM9-SOLID-004" | "SolidBooleanSplit" | "SolidCreateSolid") => flag!(p.difference_delete_input),
                        "Copy" if p.feature == "OM9-SOLID-006" => flag!(p.extract_copy),
                        "NextRadius" if p.feature == "OM9-SOLID-007" => option.label = numeric_label(name, p.fillet_radius),
                        "CurrentRadius" if p.feature == "OM9-SOLID-007" => option.label = numeric_label(name, self.solid_fillet_current_radius()),
                        "Thickness" if p.feature == "OM9-SOLID-029" => option.label = numeric_label(name, p.shell_thickness),
                        "ShowRadius" if p.feature == "OM9-SOLID-007" => flag!(p.fillet_show_radius),
                        "LinkHandles" if p.feature == "OM9-SOLID-007" => flag!(p.fillet_link_handles),
                        "OutputLayer" if p.feature == "OM9-SOLID-006" => {
                            option.label = format!("OutputLayer={}", if p.extract_input_layer { "Input" } else { "Current" });
                        }
                        "Union" | "Difference" | "ReverseDifference" | "Intersection" | "InversionIntersection" if p.feature == "OM9-SOLID-004" => {
                            if let Some(active) = self.solid_boolean_two_preview_mode() {
                                option.label = format!("{name} ({})", yes_no(name == active));
                            }
                        }
                        "Width" => option.label = numeric_label(name, p.box_width),
                        "Length" => option.label = format!("Length={}", p.box_length),
                        "Solution=1" | "Solution=2" => {
                            option.label = format!(
                                "{name} ({})",
                                yes_no(name == format!("Solution={}", p.sphere_solution()))
                            );
                        }
                        _ => {}
                    }
                }
            }
            "spiral" => {
                numbers!(spiral_pending, "Height" => height, "Turns" => turns);
                optional_numbers!(spiral_pending, "StartRadius" => start_radius, "EndRadius" => end_radius);
                if name == "Axis"
                    && let Some([x, y, z]) = self.spiral_pending.as_ref().and_then(|p| p.axis)
                {
                    option.label = format!("Axis={x},{y},{z}");
                }
            }
            "through_points" => {
                numbers!(through_points_pending, "Degree" => degree);
                if let Some(p) = &self.through_points_pending {
                    match name {
                        "Closed" => flag!(p.closed),
                        "Open" => flag!(!p.closed),
                        _ => {}
                    }
                }
            }
            "to_lines" => {
                numbers!(to_lines_pending, "Count" => count);
            }
            "tween_curves" => {
                numbers!(tween_curves_pending, "Count" => count);
            }
            _ => {}
        }
    }
}
