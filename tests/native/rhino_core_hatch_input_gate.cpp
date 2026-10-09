// SPDX-License-Identifier: LGPL-2.1-or-later
#include "ThreeDmHatch.h"
#include "ThreeDmMerge.h"
#include <QJsonArray>
#include <QJsonDocument>
#include <QTemporaryDir>
#include <QFile>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
namespace {
QJsonArray checks,cases;
void check(bool v,const char* name){checks.append(QJsonObject{{"name",name},{"passed",v}});if(!v)throw ExchangeError(name);}
template<class F>void reject(F&& f,const char* name){QString error;try{f();}catch(const std::exception& e){error=QString::fromUtf8(e.what());}cases.append(QJsonObject{{"name",name},{"error",error}});check(error.contains("Invalid Hatch loop basis input"),name);}
QByteArray bytes(const std::filesystem::path& p){QFile f(QString::fromStdWString(p.wstring()));check(f.open(QIODevice::ReadOnly),"sentinel opened");return f.readAll();}
void run(const std::filesystem::path& directory){
 QTemporaryDir temp;check(temp.isValid(),"isolated native fixture directory");auto dir=std::filesystem::path(temp.path().toStdWString());auto missing=dir/"absent.3dm",target=dir/"existing.3dm";
 QFile sentinel(QString::fromStdWString(target.wstring()));check(sentinel.open(QIODevice::WriteOnly),"existing target created");sentinel.write("unchanged-native-sentinel");sentinel.close();auto original=bytes(target);
 QJsonObject curve{{"class_name","ON_NurbsCurve"},{"dimension",2},{"source_index",QJsonValue::Null},{"domain",QJsonArray{0,4}},{"order",2},{"rational",false},{"cvs",QJsonArray{QJsonArray{0,0},QJsonArray{10,0},QJsonArray{10,10},QJsonArray{0,10},QJsonArray{0,0}}},{"knots",QJsonArray{0,1,2,3,4}}};
 auto payload=[&](const QJsonObject& c){return QJsonObject{{"schema_version",1},{"loops",QJsonArray{QJsonObject{{"type",0},{"source_index",QJsonValue::Null},{"curve",c}}}}};};auto good=payload(curve);preflightHatchLoopFields(good);check(true,"valid native representation passes portable preflight");
 auto badCurve=curve;badCurve["knots"]=QJsonArray{0,1,2,3};auto bad=payload(badCurve);ON_Hatch invalidSource;
 reject([&]{applyHatchLoopFields(invalidSource,bad);},"bad basis before native source validation");check(invalidSource.LoopCount()==0,"invalid source remains untouched");
 const QString ns="12345678-1234-1234-1234-123456789abc",uuid="abcdef12-1234-1234-1234-123456789abc";
 QJsonObject source{{"namespace",ns},{"snapshot",QString::fromStdWString(missing.wstring())},{"archive_sha256",QString(64,'0')},{"scale_mm",1.}};
 QJsonObject row{{"namespace",ns},{"source_uuid",uuid},{"host_id","Hatch"},{"action","transform"},{"hatch_fields",QJsonObject{{"loops",bad}}}};
 auto request=[&](QJsonObject edit){return QJsonObject{{"schema_version",1},{"sources",QJsonArray{source}},{"selected",QJsonArray{edit}}};};
 reject([&]{writePreservedArchive(request(row),target);},"selected bad basis before archive read");check(bytes(target)==original,"selected rejection preserves target bytes");
 auto duplicate=row;duplicate["action"]="duplicate";duplicate["duplicate_action"]="transform";
 reject([&]{writePreservedArchive(request(duplicate),target);},"duplicate bad basis before archive read");check(bytes(target)==original,"duplicate rejection preserves target bytes");
 auto unchanged=row;unchanged.remove("hatch_fields");unchanged["action"]="unchanged";auto dependency=request(unchanged);dependency["dependency_overlays"]=QJsonArray{row};
 reject([&]{writePreservedArchive(dependency,target);},"dependency bad basis before archive read");check(bytes(target)==original,"dependency rejection preserves target bytes");
 auto independent=request(unchanged);independent["member_copies"]=QJsonArray{QJsonObject{{"independent_overlay",row}}};
 reject([&]{writePreservedArchive(independent,target);},"independent copy bad basis before archive read or clone");check(bytes(target)==original,"independent rejection preserves target bytes");
 auto valid=row;valid["hatch_fields"]=QJsonObject{{"loops",good}};auto later=request(valid);later["selected"]=QJsonArray{valid,row};
 reject([&]{writePreservedArchive(later,target);},"later bad basis rejects before earlier native reconstruction");check(bytes(target)==original,"later-row rejection preserves target bytes");
 auto multi=request(row);auto source2=source;source2["namespace"]="12345678-1234-1234-1234-123456789abd";multi["sources"]=QJsonArray{source,source2};
 reject([&]{writePreservedArchive(multi,target);},"multiple-source bad basis before native merge");check(bytes(target)==original,"multiple-source rejection preserves target bytes");
 QString validError;try{writePreservedArchive(request(valid),target);}catch(const std::exception& e){validError=QString::fromUtf8(e.what());}cases.append(QJsonObject{{"name","valid basis source access"},{"error",validError}});check(!validError.isEmpty()&&!validError.contains("Invalid Hatch loop basis input"),"valid basis reaches absent source error");
 check(!std::filesystem::exists(missing)&&bytes(target)==original,"source absent and destination intact after all native calls");
}
}
int main(int argc,char** argv){ON::Begin();QString error;auto path=argc>1?std::filesystem::path(argv[1]):std::filesystem::temp_directory_path()/"hatch-input-results.json";try{run(path.parent_path());}catch(const Standard_Failure& e){error=e.GetMessageString();}catch(const std::exception& e){error=e.what();}QFile f(QString::fromStdWString(path.wstring()));if(!f.open(QIODevice::WriteOnly))return 2;f.write(QJsonDocument(QJsonObject{{"checks",checks},{"cases",cases},{"error",error}}).toJson());std::cout<<checks.size()<<" checks; "<<(error.isEmpty()?"PASS":error.toStdString())<<"\n";return error.isEmpty()?0:1;}
