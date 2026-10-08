#pragma once
#include "ThreeDmArchive.h"
#include <QJsonObject>
namespace OpenMatrix9Gui::ThreeDm {
void transformCurveOnSurfaceNative(ON_CurveOnSurface& curve,const ON_Xform& transform);
void validateCurveOnSurfaceRhino5(const ON_CurveOnSurface& curve);
QJsonObject curveOnSurfaceNativeFields(const ON_CurveOnSurface& curve);
QJsonObject nativeCurveTreeFields(const ON_Curve& curve);
bool hasPolyEdgeReference(const ON_Curve& curve);
}
