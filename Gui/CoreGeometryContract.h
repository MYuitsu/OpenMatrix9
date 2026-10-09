// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Python.h>

namespace OpenMatrix9Gui {
// Read-only GUI-thread companion. Python receives copied Unicode JSON values;
// no native handle/pointer, document property or transaction is created.
void addGeometryContractMethods(PyObject* module);
}
