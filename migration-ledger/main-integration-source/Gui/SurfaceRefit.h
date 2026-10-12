// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Python.h>
#include <array>
namespace OpenMatrix9Gui {
// Positive-weight trimmed NURBS hulls certify a symmetric curve distance bound.
double certifySurfaceRefit(PyObject* original,PyObject* candidate,double tolerance);
std::array<double,3> surfaceRailPoint(PyObject* wire,double arcFraction);
double surfaceRailFraction(PyObject* wire,const std::array<double,3>& worldPoint);
}
