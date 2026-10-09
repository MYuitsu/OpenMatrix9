// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "ThreeDmGeometry.h"
#include <QJsonObject>
#include <cstdint>
#include <TopoDS_Wire.hxx>
namespace OpenMatrix9Gui::ThreeDm::Polycurve {
struct Input {
 std::size_t count;
 std::uint32_t parent_known,closed,policy,reserved;
 double length_tolerance,parameter_tolerance;
 std::array<double,2> parent_domain;
};
struct Source {
 std::size_t ordinal;std::uint32_t family,parent_known;
 std::array<double,2> child_domain,parent_domain;
 std::array<double,3> start,end;
};
struct Native {
 std::size_t ordinal;std::uint32_t direction,shared_previous;
 std::array<double,3> start,end;
};
struct Summary {std::uint32_t status,schema;std::array<double,4> metrics;};
static_assert(sizeof(Input)==56 && alignof(Input)==8 && offsetof(Input,length_tolerance)==24);
static_assert(sizeof(Source)==96 && alignof(Source)==8 && offsetof(Source,start)==48);
static_assert(sizeof(Native)==64 && alignof(Native)==8 && offsetof(Native,start)==16);
static_assert(sizeof(Summary)==40 && alignof(Summary)==8 && offsetof(Summary,metrics)==8);
extern "C" {
 std::size_t om9_polycurve_limit();
 std::uint32_t om9_polycurve_preflight(Input);
 Summary om9_polycurve_source(Input,Source,Source,std::uint32_t);
 Summary om9_polycurve_native(Input,Source,Native,Source,Native,std::uint32_t);
 Summary om9_polycurve_finish(Input,Source,Source,Native,Native,std::size_t,std::uint32_t,std::uint32_t);
}
struct Options {double lengthTolerance,parameterTolerance;};
struct Report {
 Input input{};std::vector<Source> source;std::vector<Native> native;
 std::vector<Summary> measurements;Summary closure{};
 QJsonObject json() const;
};
struct ObservedWire {TopoDS_Wire wire;Report report;};
// Synchronous live native owners remain caller-owned; returned wire/report own
// independent geometry/values. Flat ordinary owned children only, no healing.
ObservedWire importPolyCurveObserved(const ON_PolyCurve&,Options);
std::unique_ptr<ON_PolyCurve> exportWireChecked(const TopoDS_Wire&,Options);
void verifyWirePartition(const TopoDS_Wire& expected,const TopoDS_Wire& actual,Options);
}
