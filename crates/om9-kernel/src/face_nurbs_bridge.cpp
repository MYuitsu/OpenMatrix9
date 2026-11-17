// SPDX-License-Identifier: LGPL-2.1-or-later
// Exact NURBS of BRep faces, for add-ons: each face's underlying surface and its trim loops
// (3D edge curves, outer loop first, edges in wire order and direction). Read-only: the shape
// is parsed from its canonical source and nothing is built. Written as text through a byte sink.
#include <Geom2d_Curve.hxx>
#include <TopoDS_Vertex.hxx>
#include <gp_Pnt2d.hxx>
#include <algorithm>
#include <cmath>
#include <BRepCheck_Analyzer.hxx>
#include <BRepTools.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <GeomConvert.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BoundedSurface.hxx>
#include <Geom_Curve.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_Surface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Standard_Failure.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>

#include <cstdint>
#include <cstdio>
#include <iomanip>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

void face_nurbs_failure(char* message, int capacity, const char* text) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", text);
}

TopoDS_Shape face_nurbs_source(const char* source, size_t length) {
  if (!source || !length) throw std::invalid_argument("missing canonical BRep source");
  TopoDS_Shape shape;
  BRep_Builder builder;
  std::istringstream input(std::string(source, length));
  BRepTools::Read(shape, input, builder);
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid())
    throw std::invalid_argument("invalid canonical BRep source");
  return shape;
}

void write_curve(std::ostringstream& out, Handle(Geom_BSplineCurve) curve) {
  // SetNotPeriodic keeps the periodic knot vector's unclamped ends; Segment over the whole
  // range clamps them (end knots of multiplicity degree + 1) without moving the curve.
  if (curve->IsPeriodic()) {
    const Standard_Real first = curve->FirstParameter(), last = curve->LastParameter();
    curve->SetNotPeriodic();
    curve->Segment(first, last);
  }
  const int poles = curve->NbPoles(), degree = curve->Degree();
  TColStd_Array1OfReal knots(1, poles + degree + 1);
  curve->KnotSequence(knots);
  out << "C " << degree << ' ' << poles << '\n';
  for (int i = knots.Lower(); i <= knots.Upper(); ++i) out << knots(i) << ' ';
  out << '\n';
  for (int i = 1; i <= poles; ++i) {
    const gp_Pnt p = curve->Pole(i);
    out << p.X() << ' ' << p.Y() << ' ' << p.Z() << ' ' << curve->Weight(i) << ' ';
  }
  out << '\n';
}

// The edges of a wire that carry a 3D curve, in order and direction along the wire.
std::vector<Handle(Geom_BSplineCurve)> wire_curves(const TopoDS_Wire& wire, const TopoDS_Face& face) {
  std::vector<Handle(Geom_BSplineCurve)> out;
  for (BRepTools_WireExplorer it(wire, face); it.More(); it.Next()) {
    const TopoDS_Edge& edge = it.Current();
    if (BRep_Tool::Degenerated(edge)) continue;
    Standard_Real first = 0, last = 0;
    const Handle(Geom_Curve) curve = BRep_Tool::Curve(edge, first, last);
    if (curve.IsNull()) continue;
    Handle(Geom_BSplineCurve) bs = GeomConvert::CurveToBSplineCurve(new Geom_TrimmedCurve(curve, first, last));
    if (bs.IsNull()) throw std::runtime_error("edge curve not convertible to NURBS");
    // The edge's own orientation in the wire: WireExplorer::Orientation() reports FORWARD for
    // a hole made of one closed edge that the wire uses reversed.
    if (edge.Orientation() == TopAbs_REVERSED) bs->Reverse();
    // End exactly on the edge's vertices (in loop order), so a loop closes and neighbouring
    // faces meet without the conversion's last digits (a boolean leaves gaps of 1e-7).
    TopoDS_Vertex first_vertex, last_vertex;
    TopExp::Vertices(edge, first_vertex, last_vertex, Standard_True);
    if (!first_vertex.IsNull()) bs->SetPole(1, BRep_Tool::Pnt(first_vertex));
    if (!last_vertex.IsNull()) bs->SetPole(bs->NbPoles(), BRep_Tool::Pnt(last_vertex));
    out.push_back(bs);
  }
  return out;
}

