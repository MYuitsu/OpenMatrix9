#include "ThreeDmMerge.h"
#include "ThreeDmCurveOnSurface.h"
#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <QUuid>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static int checks=0;
static void require(bool v,const char* message){++checks;if(!v)throw ExchangeError(message);}
static QByteArray bytes(QString path){QFile f(path);require(f.open(QIODevice::ReadOnly),"Read immutable seam source");return f.readAll();}
static ON_3dPoint analytical(int kind,double f){
    if(kind<2){double latitude=-ON_PI/2+ON_PI*f;return ON_3dPoint(3*std::cos(latitude),0,3*std::sin(latitude));}
    double angle=2*ON_PI*f;return ON_3dPoint(3*std::cos(angle),3*std::sin(angle),kind==3?3.5:0);
}
int main(){try{ON::Begin();QDir out(QStringLiteral(OM9_SEAM_DIR));require(QDir().mkpath(out.path()),"Owned seam proof folder");QJsonArray cases;
    for(int kind=0;kind<4;++kind)for(bool reverse:{false,true}){
        ON_RevSurface* surface=nullptr;
        if(kind<3)surface=ON_Sphere(ON_3dPoint::Origin,3).RevSurfaceForm(false);
        else{surface=new ON_RevSurface;surface->m_curve=new ON_LineCurve(ON_3dPoint(3,0,0),ON_3dPoint(3,0,7));require(surface->m_curve->SetDomain(0,7),"Independent cylinder axial parameter");surface->m_axis=ON_Line(ON_3dPoint::Origin,ON_3dPoint(0,0,1));surface->m_angle=ON_Interval(0,2*ON_PI);surface->m_t=surface->m_angle;}
        require(surface&&surface->IsValid(),"Exact periodic native revolution surface");
        ON_3dPoint start,end;if(kind<2){double u=kind==1?2*ON_PI:0;start=ON_3dPoint(u,-ON_PI/2,0);end=ON_3dPoint(u,ON_PI/2,0);}else{double v=kind==3?3.5:0;start=ON_3dPoint(0,v,0);end=ON_3dPoint(2*ON_PI,v,0);}
        if(reverse)std::swap(start,end);auto uv=new ON_LineCurve(start,end);require(uv->ChangeDimension(2)&&uv->SetDomain(31,47),"Controlled native UV endpoints/direction");auto root=new ON_CurveOnSurface(uv,nullptr,surface);root->SetUserString(L"Profile",L"Seam and singular endpoints");require(root->IsValid(),"Valid no-C3 seam profile");auto fields=curveOnSurfaceNativeFields(*root);QJsonArray samples;
        for(int i=0;i<=16;++i){auto expected=analytical(kind,reverse?1-i/16.:i/16.);require(root->PointAt(31+i).DistanceTo(expected)<=1e-12,"Independent analytic pole/seam/closed-circle physical point");samples.append(QJsonArray{expected.x,expected.y,expected.z});}
        ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);ON_3dmObjectAttributes attrs;auto name=QString("seam-kind-%1-reversed-%2.3dm").arg(kind).arg(reverse?1:0);attrs.m_uuid=ON_UuidFromString(QUuid::createUuidV5(QUuid("33445566-3344-5566-7788-334455667788"),name.toLatin1()).toString(QUuid::WithoutBraces).toLatin1().constData());attrs.m_name=L"Analytic seam profile";require(!model.AddModelGeometryComponent(root,&attrs).IsEmpty(),"Stable owned seam root identity");delete root;
        auto source=out.filePath("source-"+name);require(model.Write(std::filesystem::path(source.toStdWString()).c_str(),5,nullptr),"Create source fixture");auto immutable=bytes(source);auto input=inspectArchive(std::filesystem::path(source.toStdWString()));require(input.document["issues"].toArray().isEmpty(),"Complete seam input graph");auto record=input.document["records"].toArray()[0].toObject();require(record["curve_on_surface_native"].toObject()==fields,"Exact decoded source native children");
        QJsonObject archive{{"namespace","33445566-3344-5566-7788-334455667788"},{"snapshot",source},{"archive_sha256",input.document["archive_sha256"]},{"scale_mm",1}};
        QJsonObject selection{{"namespace",archive["namespace"]},{"source_uuid",record["source_uuid"]},{"host_id","seam"},{"action","unchanged"}};
        auto target=std::filesystem::path(out.filePath(name).toStdWString());writePreservedArchive(QJsonObject{{"schema_version",1},{"sources",QJsonArray{archive}},{"selected",QJsonArray{selection}}},target);auto result=inspectArchive(target);auto row=result.document["records"].toArray()[0].toObject();require(result.document["source_version"]==50&&result.document["issues"].toArray().isEmpty()&&row["source_uuid"]==record["source_uuid"]&&row["curve_on_surface_native"].toObject()==fields,"Public V5 selected writer keeps class/UV/surface/domain/metadata exact");require(bytes(source)==immutable,"Source remains immutable");
        cases.append(QJsonObject{{"fixture",name},{"source_sha256",result.document["archive_sha256"]},{"kind",kind},{"reversed",reverse},{"native_fields",fields},{"objects",QJsonArray{QJsonObject{{"uuid",record["source_uuid"]},{"domain",QJsonArray{31,47}},{"samples",samples}}}},{"passed",true}});
    }
    require(cases.size()==8,"Eight seam/singular-endpoint profiles completed");QFile report(out.filePath("native-results.json"));require(report.open(QIODevice::WriteOnly),"Write seam oracle");report.write(QJsonDocument(QJsonObject{{"ok",true},{"checks",checks},{"cases",cases},{"actual_rhino5_verified",false},{"scope","CurveOnSurface without reference/C3; poles and periodic endpoints, not singular PolyEdge trim mapping"}}).toJson());std::cout<<checks<<" seam profile checks passed\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<"\n";return 1;}}
