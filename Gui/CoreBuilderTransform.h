// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Mod/Part/App/TopoShape.h>
#include <Base/Placement.h>
#include <QJsonObject>
#include <vector>
namespace OpenMatrix9Gui {
// Synchronous native ownership only. Rust receives independent JSON values.
struct BuilderShapePair {TopoDS_Shape source,current;unsigned sourceIndex;};
struct BuilderTrace {Part::TopoShape source,shape;std::vector<BuilderShapePair> pairs;};
BuilderTrace builderWorldTrace(const Part::TopoShape&,const double*,const Base::Placement&);
BuilderTrace builderLocalTrace(const BuilderTrace&,const Base::Placement& global,const Base::Placement& own);
QJsonObject builderTraceObservations(const BuilderTrace&,const QJsonObject& inputSnapshot,
    const double* affine,const Base::Placement& sourceWorld,const Base::Placement& outputParent);
}
