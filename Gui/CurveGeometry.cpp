#include "CurveGeometry.h"
#include "RustBridge.h"
#include <App/Document.h>
#include <cmath>
namespace OpenMatrix9Gui {
CurvePyRef publishedSplineShape() {
    // OM9-CURVE-003/009: Rust fits poles; OCCT constructs the native B-spline.
    const auto count=om9_spline_pole_count(), degree=om9_spline_degree(), knots=om9_spline_knot_count();
    if(count<=degree||count>256||!degree||knots<2)throw std::runtime_error("Invalid Rust spline output");
    CurvePyRef app(PyImport_ImportModule("FreeCAD")),part(PyImport_ImportModule("Part"));
    CurvePyRef vector(PyObject_GetAttrString(app.value,"Vector"));
    CurvePyRef poles(PyList_New(0)),mults(PyList_New(0)),values(PyList_New(0));
    for(std::size_t i=0;i<count;++i){
        const double x=om9_spline_pole(i,0),y=om9_spline_pole(i,1),z=om9_spline_pole(i,2);
        if(!std::isfinite(x)||!std::isfinite(y)||!std::isfinite(z))throw std::runtime_error("Nonfinite spline pole");
        CurvePyRef p(PyObject_CallFunction(vector.value,"ddd",x,y,z));
        if(PyList_Append(poles.value,p.value)<0)throw std::runtime_error("Cannot append spline pole");
    }
    for(std::size_t i=0;i<knots;++i){
        CurvePyRef u(PyFloat_FromDouble(om9_spline_knot(i))),m(PyLong_FromSize_t(om9_spline_multiplicity(i)));
        if(PyList_Append(values.value,u.value)<0||PyList_Append(mults.value,m.value)<0)throw std::runtime_error("Cannot append spline knot");
    }
    CurvePyRef type(PyObject_GetAttrString(part.value,"BSplineCurve")),curve(PyObject_CallNoArgs(type.value));
    CurvePyRef built(PyObject_CallMethod(curve.value,"buildFromPolesMultsKnots","OOOOn",poles.value,mults.value,values.value,om9_spline_periodic()?Py_True:Py_False,Py_ssize_t(degree)));
    CurvePyRef shape(PyObject_CallMethod(curve.value,"toShape",nullptr));
    CurvePyRef valid(PyObject_CallMethod(shape.value,"isValid",nullptr));
    if(PyObject_IsTrue(valid.value)!=1)throw std::runtime_error("Part produced invalid spline geometry");
    return shape;
}
CurvePyRef createCurveFeature(App::Document& doc,PyObject* shape,const char* name) {
    CurvePyRef pyDoc(doc.getPyObject()),object(PyObject_CallMethod(pyDoc.value,"addObject","ss","Part::Feature",name));
    if(PyObject_SetAttrString(object.value,"Shape",shape)<0)throw std::runtime_error("Cannot assign curve shape");
    CurvePyRef view(PyObject_GetAttrString(object.value,"ViewObject")),color(Py_BuildValue("(ddd)",0.0,130.0/255.0,85.0/255.0));
    if(PyObject_SetAttrString(view.value,"LineColor",color.value)<0)throw std::runtime_error("Cannot assign curve color");
    return object;
}
std::vector<std::array<double,3>> sampleCurve(PyObject* edge,std::size_t count) {
    CurvePyRef method(PyObject_GetAttrString(edge,"discretize")),args(PyTuple_New(0)),kwargs(Py_BuildValue("{s:n}","Number",Py_ssize_t(count)));
    CurvePyRef points(PyObject_Call(method.value,args.value,kwargs.value));
    const auto n=PySequence_Size(points.value);
    if(n!=Py_ssize_t(count))throw std::runtime_error("Curve cannot be sampled with the requested count");
    std::vector<std::array<double,3>> output;output.reserve(count);
    for(Py_ssize_t i=0;i<n;++i){
        CurvePyRef p(PySequence_GetItem(points.value,i));std::array<double,3> xyz;
        for(int j=0;j<3;++j){CurvePyRef v(PyObject_GetAttrString(p.value,j==0?"x":j==1?"y":"z"));xyz[j]=PyFloat_AsDouble(v.value);if(PyErr_Occurred()||!std::isfinite(xyz[j]))throw std::runtime_error("Invalid sampled coordinate");}
        output.push_back(xyz);
    }
    return output;
}
}
