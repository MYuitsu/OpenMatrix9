#include <opennurbs.h>
#include <QCoreApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <iostream>
int main(int argc,char** argv){
    QCoreApplication app(argc,argv);ON::Begin();
    if(argc!=2)return 2;
    ONX_Model model;
    if(!model.Read(QString::fromUtf8(argv[1]).toStdWString().c_str()))return 3;
    int found=0;QJsonObject result;
    for(int i=0;i<model.m_userdata_table.Count();++i){auto table=&model.m_userdata_table[i];
        if(!table||!ONX_Model::IsRDKDocumentInformation(*table))continue;
        ++found;ON_wString xml;
        ON_Read3dmBufferArchive archive(table->m_goo.m_value,table->m_goo.m_goo,false,table->m_usertable_3dm_version,table->m_usertable_opennurbs_version);
        int version=0;if(!archive.ReadInt(&version))return 4;
        bool decoded=ONX_Model::GetRDKDocumentInformation(*table,xml);
        result=QJsonObject{{"rdk_version",version},{"decoded",decoded},{"xml",QString::fromWCharArray(xml.Array())},
            {"table_bytes",table->m_goo.m_value}};
    }
    result["table_count"]=found;
    std::cout<<QJsonDocument(result).toJson(QJsonDocument::Compact).constData()<<'\n';
    return found==1&&result["decoded"].toBool()?0:1;
}
