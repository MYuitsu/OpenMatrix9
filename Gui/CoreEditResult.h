// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "EditGeometry.h"
#include <Mod/Part/App/TopoShape.h>
#include <Base/Placement.h>
#include <QJsonObject>
#include <QByteArray>
namespace OpenMatrix9Gui {
struct EditEvidence {QJsonObject context;std::string units;std::vector<ObjectSnapshot> sources;};
struct PreparedEditOutput {Part::TopoShape shape;std::string name,result;Base::Placement parent;Base::Matrix4D rootFrame;QByteArray masterDigest;};
// GUI thread/GIL: native shape ownership stays here; Rust owns scalar snapshots.
EditEvidence captureEditEvidence(App::Document&,const std::vector<EditInput>&,unsigned kind,unsigned flags,
    unsigned mode,std::size_t first,const std::vector<std::string>& deleting,double tolerance,
    const ToleranceSnapshot*,const EditProjection*);
PreparedEditOutput prepareEditOutput(App::Document&,PyObject*,const EditEvidence&,const std::string&,
    const Base::Placement& parent,std::size_t index,std::size_t count,std::size_t owner);
}
