#include <App/DocumentObject.h>
#include <App/PropertyStandard.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/ViewProvider.h>
#include <Mod/Part/App/PropertyTopoShape.h>
#include <BRepTools_WireExplorer.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepAdaptor_CompCurve.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <GCPnts_UniformAbscissa.hxx>
#include <Geom_BSplineCurve.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
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

namespace OpenMatrix9Gui {

void validateCurveWire(const TopoDS_Wire& wire,std::size_t expectedEdges) {
    TopTools_IndexedDataMapOfShapeListOfShape incidence;TopExp::MapShapesAndAncestors(wire,TopAbs_VERTEX,TopAbs_EDGE,incidence);
    std::vector<std::uint64_t> degrees,edges;for(int i=1;i<=incidence.Extent();++i)degrees.push_back(incidence.FindFromIndex(i).Extent());
    TopTools_IndexedMapOfShape ids;TopExp::MapShapes(wire,TopAbs_EDGE,ids);
    for(BRepTools_WireExplorer explorer(wire);explorer.More();explorer.Next())edges.push_back(ids.FindIndex(explorer.Current()));
    const Om9WireInput input{degrees.data(),degrees.size(),edges.data(),edges.size(),expectedEdges};phase2Require(om9_phase2_wire_validate(&input));
}
TopoDS_Shape publishedNativeSplineShape() {
    const auto count=om9_spline_pole_count(),degree=om9_spline_degree(),knots=om9_spline_knot_count();
    if(count<=degree||count>256||!degree||knots<2)throw std::runtime_error("Invalid Rust spline output");
    NCollection_Array1<gp_Pnt> poles(1,int(count));NCollection_Array1<double> weights(1,int(count)),values(1,int(knots));NCollection_Array1<int> mults(1,int(knots));
    for(std::size_t i=0;i<count;++i){
        const double x=om9_spline_pole(i,0),y=om9_spline_pole(i,1),z=om9_spline_pole(i,2);
        if(!std::isfinite(x)||!std::isfinite(y)||!std::isfinite(z))throw std::runtime_error("Nonfinite spline pole");
        poles.SetValue(int(i)+1,gp_Pnt(x,y,z));weights.SetValue(int(i)+1,1.);
    }
    for(std::size_t i=0;i<knots;++i){const double knot=om9_spline_knot(i);if(!std::isfinite(knot))throw std::runtime_error("Nonfinite spline knot");values.SetValue(int(i)+1,knot);mults.SetValue(int(i)+1,int(om9_spline_multiplicity(i)));}
    Handle(Geom_BSplineCurve) curve=new Geom_BSplineCurve(poles,weights,values,mults,int(degree),om9_spline_periodic());
    BRepBuilderAPI_MakeEdge builder(curve);if(!builder.IsDone()||!BRepCheck_Analyzer(builder.Edge()).IsValid())throw std::runtime_error("Native kernel produced invalid spline");return builder.Edge();
}
App::DocumentObject* createCurveFeature(App::Document& doc,const TopoDS_Shape& shape,const char* name) {
    auto* object=doc.addObject("Part::Feature",name);auto* property=object?object->getPropertyByName<Part::PropertyPartShape>("Shape"):nullptr;if(!property)throw std::runtime_error("Cannot create native curve feature");property->setValue(shape);
    if(auto* gui=Gui::Application::Instance->getDocument(&doc))if(auto* view=gui->getViewProvider(object))if(auto* color=view->getPropertyByName<App::PropertyColor>("LineColor"))color->setValue(Base::Color(0.f,130.f/255.f,85.f/255.f));return object;
}
std::vector<std::array<double,3>> sampleCurve(const TopoDS_Shape& shape,std::size_t count) {
    if(count<2||count>4097)throw std::runtime_error("Native curve sampling budget exceeded");
    TopoDS_Wire wire;std::size_t edges=0;
    if(shape.ShapeType()==TopAbs_WIRE){wire=TopoDS::Wire(shape);for(TopExp_Explorer e(wire,TopAbs_EDGE);e.More();e.Next())++edges;}
    else {BRepBuilderAPI_MakeWire builder;for(TopExp_Explorer e(shape,TopAbs_EDGE);e.More();e.Next()){builder.Add(TopoDS::Edge(e.Current()));++edges;}if(!builder.IsDone())throw std::runtime_error("Native curve has no connected wire");wire=builder.Wire();}
    validateCurveWire(wire,edges);BRepAdaptor_CompCurve curve(wire,true);GCPnts_UniformAbscissa sampling(curve,int(count),1e-7);
    if(!sampling.IsDone()||sampling.NbPoints()!=int(count))throw std::runtime_error("Curve cannot be sampled completely");
    std::vector<std::array<double,3>> output;output.reserve(count);for(int i=1;i<=int(count);++i){const auto p=curve.Value(sampling.Parameter(i));if(!std::isfinite(p.X())||!std::isfinite(p.Y())||!std::isfinite(p.Z()))throw std::runtime_error("Invalid sampled coordinate");output.push_back({p.X(),p.Y(),p.Z()});}return output;
}
}
