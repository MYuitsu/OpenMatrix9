#include "ThreeDmInventory.h"
#include "ThreeDmNativeReferences.h"
#include "opennurbs_polyedgecurve.h"
#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <memory>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool b,const char* m){if(!b)throw std::runtime_error(m);}
static void graphProof(const ArchiveInventory& archive,const QJsonObject& row){
    auto key=row["source_uuid"].toString();require(archive.nativeReferences&&archive.nativeReferences->graphs.contains(key),"Linked graph exposed with lifetime ownership");
    auto graph=archive.nativeReferences->graphs.at(key);auto linked=ON_CurveOnSurface::Cast(graph->geometry.at(key).get());require(linked&&linked->IsValid(),"Actual linked native root valid");
    auto source=ON_ModelGeometryComponent::Cast(archive.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(key.toLatin1().constData())).ModelComponent());
    auto raw=source?ON_CurveOnSurface::Cast(source->Geometry(nullptr)):nullptr;require(raw&&!raw->IsValid(),"Owning source tree remains deferred and unchanged");
    ON_Write3dmBufferArchive buffer(0,1024*1024,50,ON::Version());require(buffer.WriteObject(linked),"Linked native record can serialize without writer bypass");
    ON_Read3dmBufferArchive reader(buffer.SizeOfArchive(),buffer.Buffer(),false,50,ON::Version());ON_Object* rawDecoded=nullptr;require(reader.ReadObject(&rawDecoded)==1,"Read linked native record as deferred source");std::unique_ptr<ON_Object> decodedOwner(rawDecoded);auto decoded=ON_CurveOnSurface::Cast(rawDecoded);require(decoded,"Exact native record class");
    require(nativeGeometryFacts(*decoded)["curve_on_surface_native"]==row["curve_on_surface_native"],"Native reencode retains exact reference fields and metadata");
}
static QJsonArray brepCases(QDir dir){QJsonArray cases;
    for(bool useTrim:{false,true})for(bool reversed:{false,true}){
        ONX_Model model;ON_Layer layer;layer.SetName(L"Brep owner");model.AddModelComponent(layer);
        ON_3dPoint corners[8]={ON_3dPoint(0,0,0),ON_3dPoint(2,0,0),ON_3dPoint(2,3,0),ON_3dPoint(0,3,0),ON_3dPoint(0,0,4),ON_3dPoint(2,0,4),ON_3dPoint(2,3,4),ON_3dPoint(0,3,4)};
        auto brep=ON_BrepBox(corners);require(brep&&brep->IsValid(),"Real valid native box topology");auto added=model.AddManagedModelGeometryComponent(brep,nullptr);require(!added.IsEmpty(),"Add real Brep owner");
        const ON_BrepTrim* trim=useTrim?&brep->m_T[0]:nullptr;const ON_BrepEdge* edge=trim?trim->Edge():&brep->m_E[0];require(edge,"Real native edge");auto c3=edge->EdgeCurveOf();require(c3,"Real C3");
        auto ref=new ON_PolyEdgeCurve;require(ref->Create(c3,added.ModelComponent()->Id()),"Create native component reference without broken SDK factories");auto segment=ref->SegmentCurve(0);segment->m_component_index=trim?trim->ComponentIndex():edge->ComponentIndex();segment->m_edge_domain=edge->Domain();if(trim)segment->m_trim_domain=trim->Domain();
        require(segment->SetProxyCurveDomain(edge->ProxyCurveDomain()),"Exact C3 proxy subdomain");if(reversed)require(segment->Reverse(),"Reverse native component reference");segment->SetDomain(31,47);ref->SetDomain(31,47);
        auto uv=new ON_LineCurve(ON_3dPoint(0,0,0),ON_3dPoint(1,1,0));uv->ChangeDimension(2);uv->SetDomain(31,47);auto root=new ON_CurveOnSurface(uv,ref,new ON_PlaneSurface(ON_xy_plane));require(root->IsValid(),"Valid live component fixture");model.AddManagedModelGeometryComponent(root,nullptr);
        auto path=std::filesystem::path(dir.filePath(QString("brep-trim-%1-reversed-%2.3dm").arg(useTrim).arg(reversed)).toStdWString());require(model.Write(path.c_str(),5,nullptr),"Write real component source");auto archive=inspectArchive(path,1);QJsonObject row;for(auto value:archive.document["records"].toArray())if(value.toObject()["class_name"]=="ON_CurveOnSurface")row=value.toObject();
        auto analysis=row["native_reference_analysis"].toObject();require(analysis["status"]=="linked","Brep edge/trim reference must link actual owner topology");graphProof(archive,row);
        auto graph=archive.nativeReferences->graphs.at(row["source_uuid"].toString());auto linked=ON_CurveOnSurface::Cast(graph->geometry.at(graph->rootUuid).get());auto segment2=ON_PolyEdgeCurve::Cast(linked->m_c3)->SegmentCurve(0);
        require(segment2->Brep()&&segment2->BrepEdge()&&bool(segment2->BrepTrim())==useTrim&&bool(segment2->BrepFace())==useTrim&&bool(segment2->Surface())==useTrim,"Actual native runtime topology links populated");
        for(int i=0;i<5;++i){double f=i/4.;double real=edge->ProxyCurveDomain().ParameterAt(reversed?1-f:f);auto expected=c3->PointAt(real);auto actual=segment2->PointAt(segment2->Domain().ParameterAt(f));require(actual.DistanceTo(expected)<1e-12,"Native C3 parameter mapping exact");auto onEdge=segment2->BrepEdge()->PointAt(segment2->EdgeParameter(segment2->Domain().ParameterAt(f)));require(actual.DistanceTo(onEdge)<1e-12,"Native EdgeParameter agrees with topology");}
        cases.append(QJsonObject{{"trim",useTrim},{"reversed",reversed},{"linked",true},{"topology",true}});
    }return cases;
}
static int invalidCases(QDir dir){int count=0;
    for(int kind=0;kind<8;++kind){
        ONX_Model model;ON_Layer layer;layer.SetName(L"Invalid owner");model.AddModelComponent(layer);
        ON_LineCurve live(ON_3dPoint(0,0,0),ON_3dPoint(1,1,1));live.SetDomain(11,23);
        ON_UUID owner=ON_UuidFromString("00000000-0000-4000-8000-000000000233");auto rootId=ON_UuidFromString("00000000-0000-4000-8000-000000000244");
        if(kind==1)owner=model.AddManagedModelGeometryComponent(new ON_Point(1,2,3),nullptr).ModelComponent()->Id();
        if(kind==2||kind==3){auto target=new ON_LineCurve(live);if(kind==2)target->SetDomain(100,101);owner=model.AddManagedModelGeometryComponent(target,nullptr).ModelComponent()->Id();}
        if(kind==4)owner=rootId;
        if(kind==5){auto target=new ON_LineCurve(live);target->ChangeDimension(2);owner=model.AddManagedModelGeometryComponent(target,nullptr).ModelComponent()->Id();}
        if(kind>=6){ON_3dPoint corners[8]={ON_3dPoint(0,0,0),ON_3dPoint(2,0,0),ON_3dPoint(2,3,0),ON_3dPoint(0,3,0),ON_3dPoint(0,0,4),ON_3dPoint(2,0,4),ON_3dPoint(2,3,4),ON_3dPoint(0,3,4)};auto brep=ON_BrepBox(corners);require(brep&&brep->IsValid(),"Valid invalid-link Brep owner");owner=model.AddManagedModelGeometryComponent(brep,nullptr).ModelComponent()->Id();}
        auto ref=new ON_PolyEdgeCurve;require(ref->Create(&live,owner),"Valid live source before owner lookup");if(kind==3)ref->SegmentCurve(0)->m_component_index=ON_COMPONENT_INDEX(ON_COMPONENT_INDEX::brep_edge,999);
        if(kind>=6){ref->SegmentCurve(0)->m_component_index=ON_COMPONENT_INDEX(ON_COMPONENT_INDEX::brep_edge,kind==6?999:0);ref->SegmentCurve(0)->m_edge_domain=ON_Interval(100,101);}
        auto uv=new ON_LineCurve(ON_3dPoint(0,0,0),ON_3dPoint(1,1,0));uv->ChangeDimension(2);uv->SetDomain(11,23);auto root=new ON_CurveOnSurface(uv,ref,new ON_PlaneSurface(ON_xy_plane));require(root->IsValid(),"Native pointer validity is not source-reference validity");auto attrs=new ON_3dmObjectAttributes;attrs->m_uuid=rootId;require(!model.AddManagedModelGeometryComponent(root,attrs,false).IsEmpty(),"Add fixed root identity");
        auto path=std::filesystem::path(dir.filePath(QString("invalid-%1.3dm").arg(kind)).toStdWString());require(model.Write(path.c_str(),5,nullptr),"Write malformed logical reference fixture");auto archive=inspectArchive(path,1);QJsonObject row;for(auto value:archive.document["records"].toArray())if(value.toObject()["class_name"]=="ON_CurveOnSurface")row=value.toObject();
        require(row["native_reference_analysis"].toObject()["status"]=="invalid"&&!archive.nativeReferences->graphs.contains(row["source_uuid"].toString()),"Invalid logical reference must not expose partially linked graph");
        bool diagnosed=false;for(auto value:archive.document["issues"].toArray())if(value.toObject()["code"]=="invalid_native_reference")diagnosed=true;
        require(diagnosed,"Invalid reference geometry must be diagnosed before host mutation");++count;
    }return count;
}
static int lifetimeCases(QDir dir){int count=0;for(auto name:{"curve-child-1-reversed-1.3dm","brep-trim-1-reversed-1.3dm"}){
    auto path=std::filesystem::path(dir.filePath(name).toStdWString());auto archive=inspectArchive(path,1);require(archive.nativeReferences->graphs.size()==1,"Exactly one root graph");auto graph=archive.nativeReferences->graphs.begin()->second;std::weak_ptr<ONX_Model> source=archive.nativeModel;
    auto root=ON_CurveOnSurface::Cast(graph->geometry.at(graph->rootUuid).get());auto ref=ON_PolyEdgeCurve::Cast(root->m_c3);require(ref,"Linked lifetime reference");auto before=ref->PointAt(ref->Domain().Mid());archive={};
    require(!source.expired()&&root->IsValid()&&before.DistanceTo(ref->PointAt(ref->Domain().Mid()))<1e-12,"Graph owns targets after inventory/source handles are released");graph.reset();require(source.expired(),"Graph source ownership released without dangling external model handle");++count;
}return count;}
int main(){try{ON::Begin();QDir dir(QStringLiteral(OM9_REFERENCE_EVIDENCE_DIR));require(QDir().mkpath(dir.path()),"Evidence directory");QJsonArray cases;
    for(int child=0;child<2;++child)for(bool reversed:{false,true}){
        ONX_Model model;ON_Layer layer;layer.SetName(L"Reference owner");model.AddModelComponent(layer);
        auto target=new ON_LineCurve(ON_3dPoint(0,0,0),ON_3dPoint(1,1,child==0?0:1));target->SetDomain(11,23);if(child==0)target->ChangeDimension(2);
        auto added=model.AddManagedModelGeometryComponent(target,nullptr);require(!added.IsEmpty(),"Add owner");auto owner=added.ModelComponent()->Id();
        auto ref=new ON_PolyEdgeCurve;require(ref->Create(target,owner),"Create live native PolyEdge");auto segment=ref->SegmentCurve(0);require(segment,"Native reference child");
        require(segment->SetProxyCurveDomain(ON_Interval(13,21)),"Strict native subdomain");if(reversed)require(segment->Reverse(),"Reverse proxy");segment->SetDomain(31,47);ref->SetDomain(31,47);
        auto parameter=new ON_LineCurve(ON_3dPoint(0,0,0),ON_3dPoint(1,1,0));parameter->ChangeDimension(2);parameter->SetDomain(31,47);
        auto plane=new ON_PlaneSurface(ON_xy_plane);plane->SetDomain(0,0,1);plane->SetDomain(1,0,1);
        auto root=new ON_CurveOnSurface(child==0?static_cast<ON_Curve*>(ref):parameter,child==0?nullptr:ref,plane);if(child==0)delete parameter;root->SetUserString(L"Root",L"Keep exact native data");require(root->IsValid(),"Live fixture valid");
        auto rootRef=model.AddManagedModelGeometryComponent(root,nullptr);require(!rootRef.IsEmpty(),"Add exact native root");
        auto path=std::filesystem::path(dir.filePath(QString("curve-child-%1-reversed-%2.3dm").arg(child).arg(reversed)).toStdWString());require(model.Write(path.c_str(),5,nullptr),"Write deferred reference fixture");
        auto archive=inspectArchive(path,1);QJsonObject row;for(auto value:archive.document["records"].toArray())if(value.toObject()["class_name"]=="ON_CurveOnSurface")row=value.toObject();
        auto analysis=row["native_reference_analysis"].toObject();require(analysis["status"]=="linked"&&analysis["native_valid"].toBool(),"Owning-model native references must link into a valid detached CurveOnSurface");
        auto refs=analysis["references"].toArray();require(refs.size()==1&&refs[0].toObject()["target_kind"]=="curve"&&refs[0].toObject()["reversed"]==reversed,"Report exact resolved curve reference kind and reversal");
        auto samples=refs[0].toObject()["samples"].toArray();require(samples.size()==5,"Independent parameter mapping samples");
        for(int i=0;i<5;++i){double f=i/4.;double t=13+8*(reversed?1-f:f);auto expected=target->PointAt(t);auto p=samples[i].toArray();require(p.size()==3&&std::abs(p[0].toDouble()-expected.x)<1e-12&&std::abs(p[1].toDouble()-expected.y)<1e-12&&std::abs(p[2].toDouble()-expected.z)<1e-12,"Reference proxy mapping and native coordinates exact");}
        require(row["curve_on_surface_native"].toObject()[child==0?"parameter":"approximation"].toObject()["dimension"]==0,"Source schema stays deferred, independent of detached linked analysis");
        graphProof(archive,row);
        cases.append(QJsonObject{{"child",child},{"reversed",reversed},{"linked",true},{"samples",true}});
    }
    auto topology=brepCases(dir);
    auto invalid=invalidCases(dir);
    auto lifetime=lifetimeCases(dir);
    QFile report(dir.filePath("results.json"));require(report.open(QIODevice::WriteOnly),"Evidence report");report.write(QJsonDocument(QJsonObject{{"ok",true},{"valid_cases",cases.size()},{"brep_cases",topology.size()},{"invalid_cases",invalid},{"lifetime_cases",lifetime},{"cases",cases},{"topology",topology}}).toJson());std::cout<<"Native source-model reference resolution PASS\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
