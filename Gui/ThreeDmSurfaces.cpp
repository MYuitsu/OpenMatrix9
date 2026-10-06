#include "ThreeDmGeometry.h"
#include <Geom_BSplineSurface.hxx>
#include <GeomConvert.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TColStd_Array2OfReal.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColStd_Array1OfInteger.hxx>
namespace OpenMatrix9Gui::ThreeDm {
Handle(Geom_Surface) importSurface(const ON_Surface& surface){
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
