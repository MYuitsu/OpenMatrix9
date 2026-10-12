#include "ThreeDmInventory.h"
#include "ThreeDmCurveOnSurface.h"
#include "opennurbs_polyedgecurve.h"
#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool value,const char* text){if(!value)throw ExchangeError(text);}
int main(){try{ON::Begin();QDir directory(QStringLiteral(OM9_REFERENCE_TARGET_DIR));require(QDir().mkpath(directory.path()),"Reference target fixture folder");QJsonArray reports;
    for(int kind=0;kind<4;++kind){
        ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
        ON_Curve* uv=nullptr;ON_Curve* approximation=nullptr;ON_Surface* surface=nullptr;
        if(kind<2){uv=new ON_LineCurve(ON_3dPoint(0,0,0),ON_3dPoint(1,1,0));uv->ChangeDimension(2);uv->SetDomain(31,47);surface=new ON_PlaneSurface(ON_xy_plane);
            if(kind==1){approximation=new ON_LineCurve(ON_3dPoint(0,0,0),ON_3dPoint(1,1,0));approximation->SetDomain(31,47);}}
        else{
            auto nativeSurface=new ON_NurbsSurface(3,false,3,2,3,2);for(int i=0;i<4;++i)nativeSurface->SetKnot(0,i,i<2?0:1);nativeSurface->SetKnot(1,0,0);nativeSurface->SetKnot(1,1,1);
            for(int i=0;i<3;++i)for(int j=0;j<2;++j)nativeSurface->SetCV(i,j,ON_3dPoint(i,3*j+(i==1?1:0),kind==3&&i==1?2:0));
            auto brep=new ON_Brep;require(brep->NewFace(*nativeSurface)&&brep->IsValid(),"Coherent native Brep");auto component=model.AddManagedModelGeometryComponent(brep,nullptr);require(!component.IsEmpty(),"Own coherent referenced Brep");
            auto& trim=brep->m_T[0];auto edge=trim.Edge();auto poly=new ON_PolyEdgeCurve;require(poly->Create(edge->EdgeCurveOf(),component.ModelComponent()->Id()),"Coherent edge reference");auto segment=poly->SegmentCurve(0);segment->m_component_index=trim.ComponentIndex();segment->m_edge_domain=ON_Interval(.25,.75);segment->m_trim_domain=ON_Interval(.25,.75);require(segment->SetProxyCurveDomain(ON_Interval(.25,.75)),"Coherent referenced subdomain");segment->SetDomain(31,47);poly->SetDomain(31,47);approximation=poly;
            uv=new ON_LineCurve(ON_3dPoint(.25,0,0),ON_3dPoint(.75,0,0));uv->ChangeDimension(2);uv->SetDomain(31,47);surface=nativeSurface;
        }
        auto root=new ON_CurveOnSurface(uv,approximation,surface);require(root->IsValid(),"Coherent native surface curve");double maximum=0;
        for(int i=0;i<=16;++i){double t=31+i;auto p=uv->PointAt(t);auto physical=surface->PointAt(p.x,p.y);if(approximation)maximum=std::max(maximum,physical.DistanceTo(approximation->PointAt(t)));}
        require(maximum<1e-12,"Optional approximation matches authoritative UV/surface throughout analytical fixtures");
        auto attributes=new ON_3dmObjectAttributes;attributes->m_name=L"Coherent CurveOnSurface";auto added=model.AddManagedModelGeometryComponent(root,attributes);require(!added.IsEmpty(),"Own coherent root and heap attributes");
        auto name=QString("coherent-reference-%1.3dm").arg(kind);auto path=std::filesystem::path(directory.filePath(name).toStdWString());require(model.Write(path.c_str(),5,nullptr),"Coherent Rhino5 fixture write");std::cerr<<"Inspect coherent fixture "<<kind<<'\n';ONX_Model reread;require(reread.Read(path.c_str(),nullptr),"Coherent basic read");std::cerr<<"Basic read passed\n";auto inventory=inspectArchive(path);if(!inventory.document["issues"].toArray().isEmpty())std::cerr<<QJsonDocument(inventory.document).toJson().toStdString();require(inventory.document["issues"].toArray().isEmpty(),"Coherent native reread and reference graph");
        QJsonArray expected;for(auto row:inventory.document["records"].toArray()){auto item=row.toObject();QJsonObject target{{"uuid",item["source_uuid"]},{"native_class",item["class_name"]}};
            if(item["class_name"]=="ON_CurveOnSurface"){QJsonArray samples;for(int i=0;i<=16;++i){const auto uvPoint=uv->PointAt(31+i);const auto point=surface->PointAt(uvPoint.x,uvPoint.y);require(point.DistanceTo(root->PointAt(31+i))<1e-12,"Composed native root matches physical oracle");samples.append(QJsonArray{point.x,point.y,point.z});}target["samples"]=samples;target["domain"]=QJsonArray{31,47};}expected.append(target);}
        reports.append(QJsonObject{{"fixture",name},{"expected_objects",kind<2?1:2},{"objects",expected},{"maximum_native_approximation_deviation",maximum},{"source_sha256",inventory.document["archive_sha256"]},{"passed",true}});
    }
    QFile output(directory.filePath("native-results.json"));require(output.open(QIODevice::WriteOnly),"Reference target oracle report");output.write(QJsonDocument(QJsonObject{{"ok",true},{"cases",reports}}).toJson());
    QJsonArray standalone;
    for(int rootKind=0;rootKind<2;++rootKind)for(int ownerKind=0;ownerKind<3;++ownerKind)for(bool reversed:{false,true}){
        ONX_Model source;source.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);ON_Layer layer;layer.SetName(L"Standalone reference owner");source.AddModelComponent(layer);
        ON_Curve* target=nullptr;const ON_BrepEdge* edge=nullptr;const ON_BrepTrim* trim=nullptr;ON_UUID ownerId;QJsonObject ownerOracle;
        if(ownerKind==0){auto line=new ON_LineCurve(ON_3dPoint(0,0,0),ON_3dPoint(2,3,4));line->SetDomain(11,23);target=line;auto added=source.AddManagedModelGeometryComponent(line,nullptr);require(!added.IsEmpty(),"Standalone line owner");ownerId=added.ModelComponent()->Id();QJsonArray points;for(int i=0;i<17;++i){auto p=line->PointAt(11+12*i/16.);points.append(QJsonArray{p.x,p.y,p.z});}ownerOracle=QJsonObject{{"domain",QJsonArray{11,23}},{"samples",points}};}
        else{ON_3dPoint corners[8]={ON_3dPoint(0,0,0),ON_3dPoint(2,0,0),ON_3dPoint(2,3,0),ON_3dPoint(0,3,0),ON_3dPoint(0,0,4),ON_3dPoint(2,0,4),ON_3dPoint(2,3,4),ON_3dPoint(0,3,4)};auto brep=ON_BrepBox(corners);require(brep&&brep->IsValid(),"Standalone valid independent box owner");auto added=source.AddManagedModelGeometryComponent(brep,nullptr);require(!added.IsEmpty(),"Standalone box ownership");ownerId=added.ModelComponent()->Id();trim=ownerKind==2?&brep->m_T[0]:nullptr;edge=trim?trim->Edge():&brep->m_E[0];target=const_cast<ON_Curve*>(edge->EdgeCurveOf());ownerOracle=QJsonObject{{"faces",6},{"volume_mm3",24}};}
        char ownerText[37]{};ON_UuidToString(ownerId,ownerText);ownerOracle["uuid"]=ownerText;
        auto poly=new ON_PolyEdgeCurve;require(poly->Create(target,ownerId),"Standalone live reference factory");auto segment=poly->SegmentCurve(0);auto domain=target->Domain();double first=domain.ParameterAt(.25),last=domain.ParameterAt(.75);
        if(edge){segment->m_component_index=trim?trim->ComponentIndex():edge->ComponentIndex();segment->m_edge_domain=ON_Interval(edge->Domain().ParameterAt(.25),edge->Domain().ParameterAt(.75));first=edge->RealCurveParameter(segment->m_edge_domain[0]);last=edge->RealCurveParameter(segment->m_edge_domain[1]);if(first>last)std::swap(first,last);if(trim)segment->m_trim_domain=ON_Interval(trim->Domain().ParameterAt(.25),trim->Domain().ParameterAt(.75));}
        require(segment->SetProxyCurveDomain(ON_Interval(first,last)),"Standalone proxy subdomain");if(reversed)require(segment->Reverse(),"Standalone reference reversal");require(segment->SetDomain(31,47)&&poly->SetDomain(31,47),"Standalone evaluation domain");ON_Curve* curve=rootKind==0?static_cast<ON_Curve*>(poly):new ON_PolyEdgeSegment(*segment);if(rootKind==1)delete poly;curve->SetUserString(L"Scope",L"Standalone target compatibility pending");require(curve->IsValid(),"Live standalone reference native-valid");
        QJsonArray points;for(int i=0;i<17;++i){auto expected=target->PointAt(first+(last-first)*(reversed?1-i/16.:i/16.));require(curve->PointAt(31+i).DistanceTo(expected)<=1e-12,"Independent standalone interior reference mapping");points.append(QJsonArray{expected.x,expected.y,expected.z});}
        auto attributes=new ON_3dmObjectAttributes;attributes->m_name=L"Standalone reference root";auto root=source.AddManagedModelGeometryComponent(curve,attributes);require(!root.IsEmpty(),"Standalone root ownership");char rootText[37]{};ON_UuidToString(root.ModelComponent()->Id(),rootText);
        auto name=QString("standalone-kind-%1-owner-%2-reversed-%3.3dm").arg(rootKind).arg(ownerKind).arg(reversed?1:0);auto path=std::filesystem::path(directory.filePath(name).toStdWString());require(source.Write(path.c_str(),5,nullptr),"Independent standalone V5 source write");auto inventory=inspectArchive(path);auto component=ON_ModelGeometryComponent::Cast(inventory.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,root.ModelComponent()->Id()).ModelComponent());auto decoded=component?ON_Curve::Cast(component->Geometry(nullptr)):nullptr;require(decoded&&decoded->ClassId()==curve->ClassId(),"Exact standalone reference class after native read");auto fields=nativeCurveTreeFields(*decoded);require(fields["object_references"].toArray()==QJsonArray{QString::fromLatin1(ownerText)},"Standalone owner UUID survives deferred native read");
        standalone.append(QJsonObject{{"fixture",name},{"source_sha256",inventory.document["archive_sha256"]},{"native_reference_fields",fields},{"root_class",curve->ClassId()->ClassName()},{"objects",QJsonArray{ownerOracle,QJsonObject{{"uuid",rootText},{"domain",QJsonArray{31,47}},{"samples",points}}}},{"passed",true}});
    }
    QFile standaloneOutput(directory.filePath("standalone-native-results.json"));require(standaloneOutput.open(QIODevice::WriteOnly),"Standalone reference independent oracle");standaloneOutput.write(QJsonDocument(QJsonObject{{"ok",true},{"cases",standalone}}).toJson());
    std::cout<<"Coherent reference target fixtures PASS4 coherent and12 standalone native controls\n";return 0;
}catch(const std::exception& e){std::cerr<<"Fixture exception: "<<e.what()<<'\n';return 1;}}
