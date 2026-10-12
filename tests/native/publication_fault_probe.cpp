// SPDX-License-Identifier: LGPL-2.1-or-later
// Test-only native faults: Python observers swallow their own exceptions, so
// native application close signals are required to exercise commit failures.
#include <Python.h>
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/DocumentObjectPy.h>
#include <App/PropertyGeo.h>
#include <boost/exception/exception.hpp>
#include <set>
#include <stdexcept>
#include <string>
#include <vector>
#include <Mod/Part/App/TopoShapePy.h>
#include <Mod/Part/App/PropertyTopoShape.h>
#include <ShapeUpgrade_RemoveLocations.hxx>
#include <BRepTools.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <sstream>

namespace {
// Legacy close notifications run in a destructor which catches boost errors.
// Exercise both legacy and checked paths without terminating the test host.
struct NativePublicationFault: std::runtime_error, boost::exception {
    using std::runtime_error::runtime_error;
};
struct Captured {
    PyObject* object;
    std::string name;
    std::string type;
};
std::vector<Captured> captured;
std::set<long> initialIDs;
std::string documentName,sourceName,stage;
bool armed=false,tracking=false,matchedBefore=false;
int calls=0;

void clear() {
    armed=false;tracking=false;matchedBefore=false;
    for (const auto& row:captured) Py_DECREF(row.object);
    captured.clear();initialIDs.clear();
}

PyObject* arm(PyObject*,PyObject* args) {
    const char *kind,*docName,*input;
    if (!PyArg_ParseTuple(args,"sss",&kind,&docName,&input)) return nullptr;
    if (std::string(kind)!="early" && std::string(kind)!="before" && std::string(kind)!="close" && std::string(kind)!="observe") {
        PyErr_SetString(PyExc_ValueError,"Unknown native publication fault");return nullptr;
    }
    auto* doc=App::GetApplication().getDocument(docName);
    if (!doc || !doc->getObject(input)) {
        PyErr_SetString(PyExc_ValueError,"Native fault requires a live document/source");return nullptr;
    }
    clear();documentName=docName;sourceName=input;stage=kind;calls=0;
    for (auto* object:doc->getObjects()) initialIDs.insert(object->getID());
    static auto created=App::GetApplication().signalNewObject.connect([](const App::DocumentObject& object) {
        if (!tracking || !object.getDocument() || object.getDocument()->getName()!=documentName
            || initialIDs.contains(object.getID()) || object.getDocument()->isPerformingTransaction()) return;
        auto* py=const_cast<App::DocumentObject&>(object).getPyObject();
        captured.push_back({py,object.getNameInDocument(),std::string(object.getTypeId().getName())});
        if (armed && stage=="early") {
            armed=false;++calls;
            auto* doc=App::GetApplication().getDocument(documentName.c_str());
            auto* source=doc?doc->getObject(sourceName.c_str()):nullptr;
            auto* placement=source?source->getPropertyByName<App::PropertyPlacement>("Placement"):nullptr;
            if (!placement) throw std::runtime_error("Native source-placement fault unavailable");
            auto changed=placement->getValue();
            changed.setPosition(changed.getPosition()+Base::Vector3d(1,0,0));
            placement->setValue(changed);
        }
    });
    static auto before=App::GetApplication().signalBeforeCloseTransaction.connect([](bool abort) {
        if (!armed || abort || stage=="early") return;
        auto* doc=App::GetApplication().getDocument(documentName.c_str());
        if (!doc || (!doc->getBookedTransactionID() && !doc->hasPendingTransaction())) return;
        matchedBefore=true;
        if (stage=="before") {
            armed=false;++calls;
            throw NativePublicationFault("Test native before-commit fault");
        }
    });
    static auto closed=App::GetApplication().signalCloseTransaction.connect([](bool abort) {
        if (armed && !abort && matchedBefore && stage=="close") {
            armed=false;++calls;
            throw NativePublicationFault("Test native close-commit fault");
        }
    });
    armed=stage!="observe";tracking=true;Py_RETURN_NONE;
}

PyObject* counts(PyObject*,PyObject*) {
    auto* out=PyList_New(captured.size());if (!out) return nullptr;
    for (std::size_t index=0;index<captured.size();++index) {
        const auto& row=captured[index];
        const bool valid=static_cast<App::DocumentObjectPy*>(row.object)->Base::PyObjectBase::isValid();
        auto* entry=Py_BuildValue("{s:s,s:s,s:n,s:O}","name",row.name.c_str(),"type",row.type.c_str(),
            "refcount",Py_REFCNT(row.object),"valid",valid?Py_True:Py_False);
        if (!entry) {Py_DECREF(out);return nullptr;}
        PyList_SET_ITEM(out,index,entry);
    }
    return out;
}
PyObject* status(PyObject*,PyObject*) {return Py_BuildValue("{s:i,s:O}","calls",calls,"armed",armed?Py_True:Py_False);}
// Diagnostic only: describe Python owners without dereferencing a dead native object.
PyObject* referrers(PyObject*,PyObject*) {
    PyObject* gc=PyImport_ImportModule("gc");if(!gc)return nullptr;
    PyObject* out=PyList_New(0);
    for(const auto& row:captured){
        PyObject* refs=PyObject_CallMethod(gc,"get_referrers","O",row.object);
        if(!refs){Py_DECREF(gc);Py_DECREF(out);return nullptr;}
        for(Py_ssize_t i=0;i<PyList_Size(refs);++i){
            PyObject* owner=PyList_GetItem(refs,i);
            PyObject* entry=Py_BuildValue("{s:s,s:s}","object",row.name.c_str(),"owner_type",Py_TYPE(owner)->tp_name);
            if(PyDict_Check(owner)){
                PyObject* keys=PyList_New(0),*key,*value;Py_ssize_t pos=0;
                while(PyDict_Next(owner,&pos,&key,&value))if(value==row.object&&PyUnicode_Check(key))PyList_Append(keys,key);
                PyDict_SetItemString(entry,"keys",keys);Py_DECREF(keys);
            }
            PyList_Append(out,entry);Py_DECREF(entry);
        }
        Py_DECREF(refs);
    }
    Py_DECREF(gc);return out;
}
// Calibration only: use OCCT's existing location normalizer, not a BRep parser.
PyObject* normalizedBrep(PyObject*,PyObject* shape){
    if(!PyObject_TypeCheck(shape,&Part::TopoShapePy::Type)){PyErr_SetString(PyExc_TypeError,"Part shape required");return nullptr;}
    try{
        auto native=static_cast<Part::TopoShapePy*>(shape)->getTopoShapePtr()->getShape();
        ShapeUpgrade_RemoveLocations normalize;normalize.SetRemoveLevel(TopAbs_SOLID);normalize.Remove(native);
        Part::TopoShape local(normalize.GetResult());std::ostringstream out;local.exportBrep(out);
        return PyUnicode_FromString(out.str().c_str());
    }catch(const Standard_Failure& error){PyErr_SetString(PyExc_RuntimeError,error.GetMessageString());return nullptr;}
}
PyObject* solverFingerprint(PyObject*,PyObject* shape){
    PyObject* copy=PyObject_CallMethod(shape,"copy",nullptr);if(!copy)return nullptr;
    try{
        auto native=static_cast<Part::TopoShapePy*>(copy)->getTopoShapePtr()->getShape();
        TopTools_IndexedMapOfShape topology;TopExp::MapShapes(native,topology);
        for(int i=1;i<=topology.Extent();++i){auto item=topology(i);item.Checked(false);}
        Part::TopoShape owned(native);std::ostringstream out;owned.exportBrep(out);Py_DECREF(copy);
        return PyUnicode_FromString(out.str().c_str());
    }catch(const Standard_Failure& error){Py_DECREF(copy);PyErr_SetString(PyExc_RuntimeError,error.GetMessageString());return nullptr;}
}
PyObject* disarm(PyObject*,PyObject*) {clear();Py_RETURN_NONE;}
PyObject* mutateOutput(PyObject*,PyObject* args){
    const char *docName,*name,*mode;if(!PyArg_ParseTuple(args,"sss",&docName,&name,&mode))return nullptr;
    auto* doc=App::GetApplication().getDocument(docName);auto* object=doc?doc->getObject(name):nullptr;
    auto* property=object?object->getPropertyByName<Part::PropertyPartShape>("Shape"):nullptr;
    if(!property){PyErr_SetString(PyExc_ValueError,"Live native Part output required");return nullptr;}
    if(std::string(mode)=="root"){
        Base::Matrix4D matrix;matrix.move(50.,60.,70.);property->setTransform(matrix);
    }else if(std::string(mode)=="orientation"){
        property->setValue(property->getValue().Reversed());
    }else{PyErr_SetString(PyExc_ValueError,"Unknown native output mutation");return nullptr;}
    Py_RETURN_NONE;
}
PyMethodDef methods[]={{"arm",arm,METH_VARARGS,nullptr},{"counts",counts,METH_NOARGS,nullptr},
    {"status",status,METH_NOARGS,nullptr},{"clear",disarm,METH_NOARGS,nullptr},
    {"referrers",referrers,METH_NOARGS,nullptr},{"normalized_brep",normalizedBrep,METH_O,nullptr},
    {"solver_fingerprint",solverFingerprint,METH_O,nullptr},{"mutate_output",mutateOutput,METH_VARARGS,nullptr},{nullptr,nullptr,0,nullptr}};
PyModuleDef module={PyModuleDef_HEAD_INIT,"OM9PublicationFault",nullptr,-1,methods};
}
PyMODINIT_FUNC PyInit_OM9PublicationFault() {return PyModule_Create(&module);}
