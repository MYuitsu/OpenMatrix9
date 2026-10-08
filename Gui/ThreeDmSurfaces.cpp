#include "ThreeDmGeometry.h"
#include <Geom_BSplineSurface.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_Circle.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <GeomConvert.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TColStd_Array2OfReal.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColStd_Array1OfInteger.hxx>
namespace OpenMatrix9Gui::ThreeDm {
std::array<double,2> importedSurfaceParameters(const ON_Surface& source,double u,double v){
    if(auto rev=ON_RevSurface::Cast(&source)){
        if(rev->m_bTransposed)std::swap(u,v);
        u=rev->m_angle.ParameterAt(rev->m_t.NormalizedParameterAt(u));
        if(auto arc=ON_ArcCurve::Cast(rev->m_curve))v=arc->m_arc.Domain().ParameterAt(arc->Domain().NormalizedParameterAt(v));
    }
    return {u,v};
}
Handle(Geom2d_Curve) importedSurfaceTrim(const ON_Surface& surface,const ON_Curve& source){
    auto curve=curve2d(source);
    if(ON_RevSurface::Cast(&surface)){
        auto spline=Handle(Geom2d_BSplineCurve)::DownCast(curve);
        if(spline.IsNull())throw ExchangeError("Revolution trim is not a NURBS curve");
        // This affine UV map changes poles only; weights, knots, direction and
        // the trim's own parameter domain are retained.
        for(int i=1;i<=spline->NbPoles();++i){auto point=spline->Pole(i);auto uv=importedSurfaceParameters(surface,point.X(),point.Y());spline->SetPole(i,gp_Pnt2d(uv[0],uv[1]));}
    }
    return curve;
}
Handle(Geom_Surface) importSurface(const ON_Surface& surface){
    if(auto rev=ON_RevSurface::Cast(&surface)){
        if(!rev->IsValid()||!rev->m_curve)throw ExchangeError("Invalid revolution surface");
        Handle(Geom_Curve) profile;
        if(auto arc=ON_ArcCurve::Cast(rev->m_curve)){
            auto& circle=arc->m_arc;auto& plane=circle.plane;
            profile=new Geom_Circle(gp_Ax2(gp_Pnt(plane.origin.x,plane.origin.y,plane.origin.z),gp_Dir(plane.zaxis.x,plane.zaxis.y,plane.zaxis.z),gp_Dir(plane.xaxis.x,plane.xaxis.y,plane.xaxis.z)),circle.radius);
        }else{
            ON_NurbsCurve check;if(rev->m_curve->GetNurbForm(check)!=1)throw ExchangeError("Revolution profile needs an explicit parameter adapter");
            profile=curve3d(*rev->m_curve);
        }
        auto direction=rev->m_axis.Direction();
        Handle(Geom_Surface) analytic=new Geom_SurfaceOfRevolution(profile,gp_Ax1(gp_Pnt(rev->m_axis.from.x,rev->m_axis.from.y,rev->m_axis.from.z),gp_Dir(direction.x,direction.y,direction.z)));
        auto profileDomain=rev->m_curve->Domain();if(auto arc=ON_ArcCurve::Cast(rev->m_curve))profileDomain=arc->m_arc.Domain();
        // A circle/revolution basis extends beyond the native arc/angle domain.
        // Keep finite native bounds so extrema, meshing and NURBS conversion
        // do not include the unused periodic part of the analytic basis.
        return new Geom_RectangularTrimmedSurface(analytic,rev->m_angle.Min(),rev->m_angle.Max(),profileDomain.Min(),profileDomain.Max());
    }
    ON_NurbsSurface n;if(!surface.GetNurbForm(n)||!n.IsValid())throw ExchangeError("Invalid NURBS surface");n.ClampEnd(0,2);n.ClampEnd(1,2);
    std::vector<double> k[2];std::vector<int> m[2];
    for(int d=0;d<2;++d){auto add=[&](double v){if(k[d].empty()||v!=k[d].back()){k[d].push_back(v);m[d].push_back(1);}else ++m[d].back();};add(n.Knot(d,0));for(int i=0;i<n.KnotCount(d);++i)add(n.Knot(d,i));add(n.Knot(d,n.KnotCount(d)-1));}
    TColgp_Array2OfPnt p(1,n.CVCount(0),1,n.CVCount(1));TColStd_Array2OfReal w(1,n.CVCount(0),1,n.CVCount(1));
    for(int u=0;u<n.CVCount(0);++u)for(int v=0;v<n.CVCount(1);++v){ON_3dPoint q;n.GetCV(u,v,q);p.SetValue(u+1,v+1,gp_Pnt(q.x,q.y,q.z));w.SetValue(u+1,v+1,n.Weight(u,v));}
    TColStd_Array1OfReal uk(1,int(k[0].size())),vk(1,int(k[1].size()));TColStd_Array1OfInteger um(1,int(m[0].size())),vm(1,int(m[1].size()));
    for(int i=0;i<int(k[0].size());++i){uk.SetValue(i+1,k[0][i]);um.SetValue(i+1,m[0][i]);}for(int i=0;i<int(k[1].size());++i){vk.SetValue(i+1,k[1][i]);vm.SetValue(i+1,m[1][i]);}
    return new Geom_BSplineSurface(p,w,uk,vk,um,vm,n.Degree(0),n.Degree(1),false,false);
}
std::unique_ptr<ON_NurbsSurface> exportSurface(const Handle(Geom_Surface)& s){
    auto n=GeomConvert::SurfaceToBSplineSurface(s);if(n->IsUPeriodic())n->SetUNotPeriodic();if(n->IsVPeriodic())n->SetVNotPeriodic();
    auto out=std::make_unique<ON_NurbsSurface>(3,n->IsURational()||n->IsVRational(),n->UDegree()+1,n->VDegree()+1,n->NbUPoles(),n->NbVPoles());
    for(int u=1;u<=n->NbUPoles();++u)for(int v=1;v<=n->NbVPoles();++v){auto p=n->Pole(u,v);double w=n->Weight(u,v);out->SetCV(u-1,v-1,ON_4dPoint(p.X()*w,p.Y()*w,p.Z()*w,w));}
    for(int d=0;d<2;++d){std::vector<double> flat;int count=d?n->NbVKnots():n->NbUKnots();for(int i=1;i<=count;++i)for(int j=0;j<(d?n->VMultiplicity(i):n->UMultiplicity(i));++j)flat.push_back(d?n->VKnot(i):n->UKnot(i));if(int(flat.size())!=out->KnotCount(d)+2)throw ExchangeError("Surface knot cardinality mismatch");for(int i=0;i<out->KnotCount(d);++i)out->SetKnot(d,i,flat[i+1]);}
    if(!out->IsValid())throw ExchangeError("Exported NURBS surface invalid");return out;
}
}
