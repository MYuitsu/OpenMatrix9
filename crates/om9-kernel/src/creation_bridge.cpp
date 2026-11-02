// SPDX-License-Identifier: LGPL-2.1-or-later
// M03 OM9-L2-01-09 geometry creation adapters.
//
// Each entry point is one complete operation: typed scalar/point arrays in, one owned exact B-Rep
// snapshot out. Validation that belongs to the domain (winding, planarity, profile simplicity,
// rail frames, sweep section placement) happens in Rust before the call; the C++ here only maps
// already-validated data onto the OCCT construction API and contains its exceptions.
#include "brep_snapshot.hxx"
#include <vector>

#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepBuilderAPI_TransitionMode.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepCheck_Shell.hxx>
#include <BRepLib.hxx>
#include <Geom_Circle.hxx>
#include <BRepOffsetAPI_MakeFilling.hxx>
#include <BRepOffsetAPI_MakePipeShell.hxx>
#include <BRepOffsetAPI_ThruSections.hxx>
#include <BRepFill_TypeOfContact.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCone.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <Geom_BSplineSurface.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TColStd_Array2OfReal.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS.hxx>
#include <TopExp_Explorer.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <ShapeAnalysis_Shell.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepPrimAPI_MakeTorus.hxx>
#include <BRep_Builder.hxx>
#include <GeomAbs_Shape.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Pnt.hxx>
#include <gp_Vec.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>

namespace {

void require_finite(const double* values, size_t count, const char* what) {
 if (!values && count) throw std::invalid_argument(what);
 for (size_t i = 0; i < count; ++i)
  if (!std::isfinite(values[i])) throw std::invalid_argument(what);
}

void require_positive(double value, const char* what) {
 if (!std::isfinite(value) || value <= 0) throw std::invalid_argument(what);
}

TopoDS_Wire make_wire(const double* points, size_t count) {
 if (!points || count < 3) throw std::invalid_argument("profile needs at least three points");
 require_finite(points, count * 3, "profile point");
 BRepBuilderAPI_MakePolygon polygon;
 for (size_t i = 0; i < count; ++i) {
  const double* p = points + 3 * i;
  polygon.Add(gp_Pnt(p[0], p[1], p[2]));
 }
 polygon.Close();
 if (!polygon.IsDone()) throw std::invalid_argument("profile is not a closed wire");
 return polygon.Wire();
}

TopoDS_Wire make_open_wire(const double* points, size_t count) {
 if (!points || count < 2) throw std::invalid_argument("open profile needs at least two points");
 require_finite(points, count * 3, "profile point");
 BRepBuilderAPI_MakePolygon polygon;
 for (size_t i = 0; i < count; ++i) {
  const double* p = points + 3 * i;
  polygon.Add(gp_Pnt(p[0], p[1], p[2]));
 }
 if (!polygon.IsDone()) throw std::invalid_argument("profile is not a wire");
 return polygon.Wire();
}

TopoDS_Face make_planar_face(const double* points, size_t count) {
 BRepBuilderAPI_MakeFace face(make_wire(points, count), true);
 if (!face.IsDone()) throw std::invalid_argument("profile is not a planar face");
 return face.Face();
}

TopoDS_Shape checked(const TopoDS_Shape& shape, const char* what) {
 if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid())
  throw std::runtime_error(what);
 return shape;
}

}  // namespace

