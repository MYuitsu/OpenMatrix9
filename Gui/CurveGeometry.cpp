#include "CurveGeometry.h"
#include "RustBridge.h"
#include <App/Document.h>
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
#include <cmath>
namespace OpenMatrix9Gui {
void validateCurveWire(const TopoDS_Wire& wire,std::size_t expectedEdges) {
    TopTools_IndexedDataMapOfShapeListOfShape incidence;TopExp::MapShapesAndAncestors(wire,TopAbs_VERTEX,TopAbs_EDGE,incidence);
    std::vector<std::uint64_t> degrees,edges;for(int i=1;i<=incidence.Extent();++i)degrees.push_back(incidence.FindFromIndex(i).Extent());
    TopTools_IndexedMapOfShape ids;TopExp::MapShapes(wire,TopAbs_EDGE,ids);
    for(BRepTools_WireExplorer explorer(wire);explorer.More();explorer.Next())edges.push_back(ids.FindIndex(explorer.Current()));
    const Om9WireInput input{degrees.data(),degrees.size(),edges.data(),edges.size(),expectedEdges};phase2Require(om9_phase2_wire_validate(&input));
}
TopoDS_Shape publishedSplineShape() {
    const auto count=om9_spline_pole_count(),degree=om9_spline_degree(),knots=om9_spline_knot_count();
    if(count<=degree||count>256||!degree||knots<2)throw std::runtime_error("Invalid Rust spline output");
    NCollection_Array1<gp_Pnt> poles(1,int(count));NCollection_Array1<double> weights(1,int(count)),values(1,int(knots));NCollection_Array1<int> mults(1,int(knots));
    for(std::size_t i=0;i<count;++i){poles.SetValue(int(i)+1,gp_Pnt(om9_spline_pole(i,0),om9_spline_pole(i,1),om9_spline_pole(i,2)));weights.SetValue(int(i)+1,1.);}
    for(std::size_t i=0;i<knots;++i){values.SetValue(int(i)+1,om9_spline_knot(i));mults.SetValue(int(i)+1,int(om9_spline_multiplicity(i)));}
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
    std::vector<std::array<double,3>> output;output.reserve(count);for(int i=1;i<=int(count);++i){const auto p=curve.Value(sampling.Parameter(i));output.push_back({p.X(),p.Y(),p.Z()});}return output;
}
}
