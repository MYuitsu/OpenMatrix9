#pragma once
#include "Phase2Rust.h"
#include <cstddef>

extern "C"
{
bool om9_curve_spline_publish();
bool om9_curve_preview_spline_closed(const double* hover,bool close);
bool om9_spline_rebuild(const double* xyz,std::size_t count,std::size_t poles,std::size_t degree,bool closed);
bool om9_spline_options(std::size_t poles,std::size_t degree);
std::size_t om9_spline_degree();
bool om9_spline_periodic();
std::size_t om9_spline_pole_count();
double om9_spline_pole(std::size_t i,std::size_t axis);
std::size_t om9_spline_knot_count();
double om9_spline_knot(std::size_t i);
std::size_t om9_spline_multiplicity(std::size_t i);
double om9_spline_value(double u,std::size_t axis);
std::size_t om9_spline_message(char* buffer,std::size_t capacity);
bool om9_3dm_overlay_allowed(unsigned int capability,unsigned int action,bool knownReferences,bool isInstance);
std::ptrdiff_t om9_3dm_dependency_closure(std::size_t nodeCount,const std::size_t* offsets,const std::size_t* edges,std::size_t edgeCount,const std::size_t* selected,std::size_t selectedCount,std::size_t* output,std::size_t capacity);
std::ptrdiff_t om9_3dm_copy_budget(std::size_t bytes,std::size_t copies,std::size_t total);
bool om9_3dm_archive_mode_valid(unsigned int mode);
bool om9_3dm_archive_capability_valid(unsigned int capability);
bool om9_3dm_archive_legacy_export_allowed(unsigned int mode);
unsigned int om9_3dm_operation(std::size_t index);
double om9_3dm_scale(double unitMm, double overrideMm);
bool om9_3dm_type_supported(unsigned int kind);
std::size_t om9_console_completion_count();
const char* om9_console_completion_name(std::size_t index);
unsigned int om9_console_completion_rank(std::size_t index,const char* input);
const char* om9_notes_metadata_key();
unsigned int om9_notes_decide(const unsigned char*, std::size_t, const unsigned char*, std::size_t);
bool om9_curve_start(const char* text);
unsigned int om9_curve_input(const char* text);
unsigned int om9_curve_point(double x,double y,double z);
void om9_curve_cancel();
bool om9_curve_active();
std::size_t om9_curve_count();
std::size_t om9_curve_preview_count();
double om9_curve_preview_coordinate(std::size_t point,std::size_t axis);
bool om9_curve_preview_point(double x,double y,double z,bool shift,double* output);
double om9_curve_coordinate(std::size_t point,std::size_t axis);
std::size_t om9_curve_message(char* buffer,std::size_t capacity);

// Commands
std::size_t om9_command_count();
std::size_t om9_workspace_command_count();
std::size_t om9_workspace_command(std::size_t index);
bool om9_command_success(std::size_t index, unsigned int effects);
unsigned int om9_command_permissions(std::size_t index);

const char* om9_command_id(std::size_t index);

const char* om9_command_menu_text(std::size_t index);

const char* om9_command_tooltip(std::size_t index);

bool om9_command_is_active(std::size_t index);

const char* om9_command_execute(std::size_t index);

// Menus
std::size_t om9_menu_group_count();

const char* om9_menu_group_title(std::size_t group);

std::size_t om9_menu_group_command_count(std::size_t group);

const char* om9_menu_group_command_id(
    std::size_t group,
    std::size_t item
);

// Toolbars
std::size_t om9_toolbar_group_count();

const char* om9_toolbar_group_title(std::size_t group);

std::size_t om9_toolbar_group_command_count(std::size_t group);

const char* om9_toolbar_group_command_id(
    std::size_t group,
    std::size_t item
);

// Workbench lifecycle
std::size_t om9_sidebar_group_count();
const char* om9_sidebar_group_title(std::size_t group);
const char* om9_sidebar_group_color(std::size_t group);
unsigned int om9_sidebar_group_kind(std::size_t group);
std::size_t om9_sidebar_group_item_count(std::size_t group);
std::size_t om9_sidebar_group_item_command(std::size_t group, std::size_t item);
std::size_t om9_sidebar_quick_count();
std::size_t om9_sidebar_quick_command(std::size_t item);
const char* om9_command_icon(std::size_t index);
const char* om9_command_native_id(std::size_t index);
std::size_t om9_sidebar_selected_group();
bool om9_sidebar_select_group(std::size_t group);
unsigned int om9_sidebar_section_flags(std::size_t section);
bool om9_sidebar_set_section(std::size_t section, bool visible, bool collapsed);
void om9_sidebar_reset();
void om9_sidebar_record_execution(std::size_t command, bool success);
std::size_t om9_sidebar_history_count();
std::size_t om9_sidebar_history_command(std::size_t item);
void om9_workbench_activated();

void om9_workbench_deactivated();
double om9_core_view_rotation(std::size_t slot,std::size_t axis,bool plane);
bool om9_core_view_perspective(std::size_t slot);
unsigned int om9_core_grid_line_kind(int index);
double om9_core_perspective_height(double distance,double angle);
double om9_view_scaled_value(bool perspective,double value,double factor);
bool om9_view_tool_start(unsigned int kind);
bool om9_view_tool_press(double x,double y);
unsigned int om9_view_tool_motion(double x,double y,double height);
unsigned int om9_view_tool_release(double x,double y);
double om9_view_tool_value(std::size_t axis);
void om9_view_tool_cancel();
bool om9_core_crosshairs();
bool om9_core_crosshairs_toggle();
unsigned int om9_core_tabs_state();
bool om9_core_tabs_load(unsigned int state);
// PictureFrame output: corner3, center3, x3, y3, normal3, width, height.
bool om9_picture_frame_plan(const double* axes,const double* corner,const double* reference,double width,double aspect,unsigned int flags,double* output);
double om9_measure_distance(double ax,double ay,double az,double bx,double by,double bz,unsigned int unit);
double om9_measure_angle(const double* points);
unsigned int om9_snap_state();
bool om9_snap_load(unsigned int state);
bool om9_snap_toggle(unsigned int bit);
// Packed world XYZ / logical screen XY. SIZE_MAX means no snap.
std::size_t om9_snap_end_pick(const double* candidates,std::size_t count,double x,double y,double radius);
std::size_t om9_snap_mode_pick(const double* candidates,std::size_t count,double x,double y,double radius,unsigned int mode);
unsigned int om9_core_tabs_apply(const unsigned char* text,std::size_t length);
bool om9_curve_frame(double ox,double oy,double oz,double xx,double xy,double xz,double yx,double yy,double yz,double zx,double zy,double zz);
std::size_t om9_keyboard_count();
const char* om9_keyboard_key(std::size_t index);
const char* om9_keyboard_target(std::size_t index);
bool om9_ortho_enabled();
void om9_ortho_load(bool value);
bool om9_ortho_toggle();
bool om9_ortho_active(bool shift);
unsigned int om9_mouse_action(unsigned int button,unsigned int modifiers,bool perspective);
bool om9_mouse_moved(double ax,double ay,double bx,double by,double threshold);
bool om9_mouse_window_contains(double x0,double y0,double x1,double y1,double b0,double b1,double b2,double b3);
unsigned int om9_curve_mouse_point(double x,double y,double z,bool shift);

std::size_t om9_viewport_mode_count();
const char* om9_viewport_mode_name(std::size_t index);
int om9_viewport_native_mode(std::size_t index);
int om9_viewport_next_single(int current,std::size_t clicked);
}

