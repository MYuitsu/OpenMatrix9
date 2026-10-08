#include "ThreeDmInventory.h"
#include "ThreeDmNativeReferences.h"
#include <QJsonDocument>
#include <QDir>
#include <QFile>
#include <QProcess>
#include <QUuid>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool value,const char* message){if(!value)throw std::runtime_error(message);}
static QJsonValue projection(QJsonValue value){
    if(value.isArray()){QJsonArray out;for(auto c:value.toArray())out.append(projection(c));return out;}
    if(value.isObject()){auto out=value.toObject();for(const auto& key:{"schema_version","object_references","userdata","resolution"})out.remove(key);for(const auto& key:out.keys())out[key]=projection(out[key]);return out;}
    return value;
}
static std::map<QString,QJsonObject> fields(const ArchiveInventory& archive){std::map<QString,QJsonObject> out;for(auto v:archive.document["records"].toArray()){auto row=v.toObject();if(row["class_name"]=="ON_CurveOnSurface")out[row["source_uuid"].toString()]=row["curve_on_surface_native"].toObject();}return out;}
int main(int argc,char** argv){try{require(argc==1||argc==2,"Optional independent reader path");const auto executable=argc==2?QString::fromLocal8Bit(argv[1]):QStringLiteral(OM9_REFERENCE_LEGACY_READER_EXE);QDir base(QStringLiteral(OM9_INTEROPERABILITY_BASE)),evidence(base.filePath("native-reference-interoperability-evidence"));require(QDir().mkpath(evidence.path()),"Evidence directory");QStringList inputs;
    // The reversed path first is a regression for the original2013 Read defect.
    inputs<<"native-reference-resolution-evidence/curve-child-1-reversed-1.3dm";
    for(int child=0;child<2;++child)for(int reverse=0;reverse<2;++reverse){auto p=QString("native-reference-resolution-evidence/curve-child-%1-reversed-%2.3dm").arg(child).arg(reverse);if(!inputs.contains(p))inputs<<p;}
    for(int trim=0;trim<2;++trim)for(int reverse=0;reverse<2;++reverse)inputs<<QString("native-reference-resolution-evidence/brep-trim-%1-reversed-%2.3dm").arg(trim).arg(reverse);
    for(int kind=0;kind<5;++kind)for(int optional=0;optional<2;++optional)inputs<<QString("curve-on-surface-schema-evidence/surface-%1-optional-%2.3dm").arg(kind).arg(optional);
    for(int shape=0;shape<2;++shape)for(int edge=0;edge<2;++edge)for(int trim=0;trim<2;++trim)for(int segment=0;segment<2;++segment)inputs<<QString("trim-domain-evidence/shape-%1-edge-%2-trim-%3-segment-%4-consistent-1.3dm").arg(shape).arg(edge).arg(trim).arg(segment);
    inputs<<"reference-graph-integrity-evidence/shared-owner.3dm"<<"reference-graph-integrity-evidence/nested-owner.3dm";
    QJsonArray cases;int roots=0;
    for(const auto& name:inputs){auto input=base.filePath(name);auto source=inspectArchive(std::filesystem::path(input.toStdWString()),1);auto expected=fields(source);require(!expected.empty(),"Native fixture roots");auto folder=evidence.filePath(QUuid::createUuid().toString(QUuid::WithoutBraces));require(QDir().mkpath(folder),"Fresh oracle folder");auto output=folder+"/reencoded.3dm",reportPath=folder+"/report.json";
        QProcess process;process.start(executable,{input,output,reportPath});bool finished=process.waitForFinished(120000);if(!finished||process.exitStatus()!=QProcess::NormalExit||process.exitCode()!=0)std::cerr<<process.readAllStandardError().toStdString();require(finished&&process.exitStatus()==QProcess::NormalExit&&process.exitCode()==0,"Independent SDK2013 decode/link/reencode must succeed");QFile report(reportPath);require(report.open(QIODevice::ReadOnly),"Read separate legacy report");auto proof=QJsonDocument::fromJson(report.readAll()).object();require(proof["sdk_version"]==201307115&&proof["source_version"]==50,"Independent target version facts");
        auto records=proof["records"].toArray();require(records.size()==expected.size(),"Legacy native root count exact");for(auto v:records){auto row=v.toObject();auto id=row["source_uuid"].toString();require(expected.contains(id),"Legacy retains native root UUID");if(projection(expected.at(id))!=row["fields"]){std::cerr<<name.toStdString()<<'\n';QFile mismatch(folder+"/mismatch.json");require(mismatch.open(QIODevice::WriteOnly),"Mismatch report open");mismatch.write(QJsonDocument(QJsonObject{{"expected",projection(expected.at(id))},{"legacy",row["fields"]}}).toJson());throw std::runtime_error("Independent SDK2013 must retain exact native class/domain/reference reversal fields");}}
        auto reread=inspectArchive(std::filesystem::path(output.toStdWString()),1);require(reread.document["source_version"]==50&&fields(reread)==expected,"Legacy reencode must preserve full modern native schema including userdata matrices");
        require(source.nativeReferences->diagnostics==reread.nativeReferences->diagnostics,"Independent target reencode must preserve native reference/trim analysis");roots+=int(expected.size());cases.append(QJsonObject{{"input",name},{"folder",folder},{"roots",int(expected.size())},{"exact_legacy_fields",true},{"exact_full_reencode_schema",true},{"reference_analysis",true}});
    }
    require(inputs.size()==36,"Full named interoperability corpus");QFile report(evidence.filePath("results.json"));require(report.open(QIODevice::WriteOnly),"Final report");report.write(QJsonDocument(QJsonObject{{"ok",true},{"cases",cases},{"archives",cases.size()},{"roots",roots},{"sdk_version",201307115},{"target_version",50}}).toJson());std::cout<<"Independent SDK2013 native reference corpus PASS36 archives "<<roots<<" roots\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
