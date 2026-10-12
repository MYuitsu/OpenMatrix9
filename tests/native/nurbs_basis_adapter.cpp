// SPDX-License-Identifier: LGPL-2.1-or-later
// Independent native inputs for the Rust basis gate; no fixture-oracle rewrites.
#include "NurbsBasisAdapter.h"
#include "ThreeDmGeometry.h"
#include <TColgp_Array1OfPnt.hxx>
#include <TColgp_Array1OfPnt2d.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <TColStd_Array2OfReal.hxx>
#include <cmath>
#include <cstdint>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>

using namespace OpenMatrix9Gui::ThreeDm;
namespace Basis = OpenMatrix9Gui::ThreeDm::BasisContract;
namespace {
void require(bool condition, const char* message)
{
    if (!condition) throw std::runtime_error(message);
}
template<class Function> void rejects(const char* message, Function action)
{
    bool rejected = false;
    try { action(); } catch (const ExchangeError&) { rejected = true; }
    require(rejected, message);
}
// Every temporary ownership/pointer change is restored during normal return
// and exception unwinding, before the SDK owner can destroy its allocation.
template<class T> class NativeFieldRestore {
public:
    explicit NativeFieldRestore(T& field) : field_(field), original_(field) {}
    ~NativeFieldRestore() noexcept { field_ = original_; }
    NativeFieldRestore(const NativeFieldRestore&) = delete;
    NativeFieldRestore& operator=(const NativeFieldRestore&) = delete;
private:
    T& field_;
    T original_;
};
class CurveKnotOwnershipRestore {
public:
    explicit CurveKnotOwnershipRestore(ON_NurbsCurve& curve) : curve_(curve)
    {
        curve_.UnmanageKnotForExperts(capacity_, pointer_);
        curve_.ManageKnotForExperts(0, pointer_);
    }
    ~CurveKnotOwnershipRestore() noexcept { curve_.ManageKnotForExperts(capacity_, pointer_); }
    CurveKnotOwnershipRestore(const CurveKnotOwnershipRestore&) = delete;
    CurveKnotOwnershipRestore& operator=(const CurveKnotOwnershipRestore&) = delete;
private:
    ON_NurbsCurve& curve_;
    int capacity_ = 0;
    double* pointer_ = nullptr;
};
ON_NurbsCurve quadratic()
{
    ON_NurbsCurve curve(3, true, 3, 3);
    const double weight = std::sqrt(0.5);
    require(curve.SetCV(0, ON_4dPoint(10, 0, 0, 1)), "quarter CV0");
    require(curve.SetCV(1, ON_4dPoint(10 * weight, 10 * weight, 0, weight)), "quarter CV1");
    require(curve.SetCV(2, ON_4dPoint(0, 10, 0, 1)), "quarter CV2");
    for (int i = 0; i < 4; ++i) require(curve.SetKnot(i, i < 2 ? 0 : 1), "quarter knots");
    return curve;
}
ON_NurbsSurface plane(int uCount = 2, int vCount = 2, bool rational = true)
{
    ON_NurbsSurface surface(3, rational, 2, 2, uCount, vCount);
    for (int u = 0; u < uCount; ++u)
        for (int v = 0; v < vCount; ++v)
            require(surface.SetCV(u, v, ON_4dPoint(u, v, 0, 1)), "surface CV");
    for (int u = 0; u < uCount; ++u) require(surface.SetKnot(0, u, u), "surface U knots");
    for (int v = 0; v < vCount; ++v) require(surface.SetKnot(1, v, v), "surface V knots");
    return surface;
}
void nativeEnvelopesAndEntryPoints()
{
    auto curve = quadratic();
    const auto curveCRC = curve.DataCRC(0);
    const auto curveCvCapacity = curve.m_cv_capacity;
    const auto curveKnotCapacity = curve.KnotCapacity();
    {
        NativeFieldRestore cvCapacity(curve.m_cv_capacity);
        CurveKnotOwnershipRestore knots(curve);
        curve.m_cv_capacity = 0;
        // The original allocations remain live. Zero capacities declare the
        // SDK's external ownership convention; no dangling pointer is tested.
        const auto basis = Basis::checkedBasis(curve);
        require(basis.poles.size() == 3 && basis.weights.size() == 3,
                "live external ON curve CV/knot buffers accepted");
    }
    require(curve.m_cv_capacity == curveCvCapacity && curve.KnotCapacity() == curveKnotCapacity &&
            curve.DataCRC(0) == curveCRC, "curve external ownership restored before destruction");
    auto surface = plane();
    const auto surfaceCRC = surface.DataCRC(0);
    const auto surfaceCvCapacity = surface.m_cv_capacity;
    const auto uKnotCapacity = surface.m_knot_capacity[0], vKnotCapacity = surface.m_knot_capacity[1];
    {
        NativeFieldRestore cvCapacity(surface.m_cv_capacity);
        NativeFieldRestore uCapacity(surface.m_knot_capacity[0]);
        NativeFieldRestore vCapacity(surface.m_knot_capacity[1]);
        surface.m_cv_capacity = surface.m_knot_capacity[0] = surface.m_knot_capacity[1] = 0;
        require(Basis::checkedBasis(surface).poles.size() == 4,
                "live external ON surface CV/knot buffers accepted");
    }
    require(surface.m_cv_capacity == surfaceCvCapacity && surface.m_knot_capacity[0] == uKnotCapacity &&
            surface.m_knot_capacity[1] == vKnotCapacity && surface.DataCRC(0) == surfaceCRC,
            "surface external ownership restored before destruction");

    // Invalid numeric addresses are only submitted to the envelope gate. The
    // null, alignment and wrapping-span guards must reject before IsValid reads
    // any storage; these cases make no allocation-liveness or UAF certificate.
    const auto wrappingAddress = std::numeric_limits<std::uintptr_t>::max() - (alignof(double) - 1);
    const std::array<double*, 3> curvePointers{nullptr,
        reinterpret_cast<double*>(reinterpret_cast<std::uintptr_t>(curve.m_cv) + 1),
        reinterpret_cast<double*>(wrappingAddress)};
    const std::array<double*, 3> surfacePointers{nullptr,
        reinterpret_cast<double*>(reinterpret_cast<std::uintptr_t>(surface.m_cv) + 1),
        reinterpret_cast<double*>(wrappingAddress)};
    for (auto pointer : curvePointers) {
        NativeFieldRestore cvPointer(curve.m_cv);
        curve.m_cv = pointer;
        rejects("ON curve null/misaligned/wrapping CV envelope rejected", [&] { Basis::checkedBasis(curve); });
        rejects("production curve3d rejects malformed CV before native reads", [&] { curve3d(curve); });
    }
    for (auto pointer : surfacePointers) {
        NativeFieldRestore cvPointer(surface.m_cv);
        surface.m_cv = pointer;
        rejects("ON surface null/misaligned/wrapping CV envelope rejected", [&] { Basis::checkedBasis(surface); });
        rejects("production importSurface rejects malformed CV before native reads", [&] { importSurface(surface); });
    }
    const std::array<double*, 3> curveKnots{nullptr,
        reinterpret_cast<double*>(reinterpret_cast<std::uintptr_t>(curve.m_knot) + 1),
        reinterpret_cast<double*>(wrappingAddress)};
    const std::array<double*, 3> surfaceKnots{nullptr,
        reinterpret_cast<double*>(reinterpret_cast<std::uintptr_t>(surface.m_knot[0]) + 1),
        reinterpret_cast<double*>(wrappingAddress)};
    for (auto pointer : curveKnots) {
        NativeFieldRestore knotPointer(curve.m_knot);
        curve.m_knot = pointer;
        rejects("ON curve null/misaligned/wrapping knot envelope rejected", [&] { Basis::checkedBasis(curve); });
    }
    for (auto pointer : surfaceKnots) {
        NativeFieldRestore knotPointer(surface.m_knot[0]);
        surface.m_knot[0] = pointer;
        rejects("ON surface null/misaligned/wrapping knot envelope rejected", [&] { Basis::checkedBasis(surface); });
    }
    {
        NativeFieldRestore count(curve.m_cv_count);
        curve.m_cv_count = 1'000'001;
        rejects("production curve3d count policy runs before GetNurbForm", [&] { curve3d(curve); });
    }
    require(curve.DataCRC(0) == curveCRC && surface.DataCRC(0) == surfaceCRC,
            "malformed envelope tests restore original source storage");

    ON_RevSurface revolution;
    auto* profile = new ON_NurbsCurve(3, false, 2, 2);
    revolution.m_curve = profile; // The revolution owns and deletes the profile.
    require(profile->SetCV(0, ON_3dPoint(2, 0, 0)) && profile->SetCV(1, ON_3dPoint(2, 0, 1)) &&
            profile->SetKnot(0, 0) && profile->SetKnot(1, 1), "independent revolution profile setup");
    revolution.m_axis = ON_Line(ON_3dPoint(0, 0, 0), ON_3dPoint(0, 0, 1));
    revolution.m_angle = ON_Interval(0, 2 * ON_PI);
    revolution.m_t = revolution.m_angle;
    revolution.m_bTransposed = false;
    require(revolution.IsValid() && !importSurface(revolution).IsNull(), "independent revolution baseline is valid");
    const auto profileCRC = profile->DataCRC(0);
    {
        NativeFieldRestore count(profile->m_cv_count);
        profile->m_cv_count = 1'000'001;
        rejects("revolution profile count gated before revolution IsValid/GetNurbForm",
                [&] { importSurface(revolution); });
    }
    {
        NativeFieldRestore pointer(profile->m_cv);
        profile->m_cv = nullptr;
        rejects("revolution null profile CV gated before revolution IsValid/GetNurbForm",
                [&] { importSurface(revolution); });
    }
    require(profile->DataCRC(0) == profileCRC && revolution.IsValid(),
            "revolution profile corruption restored before owned destruction");
}
void onInputs()
{
    auto quarter = quadratic();
    const auto curveBefore = quarter.DataCRC(0);
    const auto basis = Basis::checkedBasis(quarter);
    require(basis.axis.degree == 2 && basis.poles.size() == 3, "quarter degree/poles");
    require(basis.axis.knots == std::vector<double>{0, 1}, "restore omitted outer knots");
    require(basis.axis.multiplicities == std::vector<std::size_t>{3, 3}, "quarter multiplicities");
    require(std::abs(basis.weights[1] - std::sqrt(0.5)) < 1e-15, "rational weight retained");
    require(std::abs(basis.poles[1][0] - 10) < 1e-14 && std::abs(basis.poles[1][1] - 10) < 1e-14 &&
            basis.poles[1][2] == 0, "Euclidean poles retained");
    require(quarter.DataCRC(0) == curveBefore, "ON curve source unchanged");

    for (double weight : {0.0, -1.0, std::numeric_limits<double>::quiet_NaN()}) {
        auto invalid = quadratic();
        invalid.m_cv[invalid.m_cv_stride + invalid.m_dim] = weight;
        rejects("malformed ON curve weight", [&] { Basis::checkedBasis(invalid); });
    }
    auto badPole = quadratic();
    badPole.m_cv[0] = std::numeric_limits<double>::quiet_NaN();
    rejects("ON curve NaN pole", [&] { Basis::checkedBasis(badPole); });
    auto badKnot = quadratic();
    badKnot.m_knot[1] = std::numeric_limits<double>::infinity();
    rejects("ON curve infinite knot", [&] { Basis::checkedBasis(badKnot); });
    auto tooLarge = quadratic();
    tooLarge.m_cv_count = 1'000'001;
    rejects("count policy before ON buffer access", [&] { Basis::checkedBasis(tooLarge); });
    tooLarge.m_cv_count = 3;
    auto shortBuffer = quadratic();
    shortBuffer.m_cv_count = 20;
    rejects("ON owned capacity before native validity", [&] { Basis::checkedBasis(shortBuffer); });
    shortBuffer.m_cv_count = 3;
    auto badStride = quadratic();
    badStride.m_cv_stride = 1;
    rejects("ON curve native stride", [&] { Basis::checkedBasis(badStride); });
    badStride.m_cv_stride = 4;
    auto badOrder = quadratic();
    badOrder.m_order = std::numeric_limits<int>::min();
    rejects("malformed degree before native arithmetic", [&] { Basis::checkedBasis(badOrder); });
    badOrder.m_order = 3;
    ON_NurbsCurve degree26(3, false, 27, 27);
    for (int i = 0; i < 27; ++i) degree26.SetCV(i, ON_3dPoint(i, 0, 0));
    for (int i = 0; i < degree26.KnotCount(); ++i) degree26.SetKnot(i, i < 26 ? 0 : 1);
    require(degree26.IsValid(), "independent degree26 native curve is valid");
    rejects("explicit native degree25 policy", [&] { Basis::checkedBasis(degree26); });
    ON_NurbsCurve degree25(3, false, 26, 26);
    for (int i = 0; i < 26; ++i) degree25.SetCV(i, ON_3dPoint(i, 0, 0));
    for (int i = 0; i < degree25.KnotCount(); ++i) degree25.SetKnot(i, i < 25 ? 0 : 1);
    require(Basis::checkedBasis(degree25).axis.degree == 25, "native degree25 boundary accepted");

    const ON_3dPoint periodicPoles[] = {ON_3dPoint(0, 0, 0), ON_3dPoint(1, 2, 0), ON_3dPoint(2, 0, 0)};
    ON_NurbsCurve periodic;
    require(periodic.CreatePeriodicUniformNurbs(3, 3, 3, periodicPoles), "independent ON periodic input");
    require(periodic.IsPeriodic(), "ON source is periodic");
    const auto periodicCRC = periodic.DataCRC(0);
    const auto periodicBasis = Basis::checkedBasis(periodic);
    require(!periodicBasis.axis.periodic && periodicBasis.poles.size() == static_cast<std::size_t>(periodic.CVCount()),
            "ON ghost-pole storage uses expanded structural convention");
    require(periodic.IsPeriodic() && periodic.DataCRC(0) == periodicCRC, "ON periodic source unchanged");

    ON_NurbsCurve large(3, false, 2, 300);
    for (int i = 0; i < 300; ++i) { large.SetCV(i, ON_3dPoint(i, 0, 0)); large.SetKnot(i, i); }
    require(large.IsValid(), "large independently authored native curve");
    const auto largeBasis = Basis::checkedBasis(large);
    require(largeBasis.poles.size() == 300 && largeBasis.weights.empty(), "native policy exceeds generated256");
    require(largeBasis.axis.multiplicities.front() == 2 && largeBasis.axis.multiplicities.back() == 2,
            "large curve outer multiplicities");

    auto surface = plane();
    const auto surfaceBefore = surface.DataCRC(0);
    const auto surfaceBasis = Basis::checkedBasis(surface);
    require(surfaceBasis.uCount == 2 && surfaceBasis.vCount == 2, "surface dimensions");
    require(surfaceBasis.poles[1] == std::array<double, 3>{0, 1, 0} &&
            surfaceBasis.poles[2] == std::array<double, 3>{1, 0, 0}, "surface U-major snapshot");
    require(surface.DataCRC(0) == surfaceBefore, "ON surface source unchanged");
    for (double weight : {0.0, -1.0, std::numeric_limits<double>::quiet_NaN()}) {
        auto invalid = plane();
        invalid.m_cv[invalid.m_dim] = weight;
        rejects("malformed independently authored ON surface weight", [&] { Basis::checkedBasis(invalid); });
    }
    auto badSurfaceKnot = plane();
    badSurfaceKnot.m_knot[1][1] = std::numeric_limits<double>::quiet_NaN();
    rejects("ON surface NaN knot", [&] { Basis::checkedBasis(badSurfaceKnot); });
    auto badSurfacePole = plane();
    badSurfacePole.m_cv[0] = std::numeric_limits<double>::infinity();
    rejects("ON surface infinite pole", [&] { Basis::checkedBasis(badSurfacePole); });
    auto oversizedGrid = plane();
    oversizedGrid.m_cv_count[0] = 1'000'001;
    rejects("surface product policy before native buffer access", [&] { Basis::checkedBasis(oversizedGrid); });
    oversizedGrid.m_cv_count[0] = 2;
    auto shortSurfaceBuffer = plane();
    shortSurfaceBuffer.m_cv_count[0] = 20;
    rejects("ON surface owned capacity before native validity", [&] { Basis::checkedBasis(shortSurfaceBuffer); });
    shortSurfaceBuffer.m_cv_count[0] = 2;
    auto largeSurface = plane(20, 20, false);
    const auto largeSurfaceBasis = Basis::checkedBasis(largeSurface);
    require(largeSurfaceBasis.poles.size() == 400 && largeSurfaceBasis.weights.empty(), "native surface exceeds generated256");
}
void occtInputs()
{
    TColgp_Array1OfPnt poles(1, 3);
    poles.SetValue(1, gp_Pnt(0, 0, 0)); poles.SetValue(2, gp_Pnt(1, 2, 0)); poles.SetValue(3, gp_Pnt(2, 0, 0));
    TColStd_Array1OfReal knots(1, 2), weights(1, 3);
    TColStd_Array1OfInteger mults(1, 2);
    knots.SetValue(1, 0); knots.SetValue(2, 1); mults.Init(3); weights.Init(1); weights.SetValue(2, 0.5);
    Handle(Geom_BSplineCurve) curve = new Geom_BSplineCurve(poles, weights, knots, mults, 2);
    auto sibling = curve;
    const auto before = Basis::checkedBasis(curve);
    const auto after = Basis::checkedBasis(sibling);
    require(before.poles == after.poles && before.weights == after.weights &&
            before.axis.knots == after.axis.knots && before.axis.multiplicities == after.axis.multiplicities,
            "OCCT shared source basis unchanged");
    require(before.weights[1] == 0.5, "OCCT rational extraction");
    curve->SetPole(1, gp_Pnt(1e150, 0, 0));
    require(Basis::checkedBasis(curve).poles[0][0] == 1e150, "native coordinate boundary accepted");
    curve->SetPole(1, gp_Pnt(1e151, 0, 0));
    rejects("Rust native coordinate limit", [&] { Basis::checkedBasis(curve); });
    curve->SetPole(1, gp_Pnt(1e149, 0, 0), 1e200);
    rejects("Rust homogeneous overflow gate", [&] { Basis::checkedBasis(curve); });
    Handle(Geom_BSplineCurve) nullCurve;
    rejects("null OCCT curve", [&] { Basis::checkedBasis(nullCurve); });

    TColgp_Array1OfPnt2d poles2d(1, 3);
    poles2d.SetValue(1, gp_Pnt2d(0, 0)); poles2d.SetValue(2, gp_Pnt2d(1, 2)); poles2d.SetValue(3, gp_Pnt2d(2, 0));
    Handle(Geom2d_BSplineCurve) curve2d = new Geom2d_BSplineCurve(poles2d, weights, knots, mults, 2);
    const auto basis2d = Basis::checkedBasis(curve2d);
    require(basis2d.poles[1] == std::array<double, 3>{1, 2, 0}, "2D basis embeds zero Z");

    TColStd_Array1OfReal periodicKnots(1, 4);
    TColStd_Array1OfInteger periodicMults(1, 4);
    for (int i = 1; i <= 4; ++i) periodicKnots.SetValue(i, i - 1);
    periodicMults.Init(1);
    Handle(Geom_BSplineCurve) periodic = new Geom_BSplineCurve(poles, periodicKnots, periodicMults, 2, true);
    const auto periodicBefore = Basis::checkedBasis(periodic);
    require(periodicBefore.axis.periodic && periodicBefore.axis.multiplicities == std::vector<std::size_t>{1, 1, 1, 1},
            "OCCT compact periodic cardinality");
    Basis::preflightBasis(periodic);
    const auto periodicAfter = Basis::checkedBasis(periodic);
    require(periodic->IsPeriodic() && periodicBefore.poles == periodicAfter.poles &&
            periodicBefore.axis.domain == periodicAfter.axis.domain, "periodic OCCT curve source unchanged");

    TColgp_Array2OfPnt surfacePoles(1, 3, 1, 2);
    for (int u = 1; u <= 3; ++u)
        for (int v = 1; v <= 2; ++v)
            surfacePoles.SetValue(u, v, gp_Pnt(poles.Value(u).X(), poles.Value(u).Y(), v - 1));
    TColStd_Array1OfInteger vMults(1, 2); vMults.Init(2);
    Handle(Geom_BSplineSurface) surface = new Geom_BSplineSurface(surfacePoles, periodicKnots, knots,
        periodicMults, vMults, 2, 1, true, false);
    const auto surfaceBefore = Basis::checkedBasis(surface);
    auto surfaceSibling = surface;
    const auto surfaceAfter = Basis::checkedBasis(surfaceSibling);
    require(surfaceBefore.axes[0].periodic && !surfaceBefore.axes[1].periodic &&
            surfaceBefore.uCount == 3 && surfaceBefore.vCount == 2, "periodic OCCT surface metadata");
    require(surface->IsUPeriodic() && surfaceBefore.poles == surfaceAfter.poles &&
            surfaceBefore.axes[0].domain == surfaceAfter.axes[0].domain,
            "periodic OCCT surface source and sibling unchanged");
}
}
int main()
{
    ON::Begin();
    try {
        onInputs(); occtInputs(); nativeEnvelopesAndEntryPoints();
        std::cout << "RCORE-01 native NURBS basis adapter PASS\n";
        ON::End(); return 0;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n'; ON::End(); return 1;
    }
}
