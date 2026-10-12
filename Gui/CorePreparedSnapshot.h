// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <TopoDS_Shape.hxx>
#include <QJsonObject>
#include <QByteArray>
#include <string>
namespace OpenMatrix9Gui {
// Bounded native master digest of private topology with root Location factored
// out and Checked cache normalized. Callers compare numerical frame/orientation.
QByteArray preparedCadNativeMasterDigest(const TopoDS_Shape&);
QJsonObject preparedCadSnapshot(const TopoDS_Shape&,const QJsonObject& topology,
    const QJsonObject& validation,const QJsonObject& attributes,const std::string& unitContext,
    const QJsonObject& identity,const char* operation);
// GUI-thread native owner remains with caller. Returns independent Rust-owned
// serialized values; no document, OCCT or Python handle escapes in the result.
QJsonObject preparedCadResult(const TopoDS_Shape&,const QJsonObject& topology,
    const QJsonObject& validation,const QJsonObject& attributes,
    const QJsonObject& identity,const std::string& unitContext,const QJsonObject& operationContext={});
QJsonObject preparedGeometryResult(QJsonObject snapshot,const QJsonObject& validation,
    const QJsonObject& attributes,const QJsonObject& identity,const std::string& unitContext,const QJsonObject& operationContext={});
}
