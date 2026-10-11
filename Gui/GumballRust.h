// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <cstddef>
#include <cstdint>
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
extern "C" {
std::uint64_t om9_gumball_create();
unsigned om9_gumball_destroy(std::uint64_t);
unsigned om9_gumball_begin(std::uint64_t,const Om9GumballBegin*,const Om9GumballSource*,std::size_t);
unsigned om9_gumball_event(std::uint64_t,const Om9GumballEvent*);
unsigned om9_gumball_input(std::uint64_t,const unsigned char*,std::size_t);
unsigned om9_gumball_snapshot(std::uint64_t,Om9GumballSnapshot*);
unsigned om9_gumball_sources(std::uint64_t,Om9GumballSource*,std::size_t,std::size_t*);
unsigned om9_gumball_frame(std::uint64_t,const double*,const double*);
unsigned om9_gumball_frame_from_directions(const double*,const double*,const double*,double*);
}
