// SPDX-License-Identifier: LGPL-2.1-or-later
//! C ABI over OCCT 8.0.1 for the Rust side.
//!
//! Every function is a complete operation: no `TopoDS_Shape`, `Handle`, or OCCT pointer escapes, so the
//! Rust caller never needs to reason about OCCT lifetimes or reference counts. Failures are reported
//! as a status code plus a fixed-size message buffer; a C++ exception (including the OCCT failure
//! hierarchy, which derives from `std::exception` since 8.0) is caught here and never crosses the ABI.
#include <BRepAlgoAPI_Cut.hxx>
#include <BRep_Tool.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS_Vertex.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <GProp_GProps.hxx>
#include <Standard_Failure.hxx>
#include <Standard_Version.hxx>
#include <TopoDS_Shape.hxx>
#include <BRepBndLib.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <Bnd_Box.hxx>
#include <Geom_BSplineCurve.hxx>
#include <NCollection_Array1.hxx>
#include <Geom_SphericalSurface.hxx>
#include <IntCurvesFace_ShapeIntersector.hxx>
#include <gp_Ax1.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepOffsetAPI_MakeOffsetShape.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <Geom_Circle.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <Poly_Triangle.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>

#include <windows.h>
#include <psapi.h>

#include <chrono>
#include <array>
#include <fstream>
#include <string>
#include <vector>
#include <map>
#include <utility>
#include <cmath>
#include <cstring>
#include <exception>

namespace {

constexpr int kOk = 0;
constexpr int kInvalidInput = 1;
constexpr int kKernelFailure = 2;

/// Copies `text` into the caller buffer without ever writing past `capacity`.
void write_message(char* buffer, int capacity, const char* text) {
  if (buffer == nullptr || capacity <= 0) {
    return;
  }
  std::strncpy(buffer, text, static_cast<size_t>(capacity) - 1);
  buffer[capacity - 1] = '\0';
}

/// Reports an OCCT failure: `what()` when the exception carries a message, otherwise the OCCT type
/// name (for example `Standard_ConstructionError`), so the ABI never returns an empty explanation.
int fail_with(const Standard_Failure& failure, char* message, int capacity) {
  const char* text = failure.what();
  if (text == nullptr || *text == '\0') {
    text = failure.ExceptionType();
  }
  write_message(message, capacity, text);
  return kKernelFailure;
}

}  // namespace

