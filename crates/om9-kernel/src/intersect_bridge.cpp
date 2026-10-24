// SPDX-License-Identifier: LGPL-2.1-or-later
// Exact curve/B-Rep intersection through OCCT's BRepIntCurveSurface_Inter. Every native
// object dies before returning; only the curve fractions cross into Rust.
#include <BRepCheck_Analyzer.hxx>
#include <Adaptor3d_Curve.hxx>
#include <BRepIntCurveSurface_Inter.hxx>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAPI_ExtremaCurveCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Line.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <IntCurvesFace_ShapeIntersector.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <algorithm>
#include <cmath>
#include <cstdio>
#include <sstream>
#include <stdexcept>
#include <vector>

namespace {

bool read_shape(const char* source, size_t length, TopoDS_Shape& shape) {
  if (!source || !length) return false;
  BRep_Builder builder;
  std::istringstream input(std::string(source, length));
  BRepTools::Read(shape, input, builder);
  return !shape.IsNull() && BRepCheck_Analyzer(shape).IsValid();
}

int emit(IntCurvesFace_ShapeIntersector& inter, double u0, double u1, double* fractions,
         size_t capacity, size_t* count, char* message, int message_capacity) {
  if (!fractions || !count || !(u1 > u0)) return 1;
  std::vector<double> hits;
  for (int index = 1; index <= inter.NbPnt(); ++index) {
    const double w = inter.WParameter(index);
    if (!std::isfinite(w)) continue;
    const double fraction = (w - u0) / (u1 - u0);
    if (fraction > 1e-7 && fraction < 1.0 - 1e-7) hits.push_back(fraction);
  }
  std::sort(hits.begin(), hits.end());
  hits.erase(std::unique(hits.begin(), hits.end(),
                         [](double a, double b) { return std::fabs(a - b) <= 1e-7; }),
             hits.end());
  if (hits.size() > capacity) {
    if (message && message_capacity > 0)
      std::snprintf(message, message_capacity, "curve/B-Rep intersection budget");
    return 3;
  }
  std::copy(hits.begin(), hits.end(), fractions);
  *count = hits.size();
  return 0;
}

int run(const TopoDS_Shape& shape, const GeomAdaptor_Curve& curve, double u0, double u1,
        double tolerance, double* fractions, size_t capacity, size_t* count, char* message,
        int message_capacity) {
  IntCurvesFace_ShapeIntersector inter;
  inter.Load(shape, tolerance);
  const occ::handle<Adaptor3d_Curve> adaptor = new GeomAdaptor_Curve(curve);
  inter.Perform(adaptor, u0, u1);
  if (!inter.IsDone()) throw std::runtime_error("curve/B-Rep intersection did not complete");
  return emit(inter, u0, u1, fractions, capacity, count, message, message_capacity);
}

}  // namespace

extern "C" int om9_occt_line_brep_intersections(
    const char* source, size_t length, const double* p0, const double* p1, double tolerance,
    double* fractions, size_t capacity, size_t* count, char* message, int message_capacity) {
  try {
    if (!p0 || !p1 || !std::isfinite(tolerance) || tolerance <= 0) return 1;
    TopoDS_Shape shape;
    if (!read_shape(source, length, shape)) throw std::invalid_argument("invalid BRep source");
    const gp_Pnt start(p0[0], p0[1], p0[2]);
    const gp_Pnt end(p1[0], p1[1], p1[2]);
    const gp_Vec vector(start, end);
    const double span = vector.Magnitude();
    if (!(span > tolerance)) throw std::invalid_argument("degenerate cutting line");
    const occ::handle<Geom_Line> line = new Geom_Line(start, gp_Dir(vector));
    const GeomAdaptor_Curve adaptor(line);
    return run(shape, adaptor, 0.0, span, tolerance, fractions, capacity, count, message,
               message_capacity);
  } catch (const std::exception& error) {
    if (message && message_capacity > 0) std::snprintf(message, message_capacity, "%s", error.what());
    return 2;
  } catch (...) {
    if (message && message_capacity > 0)
      std::snprintf(message, message_capacity, "unknown curve/B-Rep intersection failure");
    return 2;
  }
}

