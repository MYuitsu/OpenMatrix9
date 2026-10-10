#include "ThreeDmMerge.h"
#include "ThreeDmNativeReferences.h"
#include "opennurbs_polyedgecurve.h"
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <QUuid>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static int checks=0;
static void require(bool v,const char* message){++checks;if(!v)throw ExchangeError(message);}
static QByteArray bytes(QString path){QFile f(path);require(f.open(QIODevice::ReadOnly),"Read immutable seam/trim fixture");return f.readAll();}
int main(){try{ON::Begin();QDir dir(QStringLiteral(OM9_SEAM_TRIM_DIR));require(QDir().mkpath(dir.path()),"Owned seam/trim evidence");QJsonArray cases;
    for(int kind=0;kind<2;++kind){
        std::unique_ptr<ON_Brep> brep(kind?ON_BrepCylinder(ON_Cylinder(ON_Circle(ON_xy_plane,3),7),true,true):ON_BrepSphere(ON_Sphere(ON_3dPoint::Origin,3)));
        require(brep&&brep->IsValid()&&brep->IsSolid(),"Independent valid closed analytic BRep");
        QJsonArray topology;int seams=0,singular=0;auto before=brep->DataCRC(0);
        for(int ti=0;ti<brep->m_T.Count();++ti){auto& trim=brep->m_T[ti];if(trim.m_type!=ON_BrepTrim::seam&&trim.m_type!=ON_BrepTrim::singular)continue;
            QJsonObject row{{"trim_index",ti},{"type",int(trim.m_type)},{"edge_index",trim.m_ei},{"vertices",QJsonArray{trim.m_vi[0],trim.m_vi[1]}},{"reversed_3d",trim.m_bRev3d}};
            if(trim.m_type==ON_BrepTrim::singular){++singular;require(trim.m_ei==-1&&!trim.Edge()&&trim.m_vi[0]==trim.m_vi[1],"Singular side has one pole vertex and no3D edge by native topology");
                auto pole=brep->m_V[trim.m_vi[0]].point;for(int i=0;i<=16;++i){auto uv=trim.PointAt(trim.Domain().ParameterAt(i/16.));auto point=trim.Face()->SurfaceOf()->PointAt(uv.x,uv.y);require(point.DistanceTo(pole)<=1e-12&&std::abs(std::abs(point.z)-3)<=1e-12,"Independent sphere singular-side samples collapse exactly to north/south pole");}row["component_curve_applicability"]="not_a_3d_edge_reference";
            }else{++seams;auto edge=trim.Edge();require(edge&&trim.Face()&&trim.Face()->SurfaceOf(),"Seam side has a genuine3D edge and owning surface");
                for(int i=0;i<=16;++i){double t=edge->Domain().ParameterAt(i/16.);auto map=nativeTrimParameterAt(trim,*edge,*trim.Face()->SurfaceOf(),trim.Domain(),t,1e-7);auto uv=trim.PointAt(map.parameter),point=trim.Face()->SurfaceOf()->PointAt(uv.x,uv.y);require(map.deviation<=1e-7&&point.DistanceTo(edge->PointAt(t))<=1e-7,"Physical seam mapping verified at endpoints and interior");
                    require(kind?std::abs(std::hypot(point.x,point.y)-3)<=1e-7&&point.z>=-1e-7&&point.z<=7+1e-7:std::abs(point.DistanceTo(ON_3dPoint::Origin)-3)<=1e-7,"Independent analytic sphere/cylinder locus");}row["interior_mapping"]="projected_samples_verified";
            }topology.append(row);
        }
        require(seams==2&&singular==(kind?0:2),"Native opposite seam sides and singular applicability count");require(brep->DataCRC(0)==before,"Read-only mapping leaves native geometry unchanged");
        auto shape=importBrep(*brep,1e-7);require(!shape.IsNull()&&BRepCheck_Analyzer(shape).IsValid(),"Editable CAD BRep includes valid seam/singular topology");GProp_GProps mass;BRepGProp::VolumeProperties(shape,mass,1e-12);double volume=kind?63*ON_PI:36*ON_PI;require(std::abs(mass.Mass()-volume)<=1e-6,"Independent analytic signed solid volume with original strict threshold");
        ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance=1e-7;
        auto name=QString("seam-trim-owner-%1.3dm").arg(kind);auto uuid=QUuid::createUuidV5(QUuid("55667788-5566-7788-9900-556677889900"),name.toLatin1()).toString(QUuid::WithoutBraces);ON_3dmObjectAttributes attributes;attributes.m_uuid=ON_UuidFromString(uuid.toLatin1().constData());attributes.m_name=L"Analytic seam/singular owner";require(!model.AddModelGeometryComponent(brep.get(),&attributes).IsEmpty(),"Stable native BRep identity");
        auto source=dir.filePath("source-"+name);require(model.Write(std::filesystem::path(source.toStdWString()).c_str(),5,nullptr),"Serialize source BRep fixture");auto immutable=bytes(source);auto inv=inspectArchive(std::filesystem::path(source.toStdWString()));require(inv.document["issues"].toArray().isEmpty(),"Source BRep graph complete");auto record=inv.document["records"].toArray()[0].toObject();
        auto target=std::filesystem::path(dir.filePath(name).toStdWString());QJsonObject archive{{"namespace","55667788-5566-7788-9900-556677889900"},{"snapshot",source},{"archive_sha256",inv.document["archive_sha256"]},{"scale_mm",1}};
        writePreservedArchive(QJsonObject{{"schema_version",1},{"sources",QJsonArray{archive}},{"selected",QJsonArray{QJsonObject{{"namespace",archive["namespace"]},{"source_uuid",uuid},{"host_id","owner"},{"action","unchanged"}}}}},target);auto saved=inspectArchive(target);require(saved.document["issues"].toArray().isEmpty()&&saved.document["records"].toArray()[0].toObject()==record,"Public V5 selection retains full BRep record/native payload");require(bytes(source)==immutable,"Original source stays immutable");
        auto native=ON_ModelGeometryComponent::Cast(saved.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,attributes.m_uuid).ModelComponent());auto output=ON_Brep::Cast(native->Geometry(nullptr));require(output&&output->m_T.Count()==brep->m_T.Count()&&output->m_E.Count()==brep->m_E.Count()&&output->m_V.Count()==brep->m_V.Count(),"Complete native topology counts retained");
        for(auto entry:topology){auto t=entry.toObject();auto& actual=output->m_T[t["trim_index"].toInt()];require(int(actual.m_type)==t["type"].toInt()&&actual.m_ei==t["edge_index"].toInt()&&actual.m_bRev3d==t["reversed_3d"].toBool()&&QJsonArray{actual.m_vi[0],actual.m_vi[1]}==t["vertices"].toArray(),"Seam/singular classification,edge/vertex slots and winding exact");}
        cases.append(QJsonObject{{"fixture",name},{"source_sha256",saved.document["archive_sha256"]},{"kind",kind},{"topology",topology},{"native_record",record},{"objects",QJsonArray{QJsonObject{{"uuid",uuid},{"faces",brep->m_F.Count()},{"volume_mm3",volume}}}},{"passed",true}});
    }
    require(cases.size()==2,"Sphere/capped-cylinder BRep applicability fixtures complete");QFile report(dir.filePath("native-results.json"));require(report.open(QIODevice::WriteOnly),"Write independent topology oracle");report.write(QJsonDocument(QJsonObject{{"ok",true},{"checks",checks},{"cases",cases},{"scope","Native BRep seam/singular applicability and editable analytic solids; does not certify every PolyEdge component/reference profile"}}).toJson());std::cout<<checks<<" seam/singular BRep applicability checks passed\n";return 0;
}catch(const std::exception& e){std::cerr<<"Check "<<checks<<": "<<e.what()<<"\n";return 1;}}
