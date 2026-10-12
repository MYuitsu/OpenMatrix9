// SPDX-License-Identifier: LGPL-2.1-or-later
#define PY_SSIZE_T_CLEAN
#include <Python.h>
#include "LayerDocumentAdapter.h"
#include <App/Application.h>
#include <App/Document.h>
#include <stdexcept>
namespace {
App::Document& document(const char* name) {
    auto* doc=App::GetApplication().getDocument(name);
    if(!doc) throw std::runtime_error("Layer document not found");
    return *doc;
}
PyObject* initialize(PyObject*,PyObject* args) {
    const char* name=nullptr;const char* json=nullptr;
    if(!PyArg_ParseTuple(args,"ss",&name,&json))return nullptr;
    try { OpenMatrix9Gui::initializeLayerDocument(document(name),json);Py_RETURN_NONE; }
    catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
PyObject* snapshot(PyObject*,PyObject* args) {
    const char* name=nullptr;
    if(!PyArg_ParseTuple(args,"s",&name))return nullptr;
    try { const auto value=OpenMatrix9Gui::layerDocumentSnapshot(document(name));return PyUnicode_DecodeUTF8(value.data(),value.size(),"strict"); }
    catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
PyObject* command(PyObject*,PyObject* args) {
    const char* name=nullptr;const char* json=nullptr;
    if(!PyArg_ParseTuple(args,"ss",&name,&json))return nullptr;
    try { const auto value=OpenMatrix9Gui::layerDocumentCommand(document(name),json);return PyUnicode_DecodeUTF8(value.data(),value.size(),"strict"); }
    catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
PyObject* panel(PyObject*,PyObject* args) {
    const char* name=nullptr;
    if(!PyArg_ParseTuple(args,"s",&name))return nullptr;
    try {const auto value=OpenMatrix9Gui::layerDocumentPanel(document(name));return PyUnicode_DecodeUTF8(value.data(),value.size(),"strict");}
    catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
PyObject* transactionFacts(PyObject*,PyObject* args) {
    const char* name=nullptr;
    if(!PyArg_ParseTuple(args,"s",&name))return nullptr;
    try {
        auto& doc=document(name);
        const auto uid=doc.Uid.getValueStr();
        const bool pending=doc.hasPendingTransaction();
        // getTransactionID(true) otherwise returns the last Undo record, not
        // proof that this document currently owns a pending transaction.
        return Py_BuildValue("{s:s,s:s,s:O,s:i,s:i,s:O,s:O}",
            "document",doc.getName(),"uid",uid.c_str(),
            "pending",pending?Py_True:Py_False,
            "pending_id",pending?doc.getTransactionID(true):0,
            "booked_id",om9BookedTransaction(doc),
            "transacting",om9ClosingTransaction(doc)?Py_True:Py_False,
            "performing",doc.isPerformingTransaction()?Py_True:Py_False);
    }
    catch(const std::exception& error){PyErr_SetString(PyExc_RuntimeError,error.what());return nullptr;}
}
}
void AddLayerDocumentMethods(PyObject* module) {
    OpenMatrix9Gui::registerLayerDocumentTypes();
    static PyMethodDef methods[]={
        {"initializeLayerDocument",initialize,METH_VARARGS,"Initialize validated canonical metadata and native projections in an owned transaction."},
        {"layerDocumentSnapshot",snapshot,METH_VARARGS,"Read validated FCStd canonical layer metadata; no geometry extraction."},
        {"layerDocumentCommand",command,METH_VARARGS,"Apply a Rust document-layer command in one native Undo transaction."},
        {"layerDocumentPanel",panel,METH_VARARGS,"Read Rust layer-only presentation with local/effective flags."},
        {"layerDocumentTransactionFacts",transactionFacts,METH_VARARGS,"Read current native transaction facts; no commits, aborts or retained native pointers."},
        {nullptr,nullptr,0,nullptr}};
    PyModule_AddFunctions(module,methods);
}
