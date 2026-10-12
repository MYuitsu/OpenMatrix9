// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Python.h>
#include <QJsonObject>
#include <stdexcept>
namespace OpenMatrix9Gui {
struct PreparedCadFailure:std::runtime_error {
    QJsonObject result;
    PreparedCadFailure(unsigned,const char*,const QJsonObject&,const std::string&);
};
// GUI thread/GIL, initialized immutable caller input for one synchronous call.
// Returns a new Python reference. CAD has private native geometry; plain
// metadata is recursively owned. Non-CAD native owners remain existing adapters.
PyObject* prepareOwnedCad(PyObject*,const char* documentName);
void setPreparedCadError(const PreparedCadFailure&) noexcept;
}
