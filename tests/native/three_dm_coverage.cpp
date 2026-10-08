#include "ThreeDmInventory.h"
#include <QFile>
#include <QJsonDocument>
#include <QRegularExpression>
#include <QCryptographicHash>
#include <iostream>
#include <set>
#include "opennurbs_polyedgecurve.h"
using namespace OpenMatrix9Gui::ThreeDm;
int main(int argc,char** argv){try{
    ON::Begin();ON_PolyEdgeCurve polyEdge; // force the application-owned reference reader translation unit
    ON_wString text;ON_TextLog log(text);ON_ClassId::Dump(log);
    const auto dump=QString::fromWCharArray(text.Array());
    const QRegularExpression pattern(QStringLiteral("([A-Za-z_][A-Za-z0-9_]*)::ClassId:"));
    auto matches=pattern.globalMatch(dump);std::set<QString> names;QJsonArray rows;
    while(matches.hasNext()){
        const auto name=matches.next().captured(1);if(!names.insert(name).second)continue;
        const auto id=ON_ClassId::ClassId(name.toLatin1().constData());
        if(!id)throw ExchangeError("Dumped runtime class cannot be resolved");
        char uuid[37]{};ON_UuidToString(id->Uuid(),uuid);
        std::unique_ptr<ON_Object> object(id->Create());
        rows.append(QJsonObject{{"class_name",name},{"class_uuid",QString::fromLatin1(uuid).toLower()},
            {"base_class",QString::fromLatin1(id->BaseClassName())},{"factory_available",bool(object)},
            {"factory_class",object?QString::fromLatin1(object->ClassId()->ClassName()):QString()},
            {"uuid_lookup_class",ON_ClassId::ClassId(id->Uuid())?QString::fromLatin1(ON_ClassId::ClassId(id->Uuid())->ClassName()):QString()}});
    }
    if(names.size()<100||!names.contains("ON_Brep")||!names.contains("ON_SubD"))throw ExchangeError("Incomplete linked runtime registry");
    QFile binary(QString::fromLocal8Bit(argv[0]));if(!binary.open(QIODevice::ReadOnly))throw ExchangeError("Probe binary hash");
    const auto digest=QCryptographicHash::hash(binary.readAll(),QCryptographicHash::Sha256).toHex();
    QFile output(QStringLiteral(OM9_COVERAGE_RUNTIME));if(!output.open(QIODevice::WriteOnly))throw ExchangeError("Coverage runtime report");
    const auto bytes=QJsonDocument(QJsonObject{{"ok",true},{"scope","linked standalone OM9ThreeDm runtime; not installed FreeCAD module evidence"},
        {"probe_sha256",QString::fromLatin1(digest)},{"classes",rows},{"runtime_count",int(names.size())}}).toJson();
    if(output.write(bytes)!=bytes.size())throw ExchangeError("Coverage report incomplete write");
    std::cout<<"Coverage runtime PASS: "<<names.size()<<" registered classes\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