extern "C" {

/// Version actually linked, e.g. `8.0.1`.
const char* om9_occt_version() {
  return OCC_VERSION_COMPLETE;
}

/// Volume of a box `width x height x depth` starting at the origin, in model units cubed.
int om9_occt_box_volume(double width, double height, double depth, double* volume, char* message, int capacity) {
  if (volume == nullptr) {
    write_message(message, capacity, "null volume pointer");
    return kInvalidInput;
  }
  if (!std::isfinite(width) || !std::isfinite(height) || !std::isfinite(depth) || width <= 0.0 ||
      height <= 0.0 || depth <= 0.0) {
    write_message(message, capacity, "box dimensions must be finite and positive");
    return kInvalidInput;
  }
  try {
    const TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), width, height, depth).Shape();
    if (box.IsNull() || !BRepCheck_Analyzer(box).IsValid()) {
      write_message(message, capacity, "box is not a valid solid");
      return kKernelFailure;
    }
    GProp_GProps properties;
    BRepGProp::VolumeProperties(box, properties);
    *volume = properties.Mass();
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

/// Volume of the box after drilling a `radius` cylinder through its whole depth along Z.
int om9_occt_drilled_volume(double width, double height, double depth, double radius, double* volume,
                            char* message, int capacity) {
  if (volume == nullptr) {
    write_message(message, capacity, "null volume pointer");
    return kInvalidInput;
  }
  if (!std::isfinite(width) || !std::isfinite(height) || !std::isfinite(depth) ||
      !std::isfinite(radius) || width <= 0.0 || height <= 0.0 || depth <= 0.0 || radius <= 0.0) {
    write_message(message, capacity, "dimensions must be finite and positive");
    return kInvalidInput;
  }
  try {
    const TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), width, height, depth).Shape();
    const gp_Ax2 axis(gp_Pnt(width / 2.0, height / 2.0, -1.0), gp_Dir(0, 0, 1));
    const TopoDS_Shape drill = BRepPrimAPI_MakeCylinder(axis, radius, depth + 2.0).Shape();
    const TopoDS_Shape cut = BRepAlgoAPI_Cut(box, drill).Shape();
    if (cut.IsNull() || !BRepCheck_Analyzer(cut).IsValid()) {
      write_message(message, capacity, "boolean result is not a valid solid");
      return kKernelFailure;
    }
    GProp_GProps properties;
    BRepGProp::VolumeProperties(cut, properties);
    *volume = properties.Mass();
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

/// Deliberately invalid kernel input: proves that an OCCT failure becomes a status code instead of
/// terminating the process.
int om9_occt_invalid_cylinder(double radius, char* message, int capacity) {
  try {
    const gp_Ax2 axis(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1));
    const TopoDS_Shape shape = BRepPrimAPI_MakeCylinder(axis, radius, 1.0).Shape();
    if (shape.IsNull()) {
      write_message(message, capacity, "a non-positive radius produced a null shape");
      return kKernelFailure;
    }
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}


/// Working set of this process in bytes, so a create/destroy loop can be checked for growth.
int om9_occt_working_set(double* bytes) {
  if (bytes == nullptr) {
    return kInvalidInput;
  }
  PROCESS_MEMORY_COUNTERS info;
  if (!GetProcessMemoryInfo(GetCurrentProcess(), &info, sizeof(info))) {
    return kKernelFailure;
  }
  *bytes = static_cast<double>(info.WorkingSetSize);
  return kOk;
}

/// Reference counting on a kernel handle: a copy raises the count and `Nullify()` restores it, which is
/// the RAII path every kernel pointer the bridge touches must follow.
int om9_occt_handle_refcount(int* before, int* copied, int* after, char* message, int capacity) {
  if (before == nullptr || copied == nullptr || after == nullptr) {
    write_message(message, capacity, "null refcount pointer");
    return kInvalidInput;
  }
  try {
    Handle(Geom_Circle) circle = new Geom_Circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 1.0);
    *before = circle->GetRefCount();
    Handle(Geom_Circle) copy = circle;
    *copied = circle->GetRefCount();
    copy.Nullify();
    *after = circle->GetRefCount();
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

/// Create and destroy `count` boxes, returning the elapsed seconds. Every shape is released at the end of
/// its iteration, so a leak shows up as working-set growth in the caller's before/after comparison.
int om9_occt_create_destroy(long long count, double* elapsed_seconds, char* message, int capacity) {
  if (elapsed_seconds == nullptr || count < 0) {
    write_message(message, capacity, "invalid create/destroy argument");
    return kInvalidInput;
  }
  try {
    const auto started = std::chrono::steady_clock::now();
    for (long long index = 0; index < count; ++index) {
      const TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), 10.0, 10.0, 4.0).Shape();
      GProp_GProps properties;
      BRepGProp::VolumeProperties(box, properties);
      if (properties.Mass() < 399.0 || properties.Mass() > 401.0) {
        write_message(message, capacity, "create/destroy produced a wrong volume");
        return kKernelFailure;
      }
    }
    const auto finished = std::chrono::steady_clock::now();
    *elapsed_seconds = std::chrono::duration<double>(finished - started).count();
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}


/// Volume and area of a primitive, measured through the kernel's own properties.
int measure(const TopoDS_Shape& shape, double* volume, double* area) {
  if (shape.IsNull() || !BRepCheck_Analyzer(shape).IsValid()) {
    return kKernelFailure;
  }
  GProp_GProps volume_properties;
  BRepGProp::VolumeProperties(shape, volume_properties);
  if (volume != nullptr) {
    *volume = volume_properties.Mass();
  }
  if (area != nullptr) {
    GProp_GProps area_properties;
    BRepGProp::SurfaceProperties(shape, area_properties);
    *area = area_properties.Mass();
  }
  return kOk;
}

int om9_occt_box_measures(double width, double height, double depth, double* volume, double* area,
                          char* message, int capacity) {
  try {
    if (!(width > 0.0) || !(height > 0.0) || !(depth > 0.0)) {
      write_message(message, capacity, "box dimensions must be positive");
      return kInvalidInput;
    }
    const TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), width, height, depth).Shape();
    const int status = measure(box, volume, area);
    if (status != kOk) {
      write_message(message, capacity, "box is not a valid solid");
    }
    return status;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

int om9_occt_sphere_measures(double radius, double* volume, double* area, char* message, int capacity) {
  try {
    if (!(radius > 0.0)) {
      write_message(message, capacity, "sphere radius must be positive");
      return kInvalidInput;
    }
    const TopoDS_Shape sphere = BRepPrimAPI_MakeSphere(radius).Shape();
    const int status = measure(sphere, volume, area);
    if (status != kOk) {
      write_message(message, capacity, "sphere is not a valid solid");
    }
    return status;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

int om9_occt_cylinder_measures(double radius, double height, double* volume, double* area,
                               char* message, int capacity) {
  try {
    if (!(radius > 0.0) || !(height > 0.0)) {
      write_message(message, capacity, "cylinder dimensions must be positive");
      return kInvalidInput;
    }
    const gp_Ax2 axis(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1));
    const TopoDS_Shape cylinder = BRepPrimAPI_MakeCylinder(axis, radius, height).Shape();
    const int status = measure(cylinder, volume, area);
    if (status != kOk) {
      write_message(message, capacity, "cylinder is not a valid solid");
    }
    return status;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

/// Fillet the vertical edge at (width, 0) of a box; returns the volume and whether the result is valid.
int om9_occt_fillet_volume(double width, double height, double depth, double radius, double* volume,
                           int* valid, char* message, int capacity) {
  try {
    const TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), width, height, depth).Shape();
    BRepFilletAPI_MakeFillet fillet(box);
    int filleted = 0;
    for (TopExp_Explorer explorer(box, TopAbs_EDGE); explorer.More(); explorer.Next()) {
      const TopoDS_Edge& edge = TopoDS::Edge(explorer.Current());
      TopExp_Explorer vertices(edge, TopAbs_VERTEX);
      gp_Pnt first, second;
      int count = 0;
      for (; vertices.More() && count < 2; vertices.Next(), ++count) {
        const TopoDS_Vertex& vertex = TopoDS::Vertex(vertices.Current());
        const gp_Pnt point = BRep_Tool::Pnt(vertex);
        (count == 0 ? first : second) = point;
      }
      const bool vertical = count == 2 && std::fabs(first.X() - second.X()) < 1e-9 &&
                            std::fabs(first.Y() - second.Y()) < 1e-9 &&
                            std::fabs(first.X() - width) < 1e-9 && std::fabs(first.Y()) < 1e-9;
      if (vertical) {
        fillet.Add(radius, edge);
        ++filleted;
      }
    }
    if (filleted == 0) {
      write_message(message, capacity, "fillet edge not found");
      return kKernelFailure;
    }
    const TopoDS_Shape result = fillet.Shape();
    *valid = (fillet.IsDone() && !result.IsNull() && BRepCheck_Analyzer(result).IsValid()) ? 1 : 0;
    if (volume != nullptr) {
      GProp_GProps properties;
      BRepGProp::VolumeProperties(result, properties);
      *volume = properties.Mass();
    }
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

/// Offset a box inward by `distance` (negative) and return the resulting volume.
int om9_occt_offset_volume(double width, double height, double depth, double distance, double* volume,
                           int* valid, char* message, int capacity) {
  try {
    const TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), width, height, depth).Shape();
    BRepOffsetAPI_MakeOffsetShape offset;
    offset.PerformByJoin(box, distance, 1e-6);
    const TopoDS_Shape result = offset.Shape();
    *valid = (offset.IsDone() && !result.IsNull() && BRepCheck_Analyzer(result).IsValid()) ? 1 : 0;
    if (volume != nullptr) {
      GProp_GProps properties;
      BRepGProp::VolumeProperties(result, properties);
      *volume = properties.Mass();
    }
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

/// Tessellate a sphere and report the largest distance from a mesh node to the exact sphere, the triangle
/// count, and whether every edge is shared by exactly two triangles.
int om9_occt_sphere_mesh(double radius, double tolerance, double* max_deviation, long long* triangles,
                         long long* boundary_edges, long long* non_manifold_edges, char* message,
                         int capacity) {
  long long histogram[3] = {0, 0, 0};
  long long* edge_total = &histogram[0];
  long long* edge_single = &histogram[1];
  long long* edge_more = &histogram[2];
  if (max_deviation == nullptr || triangles == nullptr || boundary_edges == nullptr ||
      non_manifold_edges == nullptr) {
    write_message(message, capacity, "null mesh output pointer");
    return kInvalidInput;
  }
  try {
    const TopoDS_Shape sphere = BRepPrimAPI_MakeSphere(radius).Shape();
    if (sphere.IsNull()) {
      write_message(message, capacity, "sphere is null");
      return kKernelFailure;
    }
    BRepMesh_IncrementalMesh mesher(sphere, tolerance, false, 0.5, true);
    if (!mesher.IsDone()) {
      write_message(message, capacity, "mesh failed");
      return kKernelFailure;
    }
    *max_deviation = 0.0;
    *triangles = 0;
    // Merged across the whole solid: a periodic face repeats its seam vertices, so a seam edge must be
    // counted through the position key rather than per face.
    std::map<std::pair<std::array<long long, 3>, std::array<long long, 3>>, int> edges;
    for (TopExp_Explorer explorer(sphere, TopAbs_FACE); explorer.More(); explorer.Next()) {
      const TopoDS_Face& face = TopoDS::Face(explorer.Current());
      TopLoc_Location location;
      const Handle(Poly_Triangulation)& mesh = BRep_Tool::Triangulation(face, location);
      if (mesh.IsNull()) {
        continue;
      }
      *triangles += mesh->NbTriangles();
      const gp_Trsf transform = location.Transformation();
      auto node_key = [&mesh, &transform](int index) {
        const gp_Pnt point = mesh->Node(index).Transformed(transform);
        constexpr double scale = 1e7;
        return std::array<long long, 3>{std::llround(point.X() * scale),
                                        std::llround(point.Y() * scale),
                                        std::llround(point.Z() * scale)};
      };
      for (int index = 1; index <= mesh->NbNodes(); ++index) {
        const gp_Pnt point = mesh->Node(index).Transformed(transform);
        const double deviation = std::fabs(point.Distance(gp_Pnt(0, 0, 0)) - radius);
        if (deviation > *max_deviation) {
          *max_deviation = deviation;
        }
      }
      for (int index = 1; index <= mesh->NbTriangles(); ++index) {
        int a = 0, b = 0, c = 0;
        mesh->Triangle(index).Get(a, b, c);
        const int nodes[3][2] = {{a, b}, {b, c}, {c, a}};
        for (const auto& pair : nodes) {
          auto first = node_key(pair[0]);
          auto second = node_key(pair[1]);
          if (first == second) {
            // A triangle at a pole has two vertices on the pole; that collapsed edge is not a boundary.
            continue;
          }
          if (second < first) {
            std::swap(first, second);
          }
          ++edges[std::make_pair(first, second)];
        }
      }
    }
    long long boundary = 0;
    long long non_manifold = 0;
    for (const auto& entry : edges) {
      if (entry.second == 1) {
        ++boundary;
      } else if (entry.second != 2) {
        ++non_manifold;
      }
    }
    // A mesh is closed when it has no boundary edge. OCCT's sphere mesh leaves a non-manifold edge at
    // each pole (the fan closes over a duplicated pole node); that is reported separately instead of
    // being folded into a single "watertight" flag that would hide which property failed.
    *boundary_edges = boundary;
    *non_manifold_edges = non_manifold;
    std::fprintf(stderr, "[mesh] tolerance=%.6g edges=%lld boundary=%lld non_manifold=%lld\n", tolerance,
                 static_cast<long long>(edges.size()), boundary, non_manifold);
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

/// Degenerate cases: `kind` 0 = cut a box with a touching box, 1 = offset inward past the wall,
/// 2 = a box whose size is at the kernel's resolution.
int om9_occt_degenerate(int kind, int* valid, double* volume, char* message, int capacity) {
  try {
    *valid = 0;
    *volume = 0.0;
    if (kind == 0) {
      const TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), 10.0, 10.0, 4.0).Shape();
      const TopoDS_Shape neighbour = BRepPrimAPI_MakeBox(gp_Pnt(10.0, 0, 0), 10.0, 10.0, 4.0).Shape();
      const TopoDS_Shape cut = BRepAlgoAPI_Cut(box, neighbour).Shape();
      *valid = (!cut.IsNull() && BRepCheck_Analyzer(cut).IsValid()) ? 1 : 0;
      GProp_GProps properties;
      BRepGProp::VolumeProperties(cut, properties);
      *volume = properties.Mass();
      return kOk;
    }
    if (kind == 1) {
      const TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), 10.0, 10.0, 4.0).Shape();
      BRepOffsetAPI_MakeOffsetShape offset;
      offset.PerformByJoin(box, -10.0, 1e-6);
      const TopoDS_Shape result = offset.Shape();
      *valid = (offset.IsDone() && !result.IsNull() && BRepCheck_Analyzer(result).IsValid()) ? 1 : 0;
      if (*valid) {
        GProp_GProps properties;
        BRepGProp::VolumeProperties(result, properties);
        *volume = properties.Mass();
      }
      return kOk;
    }
    const TopoDS_Shape tiny = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), 1e-12, 1e-12, 1e-12).Shape();
    *valid = (!tiny.IsNull() && BRepCheck_Analyzer(tiny).IsValid()) ? 1 : 0;
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}


/// Tessellate a sphere and write the triangles as text (`v x y z` then `f a b c`, 1-based), so the GPU
/// probe can upload exactly what the kernel produced without linking OCCT into the renderer.
int om9_occt_sphere_mesh_dump(double radius, double tolerance, const char* path, long long* vertices,
                              long long* triangles, char* message, int capacity) {
  if (path == nullptr || vertices == nullptr || triangles == nullptr) {
    write_message(message, capacity, "null dump argument");
    return kInvalidInput;
  }
  try {
    const TopoDS_Shape sphere = BRepPrimAPI_MakeSphere(radius).Shape();
    BRepMesh_IncrementalMesh mesher(sphere, tolerance, false, 0.5, true);
    if (!mesher.IsDone()) {
      write_message(message, capacity, "mesh failed");
      return kKernelFailure;
    }
    std::ofstream out(path, std::ios::binary);
    if (!out) {
      write_message(message, capacity, "cannot open the mesh output");
      return kKernelFailure;
    }
    std::vector<std::array<double, 3>> positions;
    std::vector<std::array<int, 3>> faces;
    for (TopExp_Explorer explorer(sphere, TopAbs_FACE); explorer.More(); explorer.Next()) {
      const TopoDS_Face& face = TopoDS::Face(explorer.Current());
      TopLoc_Location location;
      const Handle(Poly_Triangulation)& mesh = BRep_Tool::Triangulation(face, location);
      if (mesh.IsNull()) {
        continue;
      }
      const gp_Trsf transform = location.Transformation();
      const int base = static_cast<int>(positions.size());
      for (int index = 1; index <= mesh->NbNodes(); ++index) {
        const gp_Pnt point = mesh->Node(index).Transformed(transform);
        positions.push_back({point.X(), point.Y(), point.Z()});
      }
      for (int index = 1; index <= mesh->NbTriangles(); ++index) {
        int a = 0, b = 0, c = 0;
        mesh->Triangle(index).Get(a, b, c);
        faces.push_back({base + a, base + b, base + c});
      }
    }
    out << "# om9 mesh from OCCT " << OCC_VERSION_COMPLETE << "\n";
    for (const auto& position : positions) {
      out << "v " << position[0] << ' ' << position[1] << ' ' << position[2] << '\n';
    }
    for (const auto& face : faces) {
      out << "f " << face[0] << ' ' << face[1] << ' ' << face[2] << '\n';
    }
    out.close();
    *vertices = static_cast<long long>(positions.size());
    *triangles = static_cast<long long>(faces.size());
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}


/// Topology counts and validity of a primitive: kind 0 box, 1 sphere, 2 cylinder.
int om9_occt_topology(int kind, long long* vertices, long long* edges, long long* faces, long long* wires,
                      int* valid, char* message, int capacity) {
  if (vertices == nullptr || edges == nullptr || faces == nullptr || wires == nullptr || valid == nullptr) {
    write_message(message, capacity, "null topology pointer");
    return kInvalidInput;
  }
  try {
    TopoDS_Shape shape;
    if (kind == 0) {
      shape = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), 10.0, 10.0, 4.0).Shape();
    } else if (kind == 1) {
      shape = BRepPrimAPI_MakeSphere(5.0).Shape();
    } else if (kind == 2) {
      shape = BRepPrimAPI_MakeCylinder(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 2.0, 4.0).Shape();
    } else {
      write_message(message, capacity, "unknown primitive kind");
      return kInvalidInput;
    }
    // Unique sub-shapes, not per-face occurrences: the explorer would count a shared edge twice.
    auto count = [&shape](TopAbs_ShapeEnum type) {
      TopTools_IndexedMapOfShape map;
      TopExp::MapShapes(shape, type, map);
      return static_cast<long long>(map.Extent());
    };
    *vertices = count(TopAbs_VERTEX);
    *edges = count(TopAbs_EDGE);
    *faces = count(TopAbs_FACE);
    *wires = count(TopAbs_WIRE);
    *valid = BRepCheck_Analyzer(shape).IsValid() ? 1 : 0;
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

/// Box transform service: translate, rotate about Z, then scale; reports the resulting box and volume.
int om9_occt_transform_probe(double dx, double dy, double dz, double rotate_deg, double scale,
                             double* min_x, double* max_x, double* min_z, double* max_z, double* volume,
                             int* valid, char* message, int capacity) {
  try {
    const TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), 10.0, 10.0, 4.0).Shape();
    gp_Trsf translation;
    translation.SetTranslation(gp_Vec(dx, dy, dz));
    gp_Trsf rotation;
    rotation.SetRotation(gp_Ax1(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), rotate_deg * M_PI / 180.0);
    gp_Trsf scaling;
    scaling.SetScaleFactor(scale);
    gp_Trsf combined = scaling * rotation * translation;
    const TopoDS_Shape moved = BRepBuilderAPI_Transform(box, combined, true).Shape();
    *valid = (!moved.IsNull() && BRepCheck_Analyzer(moved).IsValid()) ? 1 : 0;
    Bnd_Box bounds;
    bounds.SetGap(0.0);
    BRepBndLib::Add(moved, bounds);
    *min_x = bounds.CornerMin().X();
    *max_x = bounds.CornerMax().X();
    *min_z = bounds.CornerMin().Z();
    *max_z = bounds.CornerMax().Z();
    GProp_GProps properties;
    BRepGProp::VolumeProperties(moved, properties);
    *volume = properties.Mass();
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

/// Evaluate a rational NURBS quarter circle of radius 5 at `u`: point and first derivative.
///
/// The curve is built from poles, weights, knots and multiplicities directly, so the measurement is of
/// `Geom_BSplineCurve` itself rather than of a conversion helper.
int om9_occt_curve_eval(double u, double* px, double* py, double* pz, double* dx, double* dy, double* dz,
                        char* message, int capacity) {
  try {
    NCollection_Array1<gp_Pnt> poles(1, 3);
    poles.SetValue(1, gp_Pnt(5.0, 0.0, 0.0));
    poles.SetValue(2, gp_Pnt(5.0, 5.0, 0.0));
    poles.SetValue(3, gp_Pnt(0.0, 5.0, 0.0));
    NCollection_Array1<double> weights(1, 3);
    weights.SetValue(1, 1.0);
    weights.SetValue(2, 1.0 / std::sqrt(2.0));
    weights.SetValue(3, 1.0);
    NCollection_Array1<double> knots(1, 3);
    knots.SetValue(1, 0.0);
    knots.SetValue(2, 0.0);
    knots.SetValue(3, 0.0);
    NCollection_Array1<double> knots_high(1, 3);
    knots_high.SetValue(1, 1.0);
    knots_high.SetValue(2, 1.0);
    knots_high.SetValue(3, 1.0);
    // A single knot vector is required: [0,0,0,1,1,1] with multiplicities [3,3] at a degree-2 curve.
    NCollection_Array1<double> knots_all(1, 2);
    knots_all.SetValue(1, 0.0);
    knots_all.SetValue(2, 1.0);
    NCollection_Array1<int> multiplicities(1, 2);
    multiplicities.SetValue(1, 3);
    multiplicities.SetValue(2, 3);
    const Handle(Geom_BSplineCurve) curve =
      new Geom_BSplineCurve(poles, weights, knots_all, multiplicities, 2);
    if (curve.IsNull()) {
      write_message(message, capacity, "curve is not valid");
      return kKernelFailure;
    }
    gp_Pnt point;
    gp_Vec first;
    curve->D1(u, point, first);
    *px = point.X();
    *py = point.Y();
    *pz = point.Z();
    *dx = first.X();
    *dy = first.Y();
    *dz = first.Z();
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

/// Evaluate a radius-5 sphere at (u,v): point and unit normal.
int om9_occt_surface_eval(double u, double v, double* px, double* py, double* pz, double* nx, double* ny,
                          double* nz, char* message, int capacity) {
  try {
    Handle(Geom_SphericalSurface) sphere =
      new Geom_SphericalSurface(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 5.0);
    gp_Pnt point;
    gp_Vec du, dv;
    sphere->D1(u, v, point, du, dv);
    const gp_Vec normal = du.Crossed(dv).Normalized();
    *px = point.X();
    *py = point.Y();
    *pz = point.Z();
    *nx = normal.X();
    *ny = normal.Y();
    *nz = normal.Z();
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

/// A line through the box along +Z: number of face hits and the first one.
int om9_occt_line_face_hits(long long* hits, double* first_x, double* first_y, double* first_z,
                            char* message, int capacity) {
  try {
    const TopoDS_Shape box = BRepPrimAPI_MakeBox(gp_Pnt(0, 0, 0), 10.0, 10.0, 4.0).Shape();
    IntCurvesFace_ShapeIntersector intersector;
    intersector.Load(box, 1e-7);
    intersector.Perform(gp_Lin(gp_Pnt(5, 5, -10), gp_Dir(0, 0, 1)), -1e6, 1e6);
    *hits = static_cast<long long>(intersector.NbPnt());
    if (*hits > 0) {
      const gp_Pnt point = intersector.Pnt(1);
      *first_x = point.X();
      *first_y = point.Y();
      *first_z = point.Z();
    }
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}


/// Tessellate a sphere into caller buffers: a first call with null buffers reports the counts, a second
/// fills them. Positions are `x,y,z` floats, indices are triangle corners, 0-based.
int om9_occt_sphere_triangles(double radius, double tolerance, long long* vertex_count,
                              long long* index_count, float* positions, unsigned int* indices,
                              char* message, int capacity) {
  if (vertex_count == nullptr || index_count == nullptr) {
    write_message(message, capacity, "null mesh count pointer");
    return kInvalidInput;
  }
  try {
    const TopoDS_Shape sphere = BRepPrimAPI_MakeSphere(radius).Shape();
    BRepMesh_IncrementalMesh mesher(sphere, tolerance, false, 0.5, true);
    if (!mesher.IsDone()) {
      write_message(message, capacity, "mesh failed");
      return kKernelFailure;
    }
    std::vector<std::array<double, 3>> points;
    std::vector<std::array<unsigned int, 3>> triangles;
    for (TopExp_Explorer explorer(sphere, TopAbs_FACE); explorer.More(); explorer.Next()) {
      const TopoDS_Face& face = TopoDS::Face(explorer.Current());
      TopLoc_Location location;
      const Handle(Poly_Triangulation)& mesh = BRep_Tool::Triangulation(face, location);
      if (mesh.IsNull()) {
        continue;
      }
      const gp_Trsf transform = location.Transformation();
      const unsigned int base = static_cast<unsigned int>(points.size());
      for (int index = 1; index <= mesh->NbNodes(); ++index) {
        const gp_Pnt point = mesh->Node(index).Transformed(transform);
        points.push_back({point.X(), point.Y(), point.Z()});
      }
      for (int index = 1; index <= mesh->NbTriangles(); ++index) {
        int a = 0, b = 0, c = 0;
        mesh->Triangle(index).Get(a, b, c);
        triangles.push_back({base + static_cast<unsigned int>(a - 1), base + static_cast<unsigned int>(b - 1),
                             base + static_cast<unsigned int>(c - 1)});
      }
    }
    *vertex_count = static_cast<long long>(points.size());
    *index_count = static_cast<long long>(triangles.size() * 3);
    if (positions != nullptr && indices != nullptr) {
      for (size_t index = 0; index < points.size(); ++index) {
        positions[index * 3 + 0] = static_cast<float>(points[index][0]);
        positions[index * 3 + 1] = static_cast<float>(points[index][1]);
        positions[index * 3 + 2] = static_cast<float>(points[index][2]);
      }
      for (size_t index = 0; index < triangles.size(); ++index) {
        indices[index * 3 + 0] = triangles[index][0];
        indices[index * 3 + 1] = triangles[index][1];
        indices[index * 3 + 2] = triangles[index][2];
      }
    }
    return kOk;
  } catch (const Standard_Failure& failure) {
    return fail_with(failure, message, capacity);
  } catch (const std::exception& failure) {
    write_message(message, capacity, failure.what());
    return kKernelFailure;
  }
}

}  // extern "C"
