//! C ABI over the pinned openNURBS: write a fixture archive and read its meshes.
//!
//! Counts first, then one fill call; the caller allocates and this file only copies. Any failure is a
//! code plus a message, never an exception across the boundary.

#include <cstdint>
#include <cstdio>
#include <cmath>
#include <cstring>
#include <memory>
#include <mutex>
#include <string>
#include <vector>

#include "opennurbs.h"

namespace {

constexpr double kPi = 3.14159265358979323846;
constexpr int kOk = 0;
constexpr int kInvalidInput = 1;
constexpr int kKernelFailure = 2;

void write_message(char* message, int capacity, const char* text) {
  if (message == nullptr || capacity <= 0) {
    return;
  }
  std::snprintf(message, static_cast<size_t>(capacity), "%s", text);
}

/// openNURBS needs one explicit begin/end around its global state.
struct Kernel {
  Kernel() { ON::Begin(); }
  ~Kernel() { ON::End(); }
};

void ensure_kernel() {
  static std::once_flag once;
  std::call_once(once, [] { static Kernel kernel; });
}

/// A unit cube as an ON_Mesh: eight vertices and twelve triangles, the shape this pinned openNURBS
/// stores (it has no quad array).
std::unique_ptr<ON_Mesh> make_cube(double x) {
  auto mesh = std::make_unique<ON_Mesh>();
  const ON_3dPoint corners[8] = {
    ON_3dPoint(x - 0.5, -0.5, -0.5), ON_3dPoint(x + 0.5, -0.5, -0.5),
    ON_3dPoint(x + 0.5, 0.5, -0.5),  ON_3dPoint(x - 0.5, 0.5, -0.5),
    ON_3dPoint(x - 0.5, -0.5, 0.5),  ON_3dPoint(x + 0.5, -0.5, 0.5),
    ON_3dPoint(x + 0.5, 0.5, 0.5),   ON_3dPoint(x - 0.5, 0.5, 0.5),
  };
  for (int index = 0; index < 8; ++index) {
    mesh->SetVertex(index, corners[index]);
  }
  const int triangles[12][3] = {
    {0, 2, 1}, {0, 3, 2}, {4, 5, 6}, {4, 6, 7}, {0, 1, 5}, {0, 5, 4},
    {2, 3, 7}, {2, 7, 6}, {1, 2, 6}, {1, 6, 5}, {0, 4, 7}, {0, 7, 3},
  };
  for (int index = 0; index < 12; ++index) {
    mesh->SetTriangle(index, triangles[index][0], triangles[index][1], triangles[index][2]);
  }
  return mesh;
}

/// Reads the archive and visits every mesh component in object-table order.
template <typename Visitor>
int visit_meshes(const char* path, char* message, int capacity, Visitor visit) {
  if (path == nullptr) {
    write_message(message, capacity, "null path");
    return kInvalidInput;
  }
  ensure_kernel();
  ONX_Model model;
  ON_wString diagnostics;
  ON_TextLog log(diagnostics);
  if (!model.Read(path, &log)) {
    write_message(message, capacity, "archive read failed");
    return kKernelFailure;
  }
  ONX_ModelComponentIterator objects(model, ON_ModelComponent::Type::ModelGeometry);
  for (const ON_ModelComponent* component = objects.FirstComponent(); component;
       component = objects.NextComponent()) {
    const auto* item = ON_ModelGeometryComponent::Cast(component);
    if (item == nullptr) {
      continue;
    }
    const ON_Geometry* geometry = item->Geometry(nullptr);
    if (geometry == nullptr) {
      continue;
    }
    const auto* mesh = ON_Mesh::Cast(geometry);
    if (mesh == nullptr) {
      continue;
    }
    visit(*mesh);
  }
  return kOk;
}

}  // namespace


/// A torus (a ring band) as an ON_Mesh: `u` around the major circle, `v` around the tube.
std::unique_ptr<ON_Mesh> make_torus(double x, double y, double major, double minor, int u, int v) {
  auto mesh = std::make_unique<ON_Mesh>();
  const int rows = v + 1;
  const int cols = u + 1;
  for (int i = 0; i <= v; ++i) {
    const double phi = 2.0 * kPi * i / v;
    for (int j = 0; j <= u; ++j) {
      const double theta = 2.0 * kPi * j / u;
      const double r = major + minor * std::cos(phi);
      mesh->SetVertex(i * cols + j, ON_3dPoint(x + r * std::cos(theta), y + r * std::sin(theta), minor * std::sin(phi)));
    }
  }
  int triangle = 0;
  for (int i = 0; i < v; ++i) {
    for (int j = 0; j < u; ++j) {
      const int a = i * cols + j;
      const int b = a + 1;
      const int c = a + cols;
      const int d = c + 1;
      mesh->SetTriangle(triangle++, a, c, b);
      mesh->SetTriangle(triangle++, b, c, d);
    }
  }
  return mesh;
}

