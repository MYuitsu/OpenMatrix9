#include "ThreeDmArchive.h"
#include "ThreeDmMerge.h"
#include <QJsonDocument>
#include <QJsonObject>
#include <QProcess>
#include <fstream>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
extern ON_XMLNode& ON_GetRdkDocNode(const ON_3dmRenderSettings&);
static void require(bool value,const char* msg){if(!value)throw std::runtime_error(msg);}
static const ONX_Model_UserData& table(const ONX_Model& model){
    for(int i=0;i<model.m_userdata_table.Count();++i)if(model.m_userdata_table[i]&&ONX_Model::IsRDKDocumentInformation(*model.m_userdata_table[i]))return *model.m_userdata_table[i];
    throw std::runtime_error("Missing RDK table");
}
static int version(const ONX_Model& model){auto& data=table(model);ON_Read3dmBufferArchive archive(data.m_goo.m_value,data.m_goo.m_goo,false,data.m_usertable_3dm_version,data.m_usertable_opennurbs_version);int v=0;require(archive.ReadInt(&v),"RDK version read");return v;}
static QJsonObject legacy(const std::filesystem::path& path,bool expected){
    QProcess child;child.start(QString::fromUtf8(OM9_RDK_LEGACY_READER),{QString::fromStdWString(path.wstring())});
    require(child.waitForFinished(60000),"Legacy reader timeout");
    auto doc=QJsonDocument::fromJson(child.readAllStandardOutput());require(doc.isObject(),"Legacy reader report");
    require(child.exitStatus()==QProcess::NormalExit&&((child.exitCode()==0)==expected),"Independent Rhino5 RDK decoding mismatch");return doc.object();
}
int main(){try{
    ON::Begin();std::filesystem::path dir=OM9_RDK_EVIDENCE_DIR;std::filesystem::create_directories(dir);
    for(int format:{5,50,80}){
        ONX_Model model;ON_Layer layer;layer.SetName(L"RDK α");model.AddModelComponent(layer);
        ON_Point point(1,2,3);ON_3dmObjectAttributes attributes;model.AddModelGeometryComponent(&point,&attributes);
        auto& xmlRoot=ON_GetRdkDocNode(model.m_settings.m_RenderSettings);
        xmlRoot.CreateNodeAtPath(L"settings/om9-test")->SetProperty(ON_XMLProperty(L"unicode",ON_XMLVariant(L"é α 한 💍")));
        auto path=dir/(std::to_string(format)+"-no-resources.3dm");require(model.Write(path.c_str(),format,nullptr),"Write RDK fixture");
        ONX_Model loaded;require(loaded.Read(path.c_str()),"Modern reread");
        ON_wString xml;require(ONX_Model::GetRDKDocumentInformation(table(loaded),xml),"Modern XML read");
        require(version(loaded)==(format<=50?3:4),"Rhino5 archive must contain genuine RDK3 layout when no resources");
        if(format<=50){auto result=legacy(path,true);require(result["rdk_version"]==3,"Legacy RDK3 version");require(result["xml"].toString()==QString::fromWCharArray(xml.Array()),"Independent exact Unicode/XML retention");
            ON_Read3dmBufferArchive a(table(loaded).m_goo.m_value,table(loaded).m_goo.m_goo,false,table(loaded).m_usertable_3dm_version,table(loaded).m_usertable_opennurbs_version);int v,n;require(a.ReadInt(&v)&&a.ReadInt(&n),"RDK3 prefix");require(table(loaded).m_goo.m_value==8+n,"RDK3 must omit the RDK4 embedded-file trailer");}
    }
    // Modern resources keep their RDK4 payload. Never relabel or discard them.
    ONX_Model embedded;ON_Layer layer;layer.SetName(L"Default");embedded.AddModelComponent(layer);
    ON_Buffer buffer;const char payload[]="texture bytes";buffer.Write(sizeof(payload),payload);buffer.SeekFromStart(0);
    ON_EmbeddedFile file;require(file.LoadFromBuffer(buffer),"Load resource fixture");file.SetFilename(L"texture α.bin");require(!embedded.AddModelComponent(file).IsEmpty(),"Add resource");
    ON_Point resourcePoint(1,2,3);ON_3dmObjectAttributes resourceAttributes;
    auto selected=embedded.AddModelGeometryComponent(&resourcePoint,&resourceAttributes);
    auto path=dir/"5-with-resources.3dm";require(embedded.Write(path.c_str(),5,nullptr),"SDK resource retention write");
    ONX_Model read;require(read.Read(path.c_str()),"Resource reread");require(version(read)==4,"Resource payload must remain RDK4");require(read.ActiveComponentCount(ON_ModelComponent::Type::EmbeddedFile)==1,"Embedded resource retained");legacy(path,false);
    auto protectedPath=dir/"protected-target.3dm";{std::ofstream stream(protectedPath,std::ios::binary);stream<<"existing target";}
    bool rejected=false;try{writeModelRhino5(read,protectedPath);}catch(const ExchangeError& error){rejected=std::string(error.what()).find("embedded resources")!=std::string::npos;}
    require(rejected,"Host must refuse embedded resources before writing");
    std::ifstream stream(protectedPath,std::ios::binary);std::string bytes((std::istreambuf_iterator<char>(stream)),{});require(bytes=="existing target","Refusal must retain target bytes");
    auto inventory=inspectArchive(path).document;char selectedId[37]{};ON_UuidToString(selected.ModelComponent()->Id(),selectedId);
    QJsonObject source{{"namespace","12345678-1234-1234-1234-123456789abc"},{"snapshot",QString::fromStdWString(path.wstring())},{"archive_sha256",inventory["archive_sha256"]},{"scale_mm",inventory["scale_mm"]}};
    QJsonObject selection{{"host_id","resource-point"},{"namespace",source["namespace"]},{"source_uuid",selectedId},{"action","unchanged"}};
    QJsonObject request{{"schema_version",1},{"sources",QJsonArray{source}},{"selected",QJsonArray{selection}}};
    for(bool merged:{false,true}){
        auto scoped=request;
        if(merged){auto second=source;second["namespace"]="12345678-1234-1234-1234-123456789abd";auto secondSelection=selection;secondSelection["namespace"]=second["namespace"];secondSelection["host_id"]="second-resource-point";scoped["sources"]=QJsonArray{source,second};scoped["selected"]=QJsonArray{selection,secondSelection};}
        rejected=false;try{writePreservedArchive(scoped,protectedPath);}catch(const ExchangeError& error){rejected=std::string(error.what()).find("embedded resources")!=std::string::npos;}
        require(rejected,"Preservation and namespace merge must refuse resources before dependency pruning");
        std::ifstream retained(protectedPath,std::ios::binary);std::string target((std::istreambuf_iterator<char>(retained)),{});require(target=="existing target","Preservation refusal must retain destination bytes");
        ONX_Model snapshot;require(snapshot.Read(path.c_str())&&snapshot.ActiveComponentCount(ON_ModelComponent::Type::EmbeddedFile)==1,"Refusal must retain source resource");
    }
    ONX_Model plain;plain.AddModelComponent(layer);auto ordinary=dir/"host-plain.3dm";require(writeModelRhino5(plain,ordinary),"Host resource-free RDK export");legacy(ordinary,true);
    std::cout<<"RDK serialization PASS: V5/V50 legacy XML exact, V80 unchanged, embedded payload retained as V4, host resource refusal atomic\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
