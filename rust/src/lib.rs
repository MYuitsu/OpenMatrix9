pub mod circle;
mod curve_taylor;
mod owned_curve_input;
#[path="circle_history_selection.rs"]
mod owned_point_selection;
pub mod gumball;
pub mod gumball_component;
pub mod gumball_component_ffi;
pub mod gumball_ffi;
pub mod gumball_view;
pub mod gumball_owner;
pub mod gumball_associative;
pub mod fit_point_input;
pub mod circle_history;
pub mod circle_tangent;
pub mod coordinates;
pub mod coordinate_input;
pub mod cplanes;
pub mod units;
pub mod tolerances;
pub mod layers;
pub mod command_completion;
pub mod command_options;
pub mod command_option_schemas;
pub mod command_options_ffi;
pub mod core_3dm;
pub mod core_3dm_archive;
pub mod core_angle;
pub mod core_distance;
pub mod core_keyboard;
pub mod core_mouse;
pub mod core_notes;
pub mod core_objects;
pub mod result_publication;
pub mod core_picture_frame;
pub mod core_snaps;
pub mod core_view_controls;
pub mod core_view_tabs;
pub mod core_views;
pub mod curve;
pub mod spline;
pub mod nurbs_basis;
pub mod nurbs_basis_ffi;
pub mod native_input_tree;
pub mod native_input_tree_ffi;
pub mod geometry_contract;
pub mod native_edit_capability;
pub mod native_surface_capability;
pub mod surface_history;
pub mod surface_result;
pub mod solid_result;
pub mod edit_result;
pub mod native_rebuild_capability;
pub mod native_curve_capability;
pub mod native_solid_capability;
pub mod native_cage_capability;
pub mod geometry_contract_ffi;
pub mod geometry_frame;
pub mod geometry_frame_ffi;
pub mod instance_graph;
pub mod instance_graph_ffi;
pub mod discrete_snapshot;
pub mod discrete_snapshot_ffi;
pub mod source_attributes;
pub mod source_attributes_ffi;
pub mod topology_snapshot;
pub mod surface_complex;
pub mod shape_validation;
pub mod prepared_cad;
pub mod prepared_result;
mod prepared_discrete;
pub mod topology_snapshot_ffi;
pub mod edge_geometry;
pub mod edge_geometry_ffi;
pub mod array_occurrences;
pub mod array_occurrences_ffi;
pub mod reference_path;
pub mod reference_path_ffi;
pub mod support_snapshot;
pub mod analytic_support;
pub mod support_snapshot_ffi;
pub mod curve_conversion;
pub mod curve_conversion_ffi;
pub mod polycurve_contract;
pub mod polycurve_contract_ffi;
pub mod hatch_loop_input;
pub mod spline_ffi;
pub mod ellipse;
mod curve_ffi;
pub mod edit;
pub mod history;
pub mod builder_history;
pub mod native_builder_capability;
mod builder_transform;
pub mod cage;
pub mod cage_commands;
pub mod retained_archive;
pub mod ffi;
pub mod menu;
pub mod modeling_exchange;
pub mod state;
pub mod surface;
pub mod solid;
pub mod rectangle;
pub mod surface_refit;
pub mod workspace;
pub mod worker_budget;

pub mod viewport_title;
pub mod derived_support;
pub mod derived_support_ffi;

pub mod phase2_curve;

pub mod phase2_ffi;

pub mod phase2_session;

pub mod phase2_snap;

mod phase2_snap_ffi;

pub mod phase3_ffi;

pub mod phase3_modeling;

pub mod layer_clipboard;

pub mod layer_codec;

pub mod layer_collection;

pub mod layer_document;

pub mod layer_exchange;

pub mod layer_ffi;

pub mod layer_legacy;

pub mod layer_lifecycle;

pub mod layer_native;

pub mod layer_retained;

pub mod layer_source_payload;

pub mod layer_state;
