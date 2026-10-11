// SPDX-License-Identifier: LGPL-2.1-or-later
// FreeCAD 1.1 FCGlobal.h initializes Qt before OM9's forced compatibility prelude.
#include <windows.h>
#include <QtCore/QtCore>
#include <string_view>
#include "freecad11/WindowsDistanceMacros.h"
#if !defined(_WINDOWS_) || defined(OPENNURBS_INC_)
#error This regression needs Windows initialized and a fresh openNURBS include
#endif
#include "ThreeDmGeometry.h"
// Points.h -> Base/Writer.h -> zipios++ uses the host's unprefixed zlib here.
#include <zlib.h>
#if defined(near) || defined(far) || defined(Z_PREFIX)
#error The host must keep CAD identifiers free and use unprefixed system zlib
#endif
#define OM9_STRINGIFY_IMPL(value) #value
#define OM9_STRINGIFY(value) OM9_STRINGIFY_IMPL(value)
static_assert(std::string_view(OM9_STRINGIFY(FAR)).empty());
#undef OM9_STRINGIFY
#undef OM9_STRINGIFY_IMPL

int main()
{
    const ON_3dPoint point(1.0, 2.0, 3.0);
    const auto* text = reinterpret_cast<const Bytef*>("123456789");
    return point.x == 1.0 && point.y == 2.0 && point.z == 3.0
        && crc32(0, text, 9) == 0xcbf43926UL ? 0 : 1;
}