/// A brilliant-cut proxy (a split bipyramid) as an ON_Mesh: `n` around the girdle.
std::unique_ptr<ON_Mesh> make_diamond(double x, double y, double z, double radius, int n) {
  auto mesh = std::make_unique<ON_Mesh>();
  for (int i = 0; i < n; ++i) {
    const double theta = 2.0 * kPi * i / n;
    mesh->SetVertex(i, ON_3dPoint(x + radius * std::cos(theta), y + radius * std::sin(theta), z));
  }
  const int top = n;
  const int bottom = n + 1;
  mesh->SetVertex(top, ON_3dPoint(x, y, z + radius * 0.9));
  mesh->SetVertex(bottom, ON_3dPoint(x, y, z - radius * 1.1));
  int triangle = 0;
  for (int i = 0; i < n; ++i) {
    const int a = i;
    const int b = (i + 1) % n;
    mesh->SetTriangle(triangle++, a, b, top);
    mesh->SetTriangle(triangle++, b, a, bottom);
  }
  return mesh;
}

/// Writes a jewelry-like archive: `rings` torus bands and `diamonds` stones on a grid.
int write_jewelry(const char* path, int rings, int diamonds, int ring_u, int ring_v, char* message, int capacity) {
  if (path == nullptr || rings < 0 || diamonds < 0 || ring_u < 3 || ring_v < 3) {
    write_message(message, capacity, "invalid jewelry fixture request");
    return kInvalidInput;
  }
  ensure_kernel();
  ONX_Model model;
  model.m_settings.m_ModelUnitsAndTolerances.m_unit_system = ON::LengthUnitSystem::Millimeters;
  model.AddDefaultLayer(L"Jewelry", ON_Color::Black);
  std::vector<std::unique_ptr<ON_Mesh>> owned;
  const int side = static_cast<int>(std::ceil(std::sqrt(static_cast<double>(rings + diamonds))));
  const double step = 12.0;
  auto append = [&](std::unique_ptr<ON_Mesh> mesh, const wchar_t* prefix, int index) -> bool {
    owned.push_back(std::move(mesh));
    ON_3dmObjectAttributes attributes;
    const std::wstring name = std::wstring(prefix) + std::to_wstring(index);
    attributes.m_name = name.c_str();
    attributes.m_layer_index = 0;
    return !model.AddModelGeometryComponent(owned.back().get(), &attributes, false).IsEmpty();
  };
  for (int index = 0; index < rings; ++index) {
    const double x = (index % side) * step;
    const double y = (index / side) * step;
    if (!append(make_torus(x, y, 4.0, 1.2, ring_u, ring_v), L"Ring", index)) {
      write_message(message, capacity, "could not add a ring component");
      return kKernelFailure;
    }
  }
  for (int index = 0; index < diamonds; ++index) {
    const double x = (index % side) * step + 6.0;
    const double y = (index / side) * step + 6.0;
    if (!append(make_diamond(x, y, 2.0, 1.0, 16), L"Stone", index)) {
      write_message(message, capacity, "could not add a stone component");
      return kKernelFailure;
    }
  }
  ON_TextLog log;
  if (!model.Write(path, 5, &log)) {
    write_message(message, capacity, "jewelry archive write failed");
    return kKernelFailure;
  }
  return kOk;
}

