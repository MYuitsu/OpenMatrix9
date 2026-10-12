#pragma once
#include <Python.h>
#include <array>
#include <vector>
#include <stdexcept>
#include <utility>
#include <string>
namespace App {class Document;}
namespace OpenMatrix9Gui {
// Owned references for the typed native Part adapter; command input is never evaluated.
class CurvePyRef {
public:
    PyObject* value;
    explicit CurvePyRef(PyObject* object):value(object) {if(!value){PyErr_Print();throw std::runtime_error("Native Part curve operation failed");}}
    ~CurvePyRef(){Py_XDECREF(value);}
    CurvePyRef(const CurvePyRef&)=delete;
    CurvePyRef& operator=(const CurvePyRef&)=delete;
    CurvePyRef(CurvePyRef&& other)noexcept:value(std::exchange(other.value,nullptr)){}
    CurvePyRef& operator=(CurvePyRef&& other)noexcept{if(this!=&other){Py_XDECREF(value);value=std::exchange(other.value,nullptr);}return *this;}
};
CurvePyRef publishedSplineShape();
std::string curveConstructionEvidence();
void persistCurveConstruction(PyObject*,const std::string&);
// Synchronous GUI-thread native capture; returned bytes own no host pointers.
std::string curveGeometryResult(App::Document&,PyObject* shape,const std::string& construction,const std::string& units,const std::string& outputName,const std::string& outputRole={});
void persistCurveGeometryResult(PyObject*,const std::string&);
CurvePyRef createCurveFeature(App::Document&,PyObject* shape,const char* name);
std::vector<std::array<double,3>> sampleCurve(PyObject* edge,std::size_t count);
}
