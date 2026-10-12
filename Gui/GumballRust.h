// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <cstddef>
#include <cstdint>
extern "C" std::int32_t om9_gumball_component_frame(const double(*)[3],std::size_t,const std::uint32_t(*)[3],std::size_t,std::uint32_t,std::uint32_t,std::uint32_t,double*);
// Value records only; native owners remain on the GUI thread. Mirrors repr(C).
struct Om9GumballSource {
    std::uint64_t document,object,generation,revision,epoch,parent;
    double world[16];
};
struct Om9GumballBegin {
    std::uint64_t document,view_generation,unit_generation;
    double mm_per_unit,frame[12],cplane[12],ray_origin[3],ray_direction[3];
    std::uint32_t handle,modifiers;
};
struct Om9GumballEvent {
    std::uint32_t kind,key,pressed,auto_repeat;
    double ray_origin[3],ray_direction[3],world_point[3];
};
struct Om9GumballSnapshot {
    std::uint32_t enabled,phase,alignment,handle,operation,copy,source_count,constrained;
    std::uint64_t generation;
    double frame[12],matrix[16],constraint;
};
static_assert(sizeof(Om9GumballSource)==176);
static_assert(sizeof(Om9GumballBegin)==280);
static_assert(sizeof(Om9GumballEvent)==88);
static_assert(sizeof(Om9GumballSnapshot)==272);
struct Om9GumballSettings {
    double colors[9],move_length,move_thickness,rotate_radius,rotate_thickness;
    double arrow_length,arrow_width,scale_size,origin_size,plane_size,plane_offset;
    double menu_size,menu_x,menu_y,pick_radius,strength;
    unsigned snappy,orbit;
};
struct Om9GumballProjection {double right[3],up[3],forward[3],world_per_pixel;};
struct Om9GumballLayout {unsigned visible[14];double anchors[3][3],reference[3];};
static_assert(sizeof(Om9GumballSettings)==200);
static_assert(sizeof(Om9GumballProjection)==80);
static_assert(sizeof(Om9GumballLayout)==152);
struct Om9GumballOwnerScale {double placement[16],factors[3];};
extern "C" {
unsigned om9_gumball_owner_scale(const double*,const double*,const double*,unsigned,Om9GumballOwnerScale*);
unsigned om9_gumball_mesh_directions(const double(*)[3],std::size_t,double*);
unsigned om9_gumball_settings(std::uint64_t,const Om9GumballSettings*,Om9GumballSettings*);
unsigned om9_gumball_restore_defaults(std::uint64_t);
unsigned om9_gumball_extents(std::uint64_t,const double(*)[3],std::size_t);
unsigned om9_gumball_layout(std::uint64_t,const Om9GumballProjection*,const double(*)[3],std::size_t,Om9GumballLayout*);
std::uint32_t om9_gumball_owner_frame(std::uint64_t,const Om9GumballSource*,const double*,const double*);
std::uint64_t om9_gumball_create();
unsigned om9_gumball_command(const unsigned char*,std::size_t,unsigned*,unsigned*);
unsigned om9_gumball_destroy(std::uint64_t);
unsigned om9_gumball_begin(std::uint64_t,const Om9GumballBegin*,const Om9GumballSource*,std::size_t);
unsigned om9_gumball_event(std::uint64_t,const Om9GumballEvent*);
unsigned om9_gumball_input(std::uint64_t,const unsigned char*,std::size_t);
unsigned om9_gumball_snapshot(std::uint64_t,Om9GumballSnapshot*);
unsigned om9_gumball_report(std::uint64_t,unsigned char*,std::size_t,std::size_t*);
unsigned om9_gumball_sources(std::uint64_t,Om9GumballSource*,std::size_t,std::size_t*);
unsigned om9_gumball_frame(std::uint64_t,const double*,const double*);
unsigned om9_gumball_frame_from_directions(const double*,const double*,const double*,double*);
unsigned om9_gumball_selection_alignment(unsigned,std::size_t,unsigned,unsigned*);
}
