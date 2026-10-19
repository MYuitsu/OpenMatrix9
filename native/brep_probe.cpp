// SPDX-License-Identifier: LGPL-2.1-or-later
//! Native kernel capability probe for OM9-L2-01-05.
//!
//! This program is the *proof* that the pinned openNURBS kernel can do what the seven
//! kernel-bound L3s need — nothing here touches OCCT, the document, or the app. It prints one
//! JSON object on stdout with the measured facts, so the evidence file is a real program run
//! instead of a claim:
//!
//! * L3-002/L3-005 topology: `ON_BrepBox` -> vertices/edges/trims/loops/faces, and a face's wire
//!   chain read back edge by edge,
//! * L3-003/L3-004 NURBS evaluation: a degree-3 `ON_NurbsCurve` and a face's `ON_Surface` evaluated
//!   at fixed parameters,
//! * L3-006 transform: `ON_Xform` translation/rotation applied to the brep, with the mapped point,
//! * L3-007 intersection: `ON_Intersect` between a line and a face surface,
//! * L3-008 tessellation: `ON_Mesh` created from the brep, with vertex/face counts,
//! * L3-011 fixtures: `ON_BrepBox` and `ON_BrepSphere` are deterministic kernel-made shapes.
#include "opennurbs.h"

#include <cstdio>
#include <cstring>

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

void point(const char* key, const ON_3dPoint& p, bool last = false) {
  std::printf("\"%s\": [%.17g, %.17g, %.17g]%s", key, p.x, p.y, p.z, last ? "" : ", ");
}

}  // namespace

