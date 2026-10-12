// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Python.h>
namespace OpenMatrix9Gui {
PyObject* rotateSurfaceWire(PyObject* wire,double fraction);
void automaticSurfaceAlignment(PyObject* reference,PyObject* profile,double& seam,bool& reverse);
}
