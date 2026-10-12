// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "SurfaceGeometry.h"
#include <Base/Placement.h>
#include <Mod/Part/App/TopoShape.h>
#include <QJsonObject>
#include <string>
namespace OpenMatrix9Gui {
// GUI-thread native capture. Independent JSON/stamps retain no host pointer.
struct SurfaceEvidence {
    std::vector<SurfaceInput> inputs;
    std::vector<ObjectSnapshot> sources;
    QJsonObject context;
    std::string recipe,units,operation;
};
struct PreparedSurfaceOutput {
    Part::TopoShape shape;
    std::string result,recipe,outputName,operation;
    std::vector<ObjectSnapshot> sources;
};
SurfaceEvidence captureSurfaceEvidence(App::Document&,const std::vector<SurfaceInput>&,const SurfaceOptions&);
// Caller retains the initialized Part shape owner and GIL for this synchronous
// call. Output owns a deep CAD copy; Rust admits owned metadata, not OCCT pointers.
PreparedSurfaceOutput prepareSurfaceOutput(App::Document&,PyObject*,const SurfaceEvidence&,
    const std::string& outputName,const Base::Placement& outputParent=Base::Placement(),
    const Base::Placement& outputOwn=Base::Placement());
void verifySurfaceOutput(App::Document&,const PreparedSurfaceOutput&);
}