extern "C" int om9_occt_primitive(int kind, const double* params, size_t count, double deflection,
                                  size_t max_points, size_t max_nodes, size_t max_bytes,
                                  Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  require_finite(params, count, "primitive parameter");
  TopoDS_Shape shape;
  switch (kind) {
   case 0: {  // box: x, y, z
    if (count != 3) throw std::invalid_argument("box needs three sizes");
    for (int i = 0; i < 3; ++i) require_positive(params[i], "box size");
    shape = BRepPrimAPI_MakeBox(params[0], params[1], params[2]).Shape();
    break;
   }
   case 1: {  // sphere: radius
    if (count != 1) throw std::invalid_argument("sphere needs one radius");
    require_positive(params[0], "sphere radius");
    shape = BRepPrimAPI_MakeSphere(params[0]).Shape();
    break;
   }
   case 2: {  // cylinder: radius, height
    if (count != 2) throw std::invalid_argument("cylinder needs radius and height");
    require_positive(params[0], "cylinder radius");
    require_positive(params[1], "cylinder height");
    shape = BRepPrimAPI_MakeCylinder(params[0], params[1]).Shape();
    break;
   }
   case 3: {  // cone: bottom radius, top radius, height
    if (count != 3) throw std::invalid_argument("cone needs two radii and a height");
    require_positive(params[0], "cone bottom radius");
    if (!std::isfinite(params[1]) || params[1] < 0) throw std::invalid_argument("cone top radius");
    require_positive(params[2], "cone height");
    shape = BRepPrimAPI_MakeCone(params[0], params[1], params[2]).Shape();
    break;
   }
   case 4: {  // torus: ring radius, tube radius
    if (count != 2) throw std::invalid_argument("torus needs ring and tube radii");
    require_positive(params[0], "torus ring radius");
    require_positive(params[1], "torus tube radius");
    if (params[1] >= params[0]) throw std::invalid_argument("torus tube is not smaller than its ring");
    shape = BRepPrimAPI_MakeTorus(params[0], params[1]).Shape();
    break;
   }
   case 5: {  // exact rational biquadratic ellipsoid net supplied by Rust
    if (count != 180) throw std::invalid_argument("ellipsoid needs 45 poles and weights");
    TColgp_Array2OfPnt poles(1, 9, 1, 5);
    TColStd_Array2OfReal weights(1, 9, 1, 5);
    for (int u = 0; u < 9; ++u) {
     for (int v = 0; v < 5; ++v) {
      const int i = u * 5 + v;
      poles.SetValue(u + 1, v + 1, gp_Pnt(params[3*i], params[3*i+1], params[3*i+2]));
      require_positive(params[135 + i], "ellipsoid weight");
      weights.SetValue(u + 1, v + 1, params[135 + i]);
     }
    }
    TColStd_Array1OfReal u_knots(1, 5), v_knots(1, 3);
    TColStd_Array1OfInteger u_mults(1, 5), v_mults(1, 3);
    for (int i = 1; i <= 5; ++i) {
     u_knots.SetValue(i, i - 1);
     u_mults.SetValue(i, (i == 1 || i == 5) ? 3 : 2);
    }
    for (int i = 1; i <= 3; ++i) {
     v_knots.SetValue(i, i - 1);
     v_mults.SetValue(i, (i == 1 || i == 3) ? 3 : 2);
    }
    Handle(Geom_BSplineSurface) surface = new Geom_BSplineSurface(
        poles, weights, u_knots, v_knots, u_mults, v_mults, 2, 2, false, false);
    // Natural bounds recognize the duplicated U seam and collapsed pole rows. A single
    // face retains the seam pcurves and degenerate pole edges without approximate sewing.
    BRepBuilderAPI_MakeFace face(surface, 1.0e-7);
    if (!face.IsDone()) throw std::runtime_error("ellipsoid face failed");
    TopoDS_Shell shell;
    BRep_Builder builder;
    builder.MakeShell(shell);
    builder.Add(shell, face.Face());
    if (BRepCheck_Shell(shell).Closed() != BRepCheck_NoError)
     throw std::runtime_error("ellipsoid shell has free boundaries");
    shell.Closed(true);
    BRepBuilderAPI_MakeSolid solid(shell);
    if (!solid.IsDone()) throw std::runtime_error("ellipsoid solid failed");
    TopoDS_Solid result = solid.Solid();
    if (!BRepLib::OrientClosedSolid(result))
     throw std::runtime_error("ellipsoid is not a closed orientable solid");
    shape = result;
    break;
   }
   case 6: {  // pyramid: sides, base circumradius, height (apex on the axis)
    if (count != 3) throw std::invalid_argument("pyramid needs sides, radius and height");
    if (!std::isfinite(params[0]) || params[0] < 3 || params[0] != std::floor(params[0]))
     throw std::invalid_argument("pyramid sides");
    require_positive(params[1], "pyramid base radius");
    require_positive(params[2], "pyramid height");
    const int sides = int(params[0]);
    BRepBuilderAPI_MakePolygon base;
    for (int i = 0; i < sides; ++i) {
     const double angle = 2.0 * M_PI * double(i) / double(sides);
     base.Add(gp_Pnt(params[1] * std::cos(angle), params[1] * std::sin(angle), 0.0));
    }
    base.Close();
    if (!base.IsDone()) throw std::runtime_error("pyramid base wire");
    BRepOffsetAPI_ThruSections sections(true, true, 1.0e-6);
    sections.CheckCompatibility(false);
    sections.AddWire(base.Wire());
    sections.AddVertex(BRepBuilderAPI_MakeVertex(gp_Pnt(0.0, 0.0, params[2])));
    sections.Build();
    if (!sections.IsDone()) throw std::runtime_error("pyramid build failed");
    shape = sections.Shape();
    break;
   }
   case 7: {  // pyramid over a Rust-validated planar profile, apex on the local axis
    if (count < 10 || (count - 1) % 3 != 0) throw std::invalid_argument("pyramid profile parameters");
    require_positive(params[0], "pyramid profile height");
    const size_t vertices = (count - 1) / 3;
    if (vertices > max_points || vertices > max_bytes / 512) throw std::invalid_argument("pyramid profile budget");
    BRepOffsetAPI_ThruSections sections(true, true, 1.0e-6);
    sections.CheckCompatibility(false);
    sections.AddWire(make_wire(params + 1, vertices));
    sections.AddVertex(BRepBuilderAPI_MakeVertex(gp_Pnt(0.0, 0.0, params[0])));
    sections.Build();
    if (!sections.IsDone()) throw std::runtime_error("pyramid profile build failed");
    shape = sections.Shape();
    break;
   }
   default:
    throw std::invalid_argument("unknown primitive kind");
  }
  return om9_emit_brep_snapshot(checked(shape, "invalid primitive shape"), deflection, max_points,
                                max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown primitive failure");
  return 2;
 }
}

