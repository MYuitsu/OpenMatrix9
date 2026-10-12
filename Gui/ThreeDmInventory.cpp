#include "ThreeDmInventory.h"
#include "ThreeDmThreadPool.h"
#include "ThreeDmPointCloud.h"
#include "ThreeDmHatch.h"
#include "ThreeDmCurveOnSurface.h"
#include "ThreeDmNativeReferences.h"
#include "opennurbs_polyedgecurve.h"
#include <QCryptographicHash>
#include <QFile>
#include <QJsonDocument>
#include <QRegularExpression>
#include <cmath>
#include <set>
namespace OpenMatrix9Gui::ThreeDm {
static QString uuid(const ON_UUID& id){char value[37]{};ON_UuidToString(id,value);return QString::fromLatin1(value);}
static QString string(const ON_wString& value){ON_String s(value);return QString::fromUtf8(s.Array());}
QJsonObject nativeGeometryFacts(const ON_Geometry& geometry){
    QJsonObject facts;
    if(auto dot=ON_TextDot::Cast(&geometry)){auto point=dot->CenterPoint();facts["text_dot"]=QJsonObject{{"point",QJsonArray{point.x,point.y,point.z}},{"primary_text",string(ON_wString(dot->PrimaryText()))},{"secondary_text",string(ON_wString(dot->SecondaryText()))},{"font_face",string(ON_wString(dot->FontFace()))},{"height_in_points",dot->HeightInPoints()},{"bold",dot->Bold()},{"italic",dot->Italic()},{"always_on_top",dot->AlwaysOnTop()},{"transparent",dot->Transparent()}};}
    if(auto cloud=ON_PointCloud::Cast(&geometry))facts["point_cloud"]=pointCloudSummary(*cloud);
    if(auto hatch=ON_Hatch::Cast(&geometry)){facts["hatch"]=hatchFacts(*hatch);facts["hatch_loop_native"]=hatchLoopInventory(*hatch);}
    if(geometry.ClassId()==&ON_CLASS_RTTI(ON_CurveOnSurface))facts["curve_on_surface_native"]=curveOnSurfaceNativeFields(static_cast<const ON_CurveOnSurface&>(geometry));
    if(auto curve=ON_Curve::Cast(&geometry);curve&&hasPolyEdgeReference(*curve))facts["poly_edge_native"]=nativeCurveTreeFields(*curve);
    return facts;
}
ArchiveInventory inspectArchive(const std::filesystem::path& path,double custom){
    std::lock_guard sdkLock(sdkArchiveMutex());
    if(std::filesystem::file_size(path)>512ULL*1024*1024)throw ExchangeError("3DM archive exceeds the 512 MiB import limit");
    ON::Begin();auto nativeModel=std::make_shared<ONX_Model>();auto& model=*nativeModel;
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
            if(auto hatch=ON_Hatch::Cast(object))indexedDependency(ON_ModelComponent::Type::HatchPattern,hatch->PatternIndex());
            if(auto layer=ON_Layer::Cast(c)){if(layer->ParentId()!=ON_nil_uuid)dependencies.append(uuid(layer->ParentId()));indexedDependency(ON_ModelComponent::Type::Material,layer->RenderMaterialIndex());indexedDependency(ON_ModelComponent::Type::LinePattern,layer->LinetypeIndex());}
            if(auto style=ON_DimStyle::Cast(c))if(style->ParentId()!=ON_nil_uuid)dependencies.append(uuid(style->ParentId()));
            ON_ClassArray<ON_UserString> userStrings;object->GetUserStrings(userStrings);
            for(int i=0;i<userStrings.Count();++i)strings.append(QJsonObject{{"key",string(userStrings[i].m_key)},{"value",string(userStrings[i].m_string_value)}});
            for(auto data=object->FirstUserData();data;data=data->Next())userdata.append(QJsonObject{{"class_uuid",uuid(data->ClassId()->Uuid())},{"class_name",data->ClassId()->ClassName()},{"capability","retained"}});
            auto nativeCurve=ON_Curve::Cast(object);const bool referenceCurve=nativeCurve&&hasPolyEdgeReference(*nativeCurve);
            bool editable=!referenceCurve&&!ON_CurveOnSurface::Cast(object)&&(ON_Point::Cast(object)||ON_Curve::Cast(object)||ON_Brep::Cast(object)||ON_Surface::Cast(object)||ON_Mesh::Cast(object));
            QJsonObject row{{"source_uuid",uuid(c->Id())},{"class_uuid",uuid(object->ClassId()->Uuid())},{"class_name",object->ClassId()->ClassName()},{"component_type",string(ON_ModelComponent::ComponentTypeToString(kind))},{"name",string(c->Name())},{"role",members.contains(uuid(c->Id()))?"definition-member":"top-level"},{"dependencies",dependencies},{"capability",editable?"editable":"retained"},{"user_strings",strings}};
            identities.insert(uuid(c->Id()));
            if(geometry)if(auto attributes=geometry->Attributes(nullptr))row["wire_density"]=attributes->m_wire_density;
            // Structural host binding needs the exact native affine matrix.
            if(auto instance=ON_InstanceRef::Cast(object)){QJsonArray matrix;for(int i=0;i<16;++i)matrix.append(instance->m_xform[i/4][i%4]);row["instance_matrix"]=matrix;row["instance_definition_uuid"]=uuid(instance->m_instance_definition_uuid);}
            if(auto definition=ON_InstanceDefinition::Cast(c)){QJsonArray ids;const auto& members=definition->InstanceGeometryIdList();for(int i=0;i<members.Count();++i)ids.append(uuid(members[i]));row["member_uuids"]=ids;row["definition_description"]=string(definition->Description());row["definition_url"]=string(definition->URL());row["definition_url_tag"]=string(definition->URL_Tag());}
            row["userdata"]=userdata;
            if(geometry)row["geometry_crc"]=QString::number(object->DataCRC(0));
            if(auto native=ON_Geometry::Cast(object)){
                auto facts=nativeGeometryFacts(*native);for(auto key:facts.keys())row[key]=facts[key];
                for(auto dependency:facts["curve_on_surface_native"].toObject()["object_references"].toArray())if(!dependencies.contains(dependency))dependencies.append(dependency);
                for(auto dependency:facts["poly_edge_native"].toObject()["object_references"].toArray())if(!dependencies.contains(dependency))dependencies.append(dependency);
                row["dependencies"]=dependencies;
            }
            if(auto hatch=ON_Hatch::Cast(object)){
                try{auto current=*hatch;if(scale!=1)transformHatchNative(current,ON_Xform::DiagonalTransformation(scale),nullptr);row["hatch_current"]=hatchCurrentFields(current,model);row["hatch_loop_current"]=hatchLoopFields(current);}
                catch(const ExchangeError& error){row["hatch_current_unavailable"]=error.what();}
            }
            row["attribute_user_strings"]=attributeStrings;row["attribute_userdata"]=attributeData;
            if(auto pattern=ON_HatchPattern::Cast(c))row["hatch_pattern"]=hatchPatternFacts(*pattern);
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
    result.document["hatch_pattern_choices"]=hatchPatternChoices(model);
    result.nativeReferences=resolveNativeReferences(nativeModel,records);
    for(int i=0;i<records.size();++i){
        auto row=records[i].toObject();auto key=row["source_uuid"].toString();
        if(result.nativeReferences->diagnostics.contains(key)){
            auto analysis=result.nativeReferences->diagnostics[key].toObject();row["native_reference_analysis"]=analysis;
            if(analysis["status"]=="invalid")issues.append(QJsonObject{{"code","invalid_native_reference"},{"severity","error"},{"source_uuid",key},{"class_name",row["class_name"]},{"message",analysis["message"]}});
            if(analysis["status"]=="limit")issues.append(QJsonObject{{"code","native_reference_limit"},{"severity","error"},{"source_uuid",key},{"class_name",row["class_name"]},{"message",analysis["message"]}});
        }records[i]=row;
    }
    result.document["records"]=records;
    result.document["issues"]=issues;
    result.nativeModel=std::move(nativeModel);
    if(inventoryJson(result).size()>32ULL*1024*1024)throw ExchangeError("3DM inventory exceeds the 32 MiB manifest limit");
    return result;
}
std::string inventoryJson(const ArchiveInventory& inventory){return QJsonDocument(inventory.document).toJson(QJsonDocument::Compact).toStdString();}
}