extern "C" int om9_occt_circle_brep_intersections(
    const char* source, size_t length, const double* center, const double* x, const double* y,
    double radius, double u0, double u1, double tolerance, double* fractions, size_t capacity,
    size_t* count, char* message, int message_capacity) {
  try {
    if (!center || !x || !y || !std::isfinite(radius) || radius <= 0 || !std::isfinite(u0) ||
        !std::isfinite(u1) || u1 <= u0 || !std::isfinite(tolerance) || tolerance <= 0)
      return 1;
    TopoDS_Shape shape;
    if (!read_shape(source, length, shape)) throw std::invalid_argument("invalid BRep source");
    const gp_Pnt origin(center[0], center[1], center[2]);
    const gp_Dir x_dir(x[0], x[1], x[2]);
    const gp_Dir y_dir(y[0], y[1], y[2]);
    const gp_Dir normal = x_dir.Crossed(y_dir);
    const occ::handle<Geom_Circle> circle = new Geom_Circle(gp_Ax2(origin, normal, x_dir), radius);
    if (u1 - u0 > 2.0 * M_PI + 1e-9) throw std::invalid_argument("circle angle span");
    const occ::handle<Geom_TrimmedCurve> trimmed = new Geom_TrimmedCurve(circle, u0, u1);
    const GeomAdaptor_Curve adaptor(trimmed);
    return run(shape, adaptor, u0, u1, tolerance, fractions, capacity, count, message,
               message_capacity);
  } catch (const std::exception& error) {
    if (message && message_capacity > 0) std::snprintf(message, message_capacity, "%s", error.what());
    return 2;
  } catch (...) {
    if (message && message_capacity > 0)
      std::snprintf(message, message_capacity, "unknown circle/B-Rep intersection failure");
    return 2;
  }
}

extern "C" int om9_occt_bspline_brep_intersections(
    const char* source, size_t length, int degree, size_t pole_count, const double* poles,
    const double* weights, size_t knot_count, const double* knots, const int* multiplicities,
    int periodic, double u0, double u1, double tolerance, double* fractions, size_t capacity,
    size_t* count, char* message, int message_capacity) {
  try {
    if (!poles || !weights || !knots || !multiplicities || degree < 1 || pole_count < 2 ||
        knot_count < 2 || !std::isfinite(u0) || !std::isfinite(u1) || u1 <= u0 ||
        !std::isfinite(tolerance) || tolerance <= 0)
      return 1;
    TopoDS_Shape shape;
    if (!read_shape(source, length, shape)) throw std::invalid_argument("invalid BRep source");
    TColgp_Array1OfPnt control_points(1, static_cast<int>(pole_count));
    TColStd_Array1OfReal control_weights(1, static_cast<int>(pole_count));
    for (size_t i = 0; i < pole_count; ++i) {
      control_points.SetValue(static_cast<int>(i + 1),
                              gp_Pnt(poles[i * 3], poles[i * 3 + 1], poles[i * 3 + 2]));
      control_weights.SetValue(static_cast<int>(i + 1), weights[i]);
    }
    TColStd_Array1OfReal unique_knots(1, static_cast<int>(knot_count));
    TColStd_Array1OfInteger mults(1, static_cast<int>(knot_count));
    for (size_t i = 0; i < knot_count; ++i) {
      unique_knots.SetValue(static_cast<int>(i + 1), knots[i]);
      mults.SetValue(static_cast<int>(i + 1), multiplicities[i]);
    }
    const occ::handle<Geom_BSplineCurve> curve =
        new Geom_BSplineCurve(control_points, control_weights, unique_knots, mults, degree,
                              periodic != 0);
    const GeomAdaptor_Curve adaptor(curve);
    return run(shape, adaptor, u0, u1, tolerance, fractions, capacity, count, message,
               message_capacity);
  } catch (const std::exception& error) {
    if (message && message_capacity > 0) std::snprintf(message, message_capacity, "%s", error.what());
    return 2;
  } catch (...) {
    if (message && message_capacity > 0)
      std::snprintf(message, message_capacity, "unknown B-spline/B-Rep intersection failure");
    return 2;
  }
}


