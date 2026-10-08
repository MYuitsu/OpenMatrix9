#include "ThreeDmClassRegistry.h"
#include "ThreeDmGeometry.h"
#include "opennurbs_polyedgecurve.h"
#include <QJsonArray>
#include <QRegularExpression>
#include <memory>
#include <set>
namespace OpenMatrix9Gui::ThreeDm {
QJsonObject linkedClassRegistry(){
    ON::Begin();ON_PolyEdgeCurve forceReferenceReader;
    ON_wString text;ON_TextLog log(text);ON_ClassId::Dump(log);
    auto matches=QRegularExpression("([A-Za-z_][A-Za-z0-9_]*)::ClassId:").globalMatch(QString::fromWCharArray(text.Array()));
    std::set<QString> names;QJsonArray rows;
    while(matches.hasNext()){
        auto name=matches.next().captured(1);if(!names.insert(name).second)continue;
        auto id=ON_ClassId::ClassId(name.toLatin1().constData());if(!id)throw ExchangeError("Dumped runtime class cannot be resolved");
        char uuid[37]{};ON_UuidToString(id->Uuid(),uuid);std::unique_ptr<ON_Object> object(id->Create());
        auto byUuid=ON_ClassId::ClassId(id->Uuid());
        rows.append(QJsonObject{{"class_name",name},{"class_uuid",QString::fromLatin1(uuid).toLower()},
            {"base_class",QString::fromLatin1(id->BaseClassName())},{"factory_available",bool(object)},
            {"factory_class",object?QString::fromLatin1(object->ClassId()->ClassName()):QString()},
            {"uuid_lookup_class",byUuid?QString::fromLatin1(byUuid->ClassName()):QString()}});
    }
    if(names.size()<100||!names.contains("ON_Brep")||!names.contains("ON_SubD"))throw ExchangeError("Incomplete linked runtime registry");
    return {{"ok",true},{"classes",rows},{"runtime_count",int(names.size())}};
}
}
