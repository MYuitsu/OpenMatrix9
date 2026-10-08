#include "ThreeDmCurveOnSurface.h"
#include "opennurbs_polyedgecurve.h"
#include <QJsonArray>
#include <cmath>
#include <set>
namespace OpenMatrix9Gui::ThreeDm {
namespace {
QString uuid(ON_UUID id){char s[37]{};ON_UuidToString(id,s);return QString::fromLatin1(s);}
// Bound source values before creating JSON arrays or converting text. These are
// per-tree limits; the archive inventory also has an aggregate manifest limit.
struct Budget {
    unsigned nodes=0;
    size_t numbers=0,textBytes=0;
    std::set<const ON_Object*> active;
    std::set<QString> references;
    void numeric(size_t n){if(n>2000000-numbers)throw ExchangeError("Native surface schema numeric limit exceeded");numbers+=n;}
    QString text(const ON_wString& s){
        const size_t n=static_cast<size_t>(s.Length());
        if(n>(32ULL*1024*1024-textBytes)/4)throw ExchangeError("Native surface schema metadata limit exceeded");
        textBytes+=4*n;ON_String converted(s);return QString::fromUtf8(converted.Array());
    }
    double number(double n){numeric(1);if(!std::isfinite(n))throw ExchangeError("Non-finite native surface schema field");return n;}
    QJsonArray interval(ON_Interval v){return {number(v[0]),number(v[1])};}
    QJsonArray point(ON_3dPoint v){return {number(v.x),number(v.y),number(v.z)};}
    QJsonArray plane(const ON_Plane& v){
        QJsonArray a;for(auto p:{v.origin,ON_3dPoint(v.xaxis),ON_3dPoint(v.yaxis),ON_3dPoint(v.zaxis)})for(int i=0;i<3;++i)a.append(number(p[i]));
        for(int i=0;i<4;++i)a.append(number(v.plane_equation[i]));return a;
    }
};
struct Visit {
    Budget& budget;const ON_Object* object;
    Visit(Budget& b,const ON_Object& o,unsigned depth):budget(b),object(&o){
        if(depth>=64||++b.nodes>16384||!b.active.insert(&o).second)throw ExchangeError("Native surface schema depth/node/cycle limit exceeded");
    }
    ~Visit(){budget.active.erase(object);}
};
QJsonObject identity(const ON_Geometry& g,Budget& b){
    QJsonArray strings,data;ON_ClassArray<ON_UserString> values;g.GetUserStrings(values);
    for(int i=0;i<values.Count();++i)strings.append(QJsonObject{{"key",b.text(values[i].m_key)},{"value",b.text(values[i].m_string_value)}});
    size_t count=0;
    for(auto d=g.FirstUserData();d;d=d->Next()){
        if(++count>16384)throw ExchangeError("Native surface schema userdata limit exceeded");
        QJsonArray matrix;for(int i=0;i<16;++i)matrix.append(b.number(d->m_userdata_xform[i/4][i%4]));
        data.append(QJsonObject{{"class_name",d->ClassId()->ClassName()},{"class_uuid",uuid(d->ClassId()->Uuid())},{"userdata_uuid",uuid(d->m_userdata_uuid)},{"application_uuid",uuid(d->m_application_uuid)},{"transform",matrix},{"payload_capability",d->ClassId()==&ON_CLASS_RTTI(ON_UserStringList)?"user_strings":"retained in source snapshot"}});
    }
    return {{"class_name",g.ClassId()->ClassName()},{"class_uuid",uuid(g.ClassId()->Uuid())},{"dimension",g.Dimension()},{"user_strings",strings},{"userdata",data}};
}
QJsonObject curve(const ON_Curve&,Budget&,unsigned);
QJsonObject surface(const ON_Surface& s,Budget& b,unsigned depth){
    Visit visit(b,s,depth);auto out=identity(s,b);auto id=s.ClassId();
    out["domains"]=QJsonArray{b.interval(s.Domain(0)),b.interval(s.Domain(1))};
    if(id==&ON_CLASS_RTTI(ON_NurbsSurface)){
        const auto& n=static_cast<const ON_NurbsSurface&>(s);QJsonArray cvs,knots;
        if(n.CVCount(0)<0||n.CVCount(1)<0||n.CVSize()<1)throw ExchangeError("Invalid native surface grid");
        b.numeric(static_cast<size_t>(n.CVCount(0))*n.CVCount(1)*n.CVSize());
        for(int i=0;i<n.CVCount(0);++i){QJsonArray row;for(int j=0;j<n.CVCount(1);++j){auto p=n.CV(i,j);if(!p)throw ExchangeError("Missing native surface CV");QJsonArray cv;for(int k=0;k<n.CVSize();++k){if(!std::isfinite(p[k]))throw ExchangeError("Non-finite native surface CV");cv.append(p[k]);}row.append(cv);}cvs.append(row);}
        for(int d=0;d<2;++d){QJsonArray k;for(int i=0;i<n.KnotCount(d);++i)k.append(b.number(n.Knot(d,i)));knots.append(k);}
        out["orders"]=QJsonArray{n.Order(0),n.Order(1)};out["rational"]=n.IsRational();out["cvs"]=cvs;out["knots"]=knots;
    }else if(id==&ON_CLASS_RTTI(ON_PlaneSurface)){
        const auto& n=static_cast<const ON_PlaneSurface&>(s);out["plane"]=b.plane(n.m_plane);out["extents"]=QJsonArray{b.interval(n.Extents(0)),b.interval(n.Extents(1))};
    }else if(id==&ON_CLASS_RTTI(ON_RevSurface)){
        const auto& n=static_cast<const ON_RevSurface&>(s);if(!n.m_curve)throw ExchangeError("Missing native revolution generatrix");
        out["generatrix"]=curve(*n.m_curve,b,depth+1);out["axis"]=QJsonArray{b.point(n.m_axis.from),b.point(n.m_axis.to)};out["angle_domain"]=b.interval(n.m_angle);out["angular_parameter_domain"]=b.interval(n.m_t);out["transposed"]=n.m_bTransposed;
    }else if(id==&ON_CLASS_RTTI(ON_SumSurface)){
        const auto& n=static_cast<const ON_SumSurface&>(s);if(!n.m_curve[0]||!n.m_curve[1])throw ExchangeError("Missing native sum child");out["curves"]=QJsonArray{curve(*n.m_curve[0],b,depth+1),curve(*n.m_curve[1],b,depth+1)};out["basepoint"]=b.point(n.m_basepoint);
    }else if(id==&ON_CLASS_RTTI(ON_Extrusion)){
        const auto& n=static_cast<const ON_Extrusion&>(s);if(!n.m_profile)throw ExchangeError("Missing native extrusion profile");
        out["profile"]=curve(*n.m_profile,b,depth+1);out["profile_count"]=n.m_profile_count;out["path"]=QJsonArray{b.point(n.m_path.from),b.point(n.m_path.to)};out["path_fraction"]=b.interval(n.m_t);out["path_domain"]=b.interval(n.m_path_domain);out["up"]=b.point(ON_3dPoint(n.m_up));out["miter_normals"]=QJsonArray{b.point(ON_3dPoint(n.m_N[0])),b.point(ON_3dPoint(n.m_N[1]))};out["has_miter_normals"]=QJsonArray{n.m_bHaveN[0],n.m_bHaveN[1]};out["caps"]=QJsonArray{n.m_bCap[0],n.m_bCap[1]};out["transposed"]=n.m_bTransposed;
    }else out["adapter_unavailable"]="Native surface class requires an explicit field/reference adapter";
    return out;
}
QJsonObject curve(const ON_Curve& c,Budget& b,unsigned depth){
    Visit visit(b,c,depth);auto out=identity(c,b);auto id=c.ClassId();out["domain"]=b.interval(c.Domain());
    if(id==&ON_CLASS_RTTI(ON_CurveOnSurface)){
        const auto& n=static_cast<const ON_CurveOnSurface&>(c);if(!n.m_c2||!n.m_s)throw ExchangeError("Missing native surface curve child");
        out["schema_version"]=1;out["parameter"]=curve(*n.m_c2,b,depth+1);out["approximation"]=n.m_c3?QJsonValue(curve(*n.m_c3,b,depth+1)):QJsonValue(QJsonValue::Null);out["surface"]=surface(*n.m_s,b,depth+1);
    }else if(id==&ON_CLASS_RTTI(ON_NurbsCurve)){
        const auto& n=static_cast<const ON_NurbsCurve&>(c);QJsonArray cvs,knots;
        if(n.CVCount()<0||n.CVSize()<1)throw ExchangeError("Invalid native curve grid");b.numeric(static_cast<size_t>(n.CVCount())*n.CVSize());
        for(int i=0;i<n.CVCount();++i){auto p=n.CV(i);if(!p)throw ExchangeError("Missing native curve CV");QJsonArray cv;for(int k=0;k<n.CVSize();++k){if(!std::isfinite(p[k]))throw ExchangeError("Non-finite native curve CV");cv.append(p[k]);}cvs.append(cv);}for(int i=0;i<n.KnotCount();++i)knots.append(b.number(n.Knot(i)));out["order"]=n.Order();out["rational"]=n.IsRational();out["cvs"]=cvs;out["knots"]=knots;
    }else if(id==&ON_CLASS_RTTI(ON_LineCurve)){
        const auto& n=static_cast<const ON_LineCurve&>(c);out["points"]=QJsonArray{b.point(n.m_line.from),b.point(n.m_line.to)};
    }else if(id==&ON_CLASS_RTTI(ON_ArcCurve)){
        const auto& n=static_cast<const ON_ArcCurve&>(c);out["plane"]=b.plane(n.m_arc.plane);out["radius"]=b.number(n.m_arc.radius);out["angle_domain"]=b.interval(n.m_arc.Domain());
    }else if(id==&ON_CLASS_RTTI(ON_PolylineCurve)){
        const auto& n=static_cast<const ON_PolylineCurve&>(c);QJsonArray points,t;for(int i=0;i<n.m_pline.Count();++i)points.append(b.point(n.m_pline[i]));for(int i=0;i<n.m_t.Count();++i)t.append(b.number(n.m_t[i]));out["points"]=points;out["parameters"]=t;
    }else if(id==&ON_CLASS_RTTI(ON_PolyEdgeSegment)){
        const auto& n=static_cast<const ON_PolyEdgeSegment&>(c);
        out["object_uuid"]=uuid(n.m_object_id);out["component_index"]=QJsonArray{static_cast<int>(n.m_component_index.m_type),n.m_component_index.m_index};
        out["edge_domain"]=b.interval(n.m_edge_domain);out["trim_domain"]=b.interval(n.m_trim_domain);out["proxy_domain"]=b.interval(n.ProxyCurveDomain());out["reversed"]=n.ProxyCurveIsReversed();
        // Archive fields are retained; no runtime pointer or evaluated geometry
        // establishes owning-model linkage, component validity or UV semantics.
        out["resolution"]="deferred";b.references.insert(uuid(n.m_object_id));
    }else if(id==&ON_CLASS_RTTI(ON_PolyCurve)||id==&ON_CLASS_RTTI(ON_PolyEdgeCurve)){
        const auto& n=static_cast<const ON_PolyCurve&>(c);QJsonArray children,t;for(int i=0;i<n.Count();++i){auto child=n.SegmentCurve(i);if(!child)throw ExchangeError("Missing native polycurve child");children.append(curve(*child,b,depth+1));}const auto& params=n.SegmentParameters();for(int i=0;i<params.Count();++i)t.append(b.number(params[i]));out["segments"]=children;out["parameters"]=t;
    }else out["adapter_unavailable"]="Native curve class requires an explicit field/reference adapter";
    return out;
}
bool polyEdgeReference(const ON_Curve& c,Budget& b,unsigned depth){
    Visit visit(b,c,depth);
    if(c.ClassId()==&ON_CLASS_RTTI(ON_PolyEdgeCurve)||c.ClassId()==&ON_CLASS_RTTI(ON_PolyEdgeSegment))return true;
    if(c.ClassId()==&ON_CLASS_RTTI(ON_PolyCurve)){
        const auto& poly=static_cast<const ON_PolyCurve&>(c);
        bool found=false;
        for(int i=0;i<poly.Count();++i){auto child=poly.SegmentCurve(i);if(!child)throw ExchangeError("Missing native polycurve child");found=polyEdgeReference(*child,b,depth+1)||found;}
        return found;
    }
    return false;
}
}
bool hasPolyEdgeReference(const ON_Curve& c){Budget budget;return polyEdgeReference(c,budget,0);}
QJsonObject nativeCurveTreeFields(const ON_Curve& c){Budget budget;auto fields=curve(c,budget,0);QJsonArray references;for(const auto& id:budget.references)references.append(id);fields["object_references"]=references;return fields;}
QJsonObject curveOnSurfaceNativeFields(const ON_CurveOnSurface& c){return nativeCurveTreeFields(c);}
}
