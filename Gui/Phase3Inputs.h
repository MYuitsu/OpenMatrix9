// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Python.h>
#include "Phase3RustBridge.h"
#include <memory>
#include <string>
#include <stdexcept>
#include <array>
#include <utility>
#include <vector>
namespace App {class Document;}
namespace OpenMatrix9Gui {
// Host references never cross FFI. All uses/destruction require the GIL.
struct CurvePyRef {
    PyObject* value;
    explicit CurvePyRef(PyObject* p):value(p){if(!p){PyErr_Clear();throw std::runtime_error("Native geometry operation failed");}}
    ~CurvePyRef(){Py_XDECREF(value);}
    CurvePyRef(const CurvePyRef&)=delete;
    CurvePyRef& operator=(const CurvePyRef&)=delete;
    CurvePyRef(CurvePyRef&& other)noexcept:value(std::exchange(other.value,nullptr)){}
};
struct Phase3SnapshotState;
using Phase3Snapshot=std::shared_ptr<Phase3SnapshotState>;
Phase3Snapshot capturePhase3Object(App::Document&,const std::string&,const std::string& sub={});
void verifyPhase3Object(App::Document&,const Phase3Snapshot&);
Om9Phase3Facts phase3ShapeFacts(PyObject*,bool protectedInput=false);
void phase3Require(std::uint32_t);
void phase3Validate(unsigned,const std::vector<Om9Phase3Facts>&,bool complete=true);
bool phase3SurfaceJoinSelection();
CurvePyRef publishedSurfaceSplineShape();
std::vector<std::array<double,3>> sampleCurve(PyObject*,std::size_t);
}
