// SPDX-License-Identifier: LGPL-2.1-or-later
#include "NurbsBasisAdapter.h"
#include <cstdint>
#include <limits>
#include <string>
#include <type_traits>

namespace OpenMatrix9Gui::ThreeDm::BasisContract {
namespace {
// Mirrors rust/src/nurbs_basis_ffi.rs #[repr(C)] by-value records. Rust borrows
// only initialized, aligned, immutable vector storage for the duration of one
// synchronous call. Neither side retains pointers or frees the other's storage.
// Native objects and exceptions never cross this ABI. Independent snapshots
// may be validated concurrently; reading a mutating native source is forbidden.
struct Limits {
    std::size_t max_degree, max_poles, max_knots, max_bytes;
    double max_coordinate;
};
struct NativeAxis {
    std::size_t degree;
    const double* knots;
    std::size_t knot_count;
    const std::size_t* multiplicities;
    std::size_t multiplicity_count;
    std::uint32_t periodic;
    double domain[2];
};
struct NativeCurve {
    const std::array<double, 3>* poles;
    std::size_t pole_count;
    const double* weights;
    std::size_t weight_count;
    NativeAxis axis;
};
struct NativeSurface {
    const std::array<double, 3>* poles;
    std::size_t pole_count;
    const double* weights;
    std::size_t weight_count, u_count, v_count;
    NativeAxis axes[2];
};
struct SurfaceCounts {
    std::size_t degrees[2], u_count, v_count, pole_count, weight_count, knot_counts[2];
};
extern "C" std::uint32_t om9_basis_curve_counts(std::size_t, std::size_t, std::size_t, std::size_t, Limits);
extern "C" std::uint32_t om9_basis_surface_counts(SurfaceCounts, Limits);
extern "C" std::uint32_t om9_basis_curve_validate(NativeCurve, Limits);
extern "C" std::uint32_t om9_basis_surface_validate(NativeSurface, Limits);
extern "C" Limits om9_basis_native_limits();

// Current supported native host/ABI is 64-bit. A new target must review both
// Rust and C++ layouts rather than silently relying on a different packing ABI.
static_assert(sizeof(std::size_t) == 8 && sizeof(void*) == 8 && sizeof(double) == 8);
static_assert(sizeof(std::array<double, 3>) == 24 && alignof(std::array<double, 3>) == 8);
static_assert(std::is_standard_layout_v<Limits> && sizeof(Limits) == 40 && alignof(Limits) == 8);
static_assert(offsetof(Limits, max_degree) == 0 && offsetof(Limits, max_poles) == 8 &&
              offsetof(Limits, max_knots) == 16 && offsetof(Limits, max_bytes) == 24 &&
              offsetof(Limits, max_coordinate) == 32);
static_assert(std::is_standard_layout_v<NativeAxis> && sizeof(NativeAxis) == 64 && alignof(NativeAxis) == 8);
static_assert(offsetof(NativeAxis, degree) == 0 && offsetof(NativeAxis, knots) == 8 &&
              offsetof(NativeAxis, knot_count) == 16 && offsetof(NativeAxis, multiplicities) == 24 &&
              offsetof(NativeAxis, multiplicity_count) == 32 && offsetof(NativeAxis, periodic) == 40 &&
              offsetof(NativeAxis, domain) == 48);
static_assert(std::is_standard_layout_v<NativeCurve> && sizeof(NativeCurve) == 96 && alignof(NativeCurve) == 8);
static_assert(offsetof(NativeCurve, poles) == 0 && offsetof(NativeCurve, pole_count) == 8 &&
              offsetof(NativeCurve, weights) == 16 && offsetof(NativeCurve, weight_count) == 24 &&
              offsetof(NativeCurve, axis) == 32);
static_assert(std::is_standard_layout_v<NativeSurface> && sizeof(NativeSurface) == 176 && alignof(NativeSurface) == 8);
static_assert(offsetof(NativeSurface, poles) == 0 && offsetof(NativeSurface, pole_count) == 8 &&
              offsetof(NativeSurface, weights) == 16 && offsetof(NativeSurface, weight_count) == 24 &&
              offsetof(NativeSurface, u_count) == 32 && offsetof(NativeSurface, v_count) == 40 &&
              offsetof(NativeSurface, axes) == 48);
static_assert(std::is_standard_layout_v<SurfaceCounts> && sizeof(SurfaceCounts) == 64 && alignof(SurfaceCounts) == 8);
static_assert(offsetof(SurfaceCounts, degrees) == 0 && offsetof(SurfaceCounts, u_count) == 16 &&
              offsetof(SurfaceCounts, v_count) == 24 && offsetof(SurfaceCounts, pole_count) == 32 &&
              offsetof(SurfaceCounts, weight_count) == 40 && offsetof(SurfaceCounts, knot_counts) == 48);

const Limits nativeLimits=om9_basis_native_limits();
void requireStatus(std::uint32_t status)
{
    if (status != 0) throw ExchangeError("Native NURBS basis rejected by Rust contract (status " +
                                         std::to_string(status) + ")");
}
std::size_t nativeCount(int count)
{
    return count > 0 ? static_cast<std::size_t>(count) : 0;
}
std::size_t onDegree(int order)
{
    // Do not call Degree() or KnotCount() with malformed signed native fields.
    return order >= 2 ? static_cast<std::size_t>(order) - 1 : 0;
}
void nativeStorage(bool condition)
{
    if (!condition) throw ExchangeError("Invalid native NURBS storage envelope");
}
void nativeAddress(const double* pointer, std::size_t count)
{
    const auto address = reinterpret_cast<std::uintptr_t>(pointer);
    nativeStorage(pointer && address % alignof(double) == 0 &&
                  count <= static_cast<std::size_t>(std::numeric_limits<std::ptrdiff_t>::max()) / sizeof(double));
    nativeStorage(address <= std::numeric_limits<std::uintptr_t>::max() - count * sizeof(double));
    // These checks reject malformed envelopes, not dangling/forged pointers.
    // The native owner still establishes allocation lifetime and readability.
}
void cvEnvelope(const double* pointer, int capacity, std::size_t lastOffset, std::size_t cvSize)
{
    const auto size = lastOffset + cvSize;
    nativeStorage(pointer && capacity >= 0 && size <= static_cast<std::size_t>(std::numeric_limits<int>::max()));
    nativeStorage(capacity == 0 || size <= static_cast<std::size_t>(capacity));
    nativeAddress(pointer, size);
    // Capacity zero is the SDK's externally managed-buffer convention. Native
    // caller guarantees live storage; an address/count check cannot prove it.
}
void knotEnvelope(const double* pointer, int capacity, std::size_t count)
{
    nativeStorage(pointer && capacity >= 0 && count <= static_cast<std::size_t>(std::numeric_limits<int>::max()));
    nativeStorage(capacity == 0 || count <= static_cast<std::size_t>(capacity));
    nativeAddress(pointer, count);
}
std::size_t distinctKnots(const double* knots, std::size_t count)
{
    // Count/degree and native envelope/validity guards have already bounded this
    // scan. Adding the omitted outer duplicates never changes distinct count.
    std::size_t distinct = 1;
    for (std::size_t i = 1; i < count; ++i) distinct += knots[i] != knots[i - 1];
    return distinct;
}
struct OnCurveCounts { std::size_t degree, poles, weights, storedKnots, knots; };
OnCurveCounts counts(const ON_NurbsCurve& curve)
{
    OnCurveCounts c{onDegree(curve.m_order), nativeCount(curve.m_cv_count),
                    curve.m_is_rat ? nativeCount(curve.m_cv_count) : 0, 0, 0};
    // Minimal axes bound degree/poles before native validity reads or loops.
    // Exact distinct-knot counts and logical bytes are guarded before buffers.
    requireStatus(om9_basis_curve_counts(c.degree, c.poles, c.weights, 2, nativeLimits));
    nativeStorage(curve.m_dim >= 1 && curve.m_dim <= 3 && (curve.m_is_rat == 0 || curve.m_is_rat == 1));
    const auto cvSize = static_cast<std::size_t>(curve.m_dim + curve.m_is_rat);
    nativeStorage(curve.m_cv_stride >= static_cast<int>(cvSize));
    c.storedKnots = c.degree + c.poles - 1;
    cvEnvelope(curve.m_cv, curve.CVCapacity(), (c.poles - 1) * nativeCount(curve.m_cv_stride), cvSize);
    knotEnvelope(curve.m_knot, curve.KnotCapacity(), c.storedKnots);
    if (!curve.IsValid()) throw ExchangeError("Invalid native NURBS curve");
    c.knots = distinctKnots(curve.m_knot, c.storedKnots);
    requireStatus(om9_basis_curve_counts(c.degree, c.poles, c.weights, c.knots, nativeLimits));
    return c;
}
struct OnSurfaceCounts { SurfaceCounts ffi; std::size_t storedKnots[2]; };
OnSurfaceCounts counts(const ON_NurbsSurface& surface)
{
    OnSurfaceCounts c{};
    c.ffi.degrees[0] = onDegree(surface.m_order[0]); c.ffi.degrees[1] = onDegree(surface.m_order[1]);
    c.ffi.u_count = nativeCount(surface.m_cv_count[0]); c.ffi.v_count = nativeCount(surface.m_cv_count[1]);
    // Positive int dimensions fit this checked 64-bit ABI product; Rust enforces
    // the grid product and total-pole bound before any native arithmetic/loops.
    c.ffi.pole_count = c.ffi.u_count * c.ffi.v_count;
    c.ffi.weight_count = surface.m_is_rat ? c.ffi.pole_count : 0;
    c.ffi.knot_counts[0] = c.ffi.knot_counts[1] = 2;
    requireStatus(om9_basis_surface_counts(c.ffi, nativeLimits));
    nativeStorage(surface.m_dim >= 1 && surface.m_dim <= 3 && (surface.m_is_rat == 0 || surface.m_is_rat == 1));
    const auto cvSize = static_cast<std::size_t>(surface.m_dim + surface.m_is_rat);
    nativeStorage(surface.m_cv_stride[0] >= static_cast<int>(cvSize) && surface.m_cv_stride[1] >= static_cast<int>(cvSize));
    cvEnvelope(surface.m_cv, surface.m_cv_capacity,
        (c.ffi.u_count - 1) * nativeCount(surface.m_cv_stride[0]) +
        (c.ffi.v_count - 1) * nativeCount(surface.m_cv_stride[1]), cvSize);
    for (int d = 0; d < 2; ++d) {
        const auto poles = d == 0 ? c.ffi.u_count : c.ffi.v_count;
        c.storedKnots[d] = c.ffi.degrees[d] + poles - 1;
        knotEnvelope(surface.m_knot[d], surface.m_knot_capacity[d], c.storedKnots[d]);
    }
    if (!surface.IsValid()) throw ExchangeError("Invalid native NURBS surface");
    for (int d = 0; d < 2; ++d) c.ffi.knot_counts[d] = distinctKnots(surface.m_knot[d], c.storedKnots[d]);
    requireStatus(om9_basis_surface_counts(c.ffi, nativeLimits));
    return c;
}
Axis onAxis(const double* knots, std::size_t count, std::size_t distinct, std::size_t degree, ON_Interval domain)
{
    Axis axis;
    axis.degree = degree; axis.domain = {domain[0], domain[1]};
    axis.knots.reserve(distinct); axis.multiplicities.reserve(distinct);
    auto append = [&](double knot) {
        if (axis.knots.empty() || knot != axis.knots.back()) {
            axis.knots.push_back(knot); axis.multiplicities.push_back(1);
        } else ++axis.multiplicities.back();
    };
    append(knots[0]);
    for (std::size_t i = 0; i < count; ++i) append(knots[i]);
    append(knots[count - 1]);
    return axis;
}
NativeAxis borrowed(const Axis& axis)
{
    return {axis.degree, axis.knots.data(), axis.knots.size(), axis.multiplicities.data(),
            axis.multiplicities.size(), axis.periodic ? 1u : 0u, {axis.domain[0], axis.domain[1]}};
}
void validate(const CurveBasisData& basis)
{
    requireStatus(om9_basis_curve_validate({basis.poles.data(), basis.poles.size(),
        basis.weights.empty() ? nullptr : basis.weights.data(), basis.weights.size(), borrowed(basis.axis)}, nativeLimits));
}
void validate(const SurfaceBasisData& basis)
{
    requireStatus(om9_basis_surface_validate({basis.poles.data(), basis.poles.size(),
        basis.weights.empty() ? nullptr : basis.weights.data(), basis.weights.size(), basis.uCount, basis.vCount,
        {borrowed(basis.axes[0]), borrowed(basis.axes[1])}}, nativeLimits));
}
template<class Curve> OnCurveCounts counts(const Curve& curve)
{
    if (curve.IsNull()) throw ExchangeError("Null native NURBS curve");
    OnCurveCounts c{nativeCount(curve->Degree()), nativeCount(curve->NbPoles()),
                   curve->IsRational() ? nativeCount(curve->NbPoles()) : 0, 0, nativeCount(curve->NbKnots())};
    requireStatus(om9_basis_curve_counts(c.degree, c.poles, c.weights, c.knots, nativeLimits));
    return c;
}
SurfaceCounts counts(const Handle(Geom_BSplineSurface)& surface)
{
    if (surface.IsNull()) throw ExchangeError("Null native NURBS surface");
    SurfaceCounts c{{nativeCount(surface->UDegree()), nativeCount(surface->VDegree())},
                    nativeCount(surface->NbUPoles()), nativeCount(surface->NbVPoles()), 0, 0,
                    {nativeCount(surface->NbUKnots()), nativeCount(surface->NbVKnots())}};
    c.pole_count = c.u_count * c.v_count;
    c.weight_count = surface->IsURational() || surface->IsVRational() ? c.pole_count : 0;
    requireStatus(om9_basis_surface_counts(c, nativeLimits));
    return c;
}
template<class Curve> CurveBasisData occtCurve(const Curve& curve)
{
    const auto c = counts(curve);
    CurveBasisData basis;
    basis.axis.degree = c.degree; basis.axis.periodic = curve->IsPeriodic();
    basis.axis.domain = {curve->FirstParameter(), curve->LastParameter()};
    basis.poles.reserve(c.poles); basis.weights.reserve(c.weights);
    basis.axis.knots.reserve(c.knots); basis.axis.multiplicities.reserve(c.knots);
    for (int i = 1; i <= curve->NbPoles(); ++i) {
        const auto point = curve->Pole(i);
        double z = 0;
        if constexpr (requires { point.Z(); }) z = point.Z();
        basis.poles.push_back({point.X(), point.Y(), z});
        if (c.weights) basis.weights.push_back(curve->Weight(i));
    }
    for (int i = 1; i <= curve->NbKnots(); ++i) {
        basis.axis.knots.push_back(curve->Knot(i));
        basis.axis.multiplicities.push_back(nativeCount(curve->Multiplicity(i)));
    }
    validate(basis);
    return basis;
}
}

void preflightBasis(const ON_NurbsCurve& curve) { (void)counts(curve); }
void preflightBasis(const ON_NurbsSurface& surface) { (void)counts(surface); }
void preflightBasis(const Handle(Geom_BSplineCurve)& curve) { (void)counts(curve); }
void preflightBasis(const Handle(Geom2d_BSplineCurve)& curve) { (void)counts(curve); }
void preflightBasis(const Handle(Geom_BSplineSurface)& surface) { (void)counts(surface); }
CurveBasisData checkedBasis(const ON_NurbsCurve& curve)
{
    const auto c = counts(curve);
    CurveBasisData basis;
    basis.axis = onAxis(curve.m_knot, c.storedKnots, c.knots, c.degree, curve.Domain());
    basis.poles.reserve(c.poles); basis.weights.reserve(c.weights);
    for (int i = 0; i < curve.m_cv_count; ++i) {
        ON_4dPoint point;
        if (!curve.GetCV(i, point)) throw ExchangeError("Cannot read native NURBS curve pole");
        basis.poles.push_back({point.x / point.w, point.y / point.w, point.z / point.w});
        if (c.weights) basis.weights.push_back(point.w);
    }
    validate(basis);
    return basis;
}
SurfaceBasisData checkedBasis(const ON_NurbsSurface& surface)
{
    const auto c = counts(surface);
    SurfaceBasisData basis;
    basis.uCount = c.ffi.u_count; basis.vCount = c.ffi.v_count;
    for (int d = 0; d < 2; ++d)
        basis.axes[d] = onAxis(surface.m_knot[d], c.storedKnots[d], c.ffi.knot_counts[d],
                               c.ffi.degrees[d], surface.Domain(d));
    basis.poles.reserve(c.ffi.pole_count); basis.weights.reserve(c.ffi.weight_count);
    for (int u = 0; u < surface.m_cv_count[0]; ++u)
        for (int v = 0; v < surface.m_cv_count[1]; ++v) {
            ON_4dPoint point;
            if (!surface.GetCV(u, v, point)) throw ExchangeError("Cannot read native NURBS surface pole");
            basis.poles.push_back({point.x / point.w, point.y / point.w, point.z / point.w});
            if (c.ffi.weight_count) basis.weights.push_back(point.w);
        }
    validate(basis);
    return basis;
}
CurveBasisData checkedBasis(const Handle(Geom_BSplineCurve)& curve) { return occtCurve(curve); }
CurveBasisData checkedBasis(const Handle(Geom2d_BSplineCurve)& curve) { return occtCurve(curve); }
SurfaceBasisData checkedBasis(const Handle(Geom_BSplineSurface)& surface)
{
    const auto c = counts(surface);
    SurfaceBasisData basis;
    basis.uCount = c.u_count; basis.vCount = c.v_count;
    double u0, u1, v0, v1;
    surface->Bounds(u0, u1, v0, v1);
    basis.axes[0].degree = c.degrees[0]; basis.axes[0].periodic = surface->IsUPeriodic(); basis.axes[0].domain = {u0, u1};
    basis.axes[1].degree = c.degrees[1]; basis.axes[1].periodic = surface->IsVPeriodic(); basis.axes[1].domain = {v0, v1};
    basis.poles.reserve(c.pole_count); basis.weights.reserve(c.weight_count);
    for (int u = 1; u <= surface->NbUPoles(); ++u)
        for (int v = 1; v <= surface->NbVPoles(); ++v) {
            const auto point = surface->Pole(u, v);
            basis.poles.push_back({point.X(), point.Y(), point.Z()});
            if (c.weight_count) basis.weights.push_back(surface->Weight(u, v));
        }
    for (int d = 0; d < 2; ++d) {
        basis.axes[d].knots.reserve(c.knot_counts[d]); basis.axes[d].multiplicities.reserve(c.knot_counts[d]);
        for (int i = 1; i <= static_cast<int>(c.knot_counts[d]); ++i) {
            basis.axes[d].knots.push_back(d == 0 ? surface->UKnot(i) : surface->VKnot(i));
            basis.axes[d].multiplicities.push_back(nativeCount(d == 0 ? surface->UMultiplicity(i) : surface->VMultiplicity(i)));
        }
    }
    validate(basis);
    return basis;
}
}
