// SPDX-License-Identifier: LGPL-2.1-or-later
// Owner-state fixtures exercise the public adapter without relying on selection
// or native geometry dispatch. Geometry commits are covered by integration tests.
use super::Controller;
use super::command_options::PROVIDERS;
use crate::command_options::{Activation, CommandOptionEffect, CommandOptionInterface};
use om9_core::{budget::Limits, geometry::Curve, model::Document};

#[test]
fn sphere_solution_choices_reflect_the_owner_branch() {
    let mut c = Controller::new(Document::new("solution"), Limits::default()).unwrap();
    let mut p = super::solid::SolidPending::sphere_pending();
    p.sphere_kind = "TangentPoint";
    p.points = vec![[0.; 3], [1., 0., 0.]];
    c.solid_pending = Some(p);
    let option = c.resolve_command_option("Solution=2").unwrap();
    c.activate_command_option(option).unwrap();
    assert_eq!(
        c.command_options()
            .into_iter()
            .find(|o| o.id.name() == "Solution=2")
            .unwrap()
            .label,
        "Solution=2 (Yes)"
    );
    assert!(c.document().entities.is_empty());
    assert_eq!(c.coordinator().undo_len(), 0);
}

#[test]
fn every_declared_owner_has_unique_options_and_safe_value_entry() {
    let curve = Curve::Line {
        start: [0.; 3],
        end: [1., 0., 0.],
        domain: [0., 1.],
    };
    let surface = om9_core::surface_kernel::plane(
        [0.; 3],
        [1., 0., 0.],
        [0., 1., 0.],
        10.,
        10.,
        &Limits::default(),
    )
    .unwrap();
    assert_eq!(PROVIDERS.len(), 61);
    for &(owner, provider) in PROVIDERS {
        let mut c = Controller::new(Document::new(owner), Limits::default()).unwrap();
        match owner {
            "solid" => c.solid_pending = Some(super::solid::SolidPending::box_pending()),
            "circle" => c.circle_pending = Some(super::circle::CirclePending::new()),
            "rect" => c.rect_pending = Some(super::rect::RectPending::new()),
            "interp" => {
                c.interp_pending = Some(super::interp::InterpPending {
                    points: Vec::new(),
                    degree: 3,
                    close: false,
                })
            }
            "ellipse" => c.ellipse_pending = Some(super::ellipse::EllipsePending::new()),
            "arc" => c.arc_pending = Some(super::arc::ArcPending::new()),
            "rebuild" => {
                c.rebuild_pending = Some(super::rebuild::RebuildPending {
                    source: curve.clone(),
                    degree: 3,
                    points: 3,
                })
            }
            "fit" => {
                c.fit_pending = Some(super::fit::FitPending {
                    source: curve.clone(),
                    tolerance: 1.,
                })
            }
            "offset_curve" => {
                c.offset_curve_pending = Some(super::offset::OffsetPending {
                    source: curve.clone(),
                    distance: 1.,
                })
            }
            "project" => {
                c.project_pending = Some(super::project::ProjectPending {
                    source: curve.clone(),
                    direction: None,
                })
            }
            "blend" => {
                c.blend_pending = Some(super::blend::BlendPending {
                    feature: "OM9-CURVE-001",
                    first: curve.clone(),
                    second: curve.clone(),
                    scale1: 1.,
                    scale2: 1.,
                })
            }
            "isocurve" => {
                c.isocurve_pending = Some(super::isocurve::IsoPending {
                    surface: surface.clone(),
                    iso: om9_core::surface_kernel::SurfaceIso::U,
                    value: 1.,
                })
            }
            "pull" => {
                c.pull_pending = Some(super::pull::PullPending {
                    curve: curve.clone(),
                    surface: surface.clone(),
                    tolerance: 1.,
                })
            }
            "intersect" => {
                c.intersect_pending = Some(super::intersect::IntersectPending {
                    first: curve.clone(),
                    second: curve.clone(),
                    tolerance: 1.,
                })
            }
            "crv2view" => {
                c.crv2view_pending = Some(super::crv2view::Crv2ViewPending {
                    first: curve.clone(),
                    second: curve.clone(),
                    ids: Vec::new(),
                    history: false,
                })
            }
            "through_points" => {
                c.through_points_pending = Some(super::through_points::ThroughPointsPending {
                    points: Vec::new(),
                    degree: 3,
                    closed: false,
                })
            }
            "polygon" => c.polygon_pending = Some(super::polygon::PolygonPending::new()),
            "helix" => c.helix_pending = Some(super::helix::HelixPending::new()),
            "spiral" => c.spiral_pending = Some(super::spiral::SpiralPending::new()),
            "fillet_corners" => {
                c.fillet_corners_pending = Some(super::fillet_corners::FilletCornersPending {
                    curve: curve.clone(),
                    radius: 1.,
                })
            }
            "divide" => {
                c.divide_pending = Some(super::divide::DividePending {
                    curve: curve.clone(),
                    count: 3,
                })
            }
            "to_lines" => {
                c.to_lines_pending = Some(super::to_lines::ToLinesPending {
                    curve: curve.clone(),
                    count: 3,
                })
            }
            "crv_seam" => {
                c.crv_seam_pending = Some(super::crv_seam::CrvSeamPending {
                    curve: curve.clone(),
                    fraction: 1.,
                })
            }
            "control_curve" => {
                c.control_curve_pending = Some(super::control_curve::ControlCurvePending {
                    points: Vec::new(),
                    degree: 3,
                })
            }
            "continue_curve" => {
                c.continue_curve_pending = Some(super::continue_curve::ContinueCurvePending {
                    points: Vec::new(),
                    degree: 3,
                })
            }
            "offset_on_surface" => {
                c.offset_on_surface_pending =
                    Some(super::offset_on_surface::OffsetOnSurfacePending {
                        curve: curve.clone(),
                        surface: surface.clone(),
                        distance: 1.,
                    })
            }
            "offset_normal" => {
                c.offset_normal_pending = Some(super::offset_normal::OffsetNormalPending {
                    curve: curve.clone(),
                    surface: surface.clone(),
                    distance: 1.,
                })
            }
            "corner" => {
                c.corner_pending = Some(super::corner_ops::CornerPending {
                    feature: "OM9-CURVE-001",
                    first: curve.clone(),
                    second: curve.clone(),
                    radius: 1.,
                    distance: 1.,
                })
            }
            "cross_section" => {
                c.cross_section_pending = Some(super::cross_section::CrossSectionPending {
                    count: 3,
                    spacing: 1.,
                })
            }
            "curve_soft_edit" => {
                c.curve_soft_edit_pending = Some(super::curve_soft_edit::CurveSoftEditPending {
                    curve: curve.clone(),
                    index: 3,
                    target: [0.; 3],
                    has_target: false,
                })
            }
            "angle_isocurve" => {
                c.angle_isocurve_pending = Some(super::angle_isocurve::AngleIsocurvePending {
                    surface: surface.clone(),
                    angle: 1.,
                    axis: 3,
                })
            }
            "extrude_surface" => {
                c.extrude_surface_pending = Some(super::extrude_surface::ExtrudePending {
                    curves: vec![curve.clone()],
                    height: 1.,
                })
            }
            "heightfield" => {
                c.heightfield_pending = Some(super::heightfield::HeightfieldPending {
                    file: None,
                    height: 1.,
                    size: 1.,
                })
            }
            "extend_curve" => {
                c.extend_curve_pending = Some(super::extend_curve::ExtendPending {
                    curve: curve.clone(),
                    length: 1.,
                    at_start: false,
                })
            }
            "tween_curves" => {
                c.tween_curves_pending = Some(super::tween_curves::TweenCurvesPending {
                    first: curve.clone(),
                    second: curve.clone(),
                    count: 3,
                })
            }
            "continue_interp" => {
                c.continue_interp_pending = Some(super::continue_interp::ContinueInterpPending {
                    points: Vec::new(),
                    degree: 3,
                })
            }
            "arc_blend" => {
                c.arc_blend_pending = Some(super::arc_blend::ArcBlendPending {
                    first: curve.clone(),
                    second: curve.clone(),
                })
            }
            "refit_srf" => {
                c.refit_srf_pending = Some(super::refit_srf::RefitSrfPending {
                    source: surface.clone(),
                    tolerance: 0.01,
                })
            }
            "settings" => {
                let row = om9_core::settings::gems::straight_row(
                    &[2.0, 2.0],
                    0.1,
                    [0.; 3],
                    [1., 0., 0.],
                    [0., 0., 1.],
                )
                .unwrap();
                let params = om9_core::settings::prong::ProngParams::default();
                let set = om9_core::settings::prong::build_prong_set(&row, &params).unwrap();
                c.settings_pending = Some(super::settings_commands::SettingsPending {
                    tool: super::settings_commands::SettingsTool::ProngBuilder,
                    draft: super::settings_commands::SettingsDraft::Prongs { row, params, set },
                    message: "Prong Builder".to_owned(),
                })
            }
            _ => continue,
        }
        let tokens = provider(&c);
        assert!(!tokens.is_empty(), "{owner}");
        let options = c.command_options();
        assert_eq!(options.len(), tokens.len(), "{owner}");
        let mut ids = std::collections::HashSet::new();
        let mut shortcuts = std::collections::HashSet::new();
        for option in options {
            assert!(ids.insert(option.id), "{owner}");
            assert!(!option.label.is_empty(), "{owner}");
            assert_eq!(c.resolve_command_option(option.id.name()), Some(option.id));
            if let Some(key) = option.shortcut {
                assert!(key.is_ascii_alphabetic(), "{owner}");
                assert!(shortcuts.insert(key), "{owner}");
                assert_eq!(
                    c.resolve_command_option(&key.to_string()),
                    Some(option.id),
                    "{owner}"
                );
            }
            if let Activation::Draft(prefix) = option.activation {
                let before = c.command_options();
                assert_eq!(
                    c.activate_command_option(option.id).unwrap(),
                    CommandOptionEffect::Draft(prefix.into())
                );
                assert_eq!(c.command_options(), before, "{owner}");
                assert!(c.document().entities.is_empty(), "{owner}");
                assert_eq!(c.coordinator().undo_len(), 0, "{owner}");
            }
        }
        let cancel = c.resolve_command_option("Cancel").unwrap();
        c.activate_command_option(cancel).unwrap();
        assert!(c.command_options().is_empty(), "{owner}");
        assert!(c.document().entities.is_empty(), "{owner}");
    }
}
