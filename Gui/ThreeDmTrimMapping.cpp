#include "ThreeDmNativeReferences.h"
#include <Adaptor3d_CurveOnSurface.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Extrema_ExtPC.hxx>
#include <Standard_Failure.hxx>
#include <cmath>
#include <limits>
namespace OpenMatrix9Gui::ThreeDm {
NativeTrimParameter nativeTrimParameterAt(const ON_BrepTrim& trim,const ON_BrepEdge& edge,const ON_Surface& surface,ON_Interval domain,double edgeParameter,double tolerance)try{
    if(trim.Edge()!=&edge||!domain.IsIncreasing()||!std::isfinite(domain[0])||!std::isfinite(domain[1])||!trim.Domain().Includes(domain)||!std::isfinite(edgeParameter)||!edge.Domain().Includes(edgeParameter)||!ON_IsValid(tolerance)||tolerance<0)throw ExchangeError("Invalid native trim mapping input");
    const auto point=edge.PointAt(edgeParameter);if(!point.IsValid())throw ExchangeError("Invalid native edge mapping evaluation");
    const double magnitude=std::max({1.,std::abs(point.x),std::abs(point.y),std::abs(point.z)});
    tolerance=std::max(tolerance,64*std::numeric_limits<double>::epsilon()*magnitude);
    if(ON_IsValid(edge.m_tolerance)&&edge.m_tolerance>=0)tolerance=std::max(tolerance,edge.m_tolerance);
    // A full closed trim has physically identical start/end points. Topology,
    // rather than closest-point distance, selects the endpoint parameter side.
    if(domain==trim.Domain())for(int endpoint=0;endpoint<2;++endpoint)if(edgeParameter==edge.Domain()[endpoint]){
        const double parameter=domain[trim.m_bRev3d?1-endpoint:endpoint];const auto uv=trim.PointAt(parameter);
        if(!uv.IsValid())throw ExchangeError("Invalid native trim topological endpoint");const auto physical=surface.PointAt(uv.x,uv.y);
        if(!physical.IsValid())throw ExchangeError("Invalid native trim surface endpoint");const double deviation=point.DistanceTo(physical);
        if(!std::isfinite(deviation)||deviation>tolerance)throw ExchangeError("Native trim topological endpoint does not match edge within source tolerance");
        return {parameter,deviation};
    }
    ON_NurbsCurve check;const int form=trim.GetNurbForm(check);if(form<1||form>2)throw ExchangeError("Native trim needs an explicit parameter adapter");
    ON_NurbsSurface checkSurface;if(!ON_RevSurface::Cast(&surface)&&surface.GetNurbForm(checkSurface)!=1)throw ExchangeError("Native trim surface needs an explicit parameter adapter");
    double first=domain[0],last=domain[1];
    if(form==2&&(!trim.GetNurbFormParameterFromCurveParameter(domain[0],&first)||!trim.GetNurbFormParameterFromCurveParameter(domain[1],&last)))throw ExchangeError("Native trim domain cannot map to NURBS parameters");
    Handle(Geom2dAdaptor_Curve) uv=new Geom2dAdaptor_Curve(importedSurfaceTrim(surface,trim),first,last);
    Handle(GeomAdaptor_Surface) modelSurface=new GeomAdaptor_Surface(importSurface(surface));
    Adaptor3d_CurveOnSurface physical(uv,modelSurface);
    const double parameterTolerance=64*std::numeric_limits<double>::epsilon()*std::max({1.,std::abs(first),std::abs(last)});
    Extrema_ExtPC search(gp_Pnt(point.x,point.y,point.z),physical,first,last,parameterTolerance);
    if(!search.IsDone())throw ExchangeError("Native trim interior projection did not converge");
    std::vector<NativeTrimParameter> candidates;
    auto candidate=[&](double nurbsParameter){
        double parameter=nurbsParameter;
        if(form==2&&!trim.GetCurveParameterFromNurbFormParameter(nurbsParameter,&parameter))return;
        if(!domain.Includes(parameter))return;
        const auto nativeUV=trim.PointAt(parameter);if(!nativeUV.IsValid())return;
        const auto nativePoint=surface.PointAt(nativeUV.x,nativeUV.y);if(!nativePoint.IsValid())return;
        const double deviation=point.DistanceTo(nativePoint);if(!std::isfinite(deviation)||deviation>tolerance)return;
        for(auto& old:candidates)if(std::abs(old.parameter-parameter)<=std::max(parameterTolerance,domain.Length()*1e-10)){if(deviation<old.deviation)old={parameter,deviation};return;}
        candidates.push_back({parameter,deviation});
    };
    for(int i=1;i<=search.NbExt();++i)candidate(search.Point(i).Parameter());candidate(first);candidate(last);
    if(candidates.empty())throw ExchangeError("Native trim interior does not correspond to edge within source tolerance");
    if(candidates.size()!=1)throw ExchangeError("Ambiguous native trim interior parameter correspondence");
    return candidates.front();
}catch(const Standard_Failure& error){
    throw ExchangeError(std::string("OCCT native trim projection failed: ")+(error.GetMessageString()?error.GetMessageString():"unknown kernel error"));
}
}
