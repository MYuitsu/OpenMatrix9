// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "SolidReferences.h"
namespace OpenMatrix9Gui {
// Exact native evaluator/kernel adapter; solver policy/numerics live in Rust.
std::array<double,7> circleSolveNative(const std::map<std::size_t,SolidCurveReference>&,
    const double* constraints,std::size_t count,const double* frame,int solution,
    bool fromFirst,bool vertical);
}
