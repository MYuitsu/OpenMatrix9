// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Python.h>
namespace OpenMatrix9Gui {
// OM9-SURFACE-009. Owned native Part shape; caller holds the GIL.
PyObject* advancedLoft(PyObject* wires, unsigned style, bool closed);
}