extern "C" int om9_occt_extrude(const double* points, size_t count, const double* direction,
                                int capped, double deflection, size_t max_points, size_t max_nodes,
                                size_t max_bytes, Om9BrepSink sink, void* context, char* message,
                                int capacity) {
 try {
  require_finite(direction, 3, "extrusion direction");
  const double length = std::sqrt(direction[0] * direction[0] + direction[1] * direction[1] +
                                  direction[2] * direction[2]);
  if (!std::isfinite(length) || length <= 1.0e-12)
   throw std::invalid_argument("extrusion direction is zero");
  if (capped < 0 || capped > 2) throw std::invalid_argument("invalid extrusion profile mode");
  const TopoDS_Shape profile = capped == 1 ? TopoDS_Shape(make_planar_face(points, count))
                               : capped == 2 ? TopoDS_Shape(make_wire(points, count))
                                             : TopoDS_Shape(make_open_wire(points, count));
  BRepPrimAPI_MakePrism prism(profile, gp_Vec(direction[0], direction[1], direction[2]));
  return om9_emit_brep_snapshot(checked(prism.Shape(), "invalid extrusion"), deflection, max_points,
                                max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown extrusion failure");
  return 2;
 }
}

extern "C" int om9_occt_revolve(const double* points, size_t count, const double* axis, double angle,
                                double deflection, size_t max_points, size_t max_nodes,
                                size_t max_bytes, Om9BrepSink sink, void* context, char* message,
                                int capacity) {
 try {
  require_finite(axis, 6, "revolution axis");
  const double length = std::sqrt(axis[3] * axis[3] + axis[4] * axis[4] + axis[5] * axis[5]);
  if (!std::isfinite(length) || length <= 1.0e-12)
   throw std::invalid_argument("revolution axis direction is zero");
  if (!std::isfinite(angle) || std::fabs(angle) <= 1.0e-12 || std::fabs(angle) > 2.0 * M_PI + 1.0e-9)
   throw std::invalid_argument("revolution angle");
  const gp_Ax1 revolution(gp_Pnt(axis[0], axis[1], axis[2]), gp_Dir(axis[3], axis[4], axis[5]));
  const TopoDS_Face profile = make_planar_face(points, count);
  BRepPrimAPI_MakeRevol revol(profile, revolution, angle, false);
  return om9_emit_brep_snapshot(checked(revol.Shape(), "invalid revolution"), deflection, max_points,
                                max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown revolution failure");
  return 2;
 }
}

extern "C" int om9_occt_planar(const double* points, size_t count, const double* plane,
                               double deflection, size_t max_points, size_t max_nodes,
                               size_t max_bytes, Om9BrepSink sink, void* context, char* message,
                               int capacity) {
 try {
  require_finite(plane, 6, "planar face plane");
  const gp_Pln surface(gp_Pnt(plane[0], plane[1], plane[2]), gp_Dir(plane[3], plane[4], plane[5]));
  BRepBuilderAPI_MakeFace face(surface, make_wire(points, count), true);
  if (!face.IsDone()) throw std::runtime_error("planar face build failed");
  return om9_emit_brep_snapshot(checked(face.Face(), "invalid planar face"), deflection, max_points,
                                max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown planar failure");
  return 2;
 }
}
extern "C" int om9_occt_loft(const double* points, size_t count, const size_t* sections,
                             size_t section_count, int solid, int ruled, double deflection,
                             size_t max_points, size_t max_nodes, size_t max_bytes,
                             Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  if (section_count < 2) throw std::invalid_argument("loft needs at least two sections");
  if (!sections) throw std::invalid_argument("loft section table");
  BRepOffsetAPI_ThruSections loft(solid != 0, ruled != 0, 1.0e-6);
  loft.CheckCompatibility(false);
  for (size_t i = 0; i < section_count; ++i) {
   const size_t offset = sections[2 * i];
   const size_t length = sections[2 * i + 1];
   if (offset + length > count) throw std::invalid_argument("loft section range");
   loft.AddWire(make_wire(points + 3 * offset, length));
  }
  loft.Build();
  if (!loft.IsDone()) throw std::runtime_error("loft build failed");
  return om9_emit_brep_snapshot(checked(loft.Shape(), "invalid loft"), deflection, max_points,
                                max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown loft failure");
  return 2;
 }
}

extern "C" int om9_occt_pipe_shell(const double* spine, size_t spine_count, const double* aux,
                                  size_t aux_count, const double* points, size_t count,
                                  const size_t* sections, size_t section_count, int solid,
                                  int aux_mode, double deflection, size_t max_points,
                                  size_t max_nodes, size_t max_bytes, Om9BrepSink sink,
                                  void* context, char* message, int capacity) {
 try {
  if (section_count < 1) throw std::invalid_argument("pipe shell needs at least one section");
  if (!sections) throw std::invalid_argument("pipe shell section table");
  BRepOffsetAPI_MakePipeShell pipe(make_open_wire(spine, spine_count));
  pipe.SetMode(false);
  if (aux_mode != 0 && aux && aux_count >= 2) {
   pipe.SetMode(make_open_wire(aux, aux_count), false, BRepFill_NoContact);
  }
  for (size_t i = 0; i < section_count; ++i) {
   const size_t offset = sections[2 * i];
   const size_t length = sections[2 * i + 1];
   if (offset + length > count) throw std::invalid_argument("pipe shell section range");
   pipe.Add(make_open_wire(points + 3 * offset, length), false, false);
  }
  pipe.Build();
  if (!pipe.IsReady()) throw std::runtime_error("pipe shell build failed");
  if (solid != 0 && !pipe.MakeSolid()) throw std::runtime_error("pipe shell solid failed");
  return om9_emit_brep_snapshot(checked(pipe.Shape(), "invalid pipe shell"), deflection,
                                max_points, max_nodes, max_bytes, sink, context, message,
                                capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown pipe shell failure");
  return 2;
 }
}

extern "C" int om9_occt_four_rail_sweep(
    const double* rails, size_t rail_count, const size_t* rail_table,
    const double* sections, size_t section_count, const size_t* section_table,
    const size_t* section_ranges, int solid, double tolerance, double deflection,
    size_t max_points, size_t max_nodes, size_t max_bytes, Om9BrepSink sink, void* context,
    char* message, int capacity) {
 try {
  if (!rail_table || !section_table || !section_ranges)
   throw std::invalid_argument("four rail table");
  if (!std::isfinite(tolerance) || tolerance <= 0)
   throw std::invalid_argument("four rail tolerance");
  std::vector<TopoDS_Shape> patches;
  patches.reserve(4);
  for (size_t q = 0; q < 4; ++q) {
   const size_t rail_offset = rail_table[2 * q];
   const size_t rail_length = rail_table[2 * q + 1];
   const size_t next = (q + 1) % 4;
   const size_t next_offset = rail_table[2 * next];
   const size_t next_length = rail_table[2 * next + 1];
   if (rail_offset + rail_length > rail_count || next_offset + next_length > rail_count)
    throw std::invalid_argument("four rail range");
   BRepOffsetAPI_MakePipeShell pipe(make_open_wire(rails + 3 * rail_offset, rail_length));
   pipe.SetMode(false);
   pipe.SetMode(make_open_wire(rails + 3 * next_offset, next_length), false,
                BRepFill_NoContact);
   const size_t first = section_ranges[2 * q];
   const size_t count = section_ranges[2 * q + 1];
   for (size_t s = 0; s < count; ++s) {
    const size_t index = first + s;
    const size_t offset = section_table[2 * index];
    const size_t length = section_table[2 * index + 1];
    if (offset + length > section_count) throw std::invalid_argument("four rail section");
    pipe.Add(make_open_wire(sections + 3 * offset, length), false, false);
   }
   pipe.Build();
   if (!pipe.IsReady()) throw std::runtime_error("four rail patch failed");
   patches.push_back(pipe.Shape());
  }
  BRepBuilderAPI_Sewing sewing(tolerance);
  for (const auto& patch : patches) sewing.Add(patch);
  sewing.Perform();
  TopoDS_Shape sewed = sewing.SewedShape();
  if (sewed.IsNull()) throw std::runtime_error("four rail sewing failed");
  if (solid != 0) {
   // A sewn open shell stays open; the command reports a polysurface, not a solid.
   throw std::runtime_error("four rail solid is not supported");
  }
  return om9_emit_brep_snapshot(sewed, deflection, max_points, max_nodes, max_bytes, sink,
                                context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown four rail failure");
  return 2;
 }
}

extern "C" int om9_occt_surface(const double* points, size_t count, const size_t* curves,
                                size_t curve_count, double tolerance, double* deviation,
                                double deflection, size_t max_points, size_t max_nodes,
                                size_t max_bytes, Om9BrepSink sink, void* context, char* message,
                                int capacity) {
 try {
  if (curve_count < 2 || curve_count > 4)
   throw std::invalid_argument("surface filling needs two to four boundary curves");
  if (!curves || !deviation) throw std::invalid_argument("surface filling table");
  if (!std::isfinite(tolerance) || tolerance <= 0)
   throw std::invalid_argument("surface filling tolerance");
  BRepOffsetAPI_MakeFilling filling(3, 15, 2, false, tolerance, tolerance, 0.01, 0.1, 8, 9);
  for (size_t i = 0; i < curve_count; ++i) {
   const size_t offset = curves[2 * i];
   const size_t length = curves[2 * i + 1];
   if (length < 2 || offset + length > count) throw std::invalid_argument("surface curve range");
   for (size_t j = 1; j < length; ++j) {
    const double* a = points + 3 * (offset + j - 1);
    const double* b = points + 3 * (offset + j);
    require_finite(a, 3, "surface curve point");
    require_finite(b, 3, "surface curve point");
    BRepBuilderAPI_MakeEdge edge(gp_Pnt(a[0], a[1], a[2]), gp_Pnt(b[0], b[1], b[2]));
    if (!edge.IsDone()) throw std::runtime_error("surface boundary edge");
    if (filling.Add(edge.Edge(), GeomAbs_C0, true) <= 0)
     throw std::runtime_error("surface boundary refused");
   }
  }
  filling.Build();
  if (!filling.IsDone()) throw std::runtime_error("surface filling failed");
  *deviation = filling.G0Error();
  return om9_emit_brep_snapshot(checked(filling.Shape(), "invalid filled surface"), deflection,
                                max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown surface failure");
  return 2;
 }
}

// A pipe/tube uses exact circular sections (Geom_Circle), so the measured volume of a straight
// pipe is pi r^2 h and not a polygon approximation. The stations (origin, tangent, in-plane x) are
// computed in Rust with rotation-minimizing frames; this adapter only builds the circles and lofts
// them.
extern "C" int om9_occt_tube(const double* stations, size_t station_count, double radius,
                             int solid, double deflection, size_t max_points, size_t max_nodes,
                             size_t max_bytes, Om9BrepSink sink, void* context, char* message,
                             int capacity) {
 try {
  if (!stations || station_count < 2) throw std::invalid_argument("tube needs two stations");
  require_positive(radius, "tube radius");
  BRepOffsetAPI_ThruSections tube(solid != 0, false, 1.0e-6);
  tube.CheckCompatibility(false);
  for (size_t i = 0; i < station_count; ++i) {
   const double* s = stations + 9 * i;
   require_finite(s, 9, "tube station");
   const gp_Pnt origin(s[0], s[1], s[2]);
   const gp_Dir tangent(s[3], s[4], s[5]);
   const gp_Dir x_axis(s[6], s[7], s[8]);
   const gp_Ax2 axes(origin, tangent, x_axis);
   Handle(Geom_Circle) circle = new Geom_Circle(axes, radius);
   BRepBuilderAPI_MakeEdge edge(circle);
   if (!edge.IsDone()) throw std::runtime_error("tube section edge");
   BRepBuilderAPI_MakeWire wire(edge.Edge());
   if (!wire.IsDone()) throw std::runtime_error("tube section wire");
   tube.AddWire(wire.Wire());
  }
  tube.Build();
  if (!tube.IsDone()) throw std::runtime_error("tube build failed");
  return om9_emit_brep_snapshot(checked(tube.Shape(), "invalid tube"), deflection, max_points,
                                max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown tube failure");
  return 2;
 }
}

// The per-station radii and frames are validated and owned by Rust. This bridge
// builds exact circular sections and delegates lofting to OCCT.
extern "C" int om9_occt_tube_radii(const double* stations, const double* radii,
                                   size_t station_count, int solid, int ruled, double deflection,
                                   size_t max_points, size_t max_nodes, size_t max_bytes,
                                   Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  if (!stations || !radii || station_count < 2) throw std::invalid_argument("tube needs two stations");
  BRepOffsetAPI_ThruSections tube(solid != 0, ruled != 0, 1.0e-6);
  tube.CheckCompatibility(false);
  for (size_t i = 0; i < station_count; ++i) {
   const double* s = stations + 9 * i;
   require_finite(s, 9, "tube station");
   require_positive(radii[i], "tube radius");
   const gp_Ax2 axes(gp_Pnt(s[0], s[1], s[2]), gp_Dir(s[3], s[4], s[5]), gp_Dir(s[6], s[7], s[8]));
   Handle(Geom_Circle) circle = new Geom_Circle(axes, radii[i]);
   BRepBuilderAPI_MakeEdge edge(circle);
   if (!edge.IsDone()) throw std::runtime_error("tube section edge");
   BRepBuilderAPI_MakeWire wire(edge.Edge());
   if (!wire.IsDone()) throw std::runtime_error("tube section wire");
   tube.AddWire(wire.Wire());
  }
  tube.Build();
  if (!tube.IsDone()) throw std::runtime_error("tube build failed");
  return om9_emit_brep_snapshot(checked(tube.Shape(), "invalid tube"), deflection, max_points,
                                max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown tube failure");
  return 2;
 }
}

// OCCT adapter for an oriented hemispherical solid used to cap an existing body.
extern "C" int om9_occt_hemisphere(const double* frame, double radius, int forward,
                                    double deflection, size_t max_points, size_t max_nodes,
                                    size_t max_bytes, Om9BrepSink sink, void* context,
                                    char* message, int capacity) {
 try {
  require_finite(frame, 6, "hemisphere frame");
  require_positive(radius, "hemisphere radius");
  const gp_Pnt center(frame[0], frame[1], frame[2]);
  const gp_Vec direction(frame[3], frame[4], frame[5]);
  if (direction.Magnitude() <= 1.0e-12) throw std::invalid_argument("hemisphere direction");
  const gp_Ax2 axis(center, gp_Dir(direction));
  const double half_pi = std::acos(-1.0) / 2.0;
  BRepPrimAPI_MakeSphere sphere(axis, radius, forward ? 0.0 : -half_pi, forward ? half_pi : 0.0);
  return om9_emit_brep_snapshot(checked(sphere.Shape(), "invalid hemisphere"), deflection,
                                max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what()); return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown hemisphere failure"); return 2;
 }
}

// OCCT adapter for a straight conical body with two exact hemispherical caps.
extern "C" int om9_occt_round_pipe(const double* endpoints, double r0, double r1,
                                    double deflection, size_t max_points, size_t max_nodes,
                                    size_t max_bytes, Om9BrepSink sink, void* context,
                                    char* message, int capacity) {
 try {
  require_finite(endpoints, 6, "round pipe endpoints");
  require_positive(r0, "round pipe start radius");
  require_positive(r1, "round pipe end radius");
  const gp_Pnt start(endpoints[0], endpoints[1], endpoints[2]);
  const gp_Pnt end(endpoints[3], endpoints[4], endpoints[5]);
  const gp_Vec direction(start, end);
  const double length = direction.Magnitude();
  if (length <= 1.0e-6) throw std::invalid_argument("round pipe rail length");
  const gp_Dir axis(direction);
  const gp_Ax2 a0(start, axis), a1(end, axis);
  const double half_pi = std::acos(-1.0) / 2.0;
  const TopoDS_Shape body = r0 == r1 ? BRepPrimAPI_MakeCylinder(a0, r0, length).Shape()
                                    : BRepPrimAPI_MakeCone(a0, r0, r1, length).Shape();
  BRepPrimAPI_MakeSphere first(a0, r0, -half_pi, 0.0);
  BRepPrimAPI_MakeSphere last(a1, r1, 0.0, half_pi);
  BRepAlgoAPI_Fuse start_join(body, first.Shape());
  start_join.Build();
  if (!start_join.IsDone()) throw std::runtime_error("round pipe start cap union failed");
  BRepAlgoAPI_Fuse end_join(start_join.Shape(), last.Shape());
  end_join.Build();
  if (!end_join.IsDone()) throw std::runtime_error("round pipe end cap union failed");
  return om9_emit_brep_snapshot(checked(end_join.Shape(), "invalid round pipe"), deflection,
                                max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown round pipe failure");
  return 2;
 }
}

// Circular sweep on an open polyline with mitered (right-corner) transitions.
extern "C" int om9_occt_pipe_miter(const double* path, size_t count, double radius,
                                    int solid, double deflection, size_t max_points,
                                    size_t max_nodes, size_t max_bytes, Om9BrepSink sink,
                                    void* context, char* message, int capacity) {
 try {
  require_finite(path, count * 3, "pipe path");
  require_positive(radius, "pipe radius");
  if (count < 2) throw std::invalid_argument("pipe path needs two points");
  const gp_Pnt start(path[0], path[1], path[2]);
  const gp_Pnt next(path[3], path[4], path[5]);
  const gp_Ax2 axes(start, gp_Dir(gp_Vec(start, next)));
  Handle(Geom_Circle) circle = new Geom_Circle(axes, radius);
  BRepBuilderAPI_MakeEdge edge(circle);
  if (!edge.IsDone()) throw std::runtime_error("pipe profile edge failed");
  BRepBuilderAPI_MakeWire section(edge.Edge());
  if (!section.IsDone()) throw std::runtime_error("pipe profile wire failed");
  BRepOffsetAPI_MakePipeShell pipe(make_open_wire(path, count));
  pipe.SetMode(false);
  pipe.SetTransitionMode(BRepBuilderAPI_RightCorner);
  pipe.Add(section.Wire(), false, false);
  pipe.Build();
  if (!pipe.IsDone()) throw std::runtime_error("pipe sweep failed");
  if (solid != 0 && !pipe.MakeSolid()) throw std::runtime_error("pipe cap failed");
  return om9_emit_brep_snapshot(checked(pipe.Shape(), "invalid pipe sweep"), deflection,
                                max_points, max_nodes, max_bytes, sink, context, message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown pipe sweep failure");
  return 2;
 }
}

// Ring builder family (`OM9-L2-04-03`, `OM9-L3-04-03-007`): sew planar quads into one
// closed solid. Rust has already validated the sections and their winding; this adapter
// refuses anything that does not sew into exactly one closed shell with a positive volume,
// so the OCCT body and the mesh body describe the same ring.
extern "C" int om9_occt_sew_solid(const double* points, size_t point_count, const int32_t* quads,
                                  size_t quad_count, double tolerance, double deflection,
                                  size_t max_points, size_t max_nodes, size_t max_bytes,
                                  Om9BrepSink sink, void* context, char* message, int capacity) {
 try {
  if (!points || point_count < 4) throw std::invalid_argument("sew solid needs four points");
  if (!quads || quad_count < 4) throw std::invalid_argument("sew solid needs four faces");
  require_finite(points, point_count * 3, "sew solid point");
  require_positive(tolerance, "sew tolerance");
  BRepBuilderAPI_Sewing sewing(tolerance);
  // A modulated section leaves a side quad slightly non-planar (the outer radius changes
  // between the two stations), and a planar face cannot be built from it. Splitting every
  // quad into two triangles keeps every face exactly planar while the surface, the volume
  // and the closed boundary stay those of the sampled sections.
  for (size_t q = 0; q < quad_count; ++q) {
   const int32_t* corner = quads + 4 * q;
   for (int half = 0; half < 2; ++half) {
    const int triangle[3] = {corner[0], corner[1 + half], corner[2 + half]};
    BRepBuilderAPI_MakePolygon polygon;
    for (int k = 0; k < 3; ++k) {
     if (triangle[k] < 0) throw std::invalid_argument("sew solid face index");
     const size_t vertex = static_cast<size_t>(triangle[k]);
     if (vertex >= point_count) throw std::invalid_argument("sew solid face index");
     const double* p = points + 3 * vertex;
     polygon.Add(gp_Pnt(p[0], p[1], p[2]));
    }
    polygon.Close();
    if (!polygon.IsDone()) throw std::runtime_error("sew solid face wire");
    BRepBuilderAPI_MakeFace face(polygon.Wire(), true);
    if (!face.IsDone()) throw std::runtime_error("sew solid face");
    sewing.Add(face.Face());
   }
  }
  sewing.Perform();
  const TopoDS_Shape sewn = sewing.SewedShape();
  if (sewn.IsNull()) throw std::runtime_error("sew solid produced no shape");
  size_t shells = 0;
  for (TopExp_Explorer explorer(sewn, TopAbs_SHELL); explorer.More(); explorer.Next()) ++shells;
  if (shells != 1) throw std::runtime_error("sew solid is not one shell");
  ShapeAnalysis_Shell shell_check;
  shell_check.LoadShells(sewn);
  if (shell_check.HasFreeEdges()) throw std::runtime_error("sew solid has free edges");
  TopoDS_Shell shell;
  for (TopExp_Explorer explorer(sewn, TopAbs_SHELL); explorer.More(); explorer.Next()) {
   shell = TopoDS::Shell(explorer.Current());
  }
  BRepBuilderAPI_MakeSolid solid(shell);
  if (!solid.IsDone()) throw std::runtime_error("sew solid failed");
  const TopoDS_Shape body = checked(solid.Solid(), "sew solid is not a valid solid");
  GProp_GProps properties;
  BRepGProp::VolumeProperties(body, properties);
  if (!(properties.Mass() > 0.0)) throw std::runtime_error("sew solid volume");
  return om9_emit_brep_snapshot(body, deflection, max_points, max_nodes, max_bytes, sink, context,
                                message, capacity);
 } catch (const Standard_Failure& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 2;
 } catch (const std::exception& e) {
  if (message && capacity > 0) std::snprintf(message, capacity, "%s", e.what());
  return 1;
 } catch (...) {
  if (message && capacity > 0) std::snprintf(message, capacity, "Unknown sew solid failure");
  return 2;
 }
}
