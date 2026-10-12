// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <TopoDS_Shape.hxx>
#include <QJsonObject>
#include <cstdint>
namespace OpenMatrix9Gui {
// Synchronous, read-only observations of a live immutable native source.
// Rust owns the combination policy. No shape/owner is retained in JSON or FFI.
QJsonObject shapeValidation(const TopoDS_Shape&,const QJsonObject& topology);
void reframeShapeValidation(QJsonObject& topology,std::int32_t worldSign);
}
