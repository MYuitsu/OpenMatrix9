// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreSupportSnapshot.h"
#include "NurbsBasisAdapter.h"
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_OffsetCurve.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Precision.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <gp_Lin.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Circ.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Elips.hxx>
#include <gp_Elips2d.hxx>
#include <gp_Hypr.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Parab.hxx>
#include <gp_Parab2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Cone.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonParseError>
#include <array>
#include <cmath>
#include <cstdint>
#include <stdexcept>
#include <type_traits>

namespace OpenMatrix9Gui {
namespace {
struct NativeAxis {std::size_t degree;const double* knots;std::size_t knot_count;const std::size_t* multiplicities;std::size_t multiplicity_count;std::uint32_t periodic;double domain[2];};
struct NativeCurve {const std::array<double,3>* poles;std::size_t pole_count;const double* weights;std::size_t weight_count;NativeAxis axis;};
struct NativeSurface {const std::array<double,3>* poles;std::size_t pole_count;const double* weights;std::size_t weight_count,u_count,v_count;NativeAxis axes[2];};
struct JsonSummary {std::uint32_t status,reserved;std::size_t needed;};
struct AnalyticInput {std::uint32_t kind,dimension;std::array<double,3> origin;std::array<std::array<double,3>,3> axes;double radii[2],extra;};
struct DerivedNode {std::uint32_t kind,dimension;std::array<double,3> direction,origin;double offset,bounds[4];std::uint32_t infinite_mask,reserved;};
struct NativeSupportTree {std::uint32_t leaf_kind,dimension;AnalyticInput analytic;NativeCurve curve;NativeSurface surface;const DerivedNode* nodes;std::size_t node_count;};
static_assert(std::is_standard_layout_v<DerivedNode> && sizeof(DerivedNode)==104 && alignof(DerivedNode)==8 && offsetof(DerivedNode,bounds)==64 && offsetof(DerivedNode,infinite_mask)==96);
static_assert(std::is_standard_layout_v<NativeSupportTree> && sizeof(NativeSupportTree)==424 && alignof(NativeSupportTree)==8 && offsetof(NativeSupportTree,curve)==136 && offsetof(NativeSupportTree,surface)==232 && offsetof(NativeSupportTree,nodes)==408);
static_assert(std::is_standard_layout_v<AnalyticInput> && sizeof(AnalyticInput)==128 && alignof(AnalyticInput)==8 && offsetof(AnalyticInput,axes)==32 && offsetof(AnalyticInput,extra)==120);
static_assert(sizeof(NativeAxis)==64 && alignof(NativeAxis)==8 && offsetof(NativeAxis,domain)==48);
static_assert(sizeof(NativeCurve)==96 && alignof(NativeCurve)==8 && offsetof(NativeCurve,axis)==32);
static_assert(sizeof(NativeSurface)==176 && alignof(NativeSurface)==8 && offsetof(NativeSurface,axes)==48);
static_assert(std::is_standard_layout_v<JsonSummary> && sizeof(JsonSummary)==16 && alignof(JsonSummary)==8 && offsetof(JsonSummary,needed)==8);
extern "C" JsonSummary om9_support_curve_json(NativeCurve,unsigned char*,std::size_t);
extern "C" JsonSummary om9_support_surface_json(NativeSurface,unsigned char*,std::size_t);
extern "C" JsonSummary om9_support_bezier_curve_json(NativeCurve,unsigned char*,std::size_t);
extern "C" JsonSummary om9_support_bezier_surface_json(NativeSurface,unsigned char*,std::size_t);
extern "C" JsonSummary om9_support_analytic_json(AnalyticInput,unsigned char*,std::size_t);
extern "C" JsonSummary om9_support_tree_json(NativeSupportTree,unsigned char*,std::size_t);
extern "C" std::uint32_t om9_support_tree_depth(std::size_t);
extern "C" std::uint32_t om9_support_counts(std::size_t,std::size_t,std::size_t,std::size_t,std::uint32_t);
extern "C" std::uint32_t om9_support_payload(std::size_t,std::size_t);
extern "C" std::uint32_t om9_topology_domain(double,double);
QJsonObject uninspected(const char* reason) {
    return {{"geometry_data",QJsonValue(QJsonValue::Null)},{"geometry_data_status",reason}};
}
NativeAxis axis(const ThreeDm::BasisContract::Axis& a) {
    return {a.degree,a.knots.data(),a.knots.size(),a.multiplicities.data(),a.multiplicities.size(),
            static_cast<std::uint32_t>(a.periodic),{a.domain[0],a.domain[1]}};
}
template<class Read> QJsonObject payload(Read read,std::size_t& used,const char* status="copied_validated_basis") {
    const auto query=read(nullptr,0);
    if(query.status==1000)return uninspected("serialized_companion_limit");
    if(query.status||query.reserved||!query.needed)throw std::runtime_error("Rust rejected owned support JSON query (status "+std::to_string(query.status)+")");
    if(om9_support_payload(used,query.needed))return uninspected("aggregate_companion_limit");
    QByteArray buffer;buffer.resize(static_cast<qsizetype>(query.needed));
    const auto result=read(reinterpret_cast<unsigned char*>(buffer.data()),static_cast<std::size_t>(buffer.size()));
    if(result.status||result.reserved||result.needed!=query.needed)throw std::runtime_error("Rust rejected owned support JSON output");
    QJsonParseError error;
    const auto document=QJsonDocument::fromJson(buffer,&error);
    if(error.error!=QJsonParseError::NoError||!document.isObject())throw std::runtime_error("Invalid owned support JSON from Rust");
    used+=result.needed; // Rust checked the sum before native allocation/copy.
    return {{"geometry_data",document.object()},{"geometry_data_status",status}};
}
std::array<double,3> xyz(const gp_Pnt& p){return {p.X(),p.Y(),p.Z()};}
std::array<double,3> xyz(const gp_Dir& p){return {p.X(),p.Y(),p.Z()};}
std::array<double,3> xyz(const gp_Pnt2d& p){return {p.X(),p.Y(),0};}
std::array<double,3> xyz(const gp_Dir2d& p){return {p.X(),p.Y(),0};}
template<class Position> void frame(AnalyticInput& i,const Position& p) {
    i.origin=xyz(p.Location());i.axes={xyz(p.XDirection()),xyz(p.YDirection()),xyz(p.Direction())};
}
void frame(AnalyticInput& i,const gp_Ax22d& p) {
    i.origin=xyz(p.Location());i.axes[0]=xyz(p.XDirection());i.axes[1]=xyz(p.YDirection());
    i.axes[2]={0,0,p.XDirection().Crossed(p.YDirection())};
}
template<class T> const auto& position(const T& v){return v.Position();}
const gp_Ax22d& position(const gp_Elips2d& v){return v.Axis();}
const gp_Ax22d& position(const gp_Hypr2d& v){return v.Axis();}
const gp_Ax22d& position(const gp_Parab2d& v){return v.Axis();}
template<class Adaptor> bool readCurve(Adaptor& a,AnalyticInput& i) {
    switch(a.GetType()) {
        case GeomAbs_Line:{const auto v=a.Line();i.kind=1;i.origin=xyz(v.Location());i.axes[0]=xyz(v.Direction());return true;}
        case GeomAbs_Circle:{const auto v=a.Circle();i.kind=2;frame(i,position(v));i.radii[0]=v.Radius();return true;}
        case GeomAbs_Ellipse:{const auto v=a.Ellipse();i.kind=3;frame(i,position(v));i.radii[0]=v.MajorRadius();i.radii[1]=v.MinorRadius();return true;}
        case GeomAbs_Hyperbola:{const auto v=a.Hyperbola();i.kind=4;frame(i,position(v));i.radii[0]=v.MajorRadius();i.radii[1]=v.MinorRadius();return true;}
        case GeomAbs_Parabola:{const auto v=a.Parabola();i.kind=5;frame(i,position(v));i.radii[0]=v.Focal();return true;}
        default:return false;
    }
}
QJsonObject analyticPayload(const AnalyticInput& i,std::size_t& used) {
    return payload([&](unsigned char* out,std::size_t size){return om9_support_analytic_json(i,out,size);},used,"copied_validated_analytic");
}
bool readSurface(const Handle(Geom_Surface)& surface,AnalyticInput& i) {
    GeomAdaptor_Surface a(surface);i.dimension=3;
    switch(a.GetType()) {
        case GeomAbs_Plane:{const auto v=a.Plane();i.kind=6;frame(i,v.Position());break;}
        case GeomAbs_Cylinder:{const auto v=a.Cylinder();i.kind=7;frame(i,v.Position());i.radii[0]=v.Radius();break;}
        case GeomAbs_Cone:{const auto v=a.Cone();i.kind=8;frame(i,v.Position());i.radii[0]=v.RefRadius();i.extra=v.SemiAngle();break;}
        case GeomAbs_Sphere:{const auto v=a.Sphere();i.kind=9;frame(i,v.Position());i.radii[0]=v.Radius();break;}
        case GeomAbs_Torus:{const auto v=a.Torus();i.kind=10;frame(i,v.Position());i.radii[0]=v.MajorRadius();i.radii[1]=v.MinorRadius();break;}
        default:return false;
    }
    return true;
}
QJsonObject analyticSurface(const Handle(Geom_Surface)& surface,std::size_t& used) {
    AnalyticInput i{};
    if(!readSurface(surface,i))return uninspected("uninspected_support_type");
    return analyticPayload(i,used);
}
QJsonObject curvePayload(const ThreeDm::BasisContract::CurveBasisData& b,std::size_t& used,bool bezier) {
    const NativeCurve input{b.poles.data(),b.poles.size(),b.weights.empty()?nullptr:b.weights.data(),b.weights.size(),axis(b.axis)};
    return payload([&](unsigned char* out,std::size_t size){return bezier?om9_support_bezier_curve_json(input,out,size):om9_support_curve_json(input,out,size);},
                   used,bezier?"copied_validated_bezier":"copied_validated_basis");
}
QJsonObject surfacePayload(const ThreeDm::BasisContract::SurfaceBasisData& b,std::size_t& used,bool bezier) {
    const NativeSurface input{b.poles.data(),b.poles.size(),b.weights.empty()?nullptr:b.weights.data(),b.weights.size(),b.uCount,b.vCount,{axis(b.axes[0]),axis(b.axes[1])}};
    return payload([&](unsigned char* out,std::size_t size){return bezier?om9_support_bezier_surface_json(input,out,size):om9_support_surface_json(input,out,size);},
                   used,bezier?"copied_validated_bezier":"copied_validated_basis");
}
template<class Curve> QJsonObject bezierCurve(const Curve& curve,std::size_t& used) {
    const int poles=curve->NbPoles();
    if(poles<=0)throw std::runtime_error("Invalid Bezier support counts");
    if(om9_support_counts(poles,0,2,0,0))return uninspected("basis_companion_limit");
    return curvePayload(ThreeDm::BasisContract::checkedBasis(curve),used,true);
}
void nativeDomain(DerivedNode& node,unsigned index,double value) {
    if(Precision::IsInfinite(value)) {
        if((index%2==0 && value>=0)||(index%2==1 && value<=0))throw std::runtime_error("Invalid native supporting infinity sign");
        node.infinite_mask|=1u<<index;node.bounds[index]=0;
    } else node.bounds[index]=value;
}
template<class Curve> bool curveLeaf(const Curve& curve,int knots,bool bezier,NativeSupportTree& input,ThreeDm::BasisContract::CurveBasisData& storage) {
    const int poles=curve->NbPoles();
    if(poles<=0||knots<=0)throw std::runtime_error("Invalid derived curve basis counts");
    if(om9_support_counts(poles,0,knots,0,0))return false;
    storage=ThreeDm::BasisContract::checkedBasis(curve);
    input.leaf_kind=bezier?4:2;
    input.curve={storage.poles.data(),storage.poles.size(),storage.weights.empty()?nullptr:storage.weights.data(),storage.weights.size(),axis(storage.axis)};
    return true;
}
template<class Surface> bool surfaceLeaf(const Surface& surface,int uk,int vk,bool bezier,NativeSupportTree& input,ThreeDm::BasisContract::SurfaceBasisData& storage) {
    const int u=surface->NbUPoles(),v=surface->NbVPoles();
    if(u<=0||v<=0||uk<=0||vk<=0)throw std::runtime_error("Invalid derived surface basis counts");
    if(om9_support_counts(u,v,uk,vk,1))return false;
    storage=ThreeDm::BasisContract::checkedBasis(surface);
    input.leaf_kind=bezier?5:3;
    input.surface={storage.poles.data(),storage.poles.size(),storage.weights.empty()?nullptr:storage.weights.data(),storage.weights.size(),storage.uCount,storage.vCount,{axis(storage.axes[0]),axis(storage.axes[1])}};
    return true;
}
// One basis per wrapper. Fixed native node storage is gated before extraction;
// Rust owns the portable chain/leaf. All handles and vector buffers remain live
// through both query/copy calls and are never retained by Rust.
QJsonObject derivedPayload(Handle(Geom_Curve) curve,Handle(Geom2d_Curve) uv,Handle(Geom_Surface) surface,std::size_t& used) {
    std::array<DerivedNode,16> nodes{};NativeSupportTree input{};
    input.dimension=uv.IsNull()?3:2;input.nodes=nodes.data();input.analytic.dimension=input.dimension;
    auto push=[&](std::uint32_t kind) -> DerivedNode& {
        if(om9_support_tree_depth(input.node_count+1))throw std::runtime_error("Excessively nested derived supporting geometry");
        auto& n=nodes[input.node_count++];n.kind=kind;n.dimension=input.dimension;
        if(!surface.IsNull()){double a,b,c,d;surface->Bounds(a,b,c,d);nativeDomain(n,0,a);nativeDomain(n,1,b);nativeDomain(n,2,c);nativeDomain(n,3,d);}
        else if(!uv.IsNull()){nativeDomain(n,0,uv->FirstParameter());nativeDomain(n,1,uv->LastParameter());}
        else {nativeDomain(n,0,curve->FirstParameter());nativeDomain(n,1,curve->LastParameter());}
        return n;
    };
    for(;;) {
        if(!surface.IsNull()) {
            const auto off=Handle(Geom_OffsetSurface)::DownCast(surface);
            if(!off.IsNull()){push(2).offset=off->Offset();surface=off->BasisSurface();continue;}
            const auto ex=Handle(Geom_SurfaceOfLinearExtrusion)::DownCast(surface);
            if(!ex.IsNull()){push(3).direction=xyz(ex->Direction());curve=ex->BasisCurve();surface.Nullify();continue;}
            const auto rev=Handle(Geom_SurfaceOfRevolution)::DownCast(surface);
            if(!rev.IsNull()){auto& n=push(4);const auto a=rev->Axis();n.direction=xyz(a.Direction());n.origin=xyz(a.Location());curve=rev->BasisCurve();surface.Nullify();continue;}
            const auto trim=Handle(Geom_RectangularTrimmedSurface)::DownCast(surface);
            if(!trim.IsNull()){push(6);surface=trim->BasisSurface();continue;}
        } else if(!uv.IsNull()) {
            const auto off=Handle(Geom2d_OffsetCurve)::DownCast(uv);
            if(!off.IsNull()){push(1).offset=off->Offset();uv=off->BasisCurve();continue;}
            const auto trim=Handle(Geom2d_TrimmedCurve)::DownCast(uv);
            if(!trim.IsNull()){push(5);uv=trim->BasisCurve();continue;}
        } else if(!curve.IsNull()) {
            const auto off=Handle(Geom_OffsetCurve)::DownCast(curve);
            if(!off.IsNull()){auto& n=push(1);n.offset=off->Offset();n.direction=xyz(off->Direction());curve=off->BasisCurve();continue;}
            const auto trim=Handle(Geom_TrimmedCurve)::DownCast(curve);
            if(!trim.IsNull()){push(5);curve=trim->BasisCurve();continue;}
        } else throw std::runtime_error("Missing derived geometry basis");
        break;
    }
    ThreeDm::BasisContract::CurveBasisData cb;ThreeDm::BasisContract::SurfaceBasisData sb;
    bool accepted=true;
    if(!surface.IsNull()) {
        const auto b=Handle(Geom_BezierSurface)::DownCast(surface);const auto s=Handle(Geom_BSplineSurface)::DownCast(surface);
        if(!b.IsNull())accepted=surfaceLeaf(b,2,2,true,input,sb);
        else if(!s.IsNull())accepted=surfaceLeaf(s,s->NbUKnots(),s->NbVKnots(),false,input,sb);
        else {input.leaf_kind=1;if(!readSurface(surface,input.analytic))return uninspected("uninspected_support_type");}
    } else if(!uv.IsNull()) {
        const auto b=Handle(Geom2d_BezierCurve)::DownCast(uv);const auto s=Handle(Geom2d_BSplineCurve)::DownCast(uv);
        if(!b.IsNull())accepted=curveLeaf(b,2,true,input,cb);
        else if(!s.IsNull())accepted=curveLeaf(s,s->NbKnots(),false,input,cb);
        else {input.leaf_kind=1;Geom2dAdaptor_Curve a(uv);if(!readCurve(a,input.analytic))return uninspected("uninspected_support_type");}
    } else {
        const auto b=Handle(Geom_BezierCurve)::DownCast(curve);const auto s=Handle(Geom_BSplineCurve)::DownCast(curve);
        if(!b.IsNull())accepted=curveLeaf(b,2,true,input,cb);
        else if(!s.IsNull())accepted=curveLeaf(s,s->NbKnots(),false,input,cb);
        else {input.leaf_kind=1;GeomAdaptor_Curve a(curve);if(!readCurve(a,input.analytic))return uninspected("uninspected_support_type");}
    }
    if(!accepted)return uninspected("basis_companion_limit");
    return payload([&](unsigned char* out,std::size_t size){return om9_support_tree_json(input,out,size);},used,"copied_validated_derived");
}
bool isDerived(const Handle(Geom_Surface)& surface) {
    return !Handle(Geom_OffsetSurface)::DownCast(surface).IsNull() || !Handle(Geom_SurfaceOfLinearExtrusion)::DownCast(surface).IsNull() || !Handle(Geom_SurfaceOfRevolution)::DownCast(surface).IsNull();
}
QJsonArray location(const TopLoc_Location& value) {
    QJsonArray rows;const auto& t=value.Transformation();
    for(int r=1;r<=3;++r){QJsonArray row;
        for(int c=1;c<=4;++c){const double x=t.Value(r,c);if(!std::isfinite(x))throw std::runtime_error("Nonfinite native support location");row.append(x);}rows.append(row);
    }
    rows.append(QJsonArray{0,0,0,1});return rows;
}
} // Native extraction helpers; public adapter functions follow.
QJsonObject curveSupportSnapshot(const Handle(Geom_Curve)& original,std::size_t& used) {
    if(original.IsNull())throw std::runtime_error("Missing3D curve support");
    auto curve=original;
    for(unsigned depth=0;depth<16;++depth){const auto trim=Handle(Geom_TrimmedCurve)::DownCast(curve);if(trim.IsNull())break;
        curve=trim->BasisCurve();if(depth==15)throw std::runtime_error("Excessively nested3D supporting curve");}
    if(!Handle(Geom_OffsetCurve)::DownCast(curve).IsNull())return derivedPayload(curve,{}, {},used);
    const auto bezier=Handle(Geom_BezierCurve)::DownCast(curve);
    if(!bezier.IsNull())return bezierCurve(bezier,used);
    const auto spline=Handle(Geom_BSplineCurve)::DownCast(curve);
    if(!spline.IsNull()){
        const int poles=spline->NbPoles(),knots=spline->NbKnots();
        if(poles<=0||knots<=0)throw std::runtime_error("Invalid3D curve support counts");
        if(om9_support_counts(poles,0,knots,0,0))return uninspected("basis_companion_limit");
        return copyCurveSupport(ThreeDm::BasisContract::checkedBasis(spline),used);
    }
    GeomAdaptor_Curve a(curve);AnalyticInput i{};i.dimension=3;
    return readCurve(a,i)?analyticPayload(i,used):uninspected("uninspected_support_type");
}
QJsonObject copyCurveSupport(const ThreeDm::BasisContract::CurveBasisData& data,std::size_t& used) {
    return curvePayload(data,used,false);
}
QJsonObject curveSupportSnapshot(const Handle(Geom2d_Curve)& original,std::size_t& used) {
    if(original.IsNull())throw std::runtime_error("Missing pcurve support");
    auto curve=original;
    for(unsigned depth=0;depth<16;++depth){
        const auto trim=Handle(Geom2d_TrimmedCurve)::DownCast(curve);
        if(trim.IsNull())break;
        curve=trim->BasisCurve();if(depth==15)throw std::runtime_error("Excessively nested pcurve supporting geometry");
    }
    const auto bezier=Handle(Geom2d_BezierCurve)::DownCast(curve);
    if(!Handle(Geom2d_OffsetCurve)::DownCast(curve).IsNull())return derivedPayload({},curve,{},used);
    if(!bezier.IsNull())return bezierCurve(bezier,used);
    const auto spline=Handle(Geom2d_BSplineCurve)::DownCast(curve);
    if(spline.IsNull()) {
        Geom2dAdaptor_Curve a(curve);AnalyticInput i{};i.dimension=2;
        return readCurve(a,i)?analyticPayload(i,used):uninspected("uninspected_support_type");
    }
    const int poles=spline->NbPoles(),knots=spline->NbKnots();
    if(poles<=0||knots<=0)throw std::runtime_error("Invalid pcurve support counts");
    if(om9_support_counts(poles,0,knots,0,0))return uninspected("basis_companion_limit");
    const auto data=ThreeDm::BasisContract::checkedBasis(spline);
    return copyCurveSupport(data,used);
}
QJsonObject faceSupportSnapshot(const TopoDS_Face& face,std::size_t& used) {
    TopLoc_Location nativeLocation;
    auto surface=BRep_Tool::Surface(face,nativeLocation);
    if(surface.IsNull())throw std::runtime_error("Missing face supporting geometry");
    double u0,u1,v0,v1;BRepTools::UVBounds(face,u0,u1,v0,v1);
    if(om9_topology_domain(u0,u1)||om9_topology_domain(v0,v1))throw std::runtime_error("Invalid face UV restriction domain");
    QJsonObject result=uninspected("uninspected_support_type");
    result["native_class"]=surface->DynamicType()->Name();
    result["uv_domain"]=QJsonArray{QJsonArray{u0,u1},QJsonArray{v0,v1}};
    result["native_location"]=location(nativeLocation);
    result["geometry_coordinates"]="native_local";
    for(unsigned depth=0;depth<16;++depth){
        const auto trim=Handle(Geom_RectangularTrimmedSurface)::DownCast(surface);
        if(trim.IsNull())break;
        surface=trim->BasisSurface();if(depth==15)throw std::runtime_error("Excessively nested face supporting geometry");
    }
    const auto bezier=Handle(Geom_BezierSurface)::DownCast(surface);
    if(isDerived(surface)) {
        const auto data=derivedPayload({}, {},surface,used);
        for(auto it=data.begin();it!=data.end();++it)result[it.key()]=it.value();
        return result;
    }
    if(!bezier.IsNull()) {
        const int u=bezier->NbUPoles(),v=bezier->NbVPoles();
        if(u<=0||v<=0)throw std::runtime_error("Invalid Bezier surface counts");
        const auto data=om9_support_counts(u,v,2,2,1)?uninspected("basis_companion_limit"):
            surfacePayload(ThreeDm::BasisContract::checkedBasis(bezier),used,true);
        for(auto it=data.begin();it!=data.end();++it)result[it.key()]=it.value();
        return result;
    }
    const auto spline=Handle(Geom_BSplineSurface)::DownCast(surface);
    if(spline.IsNull()) {
        const auto data=analyticSurface(surface,used);
        for(auto it=data.begin();it!=data.end();++it)result[it.key()]=it.value();
        return result;
    }
    const int u=spline->NbUPoles(),v=spline->NbVPoles(),uk=spline->NbUKnots(),vk=spline->NbVKnots();
    if(u<=0||v<=0||uk<=0||vk<=0)throw std::runtime_error("Invalid face support counts");
    QJsonObject data;
    if(om9_support_counts(u,v,uk,vk,1))data=uninspected("basis_companion_limit");
    else {
        const auto b=ThreeDm::BasisContract::checkedBasis(spline);
        data=surfacePayload(b,used,false);
    }
    for(auto it=data.begin();it!=data.end();++it)result[it.key()]=it.value();
    return result;
}
}
