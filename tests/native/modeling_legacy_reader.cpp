// Independent unchanged SDK20130711 library: geometry V5 decode/reencode oracle.
#include <opennurbs.h>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QFile>
#include <filesystem>
#include <iostream>
static void require(bool value,const char* message){if(!value)throw std::runtime_error(message);}
int main(int argc,char** argv){try{
    require(argc==4,"usage: ModelingLegacyReader input output report");ON::Begin();require(ON::Version()==201307115,"independent Rhino5-era SDK version");
    auto input=std::filesystem::u8path(argv[1]),output=std::filesystem::u8path(argv[2]);require(!std::filesystem::exists(output),"fresh output required");
    ONX_Model model;require(model.Read(input.c_str(),nullptr)&&model.m_3dm_file_version==50,"legacy V5 read");QJsonArray rows;int roots=0,definitionObjects=0;
    for(int i=0;i<model.m_object_table.Count();++i){auto g=ON_Geometry::Cast(model.m_object_table[i].m_object);require(g&&g->IsValid(),"decoded geometry valid");ON_BoundingBox box;require(g->GetBoundingBox(box),"decoded finite bounds");
        QJsonObject row{{"class",g->ClassId()->ClassName()},{"bounds",QJsonArray{box.m_min.x,box.m_min.y,box.m_min.z,box.m_max.x,box.m_max.y,box.m_max.z}}};
        const bool member=model.m_object_table[i].m_attributes.IsInstanceDefinitionObject();row["definition_member"]=member;if(member)++definitionObjects;else ++roots;
        if(auto instance=ON_InstanceRef::Cast(g)){
            const ON_InstanceDefinition* definition=nullptr;
            for(int d=0;d<model.m_idef_table.Count();++d)if(ON_UuidCompare(&model.m_idef_table[d].m_uuid,&instance->m_instance_definition_uuid)==0)definition=&model.m_idef_table[d];
            require(definition&&definition->IsValid()&&definition->m_object_uuid.Count()>0,"legacy instance definition resolves");
            for(int m=0;m<definition->m_object_uuid.Count();++m){bool found=false;for(int o=0;o<model.m_object_table.Count();++o)if(ON_UuidCompare(&model.m_object_table[o].m_attributes.m_uuid,&definition->m_object_uuid[m])==0&&model.m_object_table[o].m_attributes.IsInstanceDefinitionObject())found=true;require(found,"legacy instance member resolves");}
        }
        if(auto cloud=ON_PointCloud::Cast(g)){QJsonArray points;for(int j=0;j<cloud->m_P.Count();++j){auto p=cloud->m_P[j];points.append(QJsonArray{p.x,p.y,p.z});}row["points"]=points;}
        if(auto mesh=ON_Mesh::Cast(g))row["faces"]=mesh->m_F.Count();
        rows.append(row);
    }
    require(!rows.isEmpty()&&model.Write(output.c_str(),5,"OM9 independent geometry acceptance",nullptr),"legacy V5 reencode");
    QFile report(QString::fromUtf8(argv[3]));require(report.open(QIODevice::WriteOnly|QIODevice::NewOnly),"fresh report open");auto bytes=QJsonDocument(QJsonObject{{"sdk_version",201307115},{"source_version",50},{"root_records",roots},{"definition_records",definitionObjects},{"records",rows}}).toJson();require(report.write(bytes)==bytes.size(),"report write");
    std::cout<<"Independent SDK20130711 geometry decode/reencode PASS: "<<rows.size()<<" objects\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
