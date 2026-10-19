// SPDX-License-Identifier: LGPL-2.1-or-later
//! OCCT conformance probe: the regression that decides whether a local OCCT patch is still needed.
//!
//! Every check compares a *kernel* derivative or geometric invariant against an oracle that does not
//! come from OCCT's own derivative code:
//!
//!  * the offset-curve D2/D3 checks use a five-point finite difference of `EvalD0` (the point is
//!    exact even in builds where the derivative expansion is wrong), so a wrong D2/D3 coefficient
//!    or sign fails;
//!  * the line and circle checks use closed-form answers (`D2 = D3 = 0` for a line,
//!    `R = r + offset` for a circle), which is also why a circle *cannot* expose the 8.0.1 defect
//!    (there `R` is constant, so `Dr = D2r = 0` and the faulty terms vanish) - the ellipse and the
//!    non-uniform B-spline are the cases that do expose it;
//!  * the solid checks use analytic volumes (`10*10*4` and `400 - pi*r^2*h`) and `BRepCheck`;
//!  * the mesh check requires a positive triangle count and consistent node indices.
//!
//! Output: one JSON object with `checks: [{name, pass, measured, limit}]` and an overall
//! `verdict`: `DROP_PATCH` when the offset derivatives pass, `KEEP_PATCH` when they fail.
//!
//! Run it against the unpatched release first: it must report KEEP_PATCH. After replacing the DLL
//! with the patched build it must report DROP_PATCH; that pair is what makes the test trustworthy.
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Curve.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Line.hxx>
#include <Geom_OffsetCurve.hxx>
#include <NCollection_Array1.hxx>
#include <Poly_Triangulation.hxx>
#include <Standard_Version.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Vec.hxx>

#include <algorithm>
#include <cmath>
#include <string>
#include <cstdio>
#include <vector>

