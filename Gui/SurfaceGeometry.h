// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Python.h>
#include <string>
#include <vector>
namespace App { class Document; }
namespace OpenMatrix9Gui {
struct SurfaceInput { std::string object, sub; bool closed=false, reverse=false; double seam=0; };
struct SurfaceOptions { unsigned int kind=0, style=0; bool frenet=false, closed=false; };
// Returned shapes are owned references. Callers hold the GIL while using them.
PyObject* surfaceWire(App::Document&, const SurfaceInput&);
PyObject* buildSurface(App::Document&, const std::vector<SurfaceInput>&, const SurfaceOptions&);
void commitSurface(App::Document&, PyObject*, const std::vector<SurfaceInput>&, const SurfaceOptions&);
}
