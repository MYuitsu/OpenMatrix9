// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <array>
#include <cstddef>
#include <functional>
#include <string>
namespace App {class DocumentObject;}
namespace OpenMatrix9Gui {
// Value-only Rust envelope. Neither query retains host storage or geometry.
struct NativeCageRequest {
    unsigned kind=0,stage=3,phase=4,role=0,flags=0,reserved=0;
    std::size_t inputCount=0;
    std::array<std::size_t,3> counts{2,2,2},degrees{1,1,1};
    std::array<double,3> lo{},hi{};double falloff=0;
};
static_assert(sizeof(NativeCageRequest)==136&&alignof(NativeCageRequest)==8);
static_assert(offsetof(NativeCageRequest,inputCount)==24&&offsetof(NativeCageRequest,counts)==32
    &&offsetof(NativeCageRequest,lo)==80&&offsetof(NativeCageRequest,falloff)==128);
extern "C" bool om9_native_cage_declared(unsigned);
void validateNativeCageCapability(unsigned,const NativeCageRequest&,const std::string&,const std::string&);
void validateCageSessionCapability(const void*,unsigned,unsigned,unsigned,const std::string&,const std::string&);
// GUI-thread/GIL borrowed builtin owners only. Runs the supplied Rust request
// decision on concrete type, then existing basis/budget/value gates, before copy.
unsigned validateCageGeometryInput(App::DocumentObject&,const std::string&,
    const std::function<void(unsigned)>&);
}
