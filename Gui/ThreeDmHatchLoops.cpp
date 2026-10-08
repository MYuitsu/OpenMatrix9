#include "ThreeDmHatch.h"
#include <QJsonArray>
#include <QRegularExpression>
#include <set>
namespace OpenMatrix9Gui::ThreeDm {
namespace {
QString uuid(ON_UUID v){char s[37]{};ON_UuidToString(v,s);return QString::fromLatin1(s);}
QString text(const ON_wString& v){ON_String s(v);return QString::fromUtf8(s.Array());}
QJsonArray strings(const ON_Object& object){QJsonArray out;ON_ClassArray<ON_UserString> values;object.GetUserStrings(values);for(int i=0;i<values.Count();++i)out.append(QJsonObject{{"key",text(values[i].m_key)},{"value",text(values[i].m_string_value)}});return out;}
// Copy generation is SDK lifetime bookkeeping, incremented by native clones.
// It is not a stable archive semantic field; plugin payloads remain unsafe.
QJsonArray userdata(const ON_Object& object){QJsonArray out;for(auto d=object.FirstUserData();d;d=d->Next())out.append(QJsonObject{{"class_name",d->ClassId()->ClassName()},{"class_uuid",uuid(d->ClassId()->Uuid())},{"userdata_uuid",uuid(d->m_userdata_uuid)},{"application_uuid",uuid(d->m_application_uuid)}});return out;}
QJsonArray point(ON_3dPoint p){return {p.x,p.y,p.z};}
QJsonArray plane(const ON_Plane& p){QJsonArray out;for(auto v:{p.origin,ON_3dPoint(p.xaxis),ON_3dPoint(p.yaxis),ON_3dPoint(p.zaxis)})for(int i=0;i<3;++i)out.append(v[i]);out.append(p.plane_equation.x);out.append(p.plane_equation.y);out.append(p.plane_equation.z);out.append(p.plane_equation.d);return out;}
struct Budget{unsigned nodes=0;size_t numbers=0;std::set<const ON_Curve*> active;void reserve(size_t count){if(count>2000000-numbers)throw ExchangeError("Hatch loop numeric inventory limit exceeded");numbers+=count;}};
QJsonObject curve(const ON_Curve& c,unsigned depth,Budget& budget,bool safe){
    if(depth>=64||++budget.nodes>16384||!budget.active.insert(&c).second)throw ExchangeError("Hatch loop nesting/node/cycle limit exceeded");
    const auto classId=c.ClassId();const auto name=QString::fromLatin1(classId->ClassName());
    if(safe){
        for(auto d=c.FirstUserData();d;d=d->Next())if(d->ClassId()!=&ON_CLASS_RTTI(ON_UserStringList))throw ExchangeError("Unsafe native Hatch loop userdata ["+name.toStdString()+" / "+d->ClassId()->ClassName()+"]");
        // Child plugin references cannot yet be resolved/remapped by the selected
        // closure. Keep their source snapshot, but never silently copy them.
        static const QRegularExpression reference("[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}");
        for(auto v:strings(c)){auto item=v.toObject();if(reference.match(item["key"].toString()).hasMatch()||reference.match(item["value"].toString()).hasMatch())throw ExchangeError("Unresolved native Hatch loop user-text reference");}
    }
    QJsonObject out{{"class_name",name},{"class_uuid",uuid(classId->Uuid())},{"dimension",c.Dimension()},{"domain",QJsonArray{c.Domain()[0],c.Domain()[1]}},{"user_strings",strings(c)},{"userdata",userdata(c)}};
    // Exact class dispatch is essential: PolyEdgeCurve derives from PolyCurve
    // and PolyEdgeSegment from CurveProxy, but their reference semantics differ.
    if(classId==&ON_CLASS_RTTI(ON_NurbsCurve)){
        const auto& n=static_cast<const ON_NurbsCurve&>(c);QJsonArray cvs,knots;
        budget.reserve(static_cast<size_t>(n.CVCount())*n.CVSize()+n.KnotCount());
        for(int i=0;i<n.CVCount();++i){QJsonArray cv;for(int j=0;j<n.CVSize();++j)cv.append(n.CV(i)[j]);cvs.append(cv);}for(int i=0;i<n.KnotCount();++i)knots.append(n.Knot(i));
        out["order"]=n.Order();out["rational"]=n.IsRational();out["cvs"]=cvs;out["knots"]=knots;
    }else if(classId==&ON_CLASS_RTTI(ON_LineCurve)){
        const auto& n=static_cast<const ON_LineCurve&>(c);out["points"]=QJsonArray{point(n.m_line.from),point(n.m_line.to)};
    }else if(classId==&ON_CLASS_RTTI(ON_ArcCurve)){
        const auto& n=static_cast<const ON_ArcCurve&>(c);out["plane"]=plane(n.m_arc.plane);out["radius"]=n.m_arc.radius;out["angle_domain"]=QJsonArray{n.m_arc.Domain()[0],n.m_arc.Domain()[1]};
    }else if(classId==&ON_CLASS_RTTI(ON_PolylineCurve)){
        const auto& n=static_cast<const ON_PolylineCurve&>(c);budget.reserve(static_cast<size_t>(n.m_pline.Count())*3+n.m_t.Count());QJsonArray points,params;for(int i=0;i<n.m_pline.Count();++i)points.append(point(n.m_pline[i]));for(int i=0;i<n.m_t.Count();++i)params.append(n.m_t[i]);out["points"]=points;out["parameters"]=params;
    }else if(classId==&ON_CLASS_RTTI(ON_PolyCurve)){
        const auto& n=static_cast<const ON_PolyCurve&>(c);QJsonArray children,params;for(int i=0;i<n.Count();++i){auto child=n.SegmentCurve(i);if(!child)throw ExchangeError("Missing native Hatch loop segment");children.append(curve(*child,depth+1,budget,safe));}const auto& t=n.SegmentParameters();for(int i=0;i<t.Count();++i)params.append(t[i]);out["segments"]=children;out["parameters"]=params;
    }else{
        out["adapter_unavailable"]="Native curve child/reference representation requires an explicit adapter";
        if(safe)throw ExchangeError("Unsupported native Hatch loop reference/class ["+name.toStdString()+"]");
    }
    budget.active.erase(&c);return out;
}
}
QJsonArray hatchLoopInventory(const ON_Hatch& h){validateHatch(h);QJsonArray out;Budget budget;for(int i=0;i<h.LoopCount();++i)out.append(QJsonObject{{"type",static_cast<int>(h.Loop(i)->Type())},{"curve",curve(*h.Loop(i)->Curve(),0,budget,false)}});return out;}
void validateHatchRhino5Data(const ON_Hatch& h){
    validateHatch(h);
    const auto gradient=ON_UuidFromString("0c1ad613-4efa-4f47-a147-4d79d77fcb0c");
    if(h.GetGradientType()!=ON_GradientType::None||h.GetUserData(gradient))throw ExchangeError("Rhino5 cannot retain native Hatch gradient data (requires archive version6 or newer)");
    Budget budget;for(int i=0;i<h.LoopCount();++i)curve(*h.Loop(i)->Curve(),0,budget,true);
}
}
