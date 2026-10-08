// Independent SDK2013 program. No modern OM9 decoder/schema/resolver is linked.
#include "opennurbs.h"
#include "opennurbs_polyedgecurve.h"
#include <QJsonArray>
#include <QJsonObject>
#include <QJsonDocument>
#include <QFile>
#include <filesystem>
#include <iostream>
#include <stdexcept>
#include <cmath>
#include <set>
#include <map>
#include <vector>
static void require(bool value,const char* message){if(!value)throw std::runtime_error(message);}
static QString uuid(ON_UUID id){char text[37]{};ON_UuidToString(id,text);return QString::fromLatin1(text);}
static QJsonArray interval(ON_Interval d){require(std::isfinite(d[0])&&std::isfinite(d[1]),"Finite native interval");return {d[0],d[1]};}
static QJsonArray point(ON_3dPoint p){require(p.IsValid(),"Valid native point");return {p.x,p.y,p.z};}
static QJsonArray frame(const ON_Plane& p){QJsonArray out;for(auto v:{p.origin,ON_3dPoint(p.xaxis),ON_3dPoint(p.yaxis),ON_3dPoint(p.zaxis)})for(int i=0;i<3;++i)out.append(v[i]);for(double coefficient:{p.plane_equation.x,p.plane_equation.y,p.plane_equation.z,p.plane_equation.d})out.append(coefficient);return out;}
static QJsonObject identity(const ON_Geometry& geometry){
    ON_ClassArray<ON_UserString> strings;geometry.GetUserStrings(strings);QJsonArray values;
    for(int i=0;i<strings.Count();++i){ON_String key(strings[i].m_key),value(strings[i].m_string_value);values.append(QJsonObject{{"key",QString::fromUtf8(key.Array())},{"value",QString::fromUtf8(value.Array())}});}
    return {{"class_name",geometry.ClassId()->ClassName()},{"class_uuid",uuid(geometry.ClassId()->Uuid())},{"dimension",geometry.Dimension()},{"user_strings",values}};
}
static QJsonObject curve(const ON_Curve&,unsigned);
static QJsonObject surface(const ON_Surface& s,unsigned depth){
    require(depth<64,"Bounded surface nesting");auto out=identity(s);out["domains"]=QJsonArray{interval(s.Domain(0)),interval(s.Domain(1))};
    if(auto n=ON_NurbsSurface::Cast(&s)){QJsonArray cvs,knots;require(n->CVCount(0)>=0&&n->CVCount(1)>=0&&1ULL*n->CVCount(0)*n->CVCount(1)*n->CVSize()<=2000000,"Bounded surface CV grid");for(int i=0;i<n->CVCount(0);++i){QJsonArray row;for(int j=0;j<n->CVCount(1);++j){QJsonArray p;for(int k=0;k<n->CVSize();++k)p.append(n->CV(i,j)[k]);row.append(p);}cvs.append(row);}for(int d=0;d<2;++d){QJsonArray k;for(int i=0;i<n->KnotCount(d);++i)k.append(n->Knot(d,i));knots.append(k);}out["rational"]=bool(n->IsRational());out["orders"]=QJsonArray{n->Order(0),n->Order(1)};out["cvs"]=cvs;out["knots"]=knots;}
    else if(auto n=ON_PlaneSurface::Cast(&s)){out["plane"]=frame(n->m_plane);out["extents"]=QJsonArray{interval(n->Extents(0)),interval(n->Extents(1))};}
    else if(auto n=ON_RevSurface::Cast(&s)){require(n->m_curve,"Revolution child");out["generatrix"]=curve(*n->m_curve,depth+1);out["axis"]=QJsonArray{point(n->m_axis.from),point(n->m_axis.to)};out["angle_domain"]=interval(n->m_angle);out["angular_parameter_domain"]=interval(n->m_t);out["transposed"]=bool(n->m_bTransposed);}
    else if(auto n=ON_SumSurface::Cast(&s)){require(n->m_curve[0]&&n->m_curve[1],"Sum children");out["curves"]=QJsonArray{curve(*n->m_curve[0],depth+1),curve(*n->m_curve[1],depth+1)};out["basepoint"]=point(n->m_basepoint);}
    else if(auto n=ON_Extrusion::Cast(&s)){require(n->m_profile,"Extrusion profile");out["profile"]=curve(*n->m_profile,depth+1);out["profile_count"]=n->m_profile_count;out["path"]=QJsonArray{point(n->m_path.from),point(n->m_path.to)};out["path_fraction"]=interval(n->m_t);out["path_domain"]=interval(n->m_path_domain);out["up"]=point(ON_3dPoint(n->m_up));out["miter_normals"]=QJsonArray{point(ON_3dPoint(n->m_N[0])),point(ON_3dPoint(n->m_N[1]))};out["has_miter_normals"]=QJsonArray{n->m_bHaveN[0],n->m_bHaveN[1]};out["caps"]=QJsonArray{n->m_bCap[0],n->m_bCap[1]};out["transposed"]=bool(n->m_bTransposed);}
    else throw std::runtime_error("Legacy surface oracle adapter unavailable");return out;
}
static QJsonObject curve(const ON_Curve& c,unsigned depth){
    require(depth<64,"Bounded curve nesting");auto out=identity(c);out["domain"]=interval(c.Domain());
    if(auto n=ON_CurveOnSurface::Cast(&c)){require(n->m_c2&&n->m_s,"Required surface curve children");out["parameter"]=curve(*n->m_c2,depth+1);out["approximation"]=n->m_c3?QJsonValue(curve(*n->m_c3,depth+1)):QJsonValue(QJsonValue::Null);out["surface"]=surface(*n->m_s,depth+1);}
    else if(auto n=ON_PolyEdgeSegment::Cast(&c)){out["object_uuid"]=uuid(n->m_object_id);out["component_index"]=QJsonArray{int(n->m_component_index.m_type),n->m_component_index.m_index};out["edge_domain"]=interval(n->m_edge_domain);out["trim_domain"]=interval(n->m_trim_domain);out["proxy_domain"]=interval(n->ProxyCurveDomain());out["reversed"]=bool(n->ProxyCurveIsReversed());}
    else if(auto n=ON_PolyCurve::Cast(&c)){require(n->Count()<=16384,"Bounded polycurve children");QJsonArray children,parameters;for(int i=0;i<n->Count();++i){require(n->SegmentCurve(i),"Required curve segment");children.append(curve(*n->SegmentCurve(i),depth+1));}const auto& t=n->SegmentParameters();for(int i=0;i<t.Count();++i)parameters.append(t[i]);out["segments"]=children;out["parameters"]=parameters;}
    else if(auto n=ON_NurbsCurve::Cast(&c)){require(n->CVCount()>=0&&1ULL*n->CVCount()*n->CVSize()<=2000000,"Bounded curve CVs");QJsonArray cvs,knots;for(int i=0;i<n->CVCount();++i){QJsonArray p;for(int k=0;k<n->CVSize();++k)p.append(n->CV(i)[k]);cvs.append(p);}for(int i=0;i<n->KnotCount();++i)knots.append(n->Knot(i));out["order"]=n->Order();out["rational"]=bool(n->IsRational());out["cvs"]=cvs;out["knots"]=knots;}
    else if(auto n=ON_ArcCurve::Cast(&c)){out["plane"]=frame(n->m_arc.plane);out["radius"]=n->m_arc.radius;out["angle_domain"]=interval(n->m_arc.Domain());}
    else if(auto n=ON_LineCurve::Cast(&c))out["points"]=QJsonArray{point(n->m_line.from),point(n->m_line.to)};
    else if(auto n=ON_PolylineCurve::Cast(&c)){QJsonArray points,parameters;for(int i=0;i<n->m_pline.Count();++i)points.append(point(n->m_pline[i]));for(int i=0;i<n->m_t.Count();++i)parameters.append(n->m_t[i]);out["points"]=points;out["parameters"]=parameters;}
    else throw std::runtime_error("Legacy curve oracle adapter unavailable");return out;
}
struct Links {
    ONX_Model& model;std::map<QString,ON_Curve*> done;std::set<QString> active;unsigned nodes=0;
    const ON_Geometry* owner(const QString& id){for(int i=0;i<model.m_object_table.Count();++i)if(uuid(model.m_object_table[i].m_attributes.m_uuid)==id)return ON_Geometry::Cast(model.m_object_table[i].m_object);throw std::runtime_error("Legacy missing owner UUID");}
    ON_Curve* bindOwner(const QString& id,unsigned depth){require(depth<64&&!active.count(id),"Legacy reference cycle/depth");if(done.count(id))return done.at(id);auto c=const_cast<ON_Curve*>(ON_Curve::Cast(owner(id)));require(c,"Legacy referenced owner is a curve");active.insert(id);walk(*c,depth);require(c->IsValid(),"Legacy linked curve valid");active.erase(id);done[id]=c;return c;}
    void walk(ON_Geometry& g,unsigned depth){require(depth<64&&++nodes<=16384,"Legacy link work bound");
        if(auto s=ON_PolyEdgeSegment::Cast(&g)){const ON_Curve* target=nullptr;const ON_Brep* brep=nullptr;const ON_BrepEdge* edge=nullptr;const ON_BrepTrim* trim=nullptr;const ON_BrepFace* face=nullptr;auto ci=s->m_component_index;
            if(ci.m_type==ON_COMPONENT_INDEX::invalid_type&&ci.m_index==-1)target=bindOwner(uuid(s->m_object_id),depth+1);
            else{brep=ON_Brep::Cast(owner(uuid(s->m_object_id)));require(brep&&brep->IsValid(),"Legacy valid Brep owner");if(ci.m_type==ON_COMPONENT_INDEX::brep_edge){require(ci.m_index>=0&&ci.m_index<brep->m_E.Count(),"Legacy edge index");edge=&brep->m_E[ci.m_index];}else{require(ci.m_type==ON_COMPONENT_INDEX::brep_trim&&ci.m_index>=0&&ci.m_index<brep->m_T.Count(),"Legacy trim index");trim=&brep->m_T[ci.m_index];edge=trim->Edge();face=trim->Face();}require(edge,"Legacy edge target");target=edge->EdgeCurveOf();}
            require(target&&target->Domain().Includes(s->ProxyCurveDomain()),"Legacy proxy target domain");const auto proxy=s->ProxyCurveDomain(),domain=s->Domain();const bool reverse=s->ProxyCurveIsReversed();s->SetProxyCurve(target,proxy);if(reverse)require(s->ON_CurveProxy::Reverse(),"Legacy proxy reverse");require(s->SetDomain(domain),"Legacy proxy domain");s->m_brep=brep;s->m_edge=edge;s->m_trim=trim;s->m_face=face;s->m_surface=face?face->SurfaceOf():nullptr;s->DestroyRuntimeCache();require(s->IsValid(),"Legacy linked reference valid");
        }else if(auto c=ON_CurveOnSurface::Cast(&g)){require(c->m_c2&&c->m_s,"Legacy required COS children");walk(*c->m_c2,depth+1);if(c->m_c3)walk(*c->m_c3,depth+1);walk(*c->m_s,depth+1);}
        else if(auto c=ON_PolyCurve::Cast(&g)){for(int i=0;i<c->Count();++i)walk(*c->SegmentCurve(i),depth+1);}
        else if(auto s=ON_RevSurface::Cast(&g))walk(*s->m_curve,depth+1);
        else if(auto s=ON_SumSurface::Cast(&g)){walk(*s->m_curve[0],depth+1);walk(*s->m_curve[1],depth+1);}
        else if(auto s=ON_Extrusion::Cast(&g))walk(*s->m_profile,depth+1);
        g.DestroyRuntimeCache();
    }
};
int main(int argc,char** argv){try{
    require(argc==4,"usage: NativeReferenceLegacyReader input output report");ON::Begin();require(ON::Version()==201307115,"Pinned legacy revision");auto input=std::filesystem::u8path(argv[1]),output=std::filesystem::u8path(argv[2]),reportPath=std::filesystem::u8path(argv[3]);require(std::filesystem::file_size(input)<=512ULL*1024*1024&&!std::filesystem::exists(output)&&!std::filesystem::exists(reportPath),"Bounded input/fresh output paths");
    ONX_Model model;require(model.Read(input.c_str(),nullptr)&&model.m_3dm_file_version==50,"Legacy version5 model read");QJsonArray records;std::vector<QString> roots;
    for(int i=0;i<model.m_object_table.Count();++i){auto c=ON_CurveOnSurface::Cast(model.m_object_table[i].m_object);if(!c)continue;auto id=uuid(model.m_object_table[i].m_attributes.m_uuid);records.append(QJsonObject{{"source_uuid",id},{"fields",curve(*c,0)}});roots.push_back(id);}
    require(!roots.empty(),"Native COS roots decoded by legacy class registry");
    QFile report(QString::fromStdWString(reportPath.wstring()));require(report.open(QIODevice::WriteOnly),"Report open");auto bytes=QJsonDocument(QJsonObject{{"sdk_version",201307115},{"source_version",50},{"read_repairs",OM9_LEGACY_REPAIR_DESCRIPTION},{"records",records}}).toJson();require(bytes.size()<=32*1024*1024&&report.write(bytes)==bytes.size(),"Bounded report write");report.close();
    Links linker{model};for(const auto& id:roots)linker.bindOwner(id,0);
    require(model.Write(output.c_str(),5,"OM9 declared SDK2013 native reference oracle",nullptr),"Legacy linked reencode version5");std::cout<<"SDK201307115 decoded/reencoded "<<roots.size()<<" native roots; declared "<<OM9_LEGACY_REPAIR_DESCRIPTION<<'\n';return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
