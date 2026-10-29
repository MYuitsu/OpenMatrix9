// SPDX-License-Identifier: LGPL-2.1-or-later
//! Command-line prompt and active-input snapshot.
//!
//! Extracted unchanged from `session.rs`; the pending-command prompt text and the option
//! snapshot stay in one place so command-line presentation can be read without the dispatcher.
use super::*;

impl Controller {
    pub fn prompt(&self) -> String {
        // A session whose context is gone asks for nothing: the prompt and the
        // phase machine must never disagree (OM9-L3-01-02-004 N5).
        if !self.is_running() {
            return "Command: ".to_owned();
        }
        if let Some(pending) = &self.head_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.settings_pending {
            return pending.prompt().to_owned();
        }
        if let Some(prompt) = self.gem_prompt() {
            return prompt;
        }
        if let Some(prompt) = self.gem_placement_prompt() {
            return prompt;
        }
        if let Some(prompt) = self.gem_cluster_prompt() {
            return prompt;
        }
        if let Some(prompt) = self.gem_edits_prompt() {
            return prompt;
        }
        if let Some(prompt) = self.gem_pave_prompt() {
            return prompt;
        }
        if let Some(prompt) = self.gem_cutter_prompt() {
            return prompt;
        }
        if let Some(prompt) = self.row_cutter_prompt() {
            return prompt;
        }
        if self.solid_pipe_needs_curve() {
            if self.solid_pending.as_ref().is_some_and(|p| {
                matches!(
                    p.feature,
                    "OM9-SOLID-020"
                        | "OM9-SOLID-016"
                        | "OM9-SOLID-019"
                        | "OM9-SOLID-017"
                        | "OM9-SOLID-018"
                ) || (p.feature == "OM9-SOLID-013" && p.ellipsoid_around)
                    || (p.feature == "OM9-SOLID-015" && p.torus_around)
            }) {
                return if self.solid_pending.as_ref().is_some_and(|p| {
                    matches!(
                        p.feature,
                        "OM9-SOLID-020" | "OM9-SOLID-016" | "OM9-SOLID-019" | "OM9-SOLID-017"
                    ) && p.cone_base_points == 6
                }) {
                    "Select tangent curve"
                } else {
                    "Select curve for AroundCurve"
                }
                .to_owned();
            }
            return if self
                .solid_pending
                .as_ref()
                .is_some_and(|p| p.pipe_multiple_selecting)
            {
                if self.solid_pending.as_ref().is_some_and(|p| p.pipe_chain) {
                    "Select connected curve segments (Enter when done)".to_owned()
                } else {
                    "Select curves for Pipe (Enter when done)".to_owned()
                }
            } else {
                "Select curve for Pipe".to_owned()
            };
        }
        if let Some(pending) = &self.solid_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.circle_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.rect_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.polyline_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.interp_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.ellipse_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.arc_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.rebuild_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.fit_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.refit_srf_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.offset_curve_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.project_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.blend_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.isocurve_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.pull_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.intersect_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.crv2view_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.crv_seam_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.control_curve_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.offset_on_surface_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.offset_normal_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.corner_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.cross_section_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.fillet_corners_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.continue_curve_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.through_points_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.polygon_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.helix_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.spiral_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.divide_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.to_lines_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.extend_curve_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.tween_curves_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.continue_interp_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.arc_blend_pending {
            return pending.prompt().to_owned();
        }
        if let Some(prompt) = self.top11_prompt() {
            return prompt.to_owned();
        }
        if let Some(pending) = &self.surface_pending {
            return if pending.feature == surface_edit::OFFSET {
                "Offset distance (Distance=): "
            } else {
                "Extrude distance (Distance=): "
            }
            .to_owned();
        }
        if self.curve_pair_pending.is_some() {
            return "Rail revolve axis (Axis=): ".to_owned();
        }
        if let Some(pending) = &self.surface_pair_pending {
            return if pending.source_edge.is_none() {
                "Match source edge (Source=): ".to_owned()
            } else if pending.target_edge.is_none() {
                "Match target edge (Target=): ".to_owned()
            } else {
                format!(
                    "Match continuity (Continuity={}, Enter to apply): ",
                    surface_edit::continuity_label(pending.continuity)
                )
            };
        }
        if let Some(pending) = &self.seam_pending {
            return if pending.axis.is_none() {
                "Seam axis (Axis=): "
            } else {
                "Seam tolerance (Tolerance=): "
            }
            .to_owned();
        }
        if let Some(pending) = &self.soft_edit_pending {
            if !pending.base_complete() {
                return if pending.base[0].is_some() {
                    "Soft edit base point (V=): ".to_owned()
                } else {
                    "Soft edit base point (U=,V=): ".to_owned()
                };
            }
            if !pending.delta_complete() {
                const NAMES: [&str; 3] = ["Dx", "Dy", "Dz"];
                let index = pending
                    .delta
                    .iter()
                    .position(Option::is_none)
                    .expect("delta value");
                return format!("Soft edit move ({}=", NAMES[index]) + "): ";
            }
            return format!(
                "Soft edit options (UDistance={} VDistance={} Copy={} FixEdges={} DirConstraint={}): ",
                pending.u_distance,
                pending.v_distance,
                if pending.copy { "Yes" } else { "No" },
                if pending.fix_edges { "Yes" } else { "No" },
                pending.constraint.label()
            );
        }
        if let Some(pending) = &self.curve_soft_edit_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.angle_isocurve_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.extrude_surface_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.heightfield_pending {
            return pending.prompt().to_owned();
        }
        if self.tween_pending.is_some() {
            return "Tween fraction (Fraction=): ".to_owned();
        }
        if let Some(pending) = &self.extend_pending {
            return if pending.edge.is_none() {
                "Extend edge (Edge=): ".to_owned()
            } else {
                "Extend distance (Distance=): ".to_owned()
            };
        }
        if let Some(pending) = &self.split_pending {
            return if pending.axis.is_none() {
                "Split axis (Axis=): ".to_owned()
            } else {
                "Split parameter (Parameter=): ".to_owned()
            };
        }
        if let Some(pending) = &self.tangent_pending {
            return if pending.edge.is_none() {
                "Tangent edge (Edge=): ".to_owned()
            } else {
                "Tangent direction (Direction=): ".to_owned()
            };
        }
        if self.extrude_all_pending.is_some() {
            return "Extrude all height (Height=): ".to_owned();
        }
        if let Some(pending) = &self.variable_offset_pending {
            return if pending.start.is_none() {
                "Variable offset start (Start=): ".to_owned()
            } else if pending.end.is_none() {
                "Variable offset end (End=): ".to_owned()
            } else {
                "Variable offset axis (Axis=): ".to_owned()
            };
        }
        if let Some(pending) = &self.shrink_pending {
            return if pending.u0.is_none() {
                "Shrink u0 (U0=): ".to_owned()
            } else if pending.u1.is_none() {
                "Shrink u1 (U1=): ".to_owned()
            } else if pending.v0.is_none() {
                "Shrink v0 (V0=): ".to_owned()
            } else {
                "Shrink v1 (V1=): ".to_owned()
            };
        }
        if let Some(pending) = &self.variable_blend_pending {
            return if pending.start_scale.is_none() {
                "Blend start scale (Start=): ".to_owned()
            } else {
                "Blend end scale (End=): ".to_owned()
            };
        }
        if self.chamfer_pending.is_some() {
            return "Chamfer setback: ".to_owned();
        }
        if let Some(pending) = &self.fillet_pending {
            if pending.feature == surface_edit::VARIABLE_FILLET {
                return if pending.start_radius.is_none() {
                    "Fillet start radius (Start=): ".to_owned()
                } else {
                    "Fillet end radius (End=): ".to_owned()
                };
            }
            return "Fillet radius (Radius=): ".to_owned();
        }
        if let Some(pending) = &self.variable_chamfer_pending {
            return if pending.start_setback.is_none() {
                "Chamfer start setback (Start=): ".to_owned()
            } else {
                "Chamfer end setback (End=): ".to_owned()
            };
        }
        if let Some(pending) = &self.history_options_pending {
            let closed = if pending.closed_available {
                format!("Closed={} ", if pending.closed { "Yes" } else { "No" })
            } else {
                String::new()
            };
            let flip = format!(
                "Flip={}{}",
                if pending.flip { "Yes" } else { "No" },
                if pending.rail_count == 2 {
                    " Slash"
                } else {
                    ""
                }
            );
            return format!("Sweep options ({closed}{flip}): ");
        }
        if let Some(pending) = &self.slash_pending {
            return if pending.points.is_empty() {
                "Slash point on first rail: "
            } else {
                "Slash point on second rail: "
            }
            .to_owned();
        }
        if let Some(pending) = &self.measure_pending {
            return pending.prompt().to_owned();
        }
        if let Some(pending) = &self.annotation_pending {
            return pending.prompt().into();
        }
        if let Some(pending) = &self.transform_pending {
            return pending.prompt().to_owned();
        }
        match &self.session {
            None => "Command: ",
            Some(s) if s.feature == POINT => "Location of point: ",
            Some(s) if surface_edit::plane_like(s.feature) => match s.points.len() {
                0 => "First corner: ",
                1 => "Second corner: ",
                _ => "Third corner: ",
            },
            Some(s) => match (s.start.is_some(), s.both_sides) {
                (false, false) => "Start of line (BothSides=No Length=): ",
                (false, true) => "Start of line (BothSides=Yes Length=): ",
                (true, false) => "End of line (BothSides=No Length=): ",
                (true, true) => "End of line (BothSides=Yes Length=): ",
            },
        }
        .to_owned()
    }
    /// Prompt text for the command line with every clickable option spelled inline.
    ///
    /// Polyline and Line already name their options inside the prompt; every other
    /// session only publishes them as option tokens, which the command bar would draw
    /// as a floating chip row over the transcript. Inserting the same signature the
    /// prompt would carry (`Location of point (Undo Cancel): `) keeps one presentation
    /// for every command and lets the existing inline renderer underline and click it.
    /// History echoes keep [`Self::prompt`] so recorded lines stay stable.
    pub fn prompt_with_options(&self) -> String {
        let prompt = self.prompt();
        if !self.is_running() {
            return prompt;
        }
        let options = crate::command_options::CommandOptionInterface::command_options(self);
        if options.is_empty() {
            return prompt;
        }
        let labels: Vec<&str> = options.iter().map(|option| option.label.as_str()).collect();
        let mut search_from = 0;
        let mut all_present = true;
        for label in &labels {
            match prompt[search_from..].find(label) {
                Some(at) => search_from += at + label.len(),
                None => {
                    all_present = false;
                    break;
                }
            }
        }
        if all_present {
            return prompt;
        }
        let signature = format!(" ({})", labels.join(" "));
        let trimmed = prompt.trim_end();
        if let Some(head) = trimmed.strip_suffix(':') {
            format!("{}{signature}: ", head.trim_end())
        } else {
            format!("{trimmed}{signature}: ")
        }
    }
    pub(super) fn validate_session(&self) -> Result<&CommandSession> {
        let s = self
            .session
            .as_ref()
            .ok_or(Error::Missing("active command"))?;
        self.validate_context(&s.context)?;
        Ok(s)
    }
    /// Owner name for positional actions that intentionally expose no option row.
    pub(super) fn active_positional_owner(&self) -> Option<&'static str> {
        if self.top11_pending.is_some() {
            Some("top11")
        } else if self.surface_pending.is_some() {
            Some("surface_distance")
        } else if self.curve_pair_pending.is_some() {
            Some("curve_pair")
        } else if self.extrude_all_pending.is_some() {
            Some("extrude_all")
        } else if self.chamfer_pending.is_some() {
            Some("surface_chamfer")
        } else if self.arc_blend_pending.is_some() {
            Some("arc_blend")
        } else if self.crv2view_pending.is_some() {
            Some("crv2view")
        } else {
            None
        }
    }
    /// Read-only active-input snapshot. Idle has no owner/options; a specialized
    /// pending command wins because its provider is the one producing options.
    pub fn active_command_input_snapshot(&self) -> CommandInputSnapshot {
        let options = crate::command_options::CommandOptionInterface::command_options(self);
        let owner = options
            .first()
            .map(|option| option.id.owner)
            .or_else(|| self.active_positional_owner());
        CommandInputSnapshot {
            owner,
            run: self.command_run,
            phase: self.phase(),
            options,
        }
    }
}
