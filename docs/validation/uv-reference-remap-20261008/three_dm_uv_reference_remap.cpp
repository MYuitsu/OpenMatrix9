#include "ThreeDmMerge.h"
#include "ThreeDmNativeReferences.h"
#include "ThreeDmCurveOnSurface.h"
#include "opennurbs_polyedgecurve.h"
#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <QUuid>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static int checks=0;
static void require(bool v,const char* message){++checks;if(!v)throw ExchangeError(message);}
static QByteArray bytes(QString path){QFile f(path);require(f.open(QIODevice::ReadOnly),"Read immutable UV reference source");return f.readAll();}
static QString identity(QString seed){return QUuid::createUuidV5(QUuid("44556677-4455-6677-8899-445566778899"),seed.toLatin1()).toString(QUuid::WithoutBraces);}
static QString alias(QString id,QString ns){return QUuid::createUuidV5(QUuid(ns),QByteArrayLiteral("OpenMatrix9.3dm.merge:")+id.toLatin1()+":0").toString(QUuid::WithoutBraces);}
static QJsonObject remapped(QJsonObject fields,QString owner){
    if(fields.contains("object_uuid"))fields["object_uuid"]=owner;
    if(!fields.value("object_references").toArray().isEmpty())fields["object_references"]=QJsonArray{owner};
    for(auto key:{"parameter","approximation","surface"})if(fields.value(key).isObject())fields[key]=remapped(fields.value(key).toObject(),owner);
    if(fields.contains("segments")){QJsonArray children;for(auto child:fields["segments"].toArray())children.append(remapped(child.toObject(),owner));fields["segments"]=children;}
    return fields;
}
static QJsonObject record(const ArchiveInventory& inv,QString id){for(auto v:inv.document["records"].toArray())if(v.toObject()["source_uuid"]==id)return v.toObject();throw ExchangeError("Missing native UV current root");}
static ON_3dPoint analytic(int kind,bool reversed,double f){double u=.25+.5*(reversed?1-f:f);return ON_3dPoint(10+2*u,20+(kind?2*u*(1-u):u),30);}
int main(){try{ON::Begin();QDir out(QStringLiteral(OM9_UV_REFERENCE_DIR));require(QDir().mkpath(out.path()),"Owned UV reference proof folder");QJsonArray evidence;
    const QString ns1="11223344-1122-3344-5566-112233445566",ns2="22334455-2233-4455-6677-223344556677";
    for(int kind=0;kind<2;++kind)for(bool wrapped:{false,true})for(bool reversed:{false,true}){
        QString name=QString("uv-reference-owner-%1-wrapper-%2-reversed-%3.3dm").arg(kind).arg(wrapped?1:0).arg(reversed?1:0),ownerId=identity(name+"-owner"),rootId=identity(name+"-root");
        std::unique_ptr<ON_Curve> owner;
        if(!kind){auto line=new ON_LineCurve(ON_3dPoint(0,0,0),ON_3dPoint(2,1,0));require(line->ChangeDimension(2)&&line->SetDomain(11,23),"Independent two-dimensional line owner");owner.reset(line);}
        else{auto c=new ON_NurbsCurve(2,false,3,3);require(c->SetCV(0,ON_3dPoint(0,0,0))&&c->SetCV(1,ON_3dPoint(1,1,0))&&c->SetCV(2,ON_3dPoint(2,0,0)),"Independent quadratic UV control points");require(c->MakeClampedUniformKnotVector(12)&&c->SetDomain(11,23),"Independent quadratic UV owner domain");owner.reset(c);}
        require(owner->IsValid()&&owner->Dimension()==2,"Valid two-dimensional owner fixture");
        auto segment=new ON_PolyEdgeSegment;segment->SetProxyCurve(owner.get(),ON_Interval(14,20));if(reversed)require(segment->ON_CurveProxy::Reverse(),"Native UV reference direction");require(segment->SetDomain(31,47),"Independent evaluation interval");segment->m_object_id=ON_UuidFromString(ownerId.toLatin1().constData());segment->m_component_index=ON_COMPONENT_INDEX(ON_COMPONENT_INDEX::invalid_type,-1);segment->SetUserString(L"UV child",L"Independent owner reference");
        ON_Curve* uv=segment;if(wrapped){auto p=new ON_PolyEdgeCurve;require(p->Append(segment),"Native UV wrapper owns reference segment");uv=p;}
        auto plane=new ON_PlaneSurface(ON_Plane(ON_3dPoint(10,20,30),ON_3dVector(0,0,1)));plane->SetExtents(0,ON_Interval(-2,4));plane->SetExtents(1,ON_Interval(-2,4));require(plane->SetDomain(0,-2,4)&&plane->SetDomain(1,-2,4),"Plane UV parameters match independent physical coordinates");
        auto root=new ON_CurveOnSurface(uv,nullptr,plane);root->SetUserString(L"Profile",L"UV reference with independent lifetime");require(root->IsValid(),"Valid linked reference UV on native surface");
        QJsonArray samples;for(int i=0;i<=16;++i){auto p=analytic(kind,reversed,i/16.);require(root->PointAt(31+i).DistanceTo(p)<=1e-12,"Independent quadratic/line physical interior samples");samples.append(QJsonArray{p.x,p.y,p.z});}
        ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);ON_3dmObjectAttributes attr;attr.m_uuid=ON_UuidFromString(ownerId.toLatin1().constData());attr.m_name=L"Independent UV owner";require(!model.AddModelGeometryComponent(owner.get(),&attr).IsEmpty(),"Persist independent UV owner identity");attr.m_uuid=ON_UuidFromString(rootId.toLatin1().constData());attr.m_name=L"Curve on referenced UV";require(!model.AddModelGeometryComponent(root,&attr).IsEmpty(),"Persist surface-curve root identity");delete root;
        auto source=out.filePath(name);require(model.Write(std::filesystem::path(source.toStdWString()).c_str(),5,nullptr),"Serialize source UV graph fixture");auto immutable=bytes(source);auto inv=inspectArchive(std::filesystem::path(source.toStdWString()));require(inv.document["issues"].toArray().isEmpty(),"Source UV graph has no broken references");auto sourceRow=record(inv,rootId);require(sourceRow["native_reference_analysis"].toObject()["status"]=="linked"&&sourceRow["dependencies"].toArray().contains(ownerId),"Decoded UV graph retains owner and lifetime");auto fields=sourceRow["curve_on_surface_native"].toObject();
        QJsonArray sources,selected;for(auto ns:{ns1,ns2}){sources.append(QJsonObject{{"namespace",ns},{"snapshot",source},{"archive_sha256",inv.document["archive_sha256"]},{"scale_mm",1}});selected.append(QJsonObject{{"namespace",ns},{"source_uuid",rootId},{"host_id","root"},{"action","unchanged"}});selected.append(QJsonObject{{"namespace",ns},{"source_uuid",rootId},{"host_id","copy"},{"action","duplicate"},{"duplicate_action","unchanged"}});}
        QJsonObject request{{"schema_version",1},{"sources",sources},{"selected",selected}};auto target=std::filesystem::path(out.filePath("current-"+name).toStdWString());writeGeometryStagingArchive(request,target);auto current=inspectArchive(target);require(current.document["issues"].toArray().isEmpty()&&current.document["records"].toArray().size()==6,"Two namespace/copy current graph has all owner dependencies");
        for(auto ns:{ns1,ns2})for(bool copy:{false,true}){auto id=copy?QUuid::createUuidV5(QUuid(ns),QByteArrayLiteral("OpenMatrix9.3dm.source-copy:")+rootId.toLatin1()+":copy").toString(QUuid::WithoutBraces):rootId;if(ns==ns2&&!copy)id=alias(id,ns);auto ownerAlias=ns==ns1?ownerId:alias(ownerId,ns);auto row=record(current,id);require(row["curve_on_surface_native"].toObject()==remapped(fields,ownerAlias),"UV child UUID remap retains domains/reversal/wrapper/metadata");require(row["native_reference_analysis"].toObject()["status"]=="linked"&&row["dependencies"].toArray().contains(ownerAlias),"Current surface-curve points to correct namespace owner");auto graph=current.nativeReferences->graphs.at(id);auto curve=ON_Curve::Cast(graph->geometry.at(id).get());for(int i=0;i<=16;++i)require(curve->PointAt(curve->Domain().ParameterAt(i/16.)).DistanceTo(analytic(kind,reversed,i/16.))<=1e-12,"Independent interior mapping retained after alias/copy/serialize");}
        auto guard=out.filePath("protected.3dm");{QFile f(guard);require(f.open(QIODevice::WriteOnly),"Prepare atomic target guard");f.write("keep-target");}bool refused=false;try{writePreservedArchive(request,std::filesystem::path(guard.toStdWString()));}catch(const ExchangeError&){refused=true;}require(refused&&bytes(guard)=="keep-target","Unverified target UV-reference class cannot overwrite V5 destination");require(bytes(source)==immutable,"Source UV archive never changed");
        evidence.append(QJsonObject{{"fixture",name},{"source_sha256",inv.document["archive_sha256"]},{"native_fields",fields},{"kind",kind},{"wrapped",wrapped},{"reversed",reversed},{"objects",QJsonArray{QJsonObject{{"uuid",ownerId},{"dimension",2}},QJsonObject{{"uuid",rootId},{"domain",QJsonArray{31,47}},{"samples",samples}}}},{"passed",true}});
    }
    require(evidence.size()==8,"Independent eight-file UV reference/remap matrix completed");QFile report(out.filePath("native-results.json"));require(report.open(QIODevice::WriteOnly),"Write UV reference independent oracle");report.write(QJsonDocument(QJsonObject{{"ok",true},{"checks",checks},{"cases",evidence},{"actual_rhino5_verified",false},{"scope","2D owner line/quadratic; segment/one-segment wrapper; internal graph/copy/remap; public V5 still guarded"}}).toJson());std::cout<<checks<<" UV reference remap checks passed\n";return 0;
}catch(const std::exception& e){std::cerr<<"Check "<<checks<<": "<<e.what()<<"\n";return 1;}}
