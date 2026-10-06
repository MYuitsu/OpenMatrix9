#include "ThreeDmGeometry.h"
#include <Geom_BSplineCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <GeomConvert.hxx>
#include <Geom2dConvert.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Tool.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TColgp_Array1OfPnt2d.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <cmath>
namespace OpenMatrix9Gui::ThreeDm {
static ON_NurbsCurve nurbs(const ON_Curve& c){ON_NurbsCurve n;if(!c.IsValid()||!c.GetNurbForm(n))throw ExchangeError("Invalid or unsupported curve");if(!n.ClampEnd(2))throw ExchangeError("Cannot clamp NURBS curve");return n;}
static void knots(const ON_NurbsCurve& n,std::vector<double>& k,std::vector<int>& m){
    auto append=[&](double v){if(!std::isfinite(v))throw ExchangeError("Nonfinite knot");if(k.empty()||v!=k.back()){k.push_back(v);m.push_back(1);}else ++m.back();};
    append(n.Knot(0));for(int i=0;i<n.KnotCount();++i)append(n.Knot(i));append(n.Knot(n.KnotCount()-1));
}
Handle(Geom_Curve) curve3d(const ON_Curve& c){
    auto n=nurbs(c);std::vector<double> k;std::vector<int> m;knots(n,k,m);
    TColgp_Array1OfPnt p(1,n.CVCount());TColStd_Array1OfReal w(1,n.CVCount()),ks(1,int(k.size()));TColStd_Array1OfInteger ms(1,int(m.size()));
    for(int i=0;i<n.CVCount();++i){ON_3dPoint v;n.GetCV(i,v);p.SetValue(i+1,gp_Pnt(v.x,v.y,v.z));w.SetValue(i+1,n.Weight(i));}
    for(int i=0;i<int(k.size());++i){ks.SetValue(i+1,k[i]);ms.SetValue(i+1,m[i]);}
    return new Geom_BSplineCurve(p,w,ks,ms,n.Degree(),false);
}
Handle(Geom2d_Curve) curve2d(const ON_Curve& c){
    auto n=nurbs(c);std::vector<double> k;std::vector<int> m;knots(n,k,m);
    TColgp_Array1OfPnt2d p(1,n.CVCount());TColStd_Array1OfReal w(1,n.CVCount()),ks(1,int(k.size()));TColStd_Array1OfInteger ms(1,int(m.size()));
    for(int i=0;i<n.CVCount();++i){ON_3dPoint v;n.GetCV(i,v);p.SetValue(i+1,gp_Pnt2d(v.x,v.y));w.SetValue(i+1,n.Weight(i));}
    for(int i=0;i<int(k.size());++i){ks.SetValue(i+1,k[i]);ms.SetValue(i+1,m[i]);}
    return new Geom2d_BSplineCurve(p,w,ks,ms,n.Degree(),false);
}
TopoDS_Shape importCurve(const ON_Curve& c,double){auto n=curve3d(c);auto d=c.Domain();BRepBuilderAPI_MakeEdge edge(n,d.Min(),d.Max());if(!edge.IsDone())throw ExchangeError("Curve edge construction failed");return edge.Edge();}
template<class Curve> static std::unique_ptr<ON_Curve> toON(const Curve& n,int dimension){
    auto out=std::make_unique<ON_NurbsCurve>(dimension,n->IsRational(),n->Degree()+1,n->NbPoles());
    for(int i=1;i<=n->NbPoles();++i){auto p=n->Pole(i);double w=n->Weight(i);double z=0;if constexpr(requires{p.Z();})z=p.Z();out->SetCV(i-1,ON_4dPoint(p.X()*w,p.Y()*w,z*w,w));}
    std::vector<double> flat;for(int i=1;i<=n->NbKnots();++i)for(int j=0;j<n->Multiplicity(i);++j)flat.push_back(n->Knot(i));
    if(int(flat.size())!=out->KnotCount()+2)throw ExchangeError("NURBS knot cardinality mismatch");
    for(int i=0;i<out->KnotCount();++i)out->SetKnot(i,flat[i+1]);ON_wString log;ON_TextLog errors(log);if(!out->IsValid(&errors))throw ExchangeError(std::string("Exported NURBS curve invalid: ")+ON_String(log).Array());return out;
}
std::unique_ptr<ON_Curve> curveON(const Handle(Geom_Curve)& c,double a,double b){
    auto n=GeomConvert::CurveToBSplineCurve(new Geom_TrimmedCurve(c,a,b));if(n->IsPeriodic())n->SetNotPeriodic();return toON(n,3);
}
std::unique_ptr<ON_Curve> curveON(const Handle(Geom2d_Curve)& c,double a,double b){
    auto spline=Handle(Geom2d_BSplineCurve)::DownCast(c);
    Handle(Geom2d_BSplineCurve) n;
    if(!spline.IsNull()){
        n=Handle(Geom2d_BSplineCurve)::DownCast(spline->Copy());
        const double parameterTolerance=1e-10*std::max({1.0,std::abs(a),std::abs(b)});
        if(std::abs(a-n->FirstParameter())>parameterTolerance||std::abs(b-n->LastParameter())>parameterTolerance)n->Segment(a,b);
    }else n=Geom2dConvert::CurveToBSplineCurve(new Geom2d_TrimmedCurve(c,a,b));
    if(n->IsPeriodic())n->SetNotPeriodic();return toON(n,2);
}
std::unique_ptr<ON_Curve> exportCurve(const TopoDS_Edge& e,double){double a,b;TopLoc_Location loc;auto c=BRep_Tool::Curve(e,loc,a,b);if(c.IsNull())throw ExchangeError("Edge has no 3D curve");c=Handle(Geom_Curve)::DownCast(c->Transformed(loc.Transformation()));auto out=curveON(c,a,b);if(e.Orientation()==TopAbs_REVERSED)out->Reverse();return out;}
}
