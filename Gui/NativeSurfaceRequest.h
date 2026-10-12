// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <cstddef>
#include <cstdint>
#include <type_traits>
namespace OpenMatrix9Gui {
// Synchronous scalar ABI. The caller owns initialized flat slash storage for
// this call only; Rust checks the envelope and retains a bounded private copy.
struct NativeSurfaceRequest {
    std::uint32_t kind,representation,stage,flags,style,sectionMode,pointCount,
                  continuityA,continuityB,railClosed;
    std::size_t inputCount;
    double tolerance;
    const double* slashPairs;
    std::size_t slashCount;
};
static_assert(std::is_standard_layout_v<NativeSurfaceRequest>);
static_assert(sizeof(NativeSurfaceRequest)==72&&alignof(NativeSurfaceRequest)==8);
static_assert(offsetof(NativeSurfaceRequest,inputCount)==40&&offsetof(NativeSurfaceRequest,tolerance)==48);
static_assert(offsetof(NativeSurfaceRequest,slashPairs)==56&&offsetof(NativeSurfaceRequest,slashCount)==64);
extern "C" unsigned om9_native_surface_capability(NativeSurfaceRequest);
extern "C" bool om9_native_surface_declared(unsigned);
}
