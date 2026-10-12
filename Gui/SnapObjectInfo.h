#pragma once
#include <Python.h>
#include <Base/Matrix.h>
#include <TopoDS_Shape.hxx>
#include <string>
namespace App { class DocumentObject; }
namespace OpenMatrix9Gui {
struct SnapObjectInfo {
    unsigned kind=9;
    bool native_cad=false,preview=false;
    std::string representation="unknown",display_mesh_state="unknown";
    const App::DocumentObject* resolved_member=nullptr;
    Base::Matrix4D global_transform;
    TopoDS_Shape shape;
};
SnapObjectInfo classifySnapObject(const App::DocumentObject*);
}
void AddModelingSnapMethods(PyObject*);
