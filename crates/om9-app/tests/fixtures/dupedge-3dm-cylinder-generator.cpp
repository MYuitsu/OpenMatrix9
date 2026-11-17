// SPDX-License-Identifier: LGPL-2.1-or-later
// Synthetic millimetre 3DM fixture with a capped cylinder, using the pinned openNURBS SDK.
#include <opennurbs.h>

int wmain(int argc, wchar_t** argv) {
    if (argc != 2) return 1;
    ON::Begin();
    ONX_Model model;
    model.m_settings.m_ModelUnitsAndTolerances.m_unit_system = ON_UnitSystem::Millimeters;
    model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance = 0.001;
    ON_Cylinder cylinder(ON_Circle(ON_xy_plane, 2.), 5.);
    ON_Brep* brep = ON_BrepCylinder(cylinder, true, true);
    auto* attrs = new ON_3dmObjectAttributes();
    attrs->m_name = L"DupEdge cylinder";
    attrs->m_layer_index = 0;
    if (!brep || model.AddManagedModelGeometryComponent(brep, attrs, false).IsEmpty()) return 2;
    ON_wString message;
    ON_TextLog log(message);
    return model.Write(argv[1], 50, &log) ? 0 : 3;
}
