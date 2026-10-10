#include "ThreeDmGeometry.h"
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <Geom_BSplineCurve.hxx>
#include <iostream>
#include <stdexcept>
using namespace OpenMatrix9Gui::ThreeDm;
static void check(bool value,const char* name){if(!value)throw std::runtime_error(name);}
int main(){try{
    ON::Begin();ON_3dPoint points[6];for(int i=0;i<6;++i)points[i]=ON_3dPoint(5*cos(i*ON_PI/3),5*sin(i*ON_PI/3),i%2);
    for(int degree:{3,5})for(bool rational:{false,true}) {
    ON_NurbsCurve source;check(source.CreatePeriodicUniformNurbs(3,degree+1,6,points,1.)&&source.IsPeriodic(),"Periodic source fixture");
    if(rational){source.MakeRational();for(int i=0;i<source.CVCount();++i){ON_3dPoint p;source.GetCV(i,p);double w=1.+.1*(i%6);source.SetCV(i,ON_4dPoint(p.x*w,p.y*w,p.z*w,w));}
        const double u[6]={0.,.6,1.7,3.,3.9,5.2};for(int i=0;i<source.KnotCount();++i){int j=i-degree+1,cycle=j/6,index=j%6;if(index<0){index+=6;--cycle;}source.SetKnot(i,u[index]+6.*cycle);}}
    check(source.IsValid()&&source.IsPeriodic(),"Rational/nonuniform periodic source fixture");
    auto shape=importCurve(source,1e-7);double first,last;auto curve=Handle(Geom_BSplineCurve)::DownCast(BRep_Tool::Curve(TopoDS::Edge(shape),first,last));
    check(!curve.IsNull()&&curve->IsPeriodic(),"Imported periodic curve keeps periodic basis");
    auto output=exportCurve(TopoDS::Edge(shape),1e-7);
    auto on=ON_NurbsCurve::Cast(output.get());std::cerr<<"OCCT poles="<<curve->NbPoles()<<" knots="<<curve->NbKnots()<<" seamMultiplicity="<<curve->Multiplicity(1)<<";ON poles="<<on->CVCount()<<" knots="<<on->KnotCount()<<" periodic="<<on->IsPeriodic()<<'\n';
    check(output->IsPeriodic(),"Exported periodic curve keeps periodic basis");
    for(int i=0;i<=64;++i){double t=source.Domain().ParameterAt(i/64.);auto p=curve->Value(t);auto a=source.PointAt(t),b=output->PointAt(output->Domain().ParameterAt(i/64.));check(p.Distance(gp_Pnt(a.x,a.y,a.z))<1e-9&&a.DistanceTo(b)<1e-9,"Periodic sample geometry and domain preserved");}
    }
    std::cout<<"Phase2 periodic import/export: 4 fixtures / 276 checks passed\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