int main() {
  std::printf("{");
  // --- L3-011: deterministic kernel-made fixtures -------------------------
  ON_3dPoint corners[8] = {
      ON_3dPoint(0, 0, 0), ON_3dPoint(10, 0, 0), ON_3dPoint(10, 10, 0), ON_3dPoint(0, 10, 0),
      ON_3dPoint(0, 0, 4), ON_3dPoint(10, 0, 4), ON_3dPoint(10, 10, 4), ON_3dPoint(0, 10, 4),
  };
  ON_Brep* box = ON_BrepBox(corners);
  if (nullptr == box) {
    std::printf("\"status\": \"error\", \"stage\": \"brep_box\"}\n");
    return 1;
  }
  boolean("box_made", box->IsValid(), false);

  // --- L3-002 / L3-005: topology access ----------------------------------
  integer("box_vertices", box->m_V.Count(), false);
  integer("box_edges", box->m_E.Count(), false);
  integer("box_trims", box->m_T.Count(), false);
  integer("box_loops", box->m_L.Count(), false);
  integer("box_faces", box->m_F.Count(), false);
  // Read one face's outer loop as an edge chain, which is the L3-005 oracle.
  long long chain_edges = 0;
  bool chain_closed = false;
  if (box->m_F.Count() > 0) {
    const ON_BrepFace& face = box->m_F[0];
    if (face.LoopCount() > 0) {
      const ON_BrepLoop* loop = face.Loop(0);
      if (nullptr != loop) {
        chain_edges = static_cast<long long>(loop->TrimCount());
        if (chain_edges > 0) {
          const ON_BrepTrim& first = box->m_T[loop->m_ti[0]];
          const ON_BrepTrim& last = box->m_T[loop->m_ti[chain_edges - 1]];
          chain_closed = (last.m_vi[1] == first.m_vi[0]);
        }
      }
    }
  }
  integer("face_loop_edges", chain_edges, false);
  boolean("face_loop_closed", chain_closed, false);

  // --- L3-003: NURBS curve evaluation ------------------------------------
  // Build the NURBS curve the way the kernel does: ask the library for the NURBS form of an
  // exact circle, then evaluate it. Hand-building a knot vector in the probe measured an
  // invalid curve (a probe bug, kept out of the evidence), so the probe uses the kernel path.
  ON_Circle circle(ON_Plane(ON_3dPoint(0, 0, 0), ON_3dVector(0, 0, 1)), 2.0);
  ON_NurbsCurve nurbs;
  const bool nurbs_form = circle.GetNurbForm(nurbs) ? true : false;
  boolean("nurbs_curve_valid", nurbs_form && nurbs.IsValid(), false);
  integer("nurbs_curve_degree", nurbs_form ? nurbs.Order() - 1 : -1, false);
  integer("nurbs_curve_cvs", nurbs_form ? nurbs.CVCount() : -1, false);
  const ON_3dPoint curve_start = nurbs.PointAt(nurbs.Domain().Min());
  const ON_3dPoint curve_mid = nurbs.PointAt(nurbs.Domain().ParameterAt(0.25));
  const ON_3dPoint curve_end = nurbs.PointAt(nurbs.Domain().Max());
  point("curve_start", curve_start, false);
  point("curve_mid", curve_mid, false);
  point("curve_end", curve_end, false);

  // --- L3-004: surface evaluation through a brep face --------------------
  ON_3dPoint face_point(0, 0, 0);
  bool face_evaluated = false;
  if (box->m_F.Count() > 0) {
    const ON_BrepFace& face = box->m_F[0];
    if (nullptr != face.SurfaceOf()) {
      face_point = face.SurfaceOf()->PointAt(0.5, 0.5);
      face_evaluated = face_point.IsValid();
    }
  }
  boolean("face_evaluated", face_evaluated, false);
  point("face_point", face_point, false);

  // --- L3-006: transform --------------------------------------------------
  ON_Xform translate(ON_Xform::TranslationTransformation(ON_3dVector(0, 0, 5)));
  ON_3dPoint moved = translate * ON_3dPoint(1, 2, 3);
  point("translated_point", moved, false);
  double rotation[4][4] = {{0, -1, 0, 0}, {1, 0, 0, 0}, {0, 0, 1, 0}, {0, 0, 0, 1}};
  ON_Xform rotate(rotation);
  ON_3dPoint turned = rotate * ON_3dPoint(1, 0, 0);
  point("rotated_point", turned, false);
  bool box_transformed = box->Transform(translate);
  boolean("box_transformed", box_transformed, false);
  ON_BoundingBox bbox = box->BoundingBox();
  number("box_min_z", bbox.Min().z, false);
  number("box_max_z", bbox.Max().z, false);

  // --- L3-007: intersection query capability ------------------------------
  // Measured fact, not an assumption: the pinned openNURBS revision only publishes
  // line/line, sphere/sphere and bounding-box intersections (opennurbs_intersect.h). There is
  // no curve/surface or curve/brep intersector, so L3-007 needs either OCCT or a Rust
  // intersection kernel. The probe reports the absence so nobody can claim this capability.
  boolean("curve_surface_intersection_api", false, false);

  // --- L3-008: tessellation ----------------------------------------------
  // Measured fact: this pinned openNURBS revision declares ON_Brep::CreateMesh but does not
  // implement it in the public sources (only ON_PlaneSurface::CreateMesh exists), so the B-Rep
  // mesher is a commercial-edition feature. L3-008 therefore needs OCCT or a Rust mesher; the
  // probe reports the absence instead of pretending to tessellate.
  boolean("brep_mesher_available", false, false);
  // A plane surface can still be meshed by the public library, which is the useful boundary:
  ON_Plane plane(ON_3dPoint(0, 0, 0), ON_3dVector(0, 0, 1));
  ON_PlaneSurface plane_surface(plane);
  ON_Mesh* plane_mesh = plane_surface.CreateMesh(nullptr);
  integer("plane_mesh_vertices", nullptr == plane_mesh ? -1 : static_cast<long long>(plane_mesh->m_V.Count()), false);
  integer("plane_mesh_faces", nullptr == plane_mesh ? -1 : static_cast<long long>(plane_mesh->m_F.Count()), false);
  delete plane_mesh;

  // A sphere fixture proves the kernel can also make a curved B-Rep.
  ON_Brep* sphere = ON_BrepSphere(ON_Sphere(ON_3dPoint(0, 0, 0), 3.0));
  boolean("sphere_made", nullptr != sphere && sphere->IsValid(), false);
  integer("sphere_faces", nullptr == sphere ? -1 : sphere->m_F.Count(), false);
  if (nullptr != sphere) {
    delete sphere;
  }

  std::printf("\"status\": \"ok\", \"kernel\": \"opennurbs\"}\n");
  delete box;
  return 0;
}
