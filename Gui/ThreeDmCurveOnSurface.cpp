#include "ThreeDmCurveOnSurface.h"
#include <cmath>
#include <memory>
#include <set>
namespace OpenMatrix9Gui::ThreeDm {
namespace {
// A detached clone must not implicitly resolve private references, clone opaque
// dependency-bearing userdata or exceed the native editor's resource limits.
struct Budget {
    unsigned nodes=0;
    std::size_t numbers=0;
    std::size_t metadataBytes=0;
    std::set<const ON_Object*> active;
    bool rhino5Profile=false;
    const ON_CurveOnSurface* allowedNestedUV=nullptr;
    void reserve(std::size_t count){if(count>2000000-numbers)throw ExchangeError("CurveOnSurface child numeric limit exceeded");numbers+=count;}
};
void safeGeometry(const ON_Geometry& geometry,unsigned depth,Budget& budget,bool nestedSurfaceCurve=true){
    if(depth>=64||++budget.nodes>16384||!budget.active.insert(&geometry).second)throw ExchangeError("CurveOnSurface native child depth/node/cycle limit exceeded");
    for(auto data=geometry.FirstUserData();data;data=data->Next()){
        if(data->ClassId()!=&ON_CLASS_RTTI(ON_UserStringList))throw ExchangeError(std::string("Unverified CurveOnSurface child userdata: ")+data->ClassId()->ClassName());
        auto bytes=static_cast<std::size_t>(data->SizeOf());
        if(bytes>32ULL*1024*1024-budget.metadataBytes)throw ExchangeError("CurveOnSurface child metadata exceeds32MiB");
        budget.metadataBytes+=bytes;
    }
    auto id=geometry.ClassId();
    if(id==&ON_CLASS_RTTI(ON_CurveOnSurface)){
        if(budget.rhino5Profile&&depth>0 && &geometry!=budget.allowedNestedUV)throw ExchangeError("Nested CurveOnSurface Rhino 5 export profile remains unverified; native data remains retained");
        if(!nestedSurfaceCurve)throw ExchangeError("CurveOnSurface native profile parameter mapping is not yet verified");
        auto& c=static_cast<const ON_CurveOnSurface&>(geometry);
        if(!c.m_c2||!c.m_s||(c.m_c3&&c.m_c3==c.m_c2))throw ExchangeError("Missing or aliased owning CurveOnSurface child");
        safeGeometry(*c.m_c2,depth+1,budget);safeGeometry(*c.m_s,depth+1,budget);
        if(c.m_c3)safeGeometry(*c.m_c3,depth+1,budget);
    }else if(id==&ON_CLASS_RTTI(ON_NurbsCurve)){
        auto& c=static_cast<const ON_NurbsCurve&>(geometry);budget.reserve(static_cast<std::size_t>(c.CVCount())*c.CVSize()+c.KnotCount());
    }else if(id==&ON_CLASS_RTTI(ON_PolyCurve)){
        auto& c=static_cast<const ON_PolyCurve&>(geometry);budget.reserve(c.SegmentParameters().Count());
        for(int i=0;i<c.Count();++i){if(!c.SegmentCurve(i))throw ExchangeError("Missing CurveOnSurface native segment");safeGeometry(*c.SegmentCurve(i),depth+1,budget,nestedSurfaceCurve);}
    }else if(id==&ON_CLASS_RTTI(ON_PolylineCurve)){
        auto& c=static_cast<const ON_PolylineCurve&>(geometry);budget.reserve(static_cast<std::size_t>(c.m_pline.Count())*3+c.m_t.Count());
    }else if(id==&ON_CLASS_RTTI(ON_ArcCurve)||id==&ON_CLASS_RTTI(ON_LineCurve)){
        budget.reserve(24);
    }else if(id==&ON_CLASS_RTTI(ON_NurbsSurface)){
        auto& s=static_cast<const ON_NurbsSurface&>(geometry);budget.reserve(static_cast<std::size_t>(s.CVCount(0))*s.CVCount(1)*s.CVSize()+s.KnotCount(0)+s.KnotCount(1));
    }else if(id==&ON_CLASS_RTTI(ON_PlaneSurface)){
        budget.reserve(24);
    }else if(id==&ON_CLASS_RTTI(ON_RevSurface)){
        auto& s=static_cast<const ON_RevSurface&>(geometry);if(!s.m_curve)throw ExchangeError("Missing CurveOnSurface revolution child");safeGeometry(*s.m_curve,depth+1,budget);
    }else if(id==&ON_CLASS_RTTI(ON_SumSurface)){
        auto& s=static_cast<const ON_SumSurface&>(geometry);for(auto c:s.m_curve){if(!c)throw ExchangeError("Missing CurveOnSurface sum child");safeGeometry(*c,depth+1,budget);}
    }else if(id==&ON_CLASS_RTTI(ON_Extrusion)){
        auto& s=static_cast<const ON_Extrusion&>(geometry);if(!s.m_profile)throw ExchangeError("Missing CurveOnSurface extrusion profile");safeGeometry(*s.m_profile,depth+1,budget,false);
    }else throw ExchangeError(std::string("Unverified CurveOnSurface native child/reference transform: ")+id->ClassName());
    if(!geometry.IsValid()||(geometry.Dimension()!=2&&geometry.Dimension()!=3))throw ExchangeError("Invalid CurveOnSurface native child or unresolved reference");
    budget.active.erase(&geometry);
}
bool positiveParameterWeights(const ON_Curve& curve){
    auto type=curve.ClassId();
    if(type==&ON_CLASS_RTTI(ON_NurbsCurve)){
        auto& n=static_cast<const ON_NurbsCurve&>(curve);
        if(n.IsRational())for(int i=0;i<n.CVCount();++i)if(!std::isfinite(n.Weight(i))||n.Weight(i)<=0)return false;
        return true;
    }
    if(type==&ON_CLASS_RTTI(ON_PolyCurve)){
        auto& p=static_cast<const ON_PolyCurve&>(curve);
        for(int i=0;i<p.Count();++i)if(!positiveParameterWeights(*p.SegmentCurve(i)))return false;
        return true;
    }
    return type==&ON_CLASS_RTTI(ON_LineCurve)||type==&ON_CLASS_RTTI(ON_ArcCurve)||type==&ON_CLASS_RTTI(ON_PolylineCurve);
}
bool verifiedNestedUVProfile(const ON_CurveOnSurface& child,const ON_Surface& outer){
    // Actual Rhino5 read/write and decoded native comparisons cover the six
    // original controls and90 sheared/rational/bicubic owning UV controls.
    // One owning nest only; positive weights, single-span unit-domain maps
    // with bounded CVs and parameters. No global correspondence certification.
    if(child.m_c3)throw ExchangeError("Rhino 5 discards CurveOnSurface objects containing m_c3; exact native data remains retained in the project");
    if(!child.m_c2||!child.m_s||child.m_c2->Dimension()!=2||child.m_s->ClassId()!=&ON_CLASS_RTTI(ON_NurbsSurface))return false;
    const auto& surface=static_cast<const ON_NurbsSurface&>(*child.m_s);
    int count=surface.Order(0);
    if(surface.Dimension()!=2||(count!=2&&count!=4)||(count==4&&surface.IsRational()))return false;
    for(int d=0;d<2;++d){
        if(surface.Order(d)!=count||surface.CVCount(d)!=count||surface.KnotCount(d)!=2*count-2)return false;
        for(int i=0;i<surface.KnotCount(d);++i)if(surface.Knot(d,i)!=(i<count-1?0:1))return false;
    }
    // Apply the existing complete traversal budget before validity/bounding-box
    // work on arbitrary parameter children; deeper/unknown children still refuse.
    Budget checked;safeGeometry(child,1,checked);
    if(!positiveParameterWeights(*child.m_c2))return false;
    ON_3dPoint points[4][4];
    for(int i=0;i<count;++i)for(int j=0;j<count;++j){
        if(surface.IsRational()&&(!std::isfinite(surface.Weight(i,j))||surface.Weight(i,j)<=0))return false;
        if(!surface.GetCV(i,j,points[i][j])||!points[i][j].IsValid()||points[i][j].z!=0)return false;
        if(!outer.Domain(0).Includes(points[i][j].x)||!outer.Domain(1).Includes(points[i][j].y))return false;
        if(i&&points[i][j].x<=points[i-1][j].x)return false;
        if(j&&points[i][j].y<=points[i][j-1].y)return false;
    }
    ON_BoundingBox box;
    if(!child.m_c2->GetBoundingBox(box)||!box.IsValid()||!surface.Domain(0).Includes(box.m_min.x)||!surface.Domain(0).Includes(box.m_max.x)||!surface.Domain(1).Includes(box.m_min.y)||!surface.Domain(1).Includes(box.m_max.y))return false;
    return true;
}
bool outOfPlane(const ON_Xform& transform){return transform[2][0]!=0||transform[2][1]!=0||transform[2][3]!=0;}
void curveTransform(ON_Curve& curve,const ON_Xform& transform){
    if(curve.ClassId()==&ON_CLASS_RTTI(ON_ArcCurve)){
        auto& arc=static_cast<ON_ArcCurve&>(curve);auto x=transform*arc.m_arc.plane.xaxis,y=transform*arc.m_arc.plane.yaxis;
        auto sx=x.Length(),sy=y.Length();
        // ON_Circle::Transform can replace a shear with an averaged radius and
        // still return true. Require a native circle-preserving plane map.
        if(!std::isfinite(sx)||!std::isfinite(sy)||sx==0||sy==0||std::abs(sx-sy)>1e-12*std::max(sx,sy)||std::abs(x*y)>1e-12*sx*sy)
            throw ExchangeError("Native ArcCurve approximation/generatrix transform requires explicit class conversion");
    }
    if(curve.ClassId()==&ON_CLASS_RTTI(ON_PolyCurve)){
        auto& poly=static_cast<ON_PolyCurve&>(curve);
        for(int i=0;i<poly.Count();++i)curveTransform(*const_cast<ON_Curve*>(poly.SegmentCurve(i)),transform);
        poly.TransformUserData(transform);poly.DestroyRuntimeCache();
        if(!poly.IsValid())throw ExchangeError("Invalid native CurveOnSurface transformed segments");
        return;
    }
    transformNativeGeometry(curve,transform);
}
void promoteNurbsSurface(ON_NurbsSurface& surface){
    // Pinned ChangeDimension can retain a short CV stride smaller than the new
    // rational CV size, overlapping rows. Rebuild exact dense native raw fields.
    ON_NurbsSurface promoted(3,surface.IsRational(),surface.Order(0),surface.Order(1),surface.CVCount(0),surface.CVCount(1));
    for(int d=0;d<2;++d)for(int i=0;i<surface.KnotCount(d);++i)
        if(!promoted.SetKnot(d,i,surface.Knot(d,i)))throw ExchangeError("Cannot preserve native surface knot in dimension promotion");
    for(int i=0;i<surface.CVCount(0);++i)for(int j=0;j<surface.CVCount(1);++j){auto cv=surface.CV(i,j);double value[4]{cv[0],cv[1],0,surface.IsRational()?cv[2]:1};
        if(!promoted.SetCV(i,j,surface.IsRational()?ON::homogeneous_rational:ON::not_rational,value))throw ExchangeError("Cannot preserve native surface CV in dimension promotion");}
    static_cast<ON_Object&>(promoted)=static_cast<const ON_Object&>(surface);
    if(!promoted.IsValid())throw ExchangeError("Invalid native surface dimension promotion");
    surface=promoted;
}
void planeTransform(ON_PlaneSurface& surface,const ON_Xform& transform){
    // The SDK skips extent scaling when determinant is1, even for anisotropic
    // axes. Compute both axis lengths explicitly and retain the original domains.
    auto frame=surface.m_plane;auto x=transform*frame.xaxis,y=transform*frame.yaxis;
    auto sx=x.Length(),sy=y.Length();
    if(!std::isfinite(sx)||!std::isfinite(sy)||sx==0||sy==0||std::abs(x*y)>1e-12*sx*sy)
        throw ExchangeError("Native PlaneSurface shear requires an explicit CurveOnSurface UV/type adapter");
    frame.origin=transform*frame.origin;x/=sx;y/=sy;frame.xaxis=x;frame.yaxis=y;frame.zaxis=ON_CrossProduct(x,y);frame.UpdateEquation();
    if(!frame.IsValid())throw ExchangeError("Cannot retain native PlaneSurface frame");
    auto a=surface.Extents(0),b=surface.Extents(1);a.Set(a[0]*sx,a[1]*sx);b.Set(b[0]*sy,b[1]*sy);
    surface.TransformUserData(transform);surface.m_plane=frame;
    if(!surface.SetExtents(0,a,false)||!surface.SetExtents(1,b,false))throw ExchangeError("Cannot retain native PlaneSurface extents");
    surface.DestroyRuntimeCache();
}
void surfaceTransform(ON_Surface& surface,const ON_Xform& transform){
    auto id=surface.ClassId();
    if(id==&ON_CLASS_RTTI(ON_PlaneSurface)){planeTransform(static_cast<ON_PlaneSurface&>(surface),transform);return;}
    if(id==&ON_CLASS_RTTI(ON_SumSurface)){
        auto& s=static_cast<ON_SumSurface&>(surface);
        // Native SumSurface calls child Transform directly, bypassing coupled
        // nested surface curves and2D promotion. Dispatch each owned child once.
        for(auto child:s.m_curve)curveTransform(*child,transform);
        s.m_basepoint=transform*s.m_basepoint-ON_3dVector(transform[0][3],transform[1][3],transform[2][3]);
        s.TransformUserData(transform);s.DestroyRuntimeCache();return;
    }
    if(id==&ON_CLASS_RTTI(ON_RevSurface)){
        auto& s=static_cast<ON_RevSurface&>(surface);
        if(transform.IsSimilarity()==0||s.m_curve->Dimension()!=3)throw ExchangeError("Native RevSurface non-similarity/2D generatrix parameter mapping requires an explicit CurveOnSurface adapter");
        std::unique_ptr<ON_Curve> child(s.m_curve->DuplicateCurve());if(!child)throw ExchangeError("Cannot stage native revolution child");
        curveTransform(*child,transform);
        if(!s.Transform(transform))throw ExchangeError("Cannot retain native revolution surface transform");
        delete s.m_curve;s.m_curve=child.release();s.DestroyRuntimeCache();return;
    }
    if(id==&ON_CLASS_RTTI(ON_Extrusion)&&transform.IsSimilarity()<=0)
        throw ExchangeError("Native Extrusion reflected/sheared UV parameter mapping requires an explicit CurveOnSurface adapter");
    if(surface.Dimension()==2&&outOfPlane(transform)){
        auto nurbs=ON_NurbsSurface::Cast(&surface);
        if(!nurbs)throw ExchangeError("Cannot promote native CurveOnSurface surface for out-of-plane placement");
        promoteNurbsSurface(*nurbs);
        if(surface.Dimension()!=3)
            throw ExchangeError("Cannot promote native CurveOnSurface surface for out-of-plane placement");
    }
    if(!surface.Transform(transform))throw ExchangeError("Cannot retain native CurveOnSurface surface transform");
}
}
void validateCurveOnSurfaceRhino5(const ON_CurveOnSurface& curve){
    // Actual Rhino 5.14 File3dm.Read kept the no-C3 control but dropped each
    // coherent C3 control. Never remove the optional child to disguise that loss.
    if(curve.m_c3)throw ExchangeError("Rhino 5 discards CurveOnSurface objects containing m_c3; exact native data remains retained in the project");
    if(!curve.m_c2||!curve.m_s||curve.m_c2->Dimension()!=2||(curve.m_s->Dimension()!=2&&curve.m_s->Dimension()!=3))
        throw ExchangeError("CurveOnSurface Rhino 5 export profile is not yet verified for these UV/surface types; native data remains retained");
    auto uv=curve.m_c2->ClassId(),surface=curve.m_s->ClassId();
    const ON_CurveOnSurface* nestedUV=nullptr;
    if(uv==&ON_CLASS_RTTI(ON_CurveOnSurface)){
        auto& child=static_cast<const ON_CurveOnSurface&>(*curve.m_c2);
        if(verifiedNestedUVProfile(child,*curve.m_s))nestedUV=&child;
    }
    if((uv!=&ON_CLASS_RTTI(ON_LineCurve)&&uv!=&ON_CLASS_RTTI(ON_ArcCurve)&&uv!=&ON_CLASS_RTTI(ON_NurbsCurve)&&uv!=&ON_CLASS_RTTI(ON_PolylineCurve)&&uv!=&ON_CLASS_RTTI(ON_PolyCurve))
        &&!nestedUV)
        throw ExchangeError("CurveOnSurface Rhino 5 export profile is not yet verified for these UV/surface types; native data remains retained");
    if(surface!=&ON_CLASS_RTTI(ON_PlaneSurface)&&surface!=&ON_CLASS_RTTI(ON_NurbsSurface)&&surface!=&ON_CLASS_RTTI(ON_RevSurface)&&surface!=&ON_CLASS_RTTI(ON_SumSurface)&&surface!=&ON_CLASS_RTTI(ON_Extrusion))
        throw ExchangeError("CurveOnSurface Rhino 5 export profile is not yet verified for these UV/surface types; native data remains retained");
    // Thirty direct and96 restricted nested UV profiles survived actual
    // Rhino5 read/write and complete native child-field comparisons.
    Budget budget;budget.rhino5Profile=true;budget.allowedNestedUV=nestedUV;safeGeometry(curve,0,budget);
}
void transformCurveOnSurfaceNative(ON_CurveOnSurface& curve,const ON_Xform& transform){
    Budget budget;safeGeometry(curve,0,budget);
    if(transform.IsIdentity(0.0))return;
    std::unique_ptr<ON_Object> owner(curve.Duplicate());
    auto staged=ON_CurveOnSurface::Cast(owner.get());
    if(!staged||!staged->IsValid())throw ExchangeError("Cannot stage exact native CurveOnSurface children");
    // The UV curve stays in parameter space. Model-coordinate children move
    // together; any native class-transform failure affects only detached copies.
    surfaceTransform(*staged->m_s,transform);
    if(staged->m_c3)curveTransform(*staged->m_c3,transform);
    staged->TransformUserData(transform);
    if(!staged->IsValid())throw ExchangeError("Coupled CurveOnSurface transform produced invalid children");
    curve.DestroyRuntimeCache();
    // Move only the ON_Object userdata base; SDK CurveOnSurface move assignment
    // explicitly calls its destructor and is unsuitable for committing live data.
    static_cast<ON_Object&>(curve)=std::move(static_cast<ON_Object&>(*staged));
    std::swap(curve.m_s,staged->m_s);std::swap(curve.m_c3,staged->m_c3);
}
}
