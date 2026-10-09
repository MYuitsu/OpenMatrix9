// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "NurbsBasisAdapter.h"
#include <QJsonObject>
#include <cstdint>
#include <span>

namespace OpenMatrix9Gui::ThreeDm::CurveConversion {
// Stable, copied value ABI. No native owner or allocator crosses this boundary.
struct Input {
    std::uint32_t form, method, frame, reserved;
    std::array<double, 2> source_domain, target_domain;
    double conversion_tolerance, length_tolerance, parameter_tolerance;
};
struct Observation {
    double source_parameter, target_parameter, inverse_parameter;
    std::uint32_t forward_success, inverse_success;
    std::array<double, 3> source_point, intermediate_point, native_point, raw_native_point;
};
struct QuerySummary {
    std::uint32_t status, schema_version, action, raw_parameter_reusable;
    Observation observation;
    std::array<double, 3> residuals;
};
static_assert(sizeof(Input) == 72 && alignof(Input) == 8);
static_assert(offsetof(Input, source_domain) == 16 && offsetof(Input, conversion_tolerance) == 48);
static_assert(sizeof(Observation) == 128 && alignof(Observation) == 8);
static_assert(offsetof(Observation, source_point) == 32);
static_assert(sizeof(QuerySummary) == 168 && alignof(QuerySummary) == 8);
static_assert(offsetof(QuerySummary, observation) == 16 && offsetof(QuerySummary, residuals) == 144);
extern "C" {
std::size_t om9_curve_conversion_query_limit();
std::uint32_t om9_curve_conversion_preflight(Input, std::size_t);
std::uint32_t om9_curve_conversion_request(Input, double);
std::uint32_t om9_curve_conversion_mapped_parameter(Input, double, std::uint32_t success, std::uint32_t direction);
QuerySummary om9_curve_conversion_observation(Input, Observation);
}
struct Options {
    double conversionTolerance;
    double lengthTolerance;
    double parameterTolerance;
};
struct QueryError : ExchangeError {
    std::uint32_t status;
    explicit QueryError(std::uint32_t code);
};
struct Report {
    Input input{};
    std::uint32_t sourceCrc = 0;
    std::array<double, 2> getNurbFormDomain{};
    bool beforeClampPeriodic = false, afterClampPeriodic = false;
    BasisContract::CurveBasisData targetBasis;
    std::vector<QuerySummary> queries;
    QJsonObject json() const;
};
struct ObservedCurve {
    TopoDS_Shape shape;
    Report report;
};
// Synchronous native exception: original source must remain live/immutable.
// Report values are independent of both source and target owners. Length units
// are unspecified by ON_Curve; a host caller must supply its actual unit context.
// Full-domain conversion only, no reversal/transform/subdomain/export mapping.
// A raw diagnostic outside target domain is unsupported, never extrapolated.
// Failed queries throw without returning any partial geometry/report.
ObservedCurve importCurveObserved(const ON_Curve&, Options, std::span<const double>);
namespace Detail {
// Internal shared converter hooks. The prepared buffer also builds the target.
void preflight(const ON_Curve&, Options, std::span<const double>);
Report observe(const ON_Curve&, const ON_NurbsCurve&, const Handle(Geom_Curve)&,
               int nativeForm, std::array<double,2> getNurbFormDomain,
               bool beforeClampPeriodic, Options, std::span<const double>);
}
}
