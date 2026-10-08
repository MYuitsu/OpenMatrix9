#pragma once
#include <Python.h>
#include <array>
#include <vector>
#include <stdexcept>
#include <utility>
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
};
CurvePyRef publishedSplineShape();
CurvePyRef createCurveFeature(App::Document&,PyObject* shape,const char* name);
std::vector<std::array<double,3>> sampleCurve(PyObject* edge,std::size_t count);
}
