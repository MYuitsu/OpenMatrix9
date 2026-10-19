// SPDX-License-Identifier: LGPL-2.1-or-later
//! OCCT capability probe for the M00-L2-02 gate (`L3-01 Toolchain and C ABI bridge proof`).
//!
//! It links the pinned OCCT 8.0.1 release import libraries and prints one JSON object with the
//! measured facts the gate asks for: the version actually linked, B-Rep topology access, shape
//! validation, transform, tessellation, intersection, a boolean (the jewellery-critical operation
//! openNURBS cannot do), a volume property, and whether an OCCT failure is caught as a C++
//! exception. Nothing here touches the document, the app or the render layer.
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepBndLib.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRep_Tool.hxx>
#include <Bnd_Box.hxx>
#include <GProp_GProps.hxx>
#include <IntCurvesFace_ShapeIntersector.hxx>
#include <Poly_Triangulation.hxx>
#include <Standard_Failure.hxx>
#include <Standard_Version.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Lin.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>

#include <cstdio>

namespace {

void number(const char* key, double value, bool last = false) {
  std::printf("\"%s\": %.17g%s", key, value, last ? "" : ", ");
}

void integer(const char* key, long long value, bool last = false) {
  std::printf("\"%s\": %lld%s", key, value, last ? "" : ", ");
}

void boolean(const char* key, bool value, bool last = false) {
  std::printf("\"%s\": %s%s", key, value ? "true" : "false", last ? "" : ", ");
}

void text(const char* key, const char* value, bool last = false) {
  std::printf("\"%s\": \"%s\"%s", key, value, last ? "" : ", ");
}

void point(const char* key, const gp_Pnt& p, bool last = false) {
  std::printf("\"%s\": [%.17g, %.17g, %.17g]%s", key, p.X(), p.Y(), p.Z(), last ? "" : ", ");
}

int count(const TopoDS_Shape& shape, TopAbs_ShapeEnum type) {
  int total = 0;
  for (TopExp_Explorer explorer(shape, type); explorer.More(); explorer.Next()) {
    ++total;
  }
  return total;
}

}  // namespace

int main() {
  std::printf("{");
  text("status", "ok", false);
  text("kernel", "occt", false);
  text("version", OCC_VERSION_COMPLETE, false);
  integer("version_major", OCC_VERSION_MAJOR, false);
  integer("version_minor", OCC_VERSION_MINOR, false);
  integer("version_maintenance", OCC_VERSION_MAINTENANCE, false);

  // --- shape creation + topology (L3-002/L3-005 style access) --------------
  TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), 10.0, 10.0, 4.0).Shape();
  boolean("box_made", !box.IsNull(), false);
  integer("box_faces", count(box, TopAbs_FACE), false);
  integer("box_edges", count(box, TopAbs_EDGE), false);
  integer("box_vertices", count(box, TopAbs_VERTEX), false);
  integer("box_wires", count(box, TopAbs_WIRE), false);
  boolean("box_valid", BRepCheck_Analyzer(box).IsValid(), false);

  // --- transform ----------------------------------------------------------
  gp_Trsf move;
  move.SetTranslation(gp_Vec(0, 0, 5));
  TopoDS_Shape moved = BRepBuilderAPI_Transform(box, move, true).Shape();
  Bnd_Box bounds;
  BRepBndLib::Add(moved, bounds);
  number("moved_min_z", bounds.CornerMin().Z(), false);
  number("moved_max_z", bounds.CornerMax().Z(), false);

  // --- tessellation -------------------------------------------------------
  BRepMesh_IncrementalMesh mesher(moved, 0.01, false, 0.5, true);
  long long nodes = 0;
  long long triangles = 0;
  for (TopExp_Explorer explorer(moved, TopAbs_FACE); explorer.More(); explorer.Next()) {
    const TopoDS_Face& face = TopoDS::Face(explorer.Current());
    TopLoc_Location location;
    const Handle(Poly_Triangulation)& mesh = BRep_Tool::Triangulation(face, location);
    if (!mesh.IsNull()) {
      nodes += mesh->NbNodes();
      triangles += mesh->NbTriangles();
    }
  }
  boolean("mesh_done", mesher.IsDone(), false);
  integer("mesh_nodes", nodes, false);
  integer("mesh_triangles", triangles, false);

  // --- intersection -------------------------------------------------------
  gp_Lin axis(gp_Pnt(5, 5, -10), gp_Dir(0, 0, 1));
  IntCurvesFace_ShapeIntersector intersector;
  intersector.Load(moved, 1e-7);
  intersector.Perform(axis, -1e6, 1e6);
  integer("line_face_hits", static_cast<long long>(intersector.NbPnt()), false);
  if (intersector.NbPnt() > 0) {
    point("first_hit", intersector.Pnt(1), false);
    number("first_hit_parameter", intersector.WParameter(1), false);
  }

  // --- boolean (the operation openNURBS does not provide) -----------------
  gp_Ax2 drill_axis(gp_Pnt(5, 5, -1), gp_Dir(0, 0, 1));
  TopoDS_Shape drill = BRepPrimAPI_MakeCylinder(drill_axis, 2.0, 6.0).Shape();
  TopoDS_Shape cut = BRepAlgoAPI_Cut(box, drill).Shape();
  boolean("boolean_done", !cut.IsNull(), false);
  boolean("boolean_valid", !cut.IsNull() && BRepCheck_Analyzer(cut).IsValid(), false);
  integer("boolean_faces", count(cut, TopAbs_FACE), false);

  // --- volume property ----------------------------------------------------
  GProp_GProps box_properties;
  BRepGProp::VolumeProperties(box, box_properties);
  number("box_volume", box_properties.Mass(), false);
  GProp_GProps cut_properties;
  BRepGProp::VolumeProperties(cut, cut_properties);
  number("cut_volume", cut_properties.Mass(), false);

  // --- exception contract ------------------------------------------------
  // A negative-radius cylinder is invalid input; the gate needs to know that the failure comes
  // back as a catchable C++ exception instead of terminating the process.
  bool caught = false;
  const char* caught_type = "none";
  try {
    // A negative radius is invalid input; OCCT 8 raises a Standard_Failure derived from
    // std::exception.
    TopoDS_Shape invalid = BRepPrimAPI_MakeCylinder(-1.0, 1.0).Shape();
    (void)invalid;
    caught = false;
  } catch (const Standard_Failure& failure) {
    caught = true;
    caught_type = failure.ExceptionType();
  } catch (const std::exception& other) {
    (void)other;
    caught = true;
    caught_type = "std::exception";
  }
  boolean("invalid_input_caught", caught, false);
  text("invalid_input_type", caught_type, true);
  std::printf("}\n");
  return 0;
}