// A face bounded only by its surface's own edges — one wire whose edges all run along the
// parameter box's sides (a box side, a cylinder's side with its seam, a sphere with its poles)
// — is the whole surface: it gets no loops.
bool natural_face(const TopoDS_Face& forward, const Handle(Geom_BSplineSurface)& surface) {
  int wires = 0;
  for (TopExp_Explorer it(forward, TopAbs_WIRE); it.More(); it.Next()) ++wires;
  if (wires != 1) return false;
  Standard_Real u0, u1, v0, v1;
  surface->Bounds(u0, u1, v0, v1);
  const double tu = 1e-9 * std::max(1.0, u1 - u0), tv = 1e-9 * std::max(1.0, v1 - v0);
  const auto side = [](double x, double a, double b, double t) {
    return std::abs(x - a) < t || std::abs(x - b) < t;
  };
  for (TopExp_Explorer it(forward, TopAbs_EDGE); it.More(); it.Next()) {
    const TopoDS_Edge& edge = TopoDS::Edge(it.Current());
    Standard_Real f = 0, l = 0;
    const Handle(Geom2d_Curve) pcurve = BRep_Tool::CurveOnSurface(edge, forward, f, l);
    if (pcurve.IsNull()) return false;
    const gp_Pnt2d a = pcurve->Value(f), m = pcurve->Value(0.5 * (f + l)), b = pcurve->Value(l);
    const bool along_u_side = side(a.X(), u0, u1, tu) && std::abs(m.X() - a.X()) < tu && std::abs(b.X() - a.X()) < tu;
    const bool along_v_side = side(a.Y(), v0, v1, tv) && std::abs(m.Y() - a.Y()) < tv && std::abs(b.Y() - a.Y()) < tv;
    if (!along_u_side && !along_v_side) return false;
  }
  return true;
}

}  // namespace

