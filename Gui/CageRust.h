// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <cstddef>
struct Om9CageHandle;
struct Om9CageDescriptor {
    std::size_t counts[3],degrees[3];
    const double* knots[3];std::size_t knot_lengths[3];
    const double* points;std::size_t point_count;
    const double* weights;std::size_t weight_count;
    double world_to_parameter[16];
    unsigned region;double local_min[3],local_max[3],falloff;
};
extern "C" {
int om9_cage_box_parameters(const double*,const double*,std::size_t*,std::size_t*);
int om9_cage_box_fill(const std::size_t*,const std::size_t*,const double*,const double*,double*,std::size_t,double*,std::size_t,double*,std::size_t,double*,std::size_t,double*);
int om9_cage_create(const Om9CageDescriptor*,Om9CageHandle**);
int om9_cage_create_placed(const Om9CageDescriptor*,const double*,Om9CageHandle**);
void om9_cage_destroy(Om9CageHandle*);
int om9_cage_deform(const Om9CageHandle*,const double*,std::size_t,double*,std::size_t);
int om9_cage_evaluate(const Om9CageHandle*,const double*,std::size_t,double*,std::size_t);
int om9_cage_inverse(const Om9CageHandle*,const double*,std::size_t,double*,std::size_t);
int om9_cage_apply(const Om9CageHandle*,const double*,const double*,std::size_t,double*,std::size_t);
int om9_cage_affine(const Om9CageHandle*,double*);
}
