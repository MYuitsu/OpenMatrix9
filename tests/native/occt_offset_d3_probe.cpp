// SPDX-License-Identifier: LGPL-2.1-or-later
// Compare the shared OCCT helper and real curve/adaptor APIs with an
// independent 70-digit rational-offset derivative fixture.
#include <Geom_BezierCurve.hxx>
#include <Geom_OffsetCurve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <Geom_OffsetCurveUtils.pxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <gp_Trsf.hxx>
#include <algorithm>
#include <cmath>
#include <iostream>

struct Query { double u, x, y, z; };
// Values derived independently from the normalized rational-offset definition.
static constexpr Query queries[] = {
    {0.1,-13.535143916888291,-0.5573614450242469,-1.864017480263903}
};
int main()
{
    TColgp_Array1OfPnt poles(1,3);
    poles(1)=gp_Pnt(2,0,1);poles(2)=gp_Pnt(4,2,3);poles(3)=gp_Pnt(3,5,6);
    TColStd_Array1OfReal weights(1,3);
    weights(1)=1;weights(2)=std::sqrt(0.5);weights(3)=1;
    occ::handle<Geom_BezierCurve> basis=new Geom_BezierCurve(poles,weights);
    occ::handle<Geom_OffsetCurve> offset=new Geom_OffsetCurve(basis,-0.4,gp_Dir(0,0,1));
    GeomAdaptor_Curve adaptor(offset);
    unsigned checks=0,passed=0;
    auto check=[&](const char* name,bool ok){++checks;if(ok)++passed;else std::cerr<<name<<" failed\n";};
    auto near=[](const gp_Vec& value,const gp_Vec& expected){
        return(value-expected).Magnitude()<=2e-9*std::max(1.0,expected.Magnitude());
    };
    for(const auto& query:queries){
        const gp_Vec expected(query.x,query.y,query.z);
        const auto native=offset->EvalD3(query.u);
        const auto lower=offset->EvalD2(query.u);
        check("native D3 matches independent derivative",near(native.D3,expected));
        check("native D3 bundle preserves D2",near(native.D2,lower.D2));
        const auto adapted=adaptor.EvalD3(query.u);
        check("adaptor D3 matches independent derivative",near(adapted.D3,expected));
        check("adaptor D3 bundle preserves D2",near(adapted.D2,lower.D2));
        for(double scale:{1.0,0.01}){
            auto source=basis->EvalD3(query.u);
            gp_Vec fourth=basis->EvalDN(query.u,4)*scale;
            source.Point=gp_Pnt(source.Point.X()*scale,source.Point.Y()*scale,source.Point.Z()*scale);
            source.D1*=scale;source.D2*=scale;source.D3*=scale;
            const double radius=source.D1.Crossed(gp_Vec(0,0,1)).Magnitude();
            const double tolerance=scale==1.0?0.0:std::pow(radius,6.5);
            check("small-radius fixture exercises alternate helper branch",scale==1.0||(std::pow(radius,7)<=tolerance&&std::pow(radius,6)>tolerance));
            const bool ok=Geom_OffsetCurveUtils::CalculateD3(source.Point,source.D1,source.D2,source.D3,fourth,gp_XYZ(0,0,1),-0.4*scale,false,tolerance);
            check("helper admits nonsingular derivative",ok);
            check("helper D3 matches independent derivative",ok&&near(source.D3,expected*scale));
            check("helper D3 bundle preserves D2",ok&&near(source.D2,lower.D2*scale));
        }
    }
    std::cout<<"{\"checks\":"<<checks<<",\"passed\":"<<passed<<",\"failed\":"<<(checks-passed)<<"}\n";
    return checks==passed?0:1;
}