extern "C" int om9_occt_face_nurbs(const char* source, size_t length, size_t max_bytes,
                                   int (*sink)(void*, const uint8_t*, size_t), void* context,
                                   char* message, int capacity) {
  try {
    if (!sink) throw std::invalid_argument("missing face NURBS sink");
    const TopoDS_Shape shape = face_nurbs_source(source, length);
    TopTools_IndexedMapOfShape faces;
    TopExp::MapShapes(shape, TopAbs_FACE, faces);
    if (faces.Extent() == 0) throw std::invalid_argument("the shape has no face");
    std::ostringstream out;
    out << std::setprecision(17);
    out << "F " << faces.Extent() << '\n';
    for (int f = 1; f <= faces.Extent(); ++f) {
      const TopoDS_Face face = TopoDS::Face(faces(f));
      Handle(Geom_Surface) surface = BRep_Tool::Surface(face);
      if (surface.IsNull()) throw std::runtime_error("a face has no surface");
      // Unbounded surfaces (planes, cylinders …) are cut to the face's parameter box; a
      // bounded one is kept whole (Untrim gives it back as it was).
      if (!surface->IsKind(STANDARD_TYPE(Geom_BoundedSurface))) {
        Standard_Real u0, u1, v0, v1;
        BRepTools::UVBounds(face, u0, u1, v0, v1);
        // A direction the face spans for a whole period stays untrimmed (periodic): the
        // conversion refuses a torus trimmed over both full periods.
        const bool u_full = surface->IsUPeriodic() && u1 - u0 >= surface->UPeriod() - 1e-9;
        const bool v_full = surface->IsVPeriodic() && v1 - v0 >= surface->VPeriod() - 1e-9;
        if (!u_full && !v_full)
          surface = new Geom_RectangularTrimmedSurface(surface, u0, u1, v0, v1);
        else if (!v_full)
          surface = new Geom_RectangularTrimmedSurface(surface, v0, v1, Standard_False);
        else if (!u_full)
          surface = new Geom_RectangularTrimmedSurface(surface, u0, u1, Standard_True);
      }
      Handle(Geom_BSplineSurface) bs = GeomConvert::SurfaceToBSplineSurface(surface);
      if (bs.IsNull()) throw std::runtime_error("surface not convertible to NURBS");
      if (bs->IsUPeriodic() || bs->IsVPeriodic()) {
        // As for curves: opening a periodic direction leaves unclamped ends; Segment over the
        // whole parameter box clamps them.
        Standard_Real a, b, c, d;
        bs->Bounds(a, b, c, d);
        if (bs->IsUPeriodic()) bs->SetUNotPeriodic();
        if (bs->IsVPeriodic()) bs->SetVNotPeriodic();
        bs->Segment(a, b, c, d);
      }
      const int nu = bs->NbUPoles(), nv = bs->NbVPoles(), du = bs->UDegree(), dv = bs->VDegree();
      out << "S " << du << ' ' << dv << ' ' << nu << ' ' << nv << ' '
          << (face.Orientation() == TopAbs_REVERSED ? 1 : 0) << '\n';
      TColStd_Array1OfReal uk(1, nu + du + 1), vk(1, nv + dv + 1);
      bs->UKnotSequence(uk);
      bs->VKnotSequence(vk);
      for (int i = uk.Lower(); i <= uk.Upper(); ++i) out << uk(i) << ' ';
      out << '\n';
      for (int i = vk.Lower(); i <= vk.Upper(); ++i) out << vk(i) << ' ';
      out << '\n';
      for (int i = 1; i <= nu; ++i)
        for (int j = 1; j <= nv; ++j) {
          const gp_Pnt p = bs->Pole(i, j);
          out << p.X() << ' ' << p.Y() << ' ' << p.Z() << ' ' << bs->Weight(i, j) << ' ';
        }
      out << '\n';
      // Loops: the outer wire first, then the holes. They run as on the forward face, i.e.
      // with the surface's own normal (outer counter-clockwise, holes clockwise in its
      // parameters), as Rhino keeps trim loops; `reversed` says the face's normal is flipped.
      const TopoDS_Face forward = TopoDS::Face(face.Oriented(TopAbs_FORWARD));
      if (natural_face(forward, bs)) {
        out << "W 0\n";
        if (static_cast<size_t>(out.tellp()) > max_bytes) throw std::length_error("face NURBS exceed the budget");
        continue;
      }
      std::vector<TopoDS_Wire> wires;
      const TopoDS_Wire outer = BRepTools::OuterWire(forward);
      if (!outer.IsNull()) wires.push_back(outer);
      for (TopExp_Explorer it(forward, TopAbs_WIRE); it.More(); it.Next()) {
        const TopoDS_Wire wire = TopoDS::Wire(it.Current());
        if (!outer.IsNull() && wire.IsSame(outer)) continue;
        wires.push_back(wire);
      }
      out << "W " << wires.size() << '\n';
      for (const TopoDS_Wire& wire : wires) {
        const auto curves = wire_curves(wire, forward);
        out << "L " << curves.size() << '\n';
        for (const auto& c : curves) write_curve(out, c);
      }
      if (static_cast<size_t>(out.tellp()) > max_bytes) throw std::length_error("face NURBS exceed the budget");
    }
    const std::string text = out.str();
    if (text.size() > max_bytes) throw std::length_error("face NURBS exceed the budget");
    return sink(context, reinterpret_cast<const uint8_t*>(text.data()), text.size()) == 0 ? 0 : 1;
  } catch (const Standard_Failure& e) {
    face_nurbs_failure(message, capacity, e.GetMessageString());
    return 2;
  } catch (const std::exception& e) {
    face_nurbs_failure(message, capacity, e.what());
    return 1;
  } catch (...) {
    face_nurbs_failure(message, capacity, "Unknown face NURBS failure");
    return 2;
  }
}