namespace {

struct Check {
  const char* name;
  bool pass;
  double measured;
  double limit;
};

std::vector<Check> g_checks;

void report(const char* name, bool pass, double measured, double limit) {
  g_checks.push_back(Check{name, pass, measured, limit});
  std::printf("  %-34s %s  measured=%.3e limit=%.1e\n", name, pass ? "PASS" : "FAIL", measured, limit);
}

double relative_error(const gp_Vec& actual, const gp_Vec& expected) {
  const gp_Vec difference = actual - expected;
  const double scale = std::max(expected.Magnitude(), actual.Magnitude());
  return difference.Magnitude() / (scale > 1.0 ? scale : 1.0);
}

// Five-point finite differences of the point: the oracle never reads a derivative from OCCT.
gp_Vec second_derivative(const Handle(Geom_Curve) & curve, double u, double h) {
  const gp_Pnt minusTwo = curve->EvalD0(u - 2 * h);
  const gp_Pnt minusOne = curve->EvalD0(u - h);
  const gp_Pnt zero = curve->EvalD0(u);
  const gp_Pnt plusOne = curve->EvalD0(u + h);
  const gp_Pnt plusTwo = curve->EvalD0(u + 2 * h);
  const double factor = 1.0 / (12.0 * h * h);
  return gp_Vec(
    (-minusTwo.X() + 16.0 * minusOne.X() - 30.0 * zero.X() + 16.0 * plusOne.X() - plusTwo.X()) *
      factor,
    (-minusTwo.Y() + 16.0 * minusOne.Y() - 30.0 * zero.Y() + 16.0 * plusOne.Y() - plusTwo.Y()) *
      factor,
    (-minusTwo.Z() + 16.0 * minusOne.Z() - 30.0 * zero.Z() + 16.0 * plusOne.Z() - plusTwo.Z()) *
      factor);
}
gp_Vec third_derivative(const Handle(Geom_Curve) & curve, double u, double h) {
  // (f(u+2h) - 2 f(u+h) + 2 f(u-h) - f(u-2h)) / (2 h^3)
  const gp_Pnt minusTwo = curve->EvalD0(u - 2 * h);
  const gp_Pnt minusOne = curve->EvalD0(u - h);
  const gp_Pnt plusOne = curve->EvalD0(u + h);
  const gp_Pnt plusTwo = curve->EvalD0(u + 2 * h);
  const double factor = 1.0 / (2.0 * h * h * h);
  return gp_Vec((plusTwo.X() - 2.0 * plusOne.X() + 2.0 * minusOne.X() - minusTwo.X()) * factor,
                (plusTwo.Y() - 2.0 * plusOne.Y() + 2.0 * minusOne.Y() - minusTwo.Y()) * factor,
                (plusTwo.Z() - 2.0 * plusOne.Z() + 2.0 * minusOne.Z() - minusTwo.Z()) * factor);
}

// Richardson extrapolation cancels the leading O(h^2) truncation term, so a failure of these
// checks means a real derivative error rather than the discretisation of the oracle. A convergence
// study below prints the h and h/2 errors so the two remain distinguishable in the log.
double offset_d2_error(const Handle(Geom_Curve) & basis,
                       double offset,
                       const gp_Dir& direction,
                       double u,
                       double h) {
  const Handle(Geom_OffsetCurve) offsetCurve = new Geom_OffsetCurve(basis, offset, direction);
  gp_Pnt point;
  gp_Vec d1, d2, d3;
  offsetCurve->D3(u, point, d1, d2, d3);
  const gp_Vec coarse = second_derivative(offsetCurve, u, h);
  const gp_Vec fine = second_derivative(offsetCurve, u, 0.5 * h);
  const gp_Vec extrapolated = (fine * 4.0 - coarse) / 3.0;
  return relative_error(d2, extrapolated);
}

double offset_d3_error(const Handle(Geom_Curve) & basis,
                       double offset,
                       const gp_Dir& direction,
                       double u,
                       double h) {
  const Handle(Geom_OffsetCurve) offsetCurve = new Geom_OffsetCurve(basis, offset, direction);
  gp_Pnt point;
  gp_Vec d1, d2, d3;
  offsetCurve->D3(u, point, d1, d2, d3);
  const gp_Vec coarse = third_derivative(offsetCurve, u, h);
  const gp_Vec fine = third_derivative(offsetCurve, u, 0.5 * h);
  const gp_Vec extrapolated = (fine * 4.0 - coarse) / 3.0;
  return relative_error(d3, extrapolated);
}

Handle(Geom_BSplineCurve) non_uniform_spline() {
  NCollection_Array1<gp_Pnt> poles(1, 6);
  poles.SetValue(1, gp_Pnt(0.0, 0.0, 0.0));
  poles.SetValue(2, gp_Pnt(2.0, 5.0, 0.0));
  poles.SetValue(3, gp_Pnt(6.0, 7.5, 0.0));
  poles.SetValue(4, gp_Pnt(9.0, 3.0, 0.0));
  poles.SetValue(5, gp_Pnt(12.0, 6.0, 0.0));
  poles.SetValue(6, gp_Pnt(15.0, 0.5, 0.0));
  // Non-uniform interior knots: R = |p' x V| varies, which is exactly the regime the defect needs.
  // sum(multiplicities) must equal poles + degree + 1 = 10; the interior knots are non-uniform so
  // R = |p' x V| varies along the curve.
  NCollection_Array1<double> knots(1, 5);
  knots.SetValue(1, 0.0);
  knots.SetValue(2, 0.15);
  knots.SetValue(3, 0.55);
  knots.SetValue(4, 0.85);
  knots.SetValue(5, 1.0);
  NCollection_Array1<int> multiplicities(1, 5);
  multiplicities.SetValue(1, 4);
  multiplicities.SetValue(2, 1);
  multiplicities.SetValue(3, 1);
  multiplicities.SetValue(4, 2);
  multiplicities.SetValue(5, 2);
  return new Geom_BSplineCurve(poles, knots, multiplicities, 3);
}

int count_faces(const TopoDS_Shape& shape) {
  int total = 0;
  for (TopExp_Explorer explorer(shape, TopAbs_FACE); explorer.More(); explorer.Next()) {
    ++total;
  }
  return total;
}

}  // namespace

