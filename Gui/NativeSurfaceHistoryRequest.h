// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <cstddef>
#include <cstdint>
#include <type_traits>
namespace OpenMatrix9Gui {
struct NativeSavedSurfaceInput { std::uint32_t count,flags; double seam; };
// Rust owns parsing/policy. Caller owns this fixed scalar result and the borrowed
// immutable UTF-8 property bytes throughout the synchronous decode call.
struct NativeSavedSurfaceRequest {
    std::uint32_t kind,style,sectionMode,pointCount,continuityA,continuityB,flags,groupCount;
    double tolerance;
    std::size_t slashCount;
    NativeSavedSurfaceInput inputs[256];
    double slashes[64][2];
};
static_assert(std::is_standard_layout_v<NativeSavedSurfaceRequest>);
static_assert(sizeof(NativeSavedSurfaceInput)==16&&sizeof(NativeSavedSurfaceRequest)==5168);
static_assert(alignof(NativeSavedSurfaceRequest)==8);
static_assert(offsetof(NativeSavedSurfaceRequest,tolerance)==32&&offsetof(NativeSavedSurfaceRequest,slashCount)==40);
static_assert(offsetof(NativeSavedSurfaceRequest,inputs)==48&&offsetof(NativeSavedSurfaceRequest,slashes)==4144);
extern "C" unsigned om9_surface_history_decode(const unsigned char*,std::size_t,std::size_t,NativeSavedSurfaceRequest*);
}
