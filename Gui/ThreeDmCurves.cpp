#include "ThreeDmGeometry.h"
#include "NurbsBasisAdapter.h"
#include "CurveConversionContract.h"
#include "NativeInputTrace.h"
#include "NativeInputTree.h"
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
struct PreparedCurve {
    ON_NurbsCurve basis;
    int form = 0;
    std::array<double,2> getNurbFormDomain{};
    bool beforeClampPeriodic = false;
};
static PreparedCurve nurbs(const ON_Curve& c,double tolerance=0.){
    InputTree::validate(c);
    PreparedCurve prepared;auto& n=prepared.basis;
    if(!c.IsValid())throw ExchangeError("Invalid or unsupported curve");
    OM9_NATIVE_INPUT_ENTRY("curve.GetNurbForm");prepared.form=c.GetNurbForm(n,tolerance);
    if(prepared.form!=1&&prepared.form!=2)throw ExchangeError("Invalid or unsupported curve");
    BasisContract::checkedBasis(n);
    prepared.getNurbFormDomain={n.Domain().Min(),n.Domain().Max()};
    prepared.beforeClampPeriodic=n.IsPeriodic();
    if(!n.ClampEnd(2))throw ExchangeError("Cannot clamp NURBS curve");
    BasisContract::checkedBasis(n);return prepared;
}
static void knots(const ON_NurbsCurve& n,std::vector<double>& k,std::vector<int>& m){
    auto append=[&](double v){if(!std::isfinite(v))throw ExchangeError("Nonfinite knot");if(k.empty()||v!=k.back()){k.push_back(v);m.push_back(1);}else ++m.back();};
    append(n.Knot(0));for(int i=0;i<n.KnotCount();++i)append(n.Knot(i));append(n.Knot(n.KnotCount()-1));
}
static Handle(Geom_Curve) curve3dFromNurbs(const ON_NurbsCurve& n){
    std::vector<double> k;std::vector<int> m;knots(n,k,m);
    TColgp_Array1OfPnt p(1,n.CVCount());TColStd_Array1OfReal w(1,n.CVCount()),ks(1,int(k.size()));TColStd_Array1OfInteger ms(1,int(m.size()));
    for(int i=0;i<n.CVCount();++i){ON_3dPoint v;n.GetCV(i,v);p.SetValue(i+1,gp_Pnt(v.x,v.y,v.z));w.SetValue(i+1,n.Weight(i));}
    for(int i=0;i<int(k.size());++i){ks.SetValue(i+1,k[i]);ms.SetValue(i+1,m[i]);}
    OM9_NATIVE_INPUT_ENTRY("curve.OCCTConstructor");return new Geom_BSplineCurve(p,w,ks,ms,n.Degree(),false);
}
Handle(Geom_Curve) curve3d(const ON_Curve& c){
    auto prepared=nurbs(c);return curve3dFromNurbs(prepared.basis);
}
Handle(Geom2d_Curve) curve2d(const ON_Curve& c){
    auto prepared=nurbs(c);const auto& n=prepared.basis;std::vector<double> k;std::vector<int> m;knots(n,k,m);
    TColgp_Array1OfPnt2d p(1,n.CVCount());TColStd_Array1OfReal w(1,n.CVCount()),ks(1,int(k.size()));TColStd_Array1OfInteger ms(1,int(m.size()));
    for(int i=0;i<n.CVCount();++i){ON_3dPoint v;n.GetCV(i,v);p.SetValue(i+1,gp_Pnt2d(v.x,v.y));w.SetValue(i+1,n.Weight(i));}
    for(int i=0;i<int(k.size());++i){ks.SetValue(i+1,k[i]);ms.SetValue(i+1,m[i]);}
    OM9_NATIVE_INPUT_ENTRY("curve2d.OCCTConstructor");return new Geom2d_BSplineCurve(p,w,ks,ms,n.Degree(),false);
}
static TopoDS_Shape curveEdge(const Handle(Geom_Curve)& n,const ON_Interval& d){BRepBuilderAPI_MakeEdge edge(n,d.Min(),d.Max());if(!edge.IsDone())throw ExchangeError("Curve edge construction failed");return edge.Edge();}
TopoDS_Shape importCurve(const ON_Curve& c,double){auto target=curve3d(c);const auto domain=c.Domain();return curveEdge(target,domain);}
CurveConversion::ObservedCurve CurveConversion::importCurveObserved(
    const ON_Curve& c,Options options,std::span<const double> parameters){
    try {
        // Domain/CRC can read raw native storage. Gate its envelope first;
        // complete shared value validation still precedes GetNurbForm below.
        InputTree::validate(c);
        Detail::preflight(c,options,parameters);
        const auto sourceCrc=c.DataCRC(0);
        auto prepared=nurbs(c,options.conversionTolerance);
        auto target=curve3dFromNurbs(prepared.basis);
        auto report=Detail::observe(c,prepared.basis,target,prepared.form,
            prepared.getNurbFormDomain,prepared.beforeClampPeriodic,options,parameters);
        auto shape=curveEdge(target,c.Domain());
        if(c.DataCRC(0)!=sourceCrc)throw ExchangeError("Curve conversion unexpectedly changed source");
        report.sourceCrc=sourceCrc;
        return {std::move(shape),std::move(report)};
    } catch(const Standard_Failure& e) {
        throw ExchangeError(std::string("Native curve query failed: ")+e.GetMessageString());
    }
}
template<class Curve> static std::unique_ptr<ON_Curve> toON(const Curve& n,int dimension){
    BasisContract::checkedBasis(n);
    OM9_NATIVE_INPUT_ENTRY("curve.ONConstructor");auto out=std::make_unique<ON_NurbsCurve>(dimension,n->IsRational(),n->Degree()+1,n->NbPoles());
    for(int i=1;i<=n->NbPoles();++i){auto p=n->Pole(i);double w=n->Weight(i);double z=0;if constexpr(requires{p.Z();})z=p.Z();out->SetCV(i-1,ON_4dPoint(p.X()*w,p.Y()*w,z*w,w));}
    std::vector<double> flat;for(int i=1;i<=n->NbKnots();++i)for(int j=0;j<n->Multiplicity(i);++j)flat.push_back(n->Knot(i));
    if(int(flat.size())!=out->KnotCount()+2)throw ExchangeError("NURBS knot cardinality mismatch");
    for(int i=0;i<out->KnotCount();++i)out->SetKnot(i,flat[i+1]);ON_wString log;ON_TextLog errors(log);if(!out->IsValid(&errors))throw ExchangeError(std::string("Exported NURBS curve invalid: ")+ON_String(log).Array());return out;
}
std::unique_ptr<ON_Curve> curveON(const Handle(Geom_Curve)& c,double a,double b){
    InputTree::validate(c);
    OM9_NATIVE_INPUT_ENTRY("curve.GeomConvert");auto n=GeomConvert::CurveToBSplineCurve(new Geom_TrimmedCurve(c,a,b));BasisContract::checkedBasis(n);if(n->IsPeriodic())n->SetNotPeriodic();return toON(n,3);
}
std::unique_ptr<ON_Curve> curveON(const Handle(Geom2d_Curve)& c,double a,double b){
    InputTree::validate(c);
    auto spline=Handle(Geom2d_BSplineCurve)::DownCast(c);
    Handle(Geom2d_BSplineCurve) n;
    if(!spline.IsNull()){
        BasisContract::checkedBasis(spline);
        OM9_NATIVE_INPUT_ENTRY("curve2d.Copy");n=Handle(Geom2d_BSplineCurve)::DownCast(spline->Copy());
        const double parameterTolerance=1e-10*std::max({1.0,std::abs(a),std::abs(b)});
        if(std::abs(a-n->FirstParameter())>parameterTolerance||std::abs(b-n->LastParameter())>parameterTolerance)n->Segment(a,b);
    }else {OM9_NATIVE_INPUT_ENTRY("curve2d.GeomConvert");n=Geom2dConvert::CurveToBSplineCurve(new Geom2d_TrimmedCurve(c,a,b));}
    BasisContract::checkedBasis(n);if(n->IsPeriodic())n->SetNotPeriodic();return toON(n,2);
}
std::unique_ptr<ON_Curve> exportCurve(const TopoDS_Edge& e,double){double a,b;TopLoc_Location loc;auto c=BRep_Tool::Curve(e,loc,a,b);if(c.IsNull())throw ExchangeError("Edge has no 3D curve");InputTree::validate(c);OM9_NATIVE_INPUT_ENTRY("curve.TransformCopy");c=Handle(Geom_Curve)::DownCast(c->Transformed(loc.Transformation()));auto out=curveON(c,a,b);if(e.Orientation()==TopAbs_REVERSED)out->Reverse();return out;}
}
