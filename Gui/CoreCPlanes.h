#pragma once
#include <Base/Placement.h>
#include <Python.h>
#include <array>
#include <cstdint>
namespace App { class Document; }
namespace Gui { class View3DInventor; }
class QMenu;
namespace OpenMatrix9Gui {
// Native adapter: view/document lifetime and Coin grid rendering stay in C++.
// Rust owns validated frames, history and named-plane identities.
Base::Placement constructionPlane(Gui::View3DInventor*, const Base::Placement& fallback);
struct ConstructionPlaneSnapshot {
    std::array<double,12> frame{};
    Base::Placement plane;
    std::uint64_t documentOwner = 0;
    std::uint64_t revision = 0;
    unsigned logicalViewSlot = 0;
};
// Read-only host capture. Runtime owners are never reused during this process;
// the logical slot is not a unique native viewport identity.
ConstructionPlaneSnapshot captureConstructionPlane(App::Document&, Gui::View3DInventor*);
std::uint64_t constructionPlaneDocumentOwner(const App::Document&);
void addCPlaneMethods(PyObject* module);
QMenu* addCPlaneMenu(QMenu* parent,Gui::View3DInventor* view);
}
