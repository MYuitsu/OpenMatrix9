// SPDX-License-Identifier: LGPL-2.1-or-later
#include "NativeInputTree.h"
#include "NurbsBasisAdapter.h"
#include "opennurbs_polyedgecurve.h"
#include <Geom_TrimmedCurve.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom_OffsetCurve.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom_Conic.hxx>
#include <Geom2d_Conic.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_ElementarySurface.hxx>
#include <BRep_TEdge.hxx>
#include <BRep_CurveRepresentation.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <cstdint>
#include <cmath>
#include <limits>
#include <array>
#include <algorithm>
#include <type_traits>
namespace OpenMatrix9Gui::ThreeDm::InputTree {
namespace {
// Exact Rust value layout. One private initialized state is exclusively borrowed
// during each synchronous call. No native owner or allocation crosses the ABI.
struct State {std::uint32_t magic,failure;std::size_t depth,nodes,bytes;std::uint64_t path[64];};
static_assert(std::is_standard_layout_v<State> && sizeof(State)==544 && alignof(State)==8);
static_assert(sizeof(std::size_t)==8 && offsetof(State,path)==32 && offsetof(State,bytes)==24);
extern "C" State om9_input_tree_init();
extern "C" std::uint32_t om9_input_tree_enter(State*,std::uint64_t);
extern "C" std::uint32_t om9_input_tree_leave(State*,std::uint64_t);
extern "C" std::uint32_t om9_input_tree_children(State*,std::size_t);
extern "C" std::uint32_t om9_input_tree_account(State*,std::size_t);
extern "C" std::uint32_t om9_input_tree_finish(State*);
void accepted(std::uint32_t status){if(status)throw ExchangeError("Native input tree rejected by Rust contract (status "+std::to_string(status)+")");}
void envelope(bool ok){if(!ok)throw ExchangeError("Invalid native input-tree storage envelope");}
class Gate {
    State state=om9_input_tree_init();
    template<class F> void node(const void* owner,F action){
        const auto id=static_cast<std::uint64_t>(reinterpret_cast<std::uintptr_t>(owner));
        accepted(om9_input_tree_enter(&state,id));action();accepted(om9_input_tree_leave(&state,id));
        // Native exceptions discard this local failed gate. No cleanup callback,
        // owner or exception crosses Rust; rejected state grants no further work.
    }
    template<class A> int array(const A& values,bool owners=false){
        const int count=values.Count();envelope(count>=0&&values.Capacity()>=count);
        if(owners)accepted(om9_input_tree_children(&state,static_cast<std::size_t>(count)));
        using Value=std::remove_cv_t<std::remove_pointer_t<decltype(values.Array())>>;
        const auto n=static_cast<std::size_t>(count);envelope(n<=static_cast<std::size_t>(std::numeric_limits<std::ptrdiff_t>::max())/sizeof(Value));
        account(n*sizeof(Value));
        if(count){const auto address=reinterpret_cast<std::uintptr_t>(values.Array());envelope(address&&address%alignof(Value)==0&&address<=std::numeric_limits<std::uintptr_t>::max()-n*sizeof(Value));}
        return count;
    }
    template<class A> int indexArray(const A& values,int limit){
        const int count=array(values);
        for(int i=0;i<count;++i)if(values[i]<0||values[i]>=limit)
            throw ExchangeError("Invalid native BRep topology reference");
        return count;
    }
    template<class T> void rawArray(const T* values,std::size_t count){
        envelope(count<=static_cast<std::size_t>(std::numeric_limits<std::ptrdiff_t>::max())/sizeof(T));
        const auto bytes=count*sizeof(T);
        if(count){const auto address=reinterpret_cast<std::uintptr_t>(values);envelope(address&&address%alignof(T)==0&&address<=std::numeric_limits<std::uintptr_t>::max()-bytes);}
        account(bytes);
    }
    static void optionalCount(int count,int expected,const char* message){
        if(count!=0&&count!=expected)throw ExchangeError(message);
    }
    template<class Basis> void basis(const Basis& value){account(BasisContract::logicalBytes(value));(void)BasisContract::checkedBasis(value);}
public:
    void account(std::size_t bytes){accepted(om9_input_tree_account(&state,bytes));}
    Summary finish(){accepted(om9_input_tree_finish(&state));return {state.nodes,state.bytes};}
    void curve(const ON_Curve* c){node(c,[&]{
        const auto id=c->ClassId();
        if(id!=&ON_CLASS_RTTI(ON_NurbsCurve)&&id!=&ON_CLASS_RTTI(ON_CurveProxy)&&
           id!=&ON_CLASS_RTTI(ON_PolyCurve)&&id!=&ON_CLASS_RTTI(ON_CurveOnSurface)&&
           id!=&ON_CLASS_RTTI(ON_PolylineCurve)&&id!=&ON_CLASS_RTTI(ON_LineCurve)&&
           id!=&ON_CLASS_RTTI(ON_ArcCurve)&&id!=&ON_CLASS_RTTI(ON_BrepEdge)&&
           id!=&ON_CLASS_RTTI(ON_BrepTrim)&&id!=&ON_CLASS_RTTI(ON_PolyEdgeCurve)&&
           id!=&ON_CLASS_RTTI(ON_PolyEdgeSegment))
            throw GeometryRepresentationUnavailable("Unverified native curve owner cannot enter NURBS conversion",0);
        if(auto n=ON_NurbsCurve::Cast(c))basis(*n);
        else if(auto p=ON_CurveProxy::Cast(c))curve(p->ProxyCurve());
        else if(auto p=ON_PolyCurve::Cast(c)){
            const auto& children=p->SegmentCurves();const int count=array(children,true);const int parameters=array(p->SegmentParameters());
            envelope(count>0&&static_cast<std::size_t>(parameters)==static_cast<std::size_t>(count)+1);
            for(int i=0;i<count;++i)curve(children[i]);
        }else if(auto p=ON_CurveOnSurface::Cast(c)){
            accepted(om9_input_tree_children(&state,p->m_c3?3:2));curve(p->m_c2);if(p->m_c3)curve(p->m_c3);surface(p->m_s);
        }else if(auto p=ON_PolylineCurve::Cast(c))basis(*p);
        else if(!ON_LineCurve::Cast(c)&&!ON_ArcCurve::Cast(c))throw GeometryRepresentationUnavailable("Unverified native curve owner cannot enter NURBS conversion",0);
    });}
    void surface(const ON_Surface* s){node(s,[&]{
        const auto id=s->ClassId();
        if(id!=&ON_CLASS_RTTI(ON_NurbsSurface)&&id!=&ON_CLASS_RTTI(ON_SurfaceProxy)&&
           id!=&ON_CLASS_RTTI(ON_RevSurface)&&id!=&ON_CLASS_RTTI(ON_SumSurface)&&
           id!=&ON_CLASS_RTTI(ON_Extrusion)&&id!=&ON_CLASS_RTTI(ON_PlaneSurface)&&
           id!=&ON_CLASS_RTTI(ON_ClippingPlaneSurface)&&id!=&ON_CLASS_RTTI(ON_BrepFace))
            throw GeometryRepresentationUnavailable("Unverified native surface owner cannot enter NURBS conversion",0);
        if(auto n=ON_NurbsSurface::Cast(s))basis(*n);
        else if(auto p=ON_SurfaceProxy::Cast(s))surface(p->ProxySurface());
        else if(auto p=ON_RevSurface::Cast(s))curve(p->m_curve);
        else if(auto p=ON_SumSurface::Cast(s)){accepted(om9_input_tree_children(&state,2));curve(p->m_curve[0]);curve(p->m_curve[1]);}
        else if(auto p=ON_Extrusion::Cast(s)){
            envelope(p->m_profile_count>0);accepted(om9_input_tree_children(&state,static_cast<std::size_t>(p->m_profile_count)));curve(p->m_profile);
        }else if(!ON_PlaneSurface::Cast(s))throw GeometryRepresentationUnavailable("Unverified native surface owner cannot enter NURBS conversion",0);
    });}
    void brep(const ON_Brep& b){node(&b,[&]{
        const int ns=array(b.m_S,true),n3=array(b.m_C3,true),n2=array(b.m_C2,true);
        const int nv=array(b.m_V,true),ne=array(b.m_E,true),nt=array(b.m_T,true),nl=array(b.m_L,true),nf=array(b.m_F,true);
        // Entire forest, including unused owned geometry, precedes the first
        // source copy. Topology/cache validity still runs on that copy.
        for(int i=0;i<ns;++i)if(b.m_S[i])surface(b.m_S[i]);
        for(int i=0;i<n3;++i)if(b.m_C3[i])curve(b.m_C3[i]);
        for(int i=0;i<n2;++i)if(b.m_C2[i])curve(b.m_C2[i]);
        for(int i=0;i<nv;++i)node(&b.m_V[i],[&]{array(b.m_V[i].m_ei);});
        for(int i=0;i<ne;++i){
            // Pinned OpenNURBS 5 SolidOrientation/IsManifold indexes the
            // second m_ti entry without a range guard. Validate references,
            // not only the ON_SimpleArray storage envelope, before any copy.
            indexArray(b.m_E[i].m_ti,nt);curve(&b.m_E[i]);
        }
        for(int i=0;i<nt;++i)curve(&b.m_T[i]);
        for(int i=0;i<nl;++i)node(&b.m_L[i],[&]{array(b.m_L[i].m_ti);});
        for(int i=0;i<nf;++i){array(b.m_F[i].m_li);surface(&b.m_F[i]);}
    });}
    void cage(const ON_NurbsCage& c){node(&c,[&]{
        if(c.m_dim!=3||(c.m_is_rat!=0&&c.m_is_rat!=1))throw ExchangeError("Invalid native NURBS cage");
        constexpr std::size_t maxControlPoints=1000000;
        const std::size_t cvDimension=static_cast<std::size_t>(c.m_dim+(c.m_is_rat?1:0));
        std::array<std::size_t,3> counts{},orders{},knots{},strides{};
        std::size_t controlPoints=1,totalKnots=0;
        auto storage=[](const double* pointer,int capacity,std::size_t count){
            if(!pointer||capacity<0||(capacity>0&&count>static_cast<std::size_t>(capacity))||
               count>static_cast<std::size_t>(std::numeric_limits<std::ptrdiff_t>::max())/sizeof(double))
                throw ExchangeError("Invalid native NURBS cage");
            const auto address=reinterpret_cast<std::uintptr_t>(pointer);
            if(address%alignof(double)!=0||address>std::numeric_limits<std::uintptr_t>::max()-count*sizeof(double))
                throw ExchangeError("Invalid native NURBS cage");
        };
        for(int d=0;d<3;++d){
            if(c.m_order[d]<2||c.m_order[d]>26||c.m_cv_count[d]<c.m_order[d]||c.m_cv_count[d]>static_cast<int>(maxControlPoints))
                throw ExchangeError("Invalid native NURBS cage");
            orders[d]=static_cast<std::size_t>(c.m_order[d]);counts[d]=static_cast<std::size_t>(c.m_cv_count[d]);
            if(controlPoints>maxControlPoints/counts[d])throw ExchangeError("Cage control-point limit exceeded");
            controlPoints*=counts[d];
            knots[d]=counts[d]+orders[d]-2;
            if(knots[d]>maxControlPoints||totalKnots>std::numeric_limits<std::size_t>::max()-knots[d])
                throw ExchangeError("Cage knot limit exceeded");
            totalKnots+=knots[d];storage(c.m_knot[d],c.m_knot_capacity[d],knots[d]);
            if(!c.m_knot[d]||!std::isfinite(c.m_knot[d][orders[d]-2])||
               !std::isfinite(c.m_knot[d][counts[d]-1])||c.m_knot[d][orders[d]-2]>=c.m_knot[d][counts[d]-1]||
               !std::isfinite(c.m_knot[d][counts[d]-1]-c.m_knot[d][orders[d]-2]))
                throw ExchangeError("Invalid native NURBS cage");
            for(std::size_t k=0;k<knots[d];++k)
                if(!std::isfinite(c.m_knot[d][k])||
                   (k&&(!std::isfinite(c.m_knot[d][k]-c.m_knot[d][k-1])||c.m_knot[d][k]<c.m_knot[d][k-1])))
                    throw ExchangeError("Invalid native NURBS cage");
            strides[d]=c.m_cv_stride[d]>0?static_cast<std::size_t>(c.m_cv_stride[d]):0;
            if(!strides[d])throw ExchangeError("Invalid native NURBS cage");
        }
        auto axes=std::array<int,3>{0,1,2};
        std::sort(axes.begin(),axes.end(),[&](int a,int b){return strides[a]<strides[b];});
        if(strides[axes[0]]<cvDimension||counts[axes[0]]>std::numeric_limits<std::size_t>::max()/strides[axes[0]]||
           strides[axes[1]]<strides[axes[0]]*counts[axes[0]]||counts[axes[1]]>std::numeric_limits<std::size_t>::max()/strides[axes[1]]||
           strides[axes[2]]<strides[axes[1]]*counts[axes[1]])
            throw ExchangeError("Invalid native NURBS cage");
        std::size_t lastOffset=0;
        for(int d=0;d<3;++d){
            if(counts[d]-1>std::numeric_limits<std::size_t>::max()/strides[d])
                throw ExchangeError("Invalid native NURBS cage");
            const auto part=(counts[d]-1)*strides[d];
            if(lastOffset>std::numeric_limits<std::size_t>::max()-part)throw ExchangeError("Invalid native NURBS cage");
            lastOffset+=part;
        }
        if(lastOffset>static_cast<std::size_t>(std::numeric_limits<int>::max())-cvDimension)
            throw ExchangeError("Invalid native NURBS cage");
        const auto cvStorage=lastOffset+cvDimension;storage(c.m_cv,c.m_cv_capacity,cvStorage);
        if(totalKnots>std::numeric_limits<std::size_t>::max()/sizeof(double)||
           controlPoints>std::numeric_limits<std::size_t>::max()/cvDimension/sizeof(double))
            throw ExchangeError("Invalid native NURBS cage");
        account(totalKnots*sizeof(double));account(controlPoints*cvDimension*sizeof(double));
        for(std::size_t i=0;i<counts[0];++i)for(std::size_t j=0;j<counts[1];++j)for(std::size_t k=0;k<counts[2];++k){
            const auto offset=i*strides[0]+j*strides[1]+k*strides[2];const auto* cv=c.m_cv+offset;
            for(std::size_t d=0;d<3;++d)if(!std::isfinite(cv[d])||std::abs(cv[d])>1e150)
                throw ExchangeError("Invalid native NURBS cage");
            if(c.m_is_rat){const double weight=cv[3];if(!std::isfinite(weight)||weight<=0||weight>1e150)
                    throw ExchangeError("Invalid native NURBS cage");
                for(int d=0;d<3;++d)if(!std::isfinite(cv[d]/weight)||std::abs(cv[d]/weight)>1e150)
                    throw ExchangeError("Invalid native NURBS cage");}
        }
    });}
    void mesh(const ON_Mesh& m){node(&m,[&]{
        if(m.ClassId()!=&ON_CLASS_RTTI(ON_Mesh))throw GeometryRepresentationUnavailable("Unverified native mesh owner cannot enter conversion",0);
        const int vertices=array(m.m_V),doubleVertices=array(m.m_dV),faces=array(m.m_F);
        envelope(vertices>0&&faces>0&&(doubleVertices==0||doubleVertices==vertices));
        const int normals=array(m.m_N),faceNormals=array(m.m_FN),legacyTexture=array(m.m_T),surfaceParameters=array(m.m_S);
        const int curvature=array(m.m_K),colors=array(m.m_C),hidden=array(m.m_H),ngonMap=array(m.m_NgonMap);
        const int ngons=array(m.m_Ngon,true),textureSets=array(m.m_TC,true);
        optionalCount(normals,vertices,"Invalid native mesh vertex-normal count");
        optionalCount(faceNormals,faces,"Invalid native mesh face-normal count");
        optionalCount(legacyTexture,vertices,"Invalid native mesh texture-coordinate count");
        optionalCount(surfaceParameters,vertices,"Invalid native mesh surface-parameter count");
        optionalCount(curvature,vertices,"Invalid native mesh curvature count");
        optionalCount(colors,vertices,"Invalid native mesh color count");
        optionalCount(ngonMap,faces,"Invalid native mesh N-gon map count");
        if(m.m_hidden_count<0||m.m_hidden_count>vertices)throw ExchangeError("Invalid native mesh hidden-vertex count");
        for(int i=0;i<vertices;++i){
            if(!m.m_V[i].IsValid()||(doubleVertices&&!m.m_dV[i].IsValid()))throw ExchangeError("Invalid native mesh vertex");
            if(normals&&!m.m_N[i].IsValid())throw ExchangeError("Invalid native mesh vertex normal");
            if(legacyTexture&&!m.m_T[i].IsValid())throw ExchangeError("Invalid native mesh texture coordinate");
            if(surfaceParameters&&!m.m_S[i].IsValid())throw ExchangeError("Invalid native mesh surface parameter");
            if(curvature){const auto& value=m.m_K[i];const bool nanSentinel=std::isnan(value.k1)&&std::isnan(value.k2);if(!nanSentinel&&!value.IsSet())throw ExchangeError("Invalid native mesh curvature");}
        }
        for(int i=0;i<faces;++i){
            const auto& face=m.m_F[i];
            for(int j=0;j<4;++j)if(face.vi[j]<0||face.vi[j]>=vertices)throw ExchangeError("Invalid native mesh face index");
            if(faceNormals&&!m.m_FN[i].IsValid())throw ExchangeError("Invalid native mesh face normal");
            if(ngonMap&&m.m_NgonMap[i]!=ON_UNSET_UINT_INDEX&&m.m_NgonMap[i]>=static_cast<unsigned int>(ngons))throw ExchangeError("Invalid native mesh N-gon index");
        }
        for(int i=0;i<textureSets;++i){const auto& coordinates=m.m_TC[i];node(&coordinates,[&]{
            if(coordinates.m_dim<1||coordinates.m_dim>3)throw ExchangeError("Invalid native mesh texture dimension");
            const int count=array(coordinates.m_T);envelope(count==vertices);
            for(int j=0;j<count;++j)if(!coordinates.m_T[j].IsValid())throw ExchangeError("Invalid native mesh texture coordinate");
        });}
        for(int i=0;i<ngons;++i){const auto* ngon=m.m_Ngon[i];if(!ngon)throw ExchangeError("Invalid native mesh N-gon owner");node(ngon,[&]{
            const auto vertexIndices=static_cast<std::size_t>(ngon->m_Vcount),faceIndices=static_cast<std::size_t>(ngon->m_Fcount);
            envelope(vertexIndices>=3&&faceIndices>0&&vertexIndices<=static_cast<std::size_t>(vertices)&&faceIndices<=static_cast<std::size_t>(faces));
            const auto capacity=static_cast<std::size_t>(ngon->Capacity());
            if(capacity==0||vertexIndices>capacity||faceIndices>capacity-vertexIndices)throw ExchangeError("Invalid native mesh N-gon storage capacity");
            rawArray(ngon->m_vi,vertexIndices);rawArray(ngon->m_fi,faceIndices);
            for(std::size_t j=0;j<vertexIndices;++j)if(ngon->m_vi[j]>=static_cast<unsigned int>(vertices))throw ExchangeError("Invalid native mesh N-gon vertex index");
            for(std::size_t j=0;j<faceIndices;++j)if(ngon->m_fi[j]>=static_cast<unsigned int>(faces))throw ExchangeError("Invalid native mesh N-gon face index");
        });}
    });}
    void pointCloud(const ON_PointCloud& cloud,bool allowEmpty=false){node(&cloud,[&]{
        if(cloud.ClassId()!=&ON_CLASS_RTTI(ON_PointCloud))throw GeometryRepresentationUnavailable("Unverified native PointCloud owner cannot enter conversion",0);
        const int points=array(cloud.m_P),normals=array(cloud.m_N),colors=array(cloud.m_C),values=array(cloud.m_V),hidden=array(cloud.m_H);
        envelope(allowEmpty?points>=0:points>0);optionalCount(normals,points,"Native PointCloud optional array count differs from points");
        optionalCount(colors,points,"Native PointCloud optional array count differs from points");
        optionalCount(values,points,"Native PointCloud optional array count differs from points");
        if(cloud.m_hidden_count>static_cast<unsigned int>(points))throw ExchangeError("Invalid native PointCloud hidden-point count");
        for(int i=0;i<points;++i)if(!cloud.m_P[i].IsValid())throw ExchangeError("Invalid native PointCloud point");
        for(int i=0;i<normals;++i)if(!cloud.m_N[i].IsValid())throw ExchangeError("Invalid native PointCloud normal");
        for(int i=0;i<values;++i)if(!std::isfinite(cloud.m_V[i]))throw ExchangeError("Invalid native PointCloud intensity");
        if(cloud.HasPlane()&&!cloud.m_plane.IsValid())throw ExchangeError("Invalid active native PointCloud plane");
    });}
    void curve(const Handle(Geom_Curve)& c){node(c.get(),[&]{
        if(auto n=Handle(Geom_BSplineCurve)::DownCast(c);!n.IsNull())basis(n);
        else if(auto n=Handle(Geom_BezierCurve)::DownCast(c);!n.IsNull())basis(n);
        else if(auto p=Handle(Geom_TrimmedCurve)::DownCast(c);!p.IsNull())curve(p->BasisCurve());
        else if(auto p=Handle(Geom_OffsetCurve)::DownCast(c);!p.IsNull())curve(p->BasisCurve());
        else if(!c->IsKind(STANDARD_TYPE(Geom_Conic))&&!c->IsKind(STANDARD_TYPE(Geom_Line)))throw GeometryRepresentationUnavailable("Unverified OCCT curve owner cannot enter NURBS conversion",0);
    });}
    void curve(const Handle(Geom2d_Curve)& c){node(c.get(),[&]{
        if(auto n=Handle(Geom2d_BSplineCurve)::DownCast(c);!n.IsNull())basis(n);
        else if(auto n=Handle(Geom2d_BezierCurve)::DownCast(c);!n.IsNull())basis(n);
        else if(auto p=Handle(Geom2d_TrimmedCurve)::DownCast(c);!p.IsNull())curve(p->BasisCurve());
        else if(auto p=Handle(Geom2d_OffsetCurve)::DownCast(c);!p.IsNull())curve(p->BasisCurve());
        else if(!c->IsKind(STANDARD_TYPE(Geom2d_Conic))&&!c->IsKind(STANDARD_TYPE(Geom2d_Line)))throw GeometryRepresentationUnavailable("Unverified OCCT 2D curve owner cannot enter NURBS conversion",0);
    });}
    void surface(const Handle(Geom_Surface)& s){node(s.get(),[&]{
        if(auto n=Handle(Geom_BSplineSurface)::DownCast(s);!n.IsNull())basis(n);
        else if(auto n=Handle(Geom_BezierSurface)::DownCast(s);!n.IsNull())basis(n);
        else if(auto p=Handle(Geom_RectangularTrimmedSurface)::DownCast(s);!p.IsNull())surface(p->BasisSurface());
        else if(auto p=Handle(Geom_OffsetSurface)::DownCast(s);!p.IsNull())surface(p->BasisSurface());
        else if(auto p=Handle(Geom_SurfaceOfRevolution)::DownCast(s);!p.IsNull())curve(p->BasisCurve());
        else if(auto p=Handle(Geom_SurfaceOfLinearExtrusion)::DownCast(s);!p.IsNull())curve(p->BasisCurve());
        else if(!s->IsKind(STANDARD_TYPE(Geom_ElementarySurface)))throw GeometryRepresentationUnavailable("Unverified OCCT surface owner cannot enter NURBS conversion",0);
    });}
    void shape(const TopoDS_Shape& s){node(s.IsNull()?nullptr:s.TShape().get(),[&]{
        if(s.ShapeType()==TopAbs_FACE){TopLoc_Location loc;surface(BRep_Tool::Surface(TopoDS::Face(s),loc));}
        if(s.ShapeType()==TopAbs_EDGE){
            auto edge=Handle(BRep_TEdge)::DownCast(s.TShape());envelope(!edge.IsNull());
            for(const auto& rep:edge->Curves())node(rep.get(),[&]{
                if(rep->IsCurve3D()){if(!rep->Curve3D().IsNull())curve(rep->Curve3D());}
                if(rep->IsCurveOnSurface()){surface(rep->Surface());curve(rep->PCurve());if(rep->IsCurveOnClosedSurface())curve(rep->PCurve2());}
                else if(rep->IsRegularity()){surface(rep->Surface());surface(rep->Surface2());}
                else if(rep->IsPolygonOnSurface())surface(rep->Surface());
            });
        }
        for(TopoDS_Iterator it(s);it.More();it.Next()){accepted(om9_input_tree_children(&state,1));shape(it.Value());}
    });}
};
}
Summary validate(const ON_Curve& c){Gate gate;gate.curve(&c);return gate.finish();}
Summary validate(const ON_Surface& s){Gate gate;gate.surface(&s);return gate.finish();}
Summary validate(const ON_Brep& b){Gate gate;gate.brep(b);return gate.finish();}
Summary validate(const ON_NurbsCage& c){Gate gate;gate.cage(c);return gate.finish();}
Summary validate(const ON_Mesh& m){Gate gate;gate.mesh(m);return gate.finish();}
Summary validate(const ON_PointCloud& cloud,bool allowEmpty){Gate gate;gate.pointCloud(cloud,allowEmpty);return gate.finish();}
Summary validate(const Handle(Geom_Curve)& c){Gate gate;gate.curve(c);return gate.finish();}
Summary validate(const Handle(Geom2d_Curve)& c){Gate gate;gate.curve(c);return gate.finish();}
Summary validate(const Handle(Geom_Surface)& s){Gate gate;gate.surface(s);return gate.finish();}
Summary validate(const TopoDS_Shape& s){Gate gate;gate.shape(s);return gate.finish();}
}
