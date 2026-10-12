// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CageGeometry.h"
#include <Base/Exception.h>
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <GeomConvert.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Precision.hxx>
#include <Standard_Failure.hxx>
#include <TopoDS.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_GTrsf.hxx>
#include <algorithm>
#include <cmath>
#include <stdexcept>

namespace OpenMatrix9Gui {
namespace {
struct Unsupported:std::runtime_error {using std::runtime_error::runtime_error;};
void valid(const TopoDS_Shape& s,const char* message){
    if(s.IsNull()||!BRepCheck_Analyzer(s,true).IsValid())throw std::runtime_error(message);
}
void oriented(const TopoDS_Shape& s){
    if(s.Orientation()!=TopAbs_FORWARD&&s.Orientation()!=TopAbs_REVERSED)
        throw Unsupported("Nonlinear cage editing requires an oriented edge or face");
}
bool finiteRange(double a,double b){return std::isfinite(a)&&std::isfinite(b)
    &&std::abs(a)<1e100&&std::abs(b)<1e100&&a<b;}
Handle(Geom_BSplineCurve) curve(const TopoDS_Shape& s){
    if(s.ShapeType()!=TopAbs_EDGE)throw Unsupported("Nonlinear cage deformation supports only a single edge or rectangular face");
    oriented(s);auto e=TopoDS::Edge(s);double first=0,last=0;
    if(BRep_Tool::Degenerated(e))throw Unsupported("Degenerate edge requires an affine cage edit");
    auto original=BRep_Tool::Curve(e,first,last);
    if(original.IsNull()||!finiteRange(first,last))throw Unsupported("Cage curve must have finite source bounds");
    Handle(Geom_TrimmedCurve) bounded=new Geom_TrimmedCurve(original,first,last);
    auto result=GeomConvert::CurveToBSplineCurve(bounded);
    if(result.IsNull())throw Unsupported("Source curve cannot be converted exactly to NURBS");
    return Handle(Geom_BSplineCurve)::DownCast(result->Copy());
}
// Certify a straight monotone pcurve; sampling alone cannot establish that a
// trim remains on one rectangle side between the sampled points.
bool straight(const Handle(Geom2d_Curve)& source,double first,double last,int fixed,double value,double lo,double hi){
    if(source.IsNull()||!finiteRange(first,last))return false;
    auto base=source;
    while(auto trimmed=Handle(Geom2d_TrimmedCurve)::DownCast(base))base=trimmed->BasisCurve();
    const double eps=Precision::PConfusion()*10;
    auto onSide=[&](const gp_Pnt2d& p){return std::abs(p.Coord(fixed+1)-value)<=eps
        &&p.Coord(2-fixed)>=lo-eps&&p.Coord(2-fixed)<=hi+eps;};
    auto a=source->Value(first),b=source->Value(last);
    if(!onSide(a)||!onSide(b))return false;
    const double av=a.Coord(2-fixed),bv=b.Coord(2-fixed);
    if(!((std::abs(av-lo)<=eps&&std::abs(bv-hi)<=eps)||(std::abs(av-hi)<=eps&&std::abs(bv-lo)<=eps)))return false;
    if(!Handle(Geom2d_Line)::DownCast(base).IsNull())return true;
    auto spline=Handle(Geom2d_BSplineCurve)::DownCast(base);
    if(spline.IsNull()||spline->IsPeriodic())return false;
    spline=Handle(Geom2d_BSplineCurve)::DownCast(spline->Copy());spline->Segment(first,last);
    double previous=av;const double sign=bv>av?1:-1;
    for(int i=1;i<=spline->NbPoles();++i){const auto p=spline->Pole(i);
        if(!onSide(p)||!std::isfinite(spline->Weight(i))||spline->Weight(i)<=0)return false;
        const double v=p.Coord(2-fixed);if(sign*(v-previous)<-eps)return false;previous=v;}
    return true;
}
Handle(Geom_BSplineSurface) surface(const TopoDS_Shape& s){
    if(s.ShapeType()!=TopAbs_FACE)throw Unsupported("Nonlinear cage deformation supports only a single edge or rectangular face");
    oriented(s);const auto face=TopoDS::Face(s);int wires=0;
    for(TopExp_Explorer it(face,TopAbs_WIRE);it.More();it.Next())++wires;
    if(wires!=1)throw Unsupported("Nonlinear cage deformation requires one rectangular face without holes");
    double u0,u1,v0,v1;BRepTools::UVBounds(face,u0,u1,v0,v1);
    if(!finiteRange(u0,u1)||!finiteRange(v0,v1))throw Unsupported("Cage face must have finite UV bounds");
    std::array<bool,4> sides{};int edges=0;
    for(TopExp_Explorer it(face,TopAbs_EDGE);it.More();it.Next()){
        const auto edge=TopoDS::Edge(it.Current());++edges;
        if(BRep_Tool::Degenerated(edge)||BRep_Tool::IsClosed(edge,face))
            throw Unsupported("Seam or collapsed face boundary requires an affine cage edit");
        double a,b;auto pc=BRep_Tool::CurveOnSurface(edge,face,a,b);int side=-1;
        if(straight(pc,a,b,0,u0,v0,v1))side=0;
        else if(straight(pc,a,b,0,u1,v0,v1))side=1;
        else if(straight(pc,a,b,1,v0,u0,u1))side=2;
        else if(straight(pc,a,b,1,v1,u0,u1))side=3;
        if(side<0||sides[side])throw Unsupported("Trimmed face boundary is not the complete UV rectangle");
        sides[side]=true;
    }
    if(edges!=4||!std::all_of(sides.begin(),sides.end(),[](bool x){return x;}))
        throw Unsupported("Trimmed face boundary is not the complete UV rectangle");
    const auto original=BRep_Tool::Surface(face);
    if(original.IsNull())throw Unsupported("Source face has no surface");
    Handle(Geom_RectangularTrimmedSurface) bounded=new Geom_RectangularTrimmedSurface(original,u0,u1,v0,v1);
    auto result=GeomConvert::SurfaceToBSplineSurface(bounded);
    if(result.IsNull())throw Unsupported("Source surface cannot be converted exactly to NURBS");
    result=Handle(Geom_BSplineSurface)::DownCast(result->Copy());
    BRepBuilderAPI_MakeFace patch(result,std::max(Precision::Confusion(),BRep_Tool::Tolerance(face)));
    if(!patch.IsDone())throw Unsupported("Cannot reconstruct source rectangular face");
    GProp_GProps sourceArea,patchArea;BRepGProp::SurfaceProperties(face,sourceArea);BRepGProp::SurfaceProperties(patch.Face(),patchArea);
    if(!std::isfinite(sourceArea.Mass())||!std::isfinite(patchArea.Mass())
       ||std::abs(sourceArea.Mass()-patchArea.Mass())>1e-7*std::max(1.,std::abs(sourceArea.Mass())))
        throw Unsupported("Rectangular NURBS patch does not preserve source face area");
    return result;
}
gp_Pnt point(const Base::Vector3d& p){
    if(!std::isfinite(p.x)||!std::isfinite(p.y)||!std::isfinite(p.z))throw std::runtime_error("Mapped cage pole is not finite");
    return gp_Pnt(p.x,p.y,p.z);
}
}
std::vector<Base::Vector3d> cageShapeControlPoints(const TopoDS_Shape& s){
    valid(s,"Invalid source shape for cage binding");std::vector<Base::Vector3d> points;
    try{
        if(s.ShapeType()==TopAbs_EDGE){auto c=curve(s);for(int i=1;i<=c->NbPoles();++i){auto p=c->Pole(i);points.emplace_back(p.X(),p.Y(),p.Z());}}
        else if(s.ShapeType()==TopAbs_FACE){auto c=surface(s);for(int u=1;u<=c->NbUPoles();++u)for(int v=1;v<=c->NbVPoles();++v){auto p=c->Pole(u,v);points.emplace_back(p.X(),p.Y(),p.Z());}}
    }catch(const Unsupported&){return {};}catch(const Standard_Failure&){return {};}
    return points;
}
TopoDS_Shape cageDeformShape(const TopoDS_Shape& source,const std::vector<Base::Vector3d>& poles,std::optional<std::array<double,16>> affine){
    const char* stage="validate native source";
    try{
        valid(source,"Invalid source shape for cage deformation");
        TopoDS_Shape result;
        if(affine){
            const auto& m=*affine;
            if(!std::all_of(m.begin(),m.end(),[](double x){return std::isfinite(x);})
               ||std::abs(m[12])>1e-12||std::abs(m[13])>1e-12||std::abs(m[14])>1e-12||std::abs(m[15]-1)>1e-12)
                throw std::runtime_error("Invalid cage affine transform");
            gp_GTrsf transform;
            for(int r=0;r<3;++r)for(int c=0;c<4;++c)transform.SetValue(r+1,c+1,m[r*4+c]);
            // Keep the full general matrix (gp_Other). SetForm is unnecessary
            // for GTransform and can reclassify a matrix populated by SetValue
            // without restoring the separate uniform-scale field.
            if(transform.IsSingular())throw std::runtime_error("Singular cage affine transform");
            // Match FreeCAD TopoShape::transformGShape: isolate native geometry
            // before GTransform, including cached intermediates from an earlier
            // transform. The recorded source and its topology remain immutable.
            stage="copy native affine source";BRepBuilderAPI_Copy copied(source,true,false);
            stage="OCCT affine transformation";BRepBuilderAPI_GTransform operation(copied.Shape(),transform,true);
            if(!operation.IsDone())throw std::runtime_error("Native affine cage transformation failed");result=operation.Shape();
        }else if(source.ShapeType()==TopAbs_EDGE){
            auto c=curve(source);
            if(poles.size()!=static_cast<std::size_t>(c->NbPoles()))throw std::runtime_error("Mapped edge cage pole count differs from source");
            for(int i=1;i<=c->NbPoles();++i)c->SetPole(i,point(poles[i-1]));
            BRepBuilderAPI_MakeEdge edge(c,c->FirstParameter(),c->LastParameter());
            if(!edge.IsDone())throw std::runtime_error("Native cage NURBS edge construction failed");result=edge.Edge();result.Orientation(source.Orientation());
        }else if(source.ShapeType()==TopAbs_FACE){
            auto c=surface(source);
            if(poles.size()!=static_cast<std::size_t>(c->NbUPoles())*c->NbVPoles())throw std::runtime_error("Mapped face cage pole count differs from source");
            std::size_t index=0;for(int u=1;u<=c->NbUPoles();++u)for(int v=1;v<=c->NbVPoles();++v)c->SetPole(u,v,point(poles[index++]));
            BRepBuilderAPI_MakeFace face(c,std::max(Precision::Confusion(),BRep_Tool::Tolerance(TopoDS::Face(source))));
            if(!face.IsDone())throw std::runtime_error("Native cage NURBS face construction failed");result=face.Face();result.Orientation(source.Orientation());
        }else throw Unsupported("Nonlinear cage deformation supports only a single edge or rectangular face; this topology requires an affine edit");
        stage="validate deformed native topology";valid(result,"Cage deformation produced invalid native topology");return result;
    }catch(const Base::Exception& e){throw std::runtime_error(std::string(stage)+": "+e.what());
    }catch(const Standard_Failure& e){throw std::runtime_error(std::string("Native cage geometry failure: ")+(e.GetMessageString()?e.GetMessageString():"OCCT failure"));}
}
}
