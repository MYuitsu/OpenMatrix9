// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <QJsonObject>
#include <TopoDS_Shape.hxx>

namespace OpenMatrix9Gui {
// Synchronous GUI-thread observation. Source must be live and immutable; caller
// verifies its identity stamp before/after. No native handles escape in JSON.
QJsonObject topologySnapshot(const TopoDS_Shape&);
}
