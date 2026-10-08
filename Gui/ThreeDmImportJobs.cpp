#include "ThreeDmStaging.h"
#include <QCoreApplication>
#include <QProcess>
#include <QFile>
#include <QJsonDocument>
#include <QElapsedTimer>
#include <QThread>
#include <memory>
#include <map>
namespace OpenMatrix9Gui::ThreeDm {
QJsonArray stagePreservedArchive(const std::filesystem::path& source,const std::filesystem::path& staging,double scale,bool members,const ONX_Model* native){
    // SDK models remain serial in the host. Workers use separate address spaces:
    // openNURBS SubD serial numbers and diagnostic counters are process globals.
    std::set<std::string> memberIds;
    ONX_ModelComponentIterator definitions(*native,ON_ModelComponent::Type::InstanceDefinition);
    for(auto c=definitions.FirstComponent();c;c=definitions.NextComponent())if(auto d=ON_InstanceDefinition::Cast(c)){const auto& ids=d->InstanceGeometryIdList();for(int i=0;i<ids.Count();++i){char text[37]{};ON_UuidToString(ids[i],text);memberIds.insert(text);}}
    std::vector<std::string> roots;
    for(auto type:{ON_ModelComponent::Type::ModelGeometry,ON_ModelComponent::Type::RenderLight}){
        ONX_ModelComponentIterator objects(*native,type);
        for(auto c=objects.FirstComponent();c;c=objects.NextComponent())if(auto object=ON_ModelGeometryComponent::Cast(c)){
            char text[37]{};ON_UuidToString(object->Id(),text);auto a=object->Attributes(nullptr);if(!a)throw ExchangeError("Missing object attributes");
            if((a->IsInstanceDefinitionObject()||memberIds.contains(text))==members)roots.emplace_back(text);
        }
    }
    if(roots.size()>1000000)throw ExchangeError("3DM object limit exceeded");
    int workers=std::clamp(QThread::idealThreadCount(),1,4);
    bool valid=false;int requested=qEnvironmentVariableIntValue("OM9_3DM_WORKERS",&valid);if(valid&&requested>=1&&requested<=4)workers=std::min(workers,requested);
    if(roots.size()<8||std::filesystem::file_size(source)>16ULL*1024*1024)workers=1;
    auto base=staging/(members?"definitions":"geometry");std::filesystem::create_directory(base);
    if(workers==1){auto model=readArchive(source,scale,true,members);QJsonArray rows;int index=0;for(auto& item:model.items){auto row=encodeExchangeItem(item,base,index++);row["tolerance"]=model.tolerance;rows.append(row);}return rows;}
    QString executable=QCoreApplication::applicationDirPath()+"/OM9ThreeDmImportWorker";
#ifdef _WIN32
    executable+=".exe";
#endif
    if(!QFile::exists(executable))throw ExchangeError("3DM import worker is missing; reinstall OM9 or set OM9_3DM_WORKERS=1");
    struct Jobs {std::vector<std::unique_ptr<QProcess>> processes;~Jobs(){for(auto& p:processes)if(p->state()!=QProcess::NotRunning){p->kill();p->waitForFinished(-1);}}} jobs;
    QElapsedTimer elapsed;elapsed.start();std::vector<QString> outputs;
    for(int w=0;w<workers;++w){
        auto dir=base/std::to_string(w);std::filesystem::create_directory(dir);QJsonArray ids;for(size_t i=w;i<roots.size();i+=workers)ids.append(QString::fromStdString(roots[i]));
        auto request=QString::fromStdWString((dir/"request.json").wstring()),result=QString::fromStdWString((dir/"result.json").wstring());
        auto bytes=QJsonDocument(QJsonObject{{"source",QString::fromStdWString(source.wstring())},{"staging",QString::fromStdWString(dir.wstring())},{"scale",scale},{"members",members},{"ids",ids}}).toJson(QJsonDocument::Compact);
        QFile file(request);if(!file.open(QIODevice::WriteOnly|QIODevice::NewOnly)||file.write(bytes)!=bytes.size())throw ExchangeError("Cannot stage worker request");file.close();
        auto process=std::make_unique<QProcess>();process->setProcessChannelMode(QProcess::SeparateChannels);process->setProgram(executable);process->setArguments({request,result});process->start();jobs.processes.push_back(std::move(process));
        if(!jobs.processes.back()->waitForStarted(10000))throw ExchangeError("Cannot start 3DM import worker");outputs.push_back(result);
    }
    std::map<std::string,QJsonObject> converted;
    for(int w=0;w<workers;++w){auto& p=jobs.processes[w];auto remaining=600000-elapsed.elapsed();
        if(remaining<=0||(p->state()!=QProcess::NotRunning&&!p->waitForFinished(static_cast<int>(remaining))))throw ExchangeError("3DM conversion worker timed out");
        if(p->exitStatus()!=QProcess::NormalExit||p->exitCode()!=0)throw ExchangeError("3DM conversion worker failed: "+p->readAllStandardError().left(4096).toStdString());
        QFile file(outputs[w]);if(!file.open(QIODevice::ReadOnly)||file.size()>128LL*1024*1024)throw ExchangeError("Invalid worker result size");QJsonParseError error;auto json=QJsonDocument::fromJson(file.readAll(),&error);
        if(error.error!=QJsonParseError::NoError||!json.isObject()||!json.object()["rows"].isArray())throw ExchangeError("Invalid worker result JSON");
        for(auto value:json.object()["rows"].toArray()){auto row=value.toObject();auto id=row["source_uuid"].toString().toStdString();if(!converted.emplace(id,row).second)throw ExchangeError("Duplicate worker source geometry");}
    }
    if(converted.size()!=roots.size())throw ExchangeError("Worker source geometry count changed");
    QJsonArray rows;for(auto& root:roots){auto item=converted.find(root);if(item==converted.end())throw ExchangeError("Missing worker source geometry");rows.append(item->second);}return rows;
}
}
