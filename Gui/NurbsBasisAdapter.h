// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once

#include "ThreeDmGeometry.h"
#include <Geom_BSplineCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <array>
#include <cstddef>
#include <vector>

namespace OpenMatrix9Gui::ThreeDm::BasisContract {
struct Axis {
    std::size_t degree = 0;
    std::vector<double> knots;
    std::vector<std::size_t> multiplicities;
    bool periodic = false;
    std::array<double, 2> domain{};
};
struct CurveBasisData {
    std::vector<std::array<double, 3>> poles;
    // Empty means nonrational; explicit rational weights remain explicit.
    std::vector<double> weights;
    Axis axis;
};
struct SurfaceBasisData {
    // U-major: (u, v) is at u * vCount + v.
    std::vector<std::array<double, 3>> poles;
    std::vector<double> weights;
    std::size_t uCount = 0;
    std::size_t vCount = 0;
    std::array<Axis, 2> axes;
};

// Permanent native exception: only OCCT/openNURBS metadata, storage-envelope
// checks and basis extraction live here; shared numerical/cardinality/budget
// validation is Rust-owned. Sources must be live and not concurrently modified.
// These functions never mutate, copy, clamp or normalize the native source.
// The explicit OM9 native policy is degree <=25, total poles <=1e6, distinct
// knots per axis <=1e6, logical input bytes <=64MiB and |coordinate| <=1e150.
// It excludes native copies, container overhead and aggregate worker budgets.
//
// openNURBS omits the two outer knots. Extraction restores duplicate endpoints
// and validates its stored ghost poles as an expanded nonperiodic axis. That
// structural view does not claim preservation of the source periodic convention.
// OCCT exposes compact periodic axes, whose periodic flag is retained.
//
// preflightBasis bounds counts/native storage before extraction buffers; only
// checkedBasis also validates all values. Re-run checkedBasis after a converter
// changes conventions. Failures are ExchangeError; native exceptions remain on
// the C++ side, never crossing the Rust ABI.
void preflightBasis(const ON_NurbsCurve&);
void preflightBasis(const ON_NurbsSurface&);
void preflightBasis(const Handle(Geom_BSplineCurve)&);
void preflightBasis(const Handle(Geom2d_BSplineCurve)&);
void preflightBasis(const Handle(Geom_BSplineSurface)&);
CurveBasisData checkedBasis(const ON_NurbsCurve&);
SurfaceBasisData checkedBasis(const ON_NurbsSurface&);
CurveBasisData checkedBasis(const Handle(Geom_BSplineCurve)&);
CurveBasisData checkedBasis(const Handle(Geom2d_BSplineCurve)&);
SurfaceBasisData checkedBasis(const Handle(Geom_BSplineSurface)&);
}