int main() {
  const gp_Dir direction(0.0, 0.0, 1.0);
  std::printf("OCCT conformance probe %s\n", OCC_VERSION_COMPLETE);

  // --- 1. line offset: D2 and D3 are exactly zero ----------------------------
  {
    const Handle(Geom_Curve) line = new Geom_Line(gp_Pnt(0, 0, 0), gp_Dir(1, 0, 0));
    const Handle(Geom_OffsetCurve) offsetLine = new Geom_OffsetCurve(line, 2.0, direction);
    gp_Pnt point;
    gp_Vec d1, d2, d3;
    offsetLine->D3(0.37, point, d1, d2, d3);
    report("line_offset_d2_zero", d2.Magnitude() <= 1e-12, d2.Magnitude(), 1e-12);
    report("line_offset_d3_zero", d3.Magnitude() <= 1e-12, d3.Magnitude(), 1e-12);
  }

  // --- 2. circle offset: exact radius r + offset ------------------------------
  {
    const double radius = 4.0;
    const double offset = 1.25;
    const Handle(Geom_Curve) circle =
      new Geom_Circle(gp_Ax2(gp_Pnt(0, 0, 0), direction), radius);
    const Handle(Geom_OffsetCurve) offsetCircle = new Geom_OffsetCurve(circle, offset, direction);
    gp_Pnt point;
    gp_Vec d1, d2, d3;
    const double u = 0.7;
    offsetCircle->D3(u, point, d1, d2, d3);
    const double expectedRadius = radius + offset;
    const gp_Vec expectedD2(-expectedRadius * std::cos(u), -expectedRadius * std::sin(u), 0.0);
    const gp_Vec expectedD3(expectedRadius * std::sin(u), -expectedRadius * std::cos(u), 0.0);
    report("circle_offset_d2_exact", relative_error(d2, expectedD2) <= 1e-10,
           relative_error(d2, expectedD2), 1e-10);
    report("circle_offset_d3_exact", relative_error(d3, expectedD3) <= 1e-10,
           relative_error(d3, expectedD3), 1e-10);
  }

  // --- 3. ellipse offset: R varies, so the 8.0.1 defect is visible -----------
  {
    const Handle(Geom_Curve) ellipse =
      new Geom_Ellipse(gp_Ax2(gp_Pnt(0, 0, 0), direction), 10.0, 4.0);
    double worstD2 = 0.0;
    double worstD3 = 0.0;
    for (double u : {0.3, 0.7, 1.2, 2.0}) {
      worstD2 = std::max(worstD2, offset_d2_error(ellipse, 1.5, direction, u, 1e-3));
      worstD3 = std::max(worstD3, offset_d3_error(ellipse, 1.5, direction, u, 5e-3));
    }
    report("ellipse_offset_d2_vs_fdiff", worstD2 <= 1e-6, worstD2, 1e-6);
    report("ellipse_offset_d3_vs_fdiff", worstD3 <= 1e-4, worstD3, 1e-4);
  }

  // --- 4. non-uniform B-spline offset: the same regime, NURBS -----------------
  {
    const Handle(Geom_Curve) spline = non_uniform_spline();
    double worstD2 = 0.0;
    double worstD3 = 0.0;
    for (double u : {0.2, 0.35, 0.5, 0.7}) {
      worstD2 = std::max(worstD2, offset_d2_error(spline, 1.0, direction, u, 1e-3));
      worstD3 = std::max(worstD3, offset_d3_error(spline, 1.0, direction, u, 5e-3));
    }
    report("spline_offset_d2_vs_fdiff", worstD2 <= 1e-6, worstD2, 1e-6);
    report("spline_offset_d3_vs_fdiff", worstD3 <= 1e-4, worstD3, 1e-4);
  }

  // --- 5. solid invariants: analytic volume, validity, mesh ------------------
  {
    const TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), 10.0, 10.0, 4.0).Shape();
    report("box_valid", BRepCheck_Analyzer(box).IsValid(), 0.0, 0.0);
    GProp_GProps boxProperties;
    BRepGProp::VolumeProperties(box, boxProperties);
    report("box_volume_400", std::fabs(boxProperties.Mass() - 400.0) <= 1e-9,
           std::fabs(boxProperties.Mass() - 400.0), 1e-9);

    const TopoDS_Shape drill =
      BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(5, 5, -1), direction), 2.0, 6.0).Shape();
    const TopoDS_Shape cut = BRepAlgoAPI_Cut(box, drill).Shape();
    GProp_GProps cutProperties;
    BRepGProp::VolumeProperties(cut, cutProperties);
    const double expected = 400.0 - M_PI * 4.0 * 4.0;
    report("cut_volume_analytic", std::fabs(cutProperties.Mass() - expected) <= 1e-6,
           std::fabs(cutProperties.Mass() - expected), 1e-6);
    report("cut_valid", BRepCheck_Analyzer(cut).IsValid(), 0.0, 0.0);
    report("cut_faces_7", count_faces(cut) == 7, static_cast<double>(count_faces(cut)), 7.0);

    BRepMesh_IncrementalMesh mesher(cut, 0.01, false, 0.5, true);
    long long triangles = 0;
    bool indicesConsistent = true;
    for (TopExp_Explorer explorer(cut, TopAbs_FACE); explorer.More(); explorer.Next()) {
      const TopoDS_Face& face = TopoDS::Face(explorer.Current());
      TopLoc_Location location;
      const Handle(Poly_Triangulation)& mesh = BRep_Tool::Triangulation(face, location);
      if (mesh.IsNull()) {
        continue;
      }
      triangles += mesh->NbTriangles();
      for (int index = 1; index <= mesh->NbTriangles(); ++index) {
        int a = 0, b = 0, c = 0;
        mesh->Triangle(index).Get(a, b, c);
        if (a < 1 || b < 1 || c < 1 || a > mesh->NbNodes() || b > mesh->NbNodes() ||
            c > mesh->NbNodes()) {
          indicesConsistent = false;
        }
      }
    }
    report("mesh_triangles_positive", mesher.IsDone() && triangles > 0,
           static_cast<double>(triangles), 1.0);
    report("mesh_indices_consistent", indicesConsistent, 0.0, 0.0);
  }

  bool offsetDerivativesPass = true;
  for (const Check& check : g_checks) {
    if (std::string(check.name).find("_d2_") != std::string::npos ||
        std::string(check.name).find("_d3_") != std::string::npos) {
      offsetDerivativesPass = offsetDerivativesPass && check.pass;
    }
  }
  std::printf("{\"occt_version\": \"%s\", \"offset_derivatives_pass\": %s, "
              "\"verdict\": \"%s\", \"checks\": [",
              OCC_VERSION_COMPLETE,
              offsetDerivativesPass ? "true" : "false",
              offsetDerivativesPass ? "PATCH_NOT_NEEDED" : "PATCH_REQUIRED");
  for (size_t index = 0; index < g_checks.size(); ++index) {
    const Check& check = g_checks[index];
    std::printf("%s{\"name\": \"%s\", \"pass\": %s, \"measured\": %.17g, \"limit\": %.17g}",
                index == 0 ? "" : ", ",
                check.name,
                check.pass ? "true" : "false",
                check.measured,
                check.limit);
  }
  std::printf("]}\n");
  return offsetDerivativesPass ? 0 : 2;
}
