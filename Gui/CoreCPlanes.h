#pragma once
#include <Base/Placement.h>
#include <Python.h>
namespace Gui { class View3DInventor; }
class QMenu;
namespace OpenMatrix9Gui {
// Native adapter: view/document lifetime and Coin grid rendering stay in C++.
// Rust owns validated frames, history and named-plane identities.
Base::Placement constructionPlane(Gui::View3DInventor*, const Base::Placement& fallback);
void addCPlaneMethods(PyObject* module);
QMenu* addCPlaneMenu(QMenu* parent,Gui::View3DInventor* view);
}
