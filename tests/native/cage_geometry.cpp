// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CageGeometry.h"
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Geom_BSplineCurve.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <TopoDS.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_Pln.hxx>
#include <cmath>
#include <functional>
#include <iostream>
#include <limits>
#include <stdexcept>
using namespace OpenMatrix9Gui;
namespace {
void require(bool ok,const char* why){if(!ok)throw std::runtime_error(why);}
void close(double a,double b){require(std::abs(a-b)<1e-7,"independent geometric expectation");}
void rejects(const std::function<void()>& operation,const char* expected){
    try{operation();}catch(const std::exception& e){
        if(std::string(e.what()).find(expected)==std::string::npos)
            throw std::runtime_error(std::string("Expected rejection '")+expected+"', actual: "+e.what());
        return;
    }
    throw std::runtime_error(std::string("Expected rejection: ")+expected);
}
double volume(const TopoDS_Shape& s){GProp_GProps g;BRepGProp::VolumeProperties(s,g);return g.Mass();}
double minX(const TopoDS_Shape& s){
    double value=std::numeric_limits<double>::infinity();
    for(TopExp_Explorer it(s,TopAbs_VERTEX);it.More();it.Next())value=std::min(value,BRep_Tool::Pnt(TopoDS::Vertex(it.Current())).X());
    return value;
}
}
int main(){try{
    TColgp_Array1OfPnt poles(1,4);poles.SetValue(1,gp_Pnt(0,0,0));poles.SetValue(2,gp_Pnt(1,0,0));
    poles.SetValue(3,gp_Pnt(2,1,0));poles.SetValue(4,gp_Pnt(3,1,0));
    TColStd_Array1OfReal knots(1,2),weights(1,4);knots.SetValue(1,2);knots.SetValue(2,5);
    weights.SetValue(1,1);weights.SetValue(2,2);weights.SetValue(3,3);weights.SetValue(4,1);
    TColStd_Array1OfInteger multiplicities(1,2);multiplicities.SetValue(1,4);multiplicities.SetValue(2,4);
    Handle(Geom_BSplineCurve) basis=new Geom_BSplineCurve(poles,weights,knots,multiplicities,3);
    auto edge=BRepBuilderAPI_MakeEdge(basis,2.,5.).Shape();edge.Reverse();
    gp_Trsf placement;placement.SetTranslation(gp_Vec(10,20,30));edge=BRepBuilderAPI_Transform(edge,placement,true).Shape();
    const auto before=cageShapeControlPoints(edge);require(before.size()==4,"rational single edge pole capture");
    close(before[0].x,10);close(before[0].y,20);close(before[0].z,30);
    auto mapped=before;for(auto& p:mapped)p.z+=(p.x-10)*(p.x-10);
    auto edited=cageDeformShape(edge,mapped,std::nullopt);
    require(BRepCheck_Analyzer(edited).IsValid()&&edited.Orientation()==edge.Orientation(),"nonlinear rational edge valid and orientation retained");
    double first,last;auto output=Handle(Geom_BSplineCurve)::DownCast(BRep_Tool::Curve(TopoDS::Edge(edited),first,last));
    require(!output.IsNull()&&output->Degree()==3&&output->NbPoles()==4,"edge structure retained");
    close(first,2);close(last,5);close(output->Knot(1),2);close(output->Knot(2),5);
    for(int i=1;i<=4;++i){close(output->Weight(i),weights.Value(i));close(output->Pole(i).Z(),mapped[i-1].z);}
    close(cageShapeControlPoints(edge)[3].z,30);close(basis->Pole(4).Z(),0); // immutable snapshots
    rejects([&]{cageDeformShape(edge,{},std::nullopt);},"pole count");
    mapped[0].z=std::numeric_limits<double>::quiet_NaN();rejects([&]{cageDeformShape(edge,mapped,std::nullopt);},"not finite");
    const gp_Pln plane(gp_Pnt(0,0,0),gp_Dir(0,0,1));auto face=BRepBuilderAPI_MakeFace(plane,0.,3.,0.,2.).Shape();
    face.Reverse();auto facePoles=cageShapeControlPoints(face);require(facePoles.size()==4,"rectangular analytic face exact NURBS poles");
    for(auto& p:facePoles)p.z=p.x*p.y;
    auto warped=cageDeformShape(face,facePoles,std::nullopt);require(BRepCheck_Analyzer(warped).IsValid(),"bilinear warped face valid");
    require(warped.Orientation()==face.Orientation(),"face orientation retained");
    auto warpedPoles=cageShapeControlPoints(warped);require(warpedPoles.size()==facePoles.size(),"surface structure retained");
    for(std::size_t i=0;i<warpedPoles.size();++i)close(warpedPoles[i].z,facePoles[i].z);
    for(const auto& p:cageShapeControlPoints(face))close(p.z,0);
    auto box=BRepPrimAPI_MakeBox(2.,3.,4.).Shape();require(cageShapeControlPoints(box).empty(),"solid requires affine edit");
    const std::array<double,16> identity{1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1};
    close(volume(cageDeformShape(box,{},identity)),24);
    rejects([&]{cageDeformShape(box,{},std::nullopt);},"requires an affine edit");
    const std::array<double,16> affine{2,.5,0,10,0,3,0,20,0,0,1,30,0,0,0,1};
    auto transformed=cageDeformShape(box,{},affine);require(BRepCheck_Analyzer(transformed).IsValid(),"arbitrary affine shear preserves valid closed solid");
    close(volume(transformed),144);close(volume(box),24);
    const std::array<double,16> uniform{2,0,0,10,0,2,0,20,0,0,2,30,0,0,0,1};
    close(volume(cageDeformShape(box,{},uniform)),192);
    const std::array<double,16> translated{1,0,0,10,0,1,0,20,0,0,1,30,0,0,0,1};
    close(volume(cageDeformShape(box,{},translated)),24);
    // Native capture already transforms its source into a world-space NURBS
    // BRep. Subsequent cage edits must also accept that converted snapshot.
    auto converted=cageDeformShape(box,{},identity);
    auto reconverted=cageDeformShape(converted,{},identity);
    auto shifted=cageDeformShape(reconverted,{},translated);
    auto chained=cageDeformShape(shifted,{},affine);
    require(BRepCheck_Analyzer(converted).IsValid()&&BRepCheck_Analyzer(reconverted).IsValid()
            &&BRepCheck_Analyzer(shifted).IsValid()&&BRepCheck_Analyzer(chained).IsValid(),"chained NURBS affine snapshots remain valid");
    close(volume(chained),144);close(minX(chained),40);
    close(volume(box),24);close(minX(box),0);close(minX(converted),0);close(minX(reconverted),0);close(minX(shifted),10);
    auto singular=affine;singular[0]=0;rejects([&]{cageDeformShape(box,{},singular);},"Singular cage affine");
    auto projective=affine;projective[12]=1;rejects([&]{cageDeformShape(box,{},projective);},"Invalid cage affine");
    BRepBuilderAPI_MakePolygon triangle;triangle.Add(gp_Pnt(0,0,0));triangle.Add(gp_Pnt(3,0,0));triangle.Add(gp_Pnt(0,2,0));triangle.Close();
    auto trimmed=BRepBuilderAPI_MakeFace(triangle.Wire()).Shape();require(cageShapeControlPoints(trimmed).empty(),"nonrectangular trim safely affine-only");
    rejects([&]{cageDeformShape(trimmed,{},std::nullopt);},"UV rectangle");
    BRepBuilderAPI_MakePolygon hole;hole.Add(gp_Pnt(1,.5,0));hole.Add(gp_Pnt(1,1.5,0));hole.Add(gp_Pnt(2,1.5,0));hole.Add(gp_Pnt(2,.5,0));hole.Close();
    BRepBuilderAPI_MakeFace perforated(plane,0.,3.,0.,2.);perforated.Add(hole.Wire());auto withHole=perforated.Shape();
    require(BRepCheck_Analyzer(withHole).IsValid(),"valid hole fixture");require(cageShapeControlPoints(withHole).empty(),"hole not silently lost");
    rejects([&]{cageDeformShape(withHole,{},std::nullopt);},"without holes");
    require(BRepCheck_Analyzer(cageDeformShape(withHole,{},affine)).IsValid(),"affine edit retains hole topology");
    std::cout<<"Cage geometry PASS; immutable NURBS edge/face, affine closed solid and explicit trim/hole rejection\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
