#include "ThreeDmInventory.h"
#include <QCryptographicHash>
#include <QFile>
#include <QJsonDocument>
#include <QRegularExpression>
#include <cmath>
#include <set>
namespace OpenMatrix9Gui::ThreeDm {
static QString uuid(const ON_UUID& id){char value[37]{};ON_UuidToString(id,value);return QString::fromLatin1(value);}
static QString string(const ON_wString& value){ON_String s(value);return QString::fromUtf8(s.Array());}
ArchiveInventory inspectArchive(const std::filesystem::path& path,double custom){
    if(std::filesystem::file_size(path)>512ULL*1024*1024)throw ExchangeError("3DM archive exceeds the 512 MiB import limit");
    ON::Begin(); ONX_Model model;
    if(!model.Read(path.c_str(),nullptr))throw ExchangeError("Cannot read 3DM archive");
    auto units=model.m_settings.m_ModelUnitsAndTolerances.m_unit_system.UnitSystem();
    double scale=(units==ON::LengthUnitSystem::None||units==ON::LengthUnitSystem::CustomUnits)?custom:ON::UnitScale(units,ON::LengthUnitSystem::Millimeters);
    if(!std::isfinite(scale)||scale<=0)throw ExchangeError("3DM file has unitless/custom units; specify millimeters per file unit");
    QFile file(QString::fromStdWString(path.wstring()));
    if(!file.open(QIODevice::ReadOnly))throw ExchangeError("Cannot hash source archive");
    QCryptographicHash hash(QCryptographicHash::Sha256);
    if(!hash.addData(&file))throw ExchangeError("Cannot hash source archive");
    QJsonArray records,components,resources,issues;
    std::set<QString> members,identities;
    ONX_ModelComponentIterator definitions(model,ON_ModelComponent::Type::InstanceDefinition);
    for(auto c=definitions.FirstComponent();c;c=definitions.NextComponent())if(auto d=ON_InstanceDefinition::Cast(c)){const auto& ids=d->InstanceGeometryIdList();for(int i=0;i<ids.Count();++i)members.insert(uuid(ids[i]));}
    for(unsigned type=1;type<static_cast<unsigned>(ON_ModelComponent::Type::NumOf);++type){
        if(type==16)continue;
        auto kind=static_cast<ON_ModelComponent::Type>(type);
        ONX_ModelComponentIterator iterator(model,kind);
        for(auto c=iterator.FirstComponent();c;c=iterator.NextComponent()){
            const ON_Object* object=c; auto geometry=ON_ModelGeometryComponent::Cast(c);
            if(geometry)object=geometry->Geometry(nullptr);
            if(!object){issues.append(QJsonObject{{"reason","missing geometry"},{"source_uuid",uuid(c->Id())}});continue;}
            QJsonArray dependencies,strings,userdata;
            auto indexedDependency=[&](ON_ModelComponent::Type target,int index){if(index<0)return;auto reference=model.ComponentFromIndex(target,index);if(auto dependency=reference.ModelComponent())dependencies.append(uuid(dependency->Id()));else issues.append(QJsonObject{{"code","missing_component"},{"severity","error"},{"source_uuid",uuid(c->Id())},{"class_name",object->ClassId()->ClassName()},{"message","Missing indexed component dependency"}});};
            QJsonArray attributeStrings,attributeData;
            if(geometry)if(auto attributes=geometry->Attributes(nullptr)){
                ON_ClassArray<ON_UserString> values;attributes->GetUserStrings(values);for(int i=0;i<values.Count();++i)attributeStrings.append(QJsonObject{{"key",string(values[i].m_key)},{"value",string(values[i].m_string_value)}});
                for(auto data=attributes->FirstUserData();data;data=data->Next())attributeData.append(QJsonObject{{"class_uuid",uuid(data->ClassId()->Uuid())},{"class_name",data->ClassId()->ClassName()},{"capability","retained"}});
                indexedDependency(ON_ModelComponent::Type::Layer,attributes->m_layer_index);indexedDependency(ON_ModelComponent::Type::Material,attributes->m_material_index);indexedDependency(ON_ModelComponent::Type::LinePattern,attributes->m_linetype_index);
                for(int i=0;i<attributes->GroupCount();++i)indexedDependency(ON_ModelComponent::Type::Group,attributes->GroupList()[i]);
            }
            if(auto ref=ON_InstanceRef::Cast(object))dependencies.append(uuid(ref->m_instance_definition_uuid));
            if(auto def=ON_InstanceDefinition::Cast(c)){const auto& ids=def->InstanceGeometryIdList();for(int i=0;i<ids.Count();++i)dependencies.append(uuid(ids[i]));}
            if(auto annotation=ON_Annotation::Cast(object)){auto id=annotation->DimensionStyleId();if(id!=ON_nil_uuid)dependencies.append(uuid(id));}
            if(auto layer=ON_Layer::Cast(c)){if(layer->ParentId()!=ON_nil_uuid)dependencies.append(uuid(layer->ParentId()));indexedDependency(ON_ModelComponent::Type::Material,layer->RenderMaterialIndex());}
            ON_ClassArray<ON_UserString> userStrings;object->GetUserStrings(userStrings);
            for(int i=0;i<userStrings.Count();++i)strings.append(QJsonObject{{"key",string(userStrings[i].m_key)},{"value",string(userStrings[i].m_string_value)}});
            for(auto data=object->FirstUserData();data;data=data->Next())userdata.append(QJsonObject{{"class_uuid",uuid(data->ClassId()->Uuid())},{"class_name",data->ClassId()->ClassName()},{"capability","retained"}});
            bool editable=ON_Point::Cast(object)||ON_Curve::Cast(object)||ON_Brep::Cast(object)||ON_Surface::Cast(object)||ON_Mesh::Cast(object);
            QJsonObject row{{"source_uuid",uuid(c->Id())},{"class_uuid",uuid(object->ClassId()->Uuid())},{"class_name",object->ClassId()->ClassName()},{"component_type",string(ON_ModelComponent::ComponentTypeToString(kind))},{"name",string(c->Name())},{"role",members.contains(uuid(c->Id()))?"definition-member":"top-level"},{"dependencies",dependencies},{"capability",editable?"editable":"retained"},{"user_strings",strings}};
            identities.insert(uuid(c->Id()));
            row["userdata"]=userdata;
            row["attribute_user_strings"]=attributeStrings;row["attribute_userdata"]=attributeData;
            if(auto material=ON_Material::Cast(c))for(int i=0;i<material->m_textures.Count();++i){auto& texture=material->m_textures[i];resources.append(QJsonObject{{"owner_uuid",uuid(c->Id())},{"kind","texture_reference"},{"full_path",string(texture.m_image_file_reference.FullPath())},{"relative_path",string(texture.m_image_file_reference.RelativePath())},{"resolution","not opened"}});}
            if(auto definition=ON_InstanceDefinition::Cast(c)){auto reference=definition->LinkedFileReference();if(!reference.FullPath().IsEmpty()||!reference.RelativePath().IsEmpty())resources.append(QJsonObject{{"owner_uuid",uuid(c->Id())},{"kind","linked_block_reference"},{"full_path",string(reference.FullPath())},{"relative_path",string(reference.RelativePath())},{"resolution","not opened"}});}
            if(geometry)records.append(row);else components.append(row);
            if(kind==ON_ModelComponent::Type::Image||kind==ON_ModelComponent::Type::EmbeddedFile)resources.append(row);
        }
    }
    for(auto collection:{records,components})for(auto value:collection)for(auto dependency:value.toObject()["dependencies"].toArray())if(!identities.contains(dependency.toString()))issues.append(QJsonObject{{"code","missing_dependency"},{"severity","error"},{"message","Missing source dependency"},{"class_name",value.toObject()["class_name"]},{"source_uuid",value.toObject()["source_uuid"]},{"dependency",dependency}});
    QJsonArray userTables;for(int i=0;i<model.m_userdata_table.Count();++i)if(auto table=model.m_userdata_table[i])userTables.append(QJsonObject{{"source_uuid",uuid(table->m_uuid)},{"source_version",static_cast<int>(table->m_usertable_3dm_version)},{"capability","retained"}});
    ON_wString propertiesReport,settingsReport;ON_TextLog propertiesLog(propertiesReport),settingsLog(settingsReport);model.m_properties.Dump(propertiesLog);model.m_settings.Dump(settingsLog);
    ArchiveInventory result{QJsonObject{{"schema_version",1},{"source_version",model.m_3dm_file_version},{"source_units",static_cast<int>(units)},{"scale_mm",scale},{"archive_sha256",QString::fromLatin1(hash.result().toHex())},{"records",records},{"components",components},{"settings",QJsonObject{{"views",model.m_settings.m_views.Count()},{"named_views",model.m_settings.m_named_views.Count()},{"plugin_userdata",userTables},{"properties_capability","retained in source snapshot"}}},{"resources",resources},{"issues",issues}}};
    if(inventoryJson(result).size()>32ULL*1024*1024)throw ExchangeError("3DM inventory exceeds the 32 MiB manifest limit");
    // Dump includes process-local serial/content counters; these are not source archive identities.
    auto stableReport=[](const ON_wString& report){auto text=string(report);text.replace(QRegularExpression("Model component [0-9]+"),"Model component <runtime>");text.replace(QRegularExpression("Content version number = [0-9]+"),"Content version number = <runtime>");return text;};
    auto settings=result.document["settings"].toObject();settings["properties_report"]=stableReport(propertiesReport);settings["settings_report"]=stableReport(settingsReport);result.document["settings"]=settings;
    if(inventoryJson(result).size()>32ULL*1024*1024)throw ExchangeError("3DM inventory exceeds the 32 MiB manifest limit");
    return result;
}
std::string inventoryJson(const ArchiveInventory& inventory){return QJsonDocument(inventory.document).toJson(QJsonDocument::Compact).toStdString();}
}