extern "C" {

/// Writes `mesh_count` cubes into a version-5 archive at `path`.
int om9_3dm_write_fixture(const char* path, int mesh_count, char* message, int capacity) {
  if (path == nullptr || mesh_count <= 0) {
    write_message(message, capacity, "invalid fixture request");
    return kInvalidInput;
  }
  ensure_kernel();
  ONX_Model model;
  model.m_settings.m_ModelUnitsAndTolerances.m_unit_system = ON::LengthUnitSystem::Millimeters;
  model.AddDefaultLayer(L"Meshes", ON_Color::Black);
  std::vector<std::unique_ptr<ON_Mesh>> owned;
  owned.reserve(static_cast<size_t>(mesh_count));
  for (int index = 0; index < mesh_count; ++index) {
    owned.push_back(make_cube(index * 1.6));
    ON_3dmObjectAttributes attributes;
    const std::wstring name = L"Mesh" + std::to_wstring(index);
    attributes.m_name = name.c_str();
    attributes.m_layer_index = 0;
    if (model.AddModelGeometryComponent(owned.back().get(), &attributes, false).IsEmpty()) {
      write_message(message, capacity, "could not add a mesh component");
      return kKernelFailure;
    }
  }
  ON_TextLog log;
  if (!model.Write(path, 5, &log)) {
    write_message(message, capacity, "archive write failed");
    return kKernelFailure;
  }
  return kOk;
}

/// Reports the mesh, vertex and triangle-corner counts of the archive.
int om9_3dm_read(const char* path, long long* mesh_count, long long* vertex_count,
                 long long* index_count, char* message, int capacity) {
  if (mesh_count == nullptr || vertex_count == nullptr || index_count == nullptr) {
    write_message(message, capacity, "null count pointer");
    return kInvalidInput;
  }
  long long meshes = 0;
  long long vertices = 0;
  long long corners = 0;
  const int code = visit_meshes(path, message, capacity, [&](const ON_Mesh& mesh) {
    ++meshes;
    vertices += mesh.m_V.Count();
    // Quads become two triangles, the shape the viewport draws.
    for (int face = 0; face < mesh.m_F.Count(); ++face) {
      corners += mesh.m_F[face].IsTriangle() ? 3 : 6;
    }
  });
  if (code != kOk) {
    return code;
  }
  *mesh_count = meshes;
  *vertex_count = vertices;
  *index_count = corners;
  return kOk;
}

/// Fills the caller buffers; `offsets` holds four numbers per mesh: vertex start/len, index start/len.
int om9_3dm_fill(const char* path, double* positions, unsigned* indices, long long* offsets,
                 char* message, int capacity) {
  if (positions == nullptr || indices == nullptr || offsets == nullptr) {
    write_message(message, capacity, "null fill buffer");
    return kInvalidInput;
  }
  long long vertex_cursor = 0;
  long long index_cursor = 0;
  long long mesh_index = 0;
  const int code = visit_meshes(path, message, capacity, [&](const ON_Mesh& mesh) {
    const long long vertex_start = vertex_cursor;
    for (int vertex = 0; vertex < mesh.m_V.Count(); ++vertex) {
      const ON_3fPoint& point = mesh.m_V[vertex];
      const size_t base = static_cast<size_t>(vertex_cursor) * 3;
      positions[base] = point.x;
      positions[base + 1] = point.y;
      positions[base + 2] = point.z;
      ++vertex_cursor;
    }
    const long long index_start = index_cursor;
    for (int face = 0; face < mesh.m_F.Count(); ++face) {
      const ON_MeshFace& entry = mesh.m_F[face];
      const long long a = entry.vi[0];
      const long long b = entry.vi[1];
      const long long c = entry.vi[2];
      if (entry.IsTriangle()) {
        indices[index_cursor++] = static_cast<unsigned>(a);
        indices[index_cursor++] = static_cast<unsigned>(b);
        indices[index_cursor++] = static_cast<unsigned>(c);
      } else {
        const long long d = entry.vi[3];
        indices[index_cursor++] = static_cast<unsigned>(a);
        indices[index_cursor++] = static_cast<unsigned>(b);
        indices[index_cursor++] = static_cast<unsigned>(c);
        indices[index_cursor++] = static_cast<unsigned>(a);
        indices[index_cursor++] = static_cast<unsigned>(c);
        indices[index_cursor++] = static_cast<unsigned>(d);
      }
    }
    offsets[mesh_index * 4] = vertex_start;
    offsets[mesh_index * 4 + 1] = vertex_cursor - vertex_start;
    offsets[mesh_index * 4 + 2] = index_start;
    offsets[mesh_index * 4 + 3] = index_cursor - index_start;
    ++mesh_index;
  });
  return code;
}


int om9_3dm_write_jewelry(const char* path, int rings, int diamonds, int ring_u, int ring_v,
                          char* message, int capacity) {
  return write_jewelry(path, rings, diamonds, ring_u, ring_v, message, capacity);
}

}  // extern "C"
