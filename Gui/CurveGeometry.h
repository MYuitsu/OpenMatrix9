#include <TopoDS_Shape.hxx>
#pragma once
#include <Python.h>
#include <array>
#include <vector>
#include <stdexcept>
#include <utility>
namespace App {class Document;class DocumentObject;}
class TopoDS_Wire;
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
TopoDS_Shape publishedNativeSplineShape();
App::DocumentObject* createCurveFeature(App::Document&,const TopoDS_Shape&,const char*);
std::vector<std::array<double,3>> sampleCurve(const TopoDS_Shape&,std::size_t);
void validateCurveWire(const TopoDS_Wire&,std::size_t);
CurvePyRef createCurveFeature(App::Document&,PyObject* shape,const char* name);
std::vector<std::array<double,3>> sampleCurve(PyObject* edge,std::size_t count);
}
