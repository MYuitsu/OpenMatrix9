// OM9-FILE-012: source-unit native field schema and object-reference closure.
#include "ThreeDmArchive.h"
#include "ThreeDmInventory.h"
#include "opennurbs_polyedgecurve.h"
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QDir>
#include <QFile>
#include <memory>
#include <filesystem>
#include <iostream>
#include <stdexcept>
#include <limits>
static void require(bool value,const char* message){if(!value)throw std::runtime_error(message);}
static QJsonArray interval(ON_Interval value){return {value[0],value[1]};}
static std::unique_ptr<ON_CurveOnSurface> fixture(int kind,bool optional){
    ON_Surface* surface=nullptr;
    if(kind==0){auto n=new ON_NurbsSurface(3,true,2,2,2,2);for(int i=0;i<2;++i)for(int j=0;j<2;++j){double w=1+.125*i+.25*j;n->SetCV(i,j,ON_4dPoint(w*(10+i),w*(20+j),w*(30+.5*i*j),w));}for(int d=0;d<2;++d){n->SetKnot(d,0,0);n->SetKnot(d,1,1);}surface=n;
    }else if(kind==1){auto p=new ON_PlaneSurface(ON_Plane(ON_3dPoint(10,20,30),ON_3dVector::ZAxis));for(int d=0;d<2;++d){p->SetDomain(d,0,1);p->SetExtents(d,ON_Interval(-.25,1.25),false);}surface=p;
    }else if(kind==2){auto r=new ON_RevSurface;r->m_curve=new ON_LineCurve(ON_3dPoint(2,0,0),ON_3dPoint(2,0,1));r->m_curve->SetDomain(0,1);r->m_axis=ON_Line(ON_3dPoint::Origin,ON_3dPoint(0,0,1));r->m_angle=ON_Interval(0,2*ON_PI);r->m_t=ON_Interval(0,1);r->m_bTransposed=false;surface=r;
    }else if(kind==3){auto s=new ON_SumSurface;ON_LineCurve a(ON_3dPoint(10,20,30),ON_3dPoint(11,20,30)),b(ON_3dPoint(0,0,0),ON_3dPoint(0,1,.5));a.SetDomain(0,1);b.SetDomain(0,1);require(s->Create(a,b),"Create native SumSurface");surface=s;
    }else{surface=ON_Extrusion::Cylinder(ON_Cylinder(ON_Circle(ON_Plane(ON_3dPoint(10,20,30),ON_3dVector::ZAxis),1),1),false,false);require(surface,"Create native Extrusion");}
    surface->SetUserString(L"Child",L"Surface");auto c2=new ON_ArcCurve(ON_Circle(ON_Plane(ON_3dPoint(.5,.5,0),ON_3dVector::ZAxis),.25),11,23);c2->ChangeDimension(2);c2->SetUserString(L"Child",L"Parameter");ON_Curve* c3=nullptr;
    if(optional){auto a=new ON_ArcCurve(ON_Circle(ON_Plane(ON_3dPoint(10.5,20.5,30),ON_3dVector::ZAxis),.25),11,23);a->SetUserString(L"Child",L"Approximation");c3=a;}
    auto result=std::make_unique<ON_CurveOnSurface>(c2,c3,surface);result->SetUserString(L"Root",L"Schema source");require(result->IsValid(),"Valid native schema source");return result;
}
static void arcFacts(const QJsonObject& fields,const ON_ArcCurve& source){require(fields["class_name"]=="ON_ArcCurve"&&fields["dimension"]==source.Dimension()&&fields["domain"].toArray()==interval(source.Domain())&&fields["radius"].toDouble()==source.m_arc.radius&&fields["angle_domain"].toArray()==interval(source.m_arc.Domain()),"Exact native Arc fields in schema");auto plane=fields["plane"].toArray();require(plane.size()==16&&plane[0].toDouble()==source.m_arc.plane.origin.x&&plane[1].toDouble()==source.m_arc.plane.origin.y&&plane[2].toDouble()==source.m_arc.plane.origin.z,"Native Arc frame must include all16 fields");}
static QString uuid(ON_UUID id){char s[37]{};ON_UuidToString(id,s);return QString::fromLatin1(s);}
static int referenceCases(QDir evidence){int count=0;
    for(bool present:{true,false})for(int child=0;child<2;++child)for(bool reversed:{false,true}){
        auto source=fixture(0,true);std::unique_ptr<ON_Curve> target(child==0?source->m_c2:source->m_c3);
        ONX_Model model;ON_Layer layer;layer.SetName(L"Reference layer");model.AddModelComponent(layer);
        auto referenceId=ON_UuidFromString("00000000-0000-4000-8000-000000000143");
        if(present){auto component=model.AddModelGeometryComponent(target.get(),nullptr);referenceId=component.ModelComponent()->Id();}
        auto reference=new ON_PolyEdgeCurve;require(reference->Create(target.get(),referenceId),"Create real referenced child");
        auto segment=reference->SegmentCurve(0);require(segment&&(!reversed||segment->Reverse()),"Reverse real reference segment");segment->m_edge_domain=ON_Interval(31,47);segment->m_trim_domain=ON_Interval(53,79);segment->SetUserString(L"Reference child",L"Raw fields");
        if(child==0)source->m_c2=reference;else source->m_c3=reference;
        // Transfer the exact native fixture without relying on a cloning path.
        // In particular, DuplicateCurve() may convert a PolyEdge proxy's type.
        model.AddManagedModelGeometryComponent(source.release(),nullptr);
        auto file=std::filesystem::path(evidence.filePath(QString("reference-child-%1-present-%2-reversed-%3.3dm").arg(child).arg(present).arg(reversed)).toStdWString());require(model.Write(file.c_str(),5,nullptr),"Write real deferred model");
        auto inventory=OpenMatrix9Gui::ThreeDm::inspectArchive(file,1);auto manifest=inventory.document;QJsonObject row;
        for(auto item:manifest["records"].toArray())if(item.toObject()["class_name"]=="ON_CurveOnSurface")row=item.toObject();
        auto fields=row["curve_on_surface_native"].toObject()[child==0?"parameter":"approximation"].toObject();
        require(fields["segments"].toArray().size()==1,"Deferred reference must inventory native segments");auto raw=fields["segments"].toArray()[0].toObject();
        require(fields["class_name"]=="ON_PolyEdgeCurve"&&fields["dimension"]==0&&raw["class_name"]=="ON_PolyEdgeSegment","Deferred reference exact class identity must not be flattened to PolyCurve");
        QFile diagnostic(evidence.filePath(QString("reference-child-%1-present-%2-reversed-%3.json").arg(child).arg(present).arg(reversed)));require(diagnostic.open(QIODevice::WriteOnly),"Open reference diagnostic");
        diagnostic.write(QJsonDocument(QJsonObject{{"inventory",manifest},{"expected",QJsonObject{{"object_uuid",uuid(referenceId)},{"component_index",QJsonArray{static_cast<int>(segment->m_component_index.m_type),segment->m_component_index.m_index}},{"domain",interval(segment->Domain())},{"edge_domain",interval(segment->m_edge_domain)},{"trim_domain",interval(segment->m_trim_domain)},{"proxy_domain",interval(segment->ProxyCurveDomain())},{"reversed",segment->ProxyCurveIsReversed()}}}}).toJson());diagnostic.close();
        require(raw["object_uuid"]==uuid(referenceId)&&raw["component_index"].toArray()==QJsonArray{static_cast<int>(segment->m_component_index.m_type),segment->m_component_index.m_index}&&raw["domain"].toArray()==interval(segment->Domain())&&raw["proxy_domain"].toArray()==interval(segment->ProxyCurveDomain())&&raw["reversed"].toBool()==segment->ProxyCurveIsReversed()&&raw["edge_domain"].toArray()==interval(segment->m_edge_domain)&&raw["trim_domain"].toArray()==interval(segment->m_trim_domain),"Every serialized PolyEdge reference field retained");
        require(raw["resolution"]=="deferred"&&!raw.contains("points")&&raw["user_strings"].toArray().size()==1,"Unresolved reference must not pretend to be evaluated geometry");
        require(row["dependencies"].toArray().contains(uuid(referenceId)),"Native child object UUID must participate in source dependency closure");
        bool missing=false;for(auto issue:manifest["issues"].toArray())if(issue.toObject()["code"]=="missing_dependency"&&issue.toObject()["dependency"]==uuid(referenceId))missing=true;
        require(missing==!present,"Missing deferred reference must be explicit; valid dependency presence is not semantic resolution");
        // Native CurveOnSurface::Write validates child geometry and refuses an
        // unresolved proxy. Inventory/dependency presence is not write support.
        auto component=ON_ModelGeometryComponent::Cast(inventory.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(row["source_uuid"].toString().toLatin1().constData())).ModelComponent());
        auto native=component?ON_CurveOnSurface::Cast(component->Geometry(nullptr)):nullptr;require(native,"Find exact retained native tree");
        ON_Write3dmBufferArchive buffer(0,1024*1024,50,ON::Version());require(!native->Write(buffer)&&buffer.SizeOfArchive()==0,"Deferred native payload writer must refuse before emitting child data");++count;
    }return count;
}
static int budgetCases(){int count=0;auto rejects=[&](ON_CurveOnSurface& c){bool failed=false;try{OpenMatrix9Gui::ThreeDm::nativeGeometryFacts(c);}catch(const OpenMatrix9Gui::ThreeDm::ExchangeError&){failed=true;}require(failed,"Invalid or oversized native tree must refuse explicitly");++count;};
    {auto c=fixture(0,true);delete c->m_c2;c->m_c2=nullptr;rejects(*c);}
    {auto c=fixture(0,true);delete c->m_s;c->m_s=nullptr;rejects(*c);}
    {auto c=fixture(0,true);ON_ArcCurve::Cast(c->m_c2)->m_arc.radius=std::numeric_limits<double>::quiet_NaN();rejects(*c);}
    {auto c=fixture(0,true);delete c->m_c3;c->m_c3=c.get();rejects(*c);c->m_c3=nullptr;}
    {auto c=fixture(0,false);auto last=c.get();for(int i=0;i<65;++i){last->m_c3=fixture(0,false).release();last=static_cast<ON_CurveOnSurface*>(last->m_c3);}rejects(*c);}
    {auto c=fixture(0,false);std::wstring text(17*1024*1024,L'x');c->m_s->SetUserString(L"Oversized",text.c_str());rejects(*c);}
    {auto c=fixture(0,false);delete c->m_c2;c->m_c2=new ON_NurbsCurve(2,true,2,666667);rejects(*c);}
    return count;
}
int main(){try{ON::Begin();QDir evidence(QStringLiteral(OM9_SURFACE_SCHEMA_EVIDENCE_DIR));require(QDir().mkpath(evidence.path()),"Evidence directory");QJsonArray cases;int count=0;
    for(int kind=0;kind<5;++kind)for(bool optional:{false,true}){
        auto curve=fixture(kind,optional);auto fields=OpenMatrix9Gui::ThreeDm::nativeGeometryFacts(*curve)["curve_on_surface_native"].toObject();
        require(fields["schema_version"]==1&&fields["class_name"]=="ON_CurveOnSurface"&&fields["dimension"]==3&&fields["domain"].toArray()==interval(curve->Domain()),"CurveOnSurface must expose versioned native field schema");
        arcFacts(fields["parameter"].toObject(),*ON_ArcCurve::Cast(curve->m_c2));if(optional)arcFacts(fields["approximation"].toObject(),*ON_ArcCurve::Cast(curve->m_c3));else require(fields["approximation"].isNull(),"Optional absence represented as null");
        auto surface=fields["surface"].toObject();require(surface["class_name"].toString()==curve->m_s->ClassId()->ClassName()&&surface["domains"].toArray()==QJsonArray{interval(curve->m_s->Domain(0)),interval(curve->m_s->Domain(1))},"Surface native identity/domains retained");
        if(kind==0){auto n=ON_NurbsSurface::Cast(curve->m_s);require(surface["rational"].toBool()&&surface["orders"].toArray()==QJsonArray{2,2},"Native rational surface orders");auto rows=surface["cvs"].toArray();for(int i=0;i<2;++i)for(int j=0;j<2;++j){auto cv=rows[i].toArray()[j].toArray();require(cv.size()==4,"Raw homogeneous CV cardinality");for(int k=0;k<4;++k)require(cv[k].toDouble()==n->CV(i,j)[k],"Raw homogeneous surface CV exact");}require(surface["knots"].toArray()==QJsonArray{QJsonArray{0.,1.},QJsonArray{0.,1.}},"Raw surface knots exact");}
        if(kind==1){require(surface["plane"].toArray().size()==16&&surface["extents"].toArray()==QJsonArray{QJsonArray{-.25,1.25},QJsonArray{-.25,1.25}},"Plane extents and domains remain distinct");}
        if(kind==2)require(surface["generatrix"].toObject()["class_name"]=="ON_LineCurve"&&surface["axis"].toArray()==QJsonArray{QJsonArray{0.,0.,0.},QJsonArray{0.,0.,1.}}&&surface["angle_domain"].toArray()==QJsonArray{0.,2*ON_PI}&&surface["angular_parameter_domain"].toArray()==QJsonArray{0.,1.}&&!surface["transposed"].toBool(),"Native revolution fields/generatrix exact");
        if(kind==3)require(surface["curves"].toArray().size()==2&&surface["basepoint"].toArray().size()==3,"Native sum children and basepoint present");
        if(kind==4){auto s=ON_Extrusion::Cast(curve->m_s);require(surface["profile"].toObject()["class_name"]==s->m_profile->ClassId()->ClassName()&&surface["profile_count"]==s->m_profile_count&&surface["path_domain"].toArray()==interval(s->m_path_domain)&&surface["path_fraction"].toArray()==interval(s->m_t)&&surface["miter_normals"].toArray().size()==2&&surface["has_miter_normals"].toArray().size()==2&&surface["caps"].toArray()==QJsonArray{false,false},"Native extrusion profile/path/miter/cap fields present");}
        require(fields["user_strings"].toArray().size()==1&&fields["parameter"].toObject()["userdata"].toArray()[0].toObject()["transform"].toArray().size()==16,"Root/child metadata and userdata matrix retained");
        ONX_Model model;ON_Layer layer;layer.SetName(L"Schema layer");model.AddModelComponent(layer);auto component=model.AddModelGeometryComponent(curve.get(),nullptr);auto id=component.ModelComponent()->Id();auto file=std::filesystem::path(evidence.filePath(QString("surface-%1-optional-%2.3dm").arg(kind).arg(optional)).toStdWString());require(model.Write(file.c_str(),5,nullptr),"Write model source");auto manifest=OpenMatrix9Gui::ThreeDm::inspectArchive(file,1).document;auto row=manifest["records"].toArray()[0].toObject();require(row["curve_on_surface_native"].toObject()==fields&&row["capability"]=="retained","Source model inventory exposes the same native schema without false editable capability");
        cases.append(QJsonObject{{"kind",kind},{"optional",optional},{"exact_schema",true},{"model_inventory",true}});++count;
    }
    const int references=referenceCases(evidence);
    const int refusals=budgetCases();
    QFile report(evidence.filePath("results.json"));require(report.open(QIODevice::WriteOnly),"Open schema report");auto data=QJsonDocument(QJsonObject{{"ok",true},{"valid_cases",count},{"reference_cases",references},{"budget_refusals",refusals},{"cases",cases}}).toJson();require(report.write(data)==data.size(),"Write schema report");std::cout<<"CurveOnSurface native schema "<<count<<" direct/model field paths, "<<references<<" reference paths, "<<refusals<<" refusals PASS\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