extern "C" int om9_occt_line_curve_parameter(
    const double* origin, const double* point, int degree, const double* poles,
    size_t pole_count, const double* weights, size_t weight_count, const double* knots,
    size_t knot_count, int periodic, double* fraction, char* message, int capacity) {
  try {
    if (!origin || !point || !poles || !weights || !knots || !fraction)
      throw std::invalid_argument("line curve table");
    if (degree < 1 || pole_count < static_cast<size_t>(degree + 1) || knot_count < 2)
      throw std::invalid_argument("line curve shape");
    gp_Vec direction(gp_Pnt(origin[0], origin[1], origin[2]),
                     gp_Pnt(point[0], point[1], point[2]));
    if (direction.Magnitude() <= 1.0e-12) throw std::invalid_argument("line direction");
    const occ::handle<Geom_Line> line =
        new Geom_Line(gp_Pnt(origin[0], origin[1], origin[2]), gp_Dir(direction));
    TColgp_Array1OfPnt control_points(1, static_cast<int>(pole_count));
    TColStd_Array1OfReal control_weights(1, static_cast<int>(pole_count));
    for (size_t i = 0; i < pole_count; ++i) {
      control_points.SetValue(static_cast<int>(i + 1),
                              gp_Pnt(poles[3 * i], poles[3 * i + 1], poles[3 * i + 2]));
      const double weight = weight_count == pole_count ? weights[i] : 1.0;
      if (!std::isfinite(weight) || weight <= 0.0) throw std::invalid_argument("weight");
      control_weights.SetValue(static_cast<int>(i + 1), weight);
    }
    std::vector<double> unique;
    std::vector<int> multiplicities;
    for (size_t i = 0; i < knot_count; ++i) {
      if (!std::isfinite(knots[i])) throw std::invalid_argument("knot");
      if (unique.empty() || std::abs(unique.back() - knots[i]) > 1.0e-12) {
        unique.push_back(knots[i]);
        multiplicities.push_back(1);
      } else {
        ++multiplicities.back();
      }
    }
    TColStd_Array1OfReal unique_knots(1, static_cast<int>(unique.size()));
    TColStd_Array1OfInteger mults(1, static_cast<int>(multiplicities.size()));
    for (size_t i = 0; i < unique.size(); ++i) {
      unique_knots.SetValue(static_cast<int>(i + 1), unique[i]);
      mults.SetValue(static_cast<int>(i + 1), multiplicities[i]);
    }
    const occ::handle<Geom_BSplineCurve> curve =
        new Geom_BSplineCurve(control_points, control_weights, unique_knots, mults, degree,
                              periodic != 0);
    GeomAPI_ExtremaCurveCurve extrema(line, curve);
    if (extrema.NbExtrema() < 1) throw std::runtime_error("no line-curve extremum");
    double best = 1.0e300;
    double best_parameter = 0.0;
    double best_point_distance = 1.0e300;
    const gp_Pnt target(point[0], point[1], point[2]);
    for (int i = 1; i <= extrema.NbExtrema(); ++i) {
      const double distance = extrema.Distance(i);
      double u1 = 0.0;
      double u2 = 0.0;
      extrema.Parameters(i, u1, u2);
      const double point_distance = curve->Value(u2).Distance(target);
      if (distance < best - 1.0e-12 ||
          (std::abs(distance - best) <= 1.0e-12 && point_distance < best_point_distance)) {
        best = distance;
        best_parameter = u2;
        best_point_distance = point_distance;
      }
    }
    const double first = curve->FirstParameter();
    const double last = curve->LastParameter();
    if (!(last > first)) throw std::runtime_error("empty curve domain");
    *fraction = (best_parameter - first) / (last - first);
    return 0;
  } catch (const std::exception& error) {
    if (message && capacity > 0) std::snprintf(message, capacity, "%s", error.what());
    return 2;
  } catch (...) {
    if (message && capacity > 0)
      std::snprintf(message, capacity, "unknown line-curve failure");
    return 2;
  }
}