extern "C" {
unsigned om9_edit_kind(const char*);
bool om9_edit_set_option(unsigned,bool);
bool om9_edit_option(unsigned);
bool om9_edit_set_tolerance(double);
double om9_edit_tolerance();
bool om9_edit_start(const char*);
void om9_edit_cancel();
unsigned om9_edit_phase();
bool om9_edit_add(const char*);
bool om9_edit_finish();
void om9_edit_back();
bool om9_edit_undo();
bool om9_edit_cycle();
unsigned om9_edit_mode();
std::size_t om9_edit_count(unsigned);
bool om9_edit_delete_input(int);
unsigned int om9_surface_kind(const char*);
unsigned int om9_surface_sweep2_contact(std::size_t);
bool om9_surface_options_valid(unsigned int,unsigned int,bool);
bool om9_surface_transport(const double*,double*);
bool om9_surface_transport_height(const double*,bool,double*);
bool om9_surface_uniform_spline(const double*,std::size_t,bool);
bool om9_surface_refit(const double*,std::size_t,double,bool);
double om9_surface_refit_deviation();
double om9_surface_slash_parameter(const double*,std::size_t,double);
bool om9_surface_chain_start();
bool om9_surface_chain_add(const char*);
bool om9_surface_chain_finish(bool);
bool om9_surface_start(const char*);
bool om9_surface_add(const char*,bool closed);
bool om9_surface_finish();
bool om9_surface_undo();
bool om9_surface_back();
void om9_surface_cancel();
unsigned int om9_surface_phase();
std::size_t om9_surface_count();
std::size_t om9_surface_message(char*,std::size_t);
}
#include "LayerRustAbi.h"
