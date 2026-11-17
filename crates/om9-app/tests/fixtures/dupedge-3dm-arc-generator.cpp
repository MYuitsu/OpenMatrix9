// SPDX-License-Identifier: LGPL-2.1-or-later
// Synthetic quarter-cylinder face with trimmed circular edges, using pinned openNURBS.
#include <opennurbs.h>
#include <cstdio>

int wmain(int argc, wchar_t** argv) {
    if (argc != 2) return 1;
    ON::Begin();
    ONX_Model model;
    model.m_settings.m_ModelUnitsAndTolerances.m_unit_system = ON_UnitSystem::Millimeters;
    model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance = 0.001;
    auto* surface = ON_RevSurface::New();
    surface->m_curve = new ON_LineCurve(ON_3dPoint(2., 0., 0.), ON_3dPoint(2., 0., 5.));
    surface->m_axis = ON_Line(ON_3dPoint(0., 0., 0.), ON_3dPoint(0., 0., 5.));
    surface->m_angle = ON_Interval(0., ON_PI * 0.5);
    surface->m_t = surface->m_angle;
    ON_Brep* brep = ON_BrepRevSurface(surface, false, false);
    if (!brep || !brep->IsValid()) return 2;
    for (int i = 0; i < brep->m_E.Count(); ++i) {
        const auto& edge = brep->m_E[i];
        const ON_3dPoint a = edge.PointAtStart(), b = edge.PointAtEnd();
        if (a.z == 5. && b.z == 5.)
            std::printf("top_edge %.17g %.17g %.17g -> %.17g %.17g %.17g\n", a.x, a.y, a.z, b.x, b.y, b.z);
    }
    auto* attrs = new ON_3dmObjectAttributes();
    attrs->m_name = L"DupEdge quarter cylinder";
    attrs->m_layer_index = 0;
    if (model.AddManagedModelGeometryComponent(brep, attrs, false).IsEmpty()) return 3;
    ON_wString message;
    ON_TextLog log(message);
    return model.Write(argv[1], 50, &log) ? 0 : 4;
}
