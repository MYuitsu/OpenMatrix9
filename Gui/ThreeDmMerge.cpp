#include "ThreeDmMerge.h"
#include "ThreeDmArchive.h"
#include "ThreeDmPointCloud.h"
#include "ThreeDmHatch.h"
#include "ThreeDmNativeReferences.h"
#include "ThreeDmCurveOnSurface.h"
#include "RustBridge.h"
#include <QFile>
#include <QJsonDocument>
#include <QSaveFile>
#include <QUuid>
#include <QXmlStreamReader>
#include <QRegularExpression>
#include <QCryptographicHash>
#include <QTemporaryDir>
#include <map>
#include <set>
#include <cmath>
#include <functional>
#include <BRepTools.hxx>
#include <BRep_Builder.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <BRepTools_WireExplorer.hxx>
namespace OpenMatrix9Gui::ThreeDm {
enum class ArchivePurpose { Rhino5Preservation, GeometryStaging };
static void writeSelectedArchive(const QJsonObject&,const std::filesystem::path&,ArchivePurpose);
static bool writeSelectedModel(ONX_Model&,const std::filesystem::path&,ArchivePurpose);
static void verifyRetainedPalette(const NativeLayerTable& expected,const ONX_Model& model){
    const auto actual=readNativeLayerTable(model);std::map<std::string,const NativeLayerRow*> palette;
    for(const auto& row:actual.rows)palette.emplace(row.id,&row);
    if(actual.rows.size()!=expected.rows.size()||actual.activeId!=expected.activeId)throw ExchangeError("Native write changed retained palette size or active layer");
    for(const auto& row:expected.rows){auto found=palette.find(row.id);if(found==palette.end())throw ExchangeError("Native write dropped retained layer identity");const auto& native=*found->second;
        if(row.parentId!=native.parentId||row.name!=native.name||row.path!=native.path||row.rgb!=native.rgb||row.locked!=native.locked||row.visible!=native.visible||(row.persistentLocked&&row.persistentLocked!=native.persistentLocked)||(row.persistentVisible&&row.persistentVisible!=native.persistentVisible))throw ExchangeError("Native write changed retained layer state: "+row.id);
    }
}
static QString id(const ON_UUID& value){char text[37]{};ON_UuidToString(value,text);return QString::fromLatin1(text);}
static QJsonArray matrixJson(const ON_Xform& transform){QJsonArray values;for(int i=0;i<16;++i)values.append(transform[i/4][i%4]);return values;}
static bool hasNativeFields(const QJsonObject& overlay){return overlay.contains("hatch_fields")||overlay.contains("text_dot")||overlay.contains("point_cloud_fields")||overlay.contains("point_cloud_sha256");}
static void validateNativeFields(const QJsonObject& overlay){
    if(!hasNativeFields(overlay))return;
    if(overlay["action"]!="transform")throw ExchangeError("Native current fields require transform action");
    bool cloud=overlay.contains("point_cloud_fields")||overlay.contains("point_cloud_sha256");
    const int kinds=(overlay.contains("hatch_fields")?1:0)+(overlay.contains("text_dot")?1:0)+(cloud?1:0);if(kinds!=1)throw ExchangeError("Ambiguous native current fields");
    if(cloud&&(overlay.contains("text_dot")||!overlay["point_cloud_fields"].isString()||!overlay["point_cloud_sha256"].isString()))throw ExchangeError("Ambiguous or incomplete native current fields");
}
static void applyTextDotFields(ON_Geometry& geometry,const QJsonValue& value){
    auto dot=ON_TextDot::Cast(&geometry);if(!dot||!value.isObject())throw ExchangeError("TextDot fields require native ON_TextDot geometry");auto fields=value.toObject();
    const QStringList keys{"point","primary_text","secondary_text","font_face","height_in_points","bold","italic","always_on_top","transparent"};
    if(fields.size()!=keys.size())throw ExchangeError("TextDot overlay requires complete known current fields");for(auto key:fields.keys())if(!keys.contains(key))throw ExchangeError("Unknown TextDot current field");
    auto point=fields["point"].toArray();if(point.size()!=3)throw ExchangeError("TextDot anchor requires3 double coordinates");ON_3dPoint anchor;for(int i=0;i<3;++i){if(!point[i].isDouble()||!std::isfinite(point[i].toDouble()))throw ExchangeError("Invalid TextDot anchor coordinate");anchor[i]=point[i].toDouble();}if(!anchor.IsValid())throw ExchangeError("Invalid TextDot anchor");
    for(auto key:{"primary_text","secondary_text","font_face"}){if(!fields[key].isString()||fields[key].toString().contains(QChar(0))||fields[key].toString().toUtf8().size()>1024*1024)throw ExchangeError("Invalid or oversized TextDot text field");}
    if(!fields["secondary_text"].toString().isEmpty()||ON_wString(dot->SecondaryText()).IsNotEmpty())throw ExchangeError("TextDot secondary text cannot be represented in Rhino5");
    auto height=fields["height_in_points"].toDouble(-1);if(!fields["height_in_points"].isDouble()||!std::isfinite(height)||height!=std::floor(height)||height<ON_TextDot::MinimumHeightInPoints||height>2147483647)throw ExchangeError("Invalid TextDot font height");
    for(auto key:{"bold","italic","always_on_top","transparent"})if(!fields[key].isBool())throw ExchangeError("Invalid TextDot display flag");
    auto original=nativeGeometryFacts(*dot)["text_dot"].toObject();dot->SetCenterPoint(anchor);
    if(fields["primary_text"]!=original["primary_text"])dot->SetPrimaryText(fields["primary_text"].toString().toStdWString().c_str());
    if(fields["font_face"]!=original["font_face"])dot->SetFontFace(fields["font_face"].toString().toStdWString().c_str());
    if(height!=dot->HeightInPoints())dot->SetHeightInPoints(static_cast<int>(height));
    if(fields["bold"].toBool()!=dot->Bold())dot->SetBold(fields["bold"].toBool());if(fields["italic"].toBool()!=dot->Italic())dot->SetItalic(fields["italic"].toBool());if(fields["always_on_top"].toBool()!=dot->AlwaysOnTop())dot->SetAlwaysOnTop(fields["always_on_top"].toBool());if(fields["transparent"].toBool()!=dot->Transparent())dot->SetTransparent(fields["transparent"].toBool());
    // Native setters trim whitespace and normalize bare CR. Require the exact
    // requested fields instead of silently accepting a different payload.
    if(!dot->IsValid()||nativeGeometryFacts(*dot)["text_dot"]!=fields)throw ExchangeError("TextDot fields require canonical native text/font values; no implicit normalization");
}
static QByteArray objectDigest(const ON_Object& object){
    // Selection can compact a custom pattern's table index. Its UUID edge and
    // complete pattern facts are verified independently; normalize only that
    // known reference slot in a clone, leaving all other native bytes checked.
    // Negative built-in indices stay exact and cannot become a custom pattern.
    std::unique_ptr<ON_Hatch> canonical;if(auto hatch=ON_Hatch::Cast(&object)){canonical=std::make_unique<ON_Hatch>(*hatch);if(canonical->PatternIndex()>=0)canonical->SetPatternIndex(0);}
    ON_Write3dmBufferArchive archive(0,512ULL*1024*1024,50,ON::Version());if(!archive.WriteObject(canonical?static_cast<const ON_Object&>(*canonical):object))throw ExchangeError("Cannot serialize bounded Rhino5 native object");return QCryptographicHash::hash(QByteArrayView(static_cast<const char*>(archive.Buffer()),static_cast<qsizetype>(archive.SizeOfArchive())),QCryptographicHash::Sha256);}
static QByteArray retainedLayerDigest(const ON_Layer& layer){
    // Archive selection compacts table indices. UUID edges and palette facts are
    // checked separately; all other native layer fields/userdata stay exact.
    ON_Layer normalized(layer);normalized.SetIndex(0);
    if(normalized.RenderMaterialIndex()>=0)normalized.SetRenderMaterialIndex(0);
    if(normalized.LinetypeIndex()>=0)normalized.SetLinetypeIndex(0);
    return objectDigest(normalized);
}
static std::size_t objectBytes(const ON_Object& object){ON_Write3dmBufferArchive archive(0,512ULL*1024*1024,50,ON::Version());if(!archive.WriteObject(object))throw ExchangeError("Cannot measure bounded Rhino5 member data");return static_cast<std::size_t>(archive.SizeOfArchive());}
static void retainSerializationCopyCounts(const ON_Geometry& source,ON_Geometry& target,unsigned depth,size_t& nodes){
    if(depth>=64||++nodes>16384||source.ClassId()!=target.ClassId())throw ExchangeError("Current UV serialization owning-tree limit or class mismatch");
    auto original=source.FirstUserData();auto copy=target.FirstUserData();
    while(original&&copy){
        if(original->ClassId()!=&ON_CLASS_RTTI(ON_UserStringList)||copy->ClassId()!=original->ClassId()||copy->m_userdata_uuid!=original->m_userdata_uuid||copy->m_application_uuid!=original->m_application_uuid)throw ExchangeError("Unverified UV serialization userdata copy");
        // Duplicate increments this serialized bookkeeping field. Linking for
        // writing must not change the retained native payload or count as edit.
        copy->m_userdata_copycount=original->m_userdata_copycount;original=original->Next();copy=copy->Next();
    }
    if(original||copy)throw ExchangeError("UV serialization clone dropped userdata");
    auto pair=[&](const ON_Geometry* a,ON_Geometry* b){if(bool(a)!=bool(b))throw ExchangeError("UV serialization clone changed owning children");if(a)retainSerializationCopyCounts(*a,*b,depth+1,nodes);};
    if(auto a=ON_CurveOnSurface::Cast(&source)){auto b=ON_CurveOnSurface::Cast(&target);pair(a->m_c2,b->m_c2);pair(a->m_c3,b->m_c3);pair(a->m_s,b->m_s);}
    else if(auto a=ON_PolyCurve::Cast(&source)){auto b=ON_PolyCurve::Cast(&target);if(a->Count()!=b->Count())throw ExchangeError("UV serialization clone changed segment count");for(int i=0;i<a->Count();++i)pair(a->SegmentCurve(i),b->SegmentCurve(i));}
    else if(auto a=ON_RevSurface::Cast(&source))pair(a->m_curve,ON_RevSurface::Cast(&target)->m_curve);
    else if(auto a=ON_SumSurface::Cast(&source)){auto b=ON_SumSurface::Cast(&target);pair(a->m_curve[0],b->m_curve[0]);pair(a->m_curve[1],b->m_curve[1]);}
    else if(auto a=ON_Extrusion::Cast(&source))pair(a->m_profile,ON_Extrusion::Cast(&target)->m_profile);
}
static std::shared_ptr<NativeReferenceResolution> currentSurfaceReferenceGraph(const ONX_Model& model,const std::set<QString>* selected=nullptr){
    QJsonArray records;ONX_ModelComponentIterator it(model,ON_ModelComponent::Type::ModelGeometry);
    for(auto component=it.FirstComponent();component;component=it.NextComponent()){
        if(selected&&!selected->contains(id(component->Id())))continue;
        auto geometry=ON_ModelGeometryComponent::Cast(component);auto curve=geometry?ON_CurveOnSurface::Cast(geometry->Geometry(nullptr)):nullptr;if(!curve)continue;
        auto fields=curveOnSurfaceNativeFields(*curve);if(fields["object_references"].toArray().isEmpty())continue;
        records.append(QJsonObject{{"source_uuid",id(component->Id())},{"class_name","ON_CurveOnSurface"},{"curve_on_surface_native",fields}});
    }
    // Borrow only the derived model for this synchronous serialization scope.
    // The returned graph owns its proxy targets; it never escapes this call chain.
    auto graph=resolveNativeReferences(std::shared_ptr<const ONX_Model>(&model,[](const ONX_Model*){}),records);
    for(auto value:records)if(graph->diagnostics[value.toObject()["source_uuid"].toString()].toObject()["status"]!="linked")throw ExchangeError("Cannot link current CurveOnSurface UV owner for serialization");
    size_t nodes=0;
    for(const auto& [uuid,entry]:graph->graphs){auto component=ON_ModelGeometryComponent::Cast(model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData())).ModelComponent());retainSerializationCopyCounts(*component->Geometry(nullptr),*entry->geometry.at(uuid),0,nodes);}
    return graph;
}
static QByteArray currentObjectDigest(const ON_Object& object,const QString& uuid,const NativeReferenceResolution& graph){
    auto found=graph.graphs.find(uuid);if(found!=graph.graphs.end())return objectDigest(*found->second->geometry.at(uuid));return objectDigest(object);
}
static bool writeSelectedModel(ONX_Model& model,const std::filesystem::path& path,ArchivePurpose purpose){
    auto graph=currentSurfaceReferenceGraph(model);
    struct Restore {
        std::vector<std::pair<ON_CurveOnSurface*,ON_CurveOnSurface*>> exchanged;
        static void swapChildren(ON_CurveOnSurface& a,ON_CurveOnSurface& b) noexcept {
            std::swap(a.m_c2,b.m_c2);std::swap(a.m_c3,b.m_c3);std::swap(a.m_s,b.m_s);
        }
        ~Restore(){for(auto [curve,linked]:exchanged)swapChildren(*curve,*linked);}
    } restore;
    // ON_CurveOnSurface::Write calls IsValid, requiring a live2D UV proxy.
    // Move the linked owning children temporarily into the derived model;
    // the graph keeps all target owners alive. Restore with no allocation.
    // The original archive and its detached reference fields never change.
    std::vector<std::pair<ON_CurveOnSurface*,ON_CurveOnSurface*>> planned;
    for(const auto& [uuid,entry]:graph->graphs){
        auto component=ON_ModelGeometryComponent::Cast(model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData())).ModelComponent());
        auto curve=component?ON_CurveOnSurface::Cast(component->ExclusiveGeometry()):nullptr;
        auto linked=ON_CurveOnSurface::Cast(entry->geometry.at(uuid).get());
        if(!curve||!linked||!linked->IsValid())throw ExchangeError("Missing exclusive current surface-curve serialization root");
        planned.emplace_back(curve,linked);
    }
    restore.exchanged.reserve(planned.size());
    for(auto [curve,linked]:planned){restore.exchanged.emplace_back(curve,linked);Restore::swapChildren(*curve,*linked);}
    return purpose==ArchivePurpose::GeometryStaging?model.Write(path.c_str(),80,nullptr):writeModelRhino5(model,path);
}
static void changeInstanceDefinition(ONX_Model& model,ON_InstanceRef* instance,QJsonObject& row,const QJsonValue& value){
    if(!instance||!om9_3dm_overlay_allowed(3,2,true,true)||!value.isString())throw ExchangeError("Reference edit requires a native instance and definition UUID");
    auto target=value.toString();QUuid parsed(target);if(parsed.isNull()||parsed.toString(QUuid::WithoutBraces)!=target)throw ExchangeError("Invalid instance definition target UUID");
    auto uuid=ON_UuidFromString(target.toLatin1().constData());if(!ON_InstanceDefinition::Cast(model.ComponentFromId(ON_ModelComponent::Type::InstanceDefinition,uuid).ModelComponent()))throw ExchangeError("Missing native instance definition target");
    auto original=row["instance_definition_uuid"].toString();if(original.isEmpty())throw ExchangeError("Missing source instance definition identity");QJsonArray dependencies;bool found=false;
    for(auto dependency:row["dependencies"].toArray()){if(dependency.toString()==original){dependencies.append(target);found=true;}else dependencies.append(dependency);}
    if(!found)throw ExchangeError("Missing source instance definition dependency");
    instance->m_instance_definition_uuid=uuid;row["instance_definition_uuid"]=target;row["dependencies"]=dependencies;row["geometry_crc"]=QString::number(instance->DataCRC(0));
}
static void safeUserdata(const QJsonObject& row){
    // The complete native payload is checked again by the shared V5 writer.
    // Reject observed target loss early, before transforms or dependency clones.
    if(row["class_name"]=="ON_CurveOnSurface"&&!row["curve_on_surface_native"].toObject()["approximation"].isNull())
        throw ExchangeError("Rhino 5 discards CurveOnSurface objects containing m_c3; exact native data remains retained in the project");
    for(auto key:{"userdata","attribute_userdata"})for(auto value:row[key].toArray()){
        auto name=value.toObject()["class_name"].toString();
        if(name!="ON_UserStringList")throw ExchangeError("Unsafe opaque userdata dependency: "+row["source_uuid"].toString().toStdString()+" ["+name.toStdString()+"]");
    }
}
static void safeRdkTable(const ONX_Model_UserData& table){
    ON_wString xml;if(!ONX_Model::IsRDKDocumentInformation(table)||!ONX_Model::GetRDKDocumentInformation(table,xml))throw ExchangeError("Opaque plugin table dependencies prevent safe selected export");
    ON_String utf8(xml);QXmlStreamReader reader(QString::fromUtf8(utf8.Array()));QStringList path;
    QRegularExpression uuidPattern("[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}");
    auto checkIds=[&](const QString& text){auto matches=uuidPattern.globalMatch(text);while(matches.hasNext()){auto value=matches.next().captured().toLower();if(value=="00000000-0000-0000-0000-000000000000")continue;bool channels=path.contains("render-channels")&&path.last()=="list"&&(value=="453a9a1c-9307-4976-b282-4ead4d539879"||value=="b752ce0b-c219-4bdd-b134-26425e1c4331");if(!channels)throw ExchangeError("RDK document contains unresolved UUID dependencies");}};
    while(!reader.atEnd()){
        reader.readNext();if(reader.isStartElement()){
            auto name=reader.name().toString();
            if(path.contains("material-section")||path.contains("environment-section")||path.contains("texture-section")||path.contains("default-content-section")||(path.contains("content")&&(name!="environment"||path.last()!="content")))throw ExchangeError("RDK content dependencies are not implemented yet");
            path.append(name);for(auto attribute:reader.attributes())checkIds(attribute.value().toString());
        }else if(reader.isEndElement()){if(!path.isEmpty())path.removeLast();}
        else if(reader.isCharacters()){
            auto text=reader.text().toString();checkIds(text);
            if(path.contains("ground-plane")&&path.last()=="material"&&!text.trimmed().isEmpty())throw ExchangeError("RDK ground-plane material dependency is not implemented yet");
        }else if(reader.isDTD())throw ExchangeError("RDK document DTD is not supported");
    }
    if(reader.hasError())throw ExchangeError("Invalid RDK document XML");
}
static std::unique_ptr<ON_Geometry> replacementGeometry(const QJsonObject& selection){
        std::unique_ptr<ON_Geometry> mesh;
        if(selection.contains("brep")){
            if(selection.contains("vertices")||selection.contains("faces"))throw ExchangeError("Ambiguous replacement geometry");auto file=std::filesystem::path(selection["brep"].toString().toStdWString());std::error_code error;auto size=std::filesystem::file_size(file,error);if(error||size==0||size>512ULL*1024*1024)throw ExchangeError("Missing or oversized replacement BRep");
            TopoDS_Shape shape;BRep_Builder builder;if(!BRepTools::Read(shape,file.string().c_str(),builder)||shape.IsNull()||!BRepCheck_Analyzer(shape).IsValid())throw ExchangeError("Invalid replacement BRep");
            if(shape.ShapeType()==TopAbs_VERTEX){auto point=BRep_Tool::Pnt(TopoDS::Vertex(shape));mesh=std::make_unique<ON_Point>(point.X(),point.Y(),point.Z());}
            else if(shape.ShapeType()==TopAbs_EDGE)mesh=exportCurve(TopoDS::Edge(shape),1e-6);
            else if(shape.ShapeType()==TopAbs_WIRE){auto poly=std::make_unique<ON_PolyCurve>();for(BRepTools_WireExplorer edges(TopoDS::Wire(shape));edges.More();edges.Next()){auto curve=exportCurve(edges.Current(),1e-6);if(!poly->Append(curve.release()))throw ExchangeError("Cannot append replacement wire curve");}mesh=std::move(poly);}
            else mesh=exportBrep(shape,1e-6);
            if(!mesh||!mesh->IsValid())throw ExchangeError("Cannot convert replacement BRep");
        }else{
        auto vertices=selection["vertices"].toArray(),faces=selection["faces"].toArray();if(vertices.isEmpty()||faces.isEmpty()||vertices.size()>1000000||faces.size()>1000000)throw ExchangeError("Invalid replacement mesh size");MeshData data;
        for(auto value:vertices){auto values=value.toArray();if(values.size()!=3)throw ExchangeError("Mesh vertex requires3 coordinates");std::array<double,3> point;for(int i=0;i<3;++i){if(!values[i].isDouble()||!std::isfinite(values[i].toDouble()))throw ExchangeError("Invalid mesh coordinate");point[i]=values[i].toDouble();}data.vertices.push_back(point);}
        for(auto value:faces){auto values=value.toArray();if(values.size()!=3&&values.size()!=4)throw ExchangeError("Mesh face requires3 or4 indices");std::array<int,4> face;for(int i=0;i<values.size();++i){auto number=values[i].toDouble(-1);if(!values[i].isDouble()||!std::isfinite(number)||number<0||number>=data.vertices.size()||number!=std::floor(number))throw ExchangeError("Mesh face index out of bounds");face[i]=static_cast<int>(number);}if(values.size()==3)face[3]=face[2];data.faces.push_back(face);}
        mesh=exportMesh(data);
        }
    return mesh;
}
static void safeMergedIdentityText(const ON_Object& object,const std::map<QString,ON_UUID>& aliases,bool canonicalBinding=false){
    ON_ClassArray<ON_UserString> strings;object.GetUserStrings(strings);
    const QRegularExpression pattern("[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}");
    for(int i=0;i<strings.Count();++i){
        // This one tag is a logical Rust binding, deliberately independent of
        // native UUID remaps. Final exact global binding validates it below.
        if(canonicalBinding&&strings[i].m_key==L"OpenMatrix9.LayerObjectId")continue;
        for(auto text:{strings[i].m_key,strings[i].m_string_value}){
        ON_String utf8(text);auto matches=pattern.globalMatch(QString::fromUtf8(utf8.Array()));
        while(matches.hasNext()){auto identity=matches.next().captured().toLower();auto found=aliases.find(identity);
            if(found!=aliases.end()&&id(found->second)!=identity)throw ExchangeError("Merged user text identity reference requires explicit remapping");}
    }}
}
static void writeMultipleSources(const QJsonObject& request,const std::filesystem::path& destination,ArchivePurpose purpose){
    const bool canonical=request["schema_version"].toInt()==2;NativeLayerSnapshot globalLayers;
    if(canonical)globalLayers=nativeLayerSnapshotFromJson(QJsonDocument(request["layer_session"].toObject()).toJson(QJsonDocument::Compact).toStdString());
    auto sources=request["sources"].toArray();if(sources.size()>128)throw ExchangeError("Too many source archives");
    auto first=sources[0].toObject();if(!canonical)for(auto value:sources)if(value.toObject()["archive_sha256"]!=first["archive_sha256"])throw ExchangeError("Different source document metadata requires an explicit document merge policy");
    QTemporaryDir staging;if(!staging.isValid())throw ExchangeError("Cannot stage source merge");
    struct Context{ArchiveInventory archive;ON_ManifestMap references;std::map<QString,ON_UUID> aliases;std::vector<ON_ModelComponentReference> added,nativeLayers;std::set<QString> bindings;QString name;};
    std::vector<std::unique_ptr<Context>> contexts;std::set<QString> namespaces,usedIds;ONX_Model output;std::uintmax_t inputBytes=0;std::optional<std::vector<QString>> documentRdk;
    int sourceIndex=0;
    for(auto value:sources){auto source=value.toObject();auto name=source["namespace"].toString();if(name.isEmpty()||!namespaces.insert(name).second)throw ExchangeError("Duplicate or empty source namespace");auto sourcePath=std::filesystem::path(source["snapshot"].toString().toStdWString());inputBytes+=std::filesystem::file_size(sourcePath);if(inputBytes>512ULL*1024*1024)throw ExchangeError("Combined source archives exceed512MiB");
        QJsonArray selection;for(auto selected:request["selected"].toArray())if(selected.toObject()["namespace"]==name)selection.append(selected);
        bool newRoots=false;for(auto key:{"new_geometry","new_instances"})for(auto geometry:request[key].toArray())if(geometry.toObject()["namespace"]==name&&geometry.toObject()["role"]=="top-level")newRoots=true;
        if(selection.isEmpty()&&!newRoots&&!canonical){for(auto key:{"dependency_overlays","definition_overlays","new_geometry","member_copies","new_definitions","new_instances"})for(auto overlay:request[key].toArray())if(overlay.toObject()["namespace"]==name)throw ExchangeError("Overlay source has no selected roots");continue;}
        auto subset=std::filesystem::path(staging.path().toStdWString())/(std::to_wstring(sourceIndex++)+L".3dm");auto subrequest=request;subrequest["sources"]=QJsonArray{source};subrequest["selected"]=selection;for(auto key:{"dependency_overlays","definition_overlays","new_geometry","member_copies","new_definitions","new_instances"}){QJsonArray scoped;for(auto overlay:request[key].toArray())if(overlay.toObject()["namespace"]==name)scoped.append(overlay);subrequest[key]=scoped;}
        std::set<QString> namespaceBindings;
        if(canonical){
            std::vector<QByteArray> ids;
            auto bindingId=[&](QJsonObject row){if(!row["layer_object_id"].isString()||row["layer_object_id"].toString().isEmpty())throw ExchangeError("Canonical namespace root requires exact logical binding");ids.push_back(row["layer_object_id"].toString().toUtf8());namespaceBindings.insert(row["layer_object_id"].toString());};
            for(auto entry:selection)bindingId(entry.toObject());
            for(auto key:{"new_geometry","new_instances"})for(auto entry:subrequest[key].toArray())if(entry.toObject()["role"]=="top-level")bindingId(entry.toObject());
            std::vector<Om9LayerByteView> views;views.reserve(ids.size());for(const auto& value:ids)views.push_back({reinterpret_cast<const unsigned char*>(value.constData()),static_cast<std::size_t>(value.size())});
            std::uint64_t handle=0;auto error=om9_layer_snapshot_subset(globalLayers.get(),views.data(),views.size(),&handle);if(error)throw ExchangeError("Rust namespace metadata partition rejected; code="+std::to_string(error));NativeLayerSnapshot partition(handle);
            subrequest["layer_session"]=QJsonDocument::fromJson(QByteArray::fromStdString(nativeLayerSnapshotJson(partition.get()))).object();
        }
        writeSelectedArchive(subrequest,subset,purpose);
        auto context=std::make_unique<Context>();context->archive=inspectArchive(subset);context->name=name;context->bindings=std::move(namespaceBindings);
        if(canonical){
            // SDK writes a default RDK document table even for a plain point.
            // Reuse the existing resource-free validator; opaque/plugin or
            // conflicting document settings remain explicit unsupported input.
            std::vector<QString> rdk;auto& tables=context->archive.nativeModel->m_userdata_table;
            for(int i=0;i<tables.Count();++i)if(auto table=tables[i]){safeRdkTable(*table);ON_wString xml;if(!ONX_Model::GetRDKDocumentInformation(*table,xml))throw ExchangeError("Cannot read verified native RDK document settings");rdk.push_back(QString::fromWCharArray(xml.Array()));}
            if(documentRdk&&*documentRdk!=rdk)throw ExchangeError("Different native RDK document settings require an explicit merge policy");documentRdk=std::move(rdk);
        }
        if(contexts.empty()){output.m_properties=context->archive.nativeModel->m_properties;output.m_settings=context->archive.nativeModel->m_settings;if(canonical){output.m_settings.SetCurrentLayerId(ON_nil_uuid);output.m_settings.SetV5CurrentLayerIndex(-1);}}
        if(canonical){auto layerAliases=mergeRetainedNativePalette(output,readNativeLayerTable(*context->archive.nativeModel));for(const auto& [from,to]:layerAliases){auto uuid=ON_UuidFromString(to.c_str());context->aliases[QString::fromStdString(from)]=uuid;usedIds.insert(id(uuid));}}
        for(unsigned type=1;type<static_cast<unsigned>(ON_ModelComponent::Type::NumOf);++type){if(type==16)continue;ONX_ModelComponentIterator it(*context->archive.nativeModel,static_cast<ON_ModelComponent::Type>(type));for(auto component=it.FirstComponent();component;component=it.NextComponent()){
            if(canonical&&component->ComponentType()==ON_ModelComponent::Type::Layer)continue;
            auto uuid=component->Id();
            if(usedIds.contains(id(uuid))){
                // Keep collision aliases stable for this ordered source set.
                // Salt also handles an occupied deterministic candidate.
                auto seed=QByteArrayLiteral("OpenMatrix9.3dm.merge:")+id(uuid).toLatin1();unsigned salt=0;
                do{
                    if(salt>=4096)throw ExchangeError("Merged identity collision limit exceeded");
                    auto scoped=QUuid::createUuidV5(QUuid(name),seed+':'+QByteArray::number(salt++));
                    uuid=ON_UuidFromString(scoped.toString(QUuid::WithoutBraces).toLatin1().constData());
                }while(usedIds.contains(id(uuid)));
            }
            usedIds.insert(id(uuid));context->aliases[id(component->Id())]=uuid;
        }}
        contexts.push_back(std::move(context));
    }
    if(contexts.empty())throw ExchangeError("No selected source records");
    std::set<QString> retainedNativeLayers;
    for(unsigned type=1;type<static_cast<unsigned>(ON_ModelComponent::Type::NumOf);++type){if(type==16)continue;auto kind=static_cast<ON_ModelComponent::Type>(type);
        for(auto& context:contexts){auto& model=*context->archive.nativeModel;ONX_ModelComponentIterator it(model,kind);for(auto component=it.FirstComponent();component;component=it.NextComponent()){
            if(canonical&&kind==ON_ModelComponent::Type::Layer){
                auto stored=output.ComponentFromId(kind,context->aliases.at(id(component->Id())));ON_ManifestMapItem mapping;if(stored.IsEmpty()||!mapping.SetSourceIdentification(component)||!mapping.SetDestinationIdentification(stored.ModelComponent())||!context->references.AddMapItem(mapping))throw ExchangeError("Cannot map canonical shared layer references");
                if(retainedNativeLayers.insert(id(stored.ModelComponent()->Id())).second){
                    safeMergedIdentityText(*component,context->aliases);auto sourceLayer=ON_Layer::Cast(component);auto targetLayer=const_cast<ON_Layer*>(ON_Layer::Cast(stored.ModelComponent()));
                    const auto nativeId=targetLayer->Id(),parent=targetLayer->ParentId();const auto index=targetLayer->Index();const auto name=targetLayer->Name();
                    *targetLayer=*sourceLayer;targetLayer->SetId(nativeId);targetLayer->SetIndex(index);targetLayer->SetName(name);targetLayer->SetParentId(parent);
                    context->nativeLayers.push_back(stored);
                }
                continue;
            }
            safeMergedIdentityText(*component,context->aliases);
            std::unique_ptr<ON_ModelComponent> copy;
            if(auto geometry=ON_ModelGeometryComponent::Cast(component)){safeMergedIdentityText(*geometry->Geometry(nullptr),context->aliases);safeMergedIdentityText(*geometry->Attributes(nullptr),context->aliases,canonical);auto attributes=*geometry->Attributes(nullptr);attributes.m_uuid=context->aliases.at(id(component->Id()));copy.reset(ON_ModelGeometryComponent::Create(*geometry->Geometry(nullptr),&attributes,nullptr));}
            else copy.reset(ON_ModelComponent::Cast(component->Duplicate()));
            if(!copy||!copy->SetId(context->aliases.at(id(component->Id()))))throw ExchangeError("Cannot allocate merged component identity");
            if(ON_ModelComponent::UniqueNameRequired(kind)&&context.get()!=contexts[0].get()){auto name=copy->Name();name+=L" [";name+=ON_wString(context->name.left(8).toStdWString().c_str());name+=L"]";if(!copy->SetName(name))throw ExchangeError("Cannot resolve source component name collision");}
            auto added=output.AddModelComponent(*copy,true);if(added.IsEmpty()||added.ModelComponent()->Id()!=context->aliases.at(id(component->Id())))throw ExchangeError("Native merge changed planned identity");
            ON_ManifestMapItem mapping;if(!mapping.SetSourceIdentification(component)||!mapping.SetDestinationIdentification(added.ModelComponent())||!context->references.AddMapItem(mapping))throw ExchangeError("Cannot map merged component references");context->added.push_back(added);
        }}
    }
    for(auto& context:contexts)for(auto& reference:context->added){auto component=const_cast<ON_ModelComponent*>(reference.ModelComponent());if(!component->UpdateReferencedComponents(context->archive.nativeModel->Manifest(),output.Manifest(),context->references))throw ExchangeError("Cannot safely update merged component references");
        // The pinned openNURBS classes do not override reference updating for
        // instance definitions or references. Remap their UUID graph explicitly.
        auto mapped=[&](ON_UUID original){auto found=context->aliases.find(id(original));if(found==context->aliases.end())throw ExchangeError("Missing merged block UUID mapping");return found->second;};
        if(auto definition=ON_InstanceDefinition::Cast(component)){ON_SimpleArray<ON_UUID> members;auto& originals=definition->InstanceGeometryIdList();for(int i=0;i<originals.Count();++i)members.Append(mapped(originals[i]));const_cast<ON_InstanceDefinition*>(definition)->SetInstanceGeometryIdList(members);}
        if(auto geometry=ON_ModelGeometryComponent::Cast(component))if(auto instance=ON_InstanceRef::Cast(geometry->ExclusiveGeometry()))instance->m_instance_definition_uuid=mapped(instance->m_instance_definition_uuid);
        if(auto geometry=ON_ModelGeometryComponent::Cast(component))if(auto curve=ON_Curve::Cast(geometry->ExclusiveGeometry())){
            auto facts=nativeGeometryFacts(*curve);
            if(!facts["poly_edge_native"].toObject()["object_references"].toArray().isEmpty()||!facts["curve_on_surface_native"].toObject()["object_references"].toArray().isEmpty())
                remapDeferredNativeReferences(*curve,context->aliases);
        }
    }
    for(const auto& context:contexts)for(const auto& reference:context->nativeLayers)if(!const_cast<ON_ModelComponent*>(reference.ModelComponent())->UpdateReferencedComponents(context->archive.nativeModel->Manifest(),output.Manifest(),context->references))throw ExchangeError("Cannot remap retained native layer references");
    std::optional<NativeLayerTable> expectedPalette;
    if(canonical){
        std::vector<NativeLayerObjectBinding> bindings;
        for(const auto& context:contexts)for(auto entry:context->archive.document["records"].toArray()){
            auto row=entry.toObject();if(row["role"]!="top-level")continue;
            auto component=ON_ModelGeometryComponent::Cast(context->archive.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(row["source_uuid"].toString().toLatin1().constData())).ModelComponent());ON_wString logical;
            if(component&&component->Attributes(nullptr)&&component->Attributes(nullptr)->GetUserString(L"OpenMatrix9.LayerObjectId",logical)){
                auto sourceId=QString::fromWCharArray(logical.Array());if(context->bindings.contains(sourceId))bindings.push_back({id(context->aliases.at(row["source_uuid"].toString())).toStdString(),sourceId.toStdString()});
            }
        }
        // Final global exact binding rejects duplicate/unused/missing metadata
        // across namespaces. Partition success alone cannot prove whole transfer.
        applyRetainedLayerOverlay(output,globalLayers.get(),bindings);expectedPalette=readNativeLayerTable(output);
    }
    std::map<QString,QByteArray> mergedLayerDigests;
    if(canonical){ONX_ModelComponentIterator layers(output,ON_ModelComponent::Type::Layer);for(auto component=layers.FirstComponent();component;component=layers.NextComponent())mergedLayerDigests.emplace(id(component->Id()),retainedLayerDigest(*ON_Layer::Cast(component)));}
    std::map<QString,QByteArray> mergedAttributeDigests;
    if(canonical)for(const auto& context:contexts)for(const auto& reference:context->added)if(auto geometry=ON_ModelGeometryComponent::Cast(reference.ModelComponent()))mergedAttributeDigests.emplace(id(geometry->Id()),objectDigest(*geometry->Attributes(nullptr)));
    auto mergedGraph=currentSurfaceReferenceGraph(output);
    auto file=std::filesystem::path(staging.path().toStdWString())/L"merged.3dm";if(!writeSelectedModel(output,file,purpose))throw ExchangeError("Cannot write merged Rhino5 archive");auto checked=inspectArchive(file);
    if(!checked.document["issues"].toArray().isEmpty()||checked.document["records"].toArray().size()!=output.ActiveComponentCount(ON_ModelComponent::Type::ModelGeometry)+output.ActiveComponentCount(ON_ModelComponent::Type::RenderLight))throw ExchangeError("Merged Rhino5 graph validation failed");
    if(expectedPalette)verifyRetainedPalette(*expectedPalette,*checked.nativeModel);
    if(canonical)for(const auto& [uuid,digest]:mergedLayerDigests){auto layer=ON_Layer::Cast(checked.nativeModel->ComponentFromId(ON_ModelComponent::Type::Layer,ON_UuidFromString(uuid.toLatin1().constData())).ModelComponent());if(!layer||retainedLayerDigest(*layer)!=digest)throw ExchangeError("Merged native layer fields changed: "+uuid.toStdString());}
    auto checkedGraph=currentSurfaceReferenceGraph(*checked.nativeModel);
    for(auto& context:contexts)for(auto& reference:context->added){auto component=reference.ModelComponent();auto reread=checked.nativeModel->ComponentFromId(component->ComponentType(),component->Id());if(reread.IsEmpty()||reread.ModelComponent()->ClassId()->Uuid()!=component->ClassId()->Uuid())throw ExchangeError("Merged Rhino5 component dropped or changed type");if(auto geometry=ON_ModelGeometryComponent::Cast(component)){auto target=ON_ModelGeometryComponent::Cast(reread.ModelComponent());if(!target||currentObjectDigest(*geometry->Geometry(nullptr),id(component->Id()),*mergedGraph)!=currentObjectDigest(*target->Geometry(nullptr),id(component->Id()),*checkedGraph))throw ExchangeError("Merged native geometry payload changed");if(canonical&&(!target->Attributes(nullptr)||mergedAttributeDigests.at(id(component->Id()))!=objectDigest(*target->Attributes(nullptr))))throw ExchangeError("Merged native object attributes changed");}}
    QFile input(QString::fromStdWString(file.wstring()));QSaveFile target(QString::fromStdWString(destination.wstring()));if(!input.open(QIODevice::ReadOnly)||!target.open(QIODevice::WriteOnly))throw ExchangeError("Cannot stage merged output");while(!input.atEnd()){auto bytes=input.read(1024*1024);if(bytes.isEmpty()&&input.error()!=QFile::NoError)throw ExchangeError("Cannot read merged output");if(target.write(bytes)!=bytes.size())throw ExchangeError("Cannot write merged output");}input.close();if(!target.commit())throw ExchangeError("Cannot replace merged output atomically");
}
static void writeSelectedArchive(const QJsonObject& request,const std::filesystem::path& destination,ArchivePurpose purpose){
    const auto schema=request["schema_version"].toInt();
    if(schema!=1&&schema!=2)throw ExchangeError("Invalid preservation export schema");
    const bool canonical=schema==2;
    if(canonical!=request.contains("layer_session"))throw ExchangeError("Preservation schema requires its matching layer metadata contract");
    NativeLayerSnapshot layerSnapshot;
    if(canonical){
        if(!request["layer_session"].isObject())throw ExchangeError("Invalid canonical preservation layer metadata");
        layerSnapshot=nativeLayerSnapshotFromJson(QJsonDocument(request["layer_session"].toObject()).toJson(QJsonDocument::Compact).toStdString());
    }
    if(!request["sources"].isArray()||!request["selected"].isArray())throw ExchangeError("Invalid preservation source or selection table");
    if(request.contains("new_geometry")&&!request["new_geometry"].isArray())throw ExchangeError("Invalid new geometry table");
    for(auto key:{"new_definitions","new_instances"})if(request.contains(key)&&!request[key].isArray())throw ExchangeError("Invalid new block table");
    if(QJsonDocument(request).toJson(QJsonDocument::Compact).size()>32*1024*1024)throw ExchangeError("Preservation request exceeds32MiB");
    // Field arrays live outside the request JSON. Bound their aggregate inputs,
    // including independent copies and all namespaces, before cloning geometry.
    std::uintmax_t fieldBytes=0;
    auto fieldBudget=[&](QJsonObject overlay){if(overlay["action"]=="duplicate")overlay["action"]=overlay["duplicate_action"];validateNativeFields(overlay);if(!overlay.contains("point_cloud_fields"))return;std::error_code error;auto size=std::filesystem::file_size(std::filesystem::path(overlay["point_cloud_fields"].toString().toStdWString()),error);if(error||size==0||size>512ULL*1024*1024-fieldBytes)throw ExchangeError("Combined PointCloud current fields exceed512MiB or are unavailable");fieldBytes+=size;};
    for(auto key:{"selected","dependency_overlays"})for(auto value:request[key].toArray())fieldBudget(value.toObject());
    for(auto value:request["member_copies"].toArray()){auto copy=value.toObject();if(copy.contains("independent_overlay"))fieldBudget(copy["independent_overlay"].toObject());}
    auto sources=request["sources"].toArray(),selected=request["selected"].toArray();
    if(!canonical)for(auto key:{"selected","new_geometry","new_instances","member_copies","dependency_overlays"})for(auto value:request[key].toArray())if(value.toObject().contains("layer_object_id"))throw ExchangeError("Canonical object binding cannot silently downgrade to legacy preservation");
    if(sources.isEmpty()&&(canonical||!request["new_geometry"].toArray().isEmpty()||!request["new_instances"].toArray().isEmpty())){
        if(!selected.isEmpty())throw ExchangeError("Source selection requires an archive");
        auto name=request["document_namespace"].toString();QUuid uuid(name);if(uuid.isNull()||uuid.toString(QUuid::WithoutBraces)!=name)throw ExchangeError("Invalid new document namespace");
        QTemporaryDir staging;if(!staging.isValid())throw ExchangeError("Cannot stage new document");ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
        if(canonical)writeNativeLayerTable(model,nativeLayerTableFromSnapshot(layerSnapshot.get()));
        else{ON_Layer layer;layer.SetName(L"OpenMatrix9");model.AddModelComponent(layer);}
        auto file=std::filesystem::path(staging.path().toStdWString())/L"new-document.3dm";if(!writeModelRhino5(model,file))throw ExchangeError("Cannot prepare new mm document");auto inventory=inspectArchive(file).document;auto scoped=request;scoped["sources"]=QJsonArray{QJsonObject{{"namespace",name},{"snapshot",QString::fromStdWString(file.wstring())},{"archive_sha256",inventory["archive_sha256"]},{"scale_mm",1.0}}};writeSelectedArchive(scoped,destination,purpose);return;
    }
    if(request.contains("dependency_overlays")&&!request["dependency_overlays"].isArray())throw ExchangeError("Invalid dependency overlay table");
    if(request.contains("definition_overlays")&&!request["definition_overlays"].isArray())throw ExchangeError("Invalid definition overlay table");
    if(request.contains("member_copies")&&!request["member_copies"].isArray())throw ExchangeError("Invalid member copy table");
    std::set<QString> sourceNames;for(auto value:sources)sourceNames.insert(value.toObject()["namespace"].toString());
    for(auto value:selected)if(!sourceNames.contains(value.toObject()["namespace"].toString()))throw ExchangeError("Selected object has no source namespace");
    for(auto value:request["dependency_overlays"].toArray())if(!sourceNames.contains(value.toObject()["namespace"].toString()))throw ExchangeError("Dependency overlay has no source namespace");
    for(auto value:request["definition_overlays"].toArray())if(!sourceNames.contains(value.toObject()["namespace"].toString()))throw ExchangeError("Definition overlay has no source namespace");
    for(auto value:request["new_geometry"].toArray())if(!sourceNames.contains(value.toObject()["namespace"].toString()))throw ExchangeError("New geometry has no document namespace");
    for(auto value:request["member_copies"].toArray())if(!sourceNames.contains(value.toObject()["namespace"].toString()))throw ExchangeError("Member copy has no source namespace");
    for(auto key:{"new_definitions","new_instances"})for(auto value:request[key].toArray())if(!sourceNames.contains(value.toObject()["namespace"].toString()))throw ExchangeError("New block has no document namespace");
    if(sources.size()>1){writeMultipleSources(request,destination,purpose);return;}
    if(sources.size()!=1)throw ExchangeError("No preservation source archive");
    if(!canonical&&selected.isEmpty()&&request["new_geometry"].toArray().isEmpty()&&request["new_instances"].toArray().isEmpty())throw ExchangeError("Select whole objects to export");
    auto source=sources[0].toObject();auto path=std::filesystem::path(source["snapshot"].toString().toStdWString());
    auto sourceNamespace=source["namespace"].toString();QUuid namespaceId(sourceNamespace);if(namespaceId.isNull()||namespaceId.toString(QUuid::WithoutBraces)!=sourceNamespace)throw ExchangeError("Invalid source import namespace");
    auto inventory=inspectArchive(path,source["scale_mm"].toDouble());auto manifest=inventory.document;
    if(manifest["archive_sha256"]!=source["archive_sha256"])throw ExchangeError("Source snapshot hash mismatch");
    if(!source["scale_mm"].isDouble()||source["scale_mm"].toDouble()!=manifest["scale_mm"].toDouble())throw ExchangeError("Source unit scale does not match snapshot units");
    if(manifest["source_version"].toInt()!=5&&manifest["source_version"].toInt()!=50)throw ExchangeError("Newer source-version compatibility has not been verified for this preservation writer");
    if(!manifest["issues"].toArray().isEmpty())throw ExchangeError("Source archive has unresolved dependencies");
    auto& model=*inventory.nativeModel;
    // Inspect before dependency pruning can discard resource components.
    // RDK3 cannot represent the resource trailer from a modern RDK4 source.
    if(model.ActiveComponentCount(ON_ModelComponent::Type::EmbeddedFile)>0)
        throw ExchangeError("Rhino5 export cannot retain RDK4 embedded resources; source archive remains preserved");
    for(int i=0;i<model.m_userdata_table.Count();++i)if(auto table=model.m_userdata_table[i])safeRdkTable(*table);
    for(auto resource:manifest["resources"].toArray())if(resource.toObject().contains("full_path"))throw ExchangeError("External resource resolution is required before preservation export");
    auto scale=manifest["scale_mm"].toDouble();
    if(scale!=1.0){
        if(model.m_settings.m_views.Count()||model.m_settings.m_named_views.Count())throw ExchangeError("View unit normalization requires verified viewport handling");
        ON_Xform scaling=ON_Xform::DiagonalTransformation(scale);
        auto normalized=manifest["records"].toArray();
        for(auto value:normalized){auto row=value.toObject();safeUserdata(row);auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(row["source_uuid"].toString().toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());auto geometry=component?component->ExclusiveGeometry():nullptr;if(!geometry)throw ExchangeError("Cannot normalize retained geometry units");
            if(auto instance=ON_InstanceRef::Cast(geometry)){for(int i=0;i<3;++i)instance->m_xform[i][3]*=scale;instance->m_bbox.Transform(scaling);}
            else if(ON_Point::Cast(geometry)||ON_TextDot::Cast(geometry)||ON_PointCloud::Cast(geometry)||ON_Hatch::Cast(geometry)||ON_Curve::Cast(geometry)||ON_Brep::Cast(geometry)||ON_Surface::Cast(geometry)||ON_Mesh::Cast(geometry)){transformNativeGeometry(*geometry,scaling,&model);}
            else throw ExchangeError("Retained class unit normalization is not verified");
            row["geometry_crc"]=QString::number(geometry->DataCRC(0));if(auto instance=ON_InstanceRef::Cast(geometry))row["instance_matrix"]=matrixJson(instance->m_xform);value=row;
        }
        manifest["records"]=normalized;
        ONX_ModelComponentIterator definitions(model,ON_ModelComponent::Type::InstanceDefinition);for(auto component=definitions.FirstComponent();component;component=definitions.NextComponent()){auto definition=const_cast<ON_InstanceDefinition*>(ON_InstanceDefinition::Cast(component));auto units=definition->UnitSystem().UnitSystem();if(units!=ON::LengthUnitSystem::None&&units!=model.m_settings.m_ModelUnitsAndTolerances.m_unit_system.UnitSystem())throw ExchangeError("Mixed block definition units require explicit normalization");auto box=definition->BoundingBox();box.Transform(scaling);definition->SetBoundingBox(box);definition->SetUnitSystem(ON::LengthUnitSystem::Millimeters);}
        ONX_ModelComponentIterator styles(model,ON_ModelComponent::Type::DimStyle);for(auto component=styles.FirstComponent();component;component=styles.NextComponent())const_cast<ON_DimStyle*>(ON_DimStyle::Cast(component))->Scale(scale);
        if(model.ActiveComponentCount(ON_ModelComponent::Type::LinePattern))throw ExchangeError("Line pattern unit normalization is not verified");
        model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance*=scale;
    }
    model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
    // Allocate copies before closure so they share the source dependency graph.
    auto records=manifest["records"].toArray();std::set<QString> hostIds;
    for(auto value:selected){auto row=value.toObject();auto host=row["host_id"].toString();if(host.isEmpty()||!hostIds.insert(host).second)throw ExchangeError("Duplicate or empty selected host identity");if(row["action"]!="duplicate")continue;
        auto sourceId=row["source_uuid"].toString();QJsonObject original;for(auto record:records)if(record.toObject()["source_uuid"]==sourceId){original=record.toObject();break;}
        if(original.isEmpty()||original["role"]!="top-level")throw ExchangeError("Duplicate requires a source top-level record");safeUserdata(original);
        for(auto key:{"user_strings","attribute_user_strings"})for(auto text:original[key].toArray())if(text.toObject()["value"].toString().contains(sourceId,Qt::CaseInsensitive))throw ExchangeError("Duplicate user text identity reference requires explicit remapping");
        if(!om9_3dm_overlay_allowed(original["capability"]=="editable"?1:3,3,true,original["class_name"]=="ON_InstanceRef"))throw ExchangeError("Duplicate overlay policy rejected");
        auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(sourceId.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());if(!component||!component->Geometry(nullptr)||!component->Attributes(nullptr))throw ExchangeError("Cannot duplicate native record");
        // Match the host's source-copy identity contract. A fresh UUID on every
        // export breaks references and copy identity after FCStd reopening.
        auto copyId=QUuid::createUuidV5(namespaceId,QByteArray("OpenMatrix9.3dm.source-copy:")+sourceId.toUtf8()+':'+host.toUtf8()).toString(QUuid::WithoutBraces);
        for(auto collection:{records,manifest["components"].toArray()})for(auto existing:collection)if(existing.toObject()["source_uuid"]==copyId)throw ExchangeError("Duplicate UUID collides with an existing source component");
        auto attributes=*component->Attributes(nullptr);attributes.m_uuid=ON_UuidFromString(copyId.toLatin1().constData());auto added=model.AddModelGeometryComponent(component->Geometry(nullptr),&attributes);if(added.IsEmpty()||added.ModelComponent()->Id()!=attributes.m_uuid)throw ExchangeError("Cannot allocate duplicate UUID");
        auto uuid=id(attributes.m_uuid);original["source_uuid"]=uuid;records.append(original);row["source_uuid"]=uuid;auto action=row["duplicate_action"].toString("unchanged");if(action!="unchanged"&&action!="replace"&&action!="instance"&&action!="transform")throw ExchangeError("Invalid duplicate overlay action");row["action"]=action;row.remove("duplicate_action");value=row;
    }
    std::set<QString> occupiedIds;for(auto collection:{records,manifest["components"].toArray()})for(auto existing:collection)occupiedIds.insert(existing.toObject()["source_uuid"].toString());QJsonArray newMembers;
    for(auto value:request["new_geometry"].toArray()){
        if(!value.isObject())throw ExchangeError("Invalid new geometry record");auto row=value.toObject();auto uuid=row["source_uuid"].toString(),host=row["host_id"].toString(),role=row["role"].toString();QUuid parsed(uuid);
        if(parsed.isNull()||parsed.toString(QUuid::WithoutBraces)!=uuid||host.isEmpty()||!hostIds.insert(host).second||row["namespace"]!=source["namespace"]||(role!="top-level"&&role!="definition-member"))throw ExchangeError("Invalid new geometry identity or role");
        for(auto key:row.keys())if(key!="source_uuid"&&key!="host_id"&&key!="namespace"&&key!="role"&&key!="brep"&&key!="vertices"&&key!="faces"&&key!="metadata"&&!(canonical&&key=="layer_object_id"))throw ExchangeError("Unsupported new geometry field");
        if(!occupiedIds.insert(uuid).second)throw ExchangeError("New geometry UUID collides with source or another new record");
        auto layerReference=model.ComponentFromId(ON_ModelComponent::Type::Layer,model.m_settings.CurrentLayerId());auto layer=ON_Layer::Cast(layerReference.ModelComponent());if(!layer){ONX_ModelComponentIterator layers(model,ON_ModelComponent::Type::Layer);layer=ON_Layer::Cast(layers.FirstComponent());}if(!layer)throw ExchangeError("New geometry requires a document layer");
        ON_3dmObjectAttributes attributes;attributes.m_uuid=ON_UuidFromString(uuid.toLatin1().constData());attributes.m_layer_index=layer->Index();if(role=="definition-member")attributes.SetMode(ON::idef_object);ON_Point placeholder(0,0,0);auto added=model.AddModelGeometryComponent(&placeholder,&attributes);if(added.IsEmpty()||id(added.ModelComponent()->Id())!=uuid)throw ExchangeError("Cannot allocate new geometry UUID");ON_wString type=ON_ModelComponent::ComponentTypeToString(ON_ModelComponent::Type::ModelGeometry);
        records.append(QJsonObject{{"source_uuid",uuid},{"class_uuid",id(placeholder.ClassId()->Uuid())},{"class_name","ON_Point"},{"component_type",QString::fromWCharArray(type.Array())},{"name",""},{"role",role},{"dependencies",QJsonArray{id(layer->Id())}},{"capability","editable"},{"geometry_crc",QString::number(placeholder.DataCRC(0))},{"wire_density",attributes.m_wire_density},{"user_strings",QJsonArray{}},{"userdata",QJsonArray{}},{"attribute_user_strings",QJsonArray{}},{"attribute_userdata",QJsonArray{}}});
        row["action"]="replace";if(role=="top-level")selected.append(row);else newMembers.append(row);
    }
    auto definitionEntries=request["definition_overlays"].toArray();auto components=manifest["components"].toArray();
    // Metadata may be large even when the copied member graph is small. Count
    // every requested native definition copy before allocating the first clone.
    std::map<QString,std::size_t> definitionCopyCounts;std::set<QString> copiedDefinitionIds;std::size_t definitionCopyBytes=0;
    for(auto value:request["new_definitions"].toArray()){
        auto row=value.toObject();if(!row.contains("source_definition_uuid"))continue;auto sourceId=row["source_definition_uuid"].toString();QUuid parsedSource(sourceId);
        if(!row["source_definition_uuid"].isString()||parsedSource.isNull()||parsedSource.toString(QUuid::WithoutBraces)!=sourceId)throw ExchangeError("Invalid copied definition source UUID");
        bool original=false;for(auto component:components)if(component.toObject()["source_uuid"]==sourceId&&component.toObject()["class_name"]=="ON_InstanceDefinition"){original=true;break;}
        if(!original)throw ExchangeError("Copied definition requires an original source definition");++definitionCopyCounts[sourceId];copiedDefinitionIds.insert(row["source_uuid"].toString());
    }
    for(auto& [uuid,count]:definitionCopyCounts){auto reference=model.ComponentFromId(ON_ModelComponent::Type::InstanceDefinition,ON_UuidFromString(uuid.toLatin1().constData()));auto native=ON_InstanceDefinition::Cast(reference.ModelComponent());if(!native)throw ExchangeError("Missing copied definition source");auto budget=om9_3dm_copy_budget(objectBytes(*native),count,definitionCopyBytes);if(budget<0)throw ExchangeError("Copied definition data exceeds512MiB");definitionCopyBytes=budget;}
    auto currentDefinitionCopyBytes=[&](){std::size_t total=0;for(auto& uuid:copiedDefinitionIds){auto reference=model.ComponentFromId(ON_ModelComponent::Type::InstanceDefinition,ON_UuidFromString(uuid.toLatin1().constData()));auto native=ON_InstanceDefinition::Cast(reference.ModelComponent());if(!native)throw ExchangeError("Missing allocated copied definition");auto budget=om9_3dm_copy_budget(objectBytes(*native),1,total);if(budget<0)throw ExchangeError("Final copied definition data exceeds512MiB");total=budget;}return total;};
    for(auto value:request["new_definitions"].toArray()){
        if(!value.isObject())throw ExchangeError("Invalid new definition record");auto row=value.toObject();auto uuid=row["source_uuid"].toString(),host=row["host_id"].toString(),name=row["name"].toString();QUuid parsed(uuid);
        for(auto key:row.keys())if(key!="namespace"&&key!="source_uuid"&&key!="host_id"&&key!="name"&&key!="member_uuids"&&key!="source_definition_uuid")throw ExchangeError("Unsupported new definition field");
        if(row["namespace"]!=source["namespace"]||parsed.isNull()||parsed.toString(QUuid::WithoutBraces)!=uuid||host.isEmpty()||!row["name"].isString()||name.isEmpty()||name.size()>1024||!row["member_uuids"].isArray()||!occupiedIds.insert(uuid).second)throw ExchangeError("Invalid or colliding new definition identity");
        ONX_ModelComponentIterator definitions(model,ON_ModelComponent::Type::InstanceDefinition);for(auto component=definitions.FirstComponent();component;component=definitions.NextComponent())if(QString::fromWCharArray(component->Name().Array()).compare(name,Qt::CaseInsensitive)==0)throw ExchangeError("New definition name collision");
        ON_InstanceDefinition definition;QJsonObject originalDefinition;
        if(row.contains("source_definition_uuid")){
            auto sourceId=row["source_definition_uuid"].toString();QUuid sourceParsed(sourceId);if(!row["source_definition_uuid"].isString()||sourceParsed.isNull()||sourceParsed.toString(QUuid::WithoutBraces)!=sourceId)throw ExchangeError("Invalid copied definition source UUID");
            for(auto sourceValue:manifest["components"].toArray())if(sourceValue.toObject()["source_uuid"]==sourceId&&sourceValue.toObject()["class_name"]=="ON_InstanceDefinition"){originalDefinition=sourceValue.toObject();break;}
            auto reference=model.ComponentFromId(ON_ModelComponent::Type::InstanceDefinition,ON_UuidFromString(sourceId.toLatin1().constData()));auto native=ON_InstanceDefinition::Cast(reference.ModelComponent());if(originalDefinition.isEmpty()||!native)throw ExchangeError("Copied definition requires an original source definition");safeUserdata(originalDefinition);
            for(auto text:originalDefinition["user_strings"].toArray())if(text.toObject()["value"].toString().contains(sourceId,Qt::CaseInsensitive))throw ExchangeError("Copied definition user text requires identity remapping");
            if(native->InstanceDefinitionType()==ON_InstanceDefinition::IDEF_UPDATE_TYPE::Linked||native->InstanceDefinitionType()==ON_InstanceDefinition::IDEF_UPDATE_TYPE::LinkedAndEmbedded||native->HasLinkedIdefReferenceComponentSettings()||!native->LinkedFileReference().FullPath().IsEmpty()||!native->LinkedFileReference().RelativePath().IsEmpty())throw ExchangeError("Copied linked definition resource semantics are not implemented yet");
            definition=*native;definition.ClearIndex();definition.ClearInstanceGeometryIdList();
        }
        if(!definition.SetId(ON_UuidFromString(uuid.toLatin1().constData()))||!definition.SetName(name.toStdWString().c_str())||!definition.SetInstanceDefinitionType(ON_InstanceDefinition::IDEF_UPDATE_TYPE::Static))throw ExchangeError("Invalid native new definition");definition.SetUnitSystem(ON::LengthUnitSystem::Millimeters);
        auto added=model.AddModelComponent(definition);if(added.IsEmpty()||id(added.ModelComponent()->Id())!=uuid||QString::fromWCharArray(added.ModelComponent()->Name().Array())!=name)throw ExchangeError("Cannot allocate new definition identity");
        QJsonObject createdDefinition=originalDefinition.isEmpty()?QJsonObject{{"class_uuid",id(definition.ClassId()->Uuid())},{"class_name","ON_InstanceDefinition"},{"component_type",QString::fromWCharArray(ON_ModelComponent::ComponentTypeToString(ON_ModelComponent::Type::InstanceDefinition).Array())},{"capability","retained"},{"user_strings",QJsonArray{}},{"userdata",QJsonArray{}},{"attribute_user_strings",QJsonArray{}},{"attribute_userdata",QJsonArray{}}}:originalDefinition;
        createdDefinition["source_uuid"]=uuid;createdDefinition["name"]=name;createdDefinition["role"]="top-level";createdDefinition["dependencies"]=QJsonArray{};createdDefinition["member_uuids"]=QJsonArray{};createdDefinition["definition_description"]=QString::fromWCharArray(definition.Description().Array());createdDefinition["definition_url"]=QString::fromWCharArray(definition.URL().Array());createdDefinition["definition_url_tag"]=QString::fromWCharArray(definition.URL_Tag().Array());components.append(createdDefinition);row.remove("source_definition_uuid");definitionEntries.append(row);
    }
    manifest["components"]=components;
    for(auto value:request["new_instances"].toArray()){
        if(!value.isObject())throw ExchangeError("Invalid new instance record");auto row=value.toObject();auto uuid=row["source_uuid"].toString(),host=row["host_id"].toString(),role=row["role"].toString(),target=row["instance_definition_uuid"].toString();QUuid parsed(uuid),targetId(target);
        for(auto key:row.keys())if(key!="namespace"&&key!="source_uuid"&&key!="host_id"&&key!="role"&&key!="instance_definition_uuid"&&key!="instance_matrix"&&key!="metadata"&&!(canonical&&key=="layer_object_id"))throw ExchangeError("Unsupported new instance field");
        if(row["namespace"]!=source["namespace"]||parsed.isNull()||parsed.toString(QUuid::WithoutBraces)!=uuid||host.isEmpty()||!hostIds.insert(host).second||!occupiedIds.insert(uuid).second||(role!="top-level"&&role!="definition-member")||targetId.isNull()||targetId.toString(QUuid::WithoutBraces)!=target||!ON_InstanceDefinition::Cast(model.ComponentFromId(ON_ModelComponent::Type::InstanceDefinition,ON_UuidFromString(target.toLatin1().constData())).ModelComponent()))throw ExchangeError("Invalid new instance identity or target");
        auto matrix=row["instance_matrix"].toArray();if(matrix.size()!=16)throw ExchangeError("New instance requires a4x4 matrix");ON_InstanceRef instance;for(int i=0;i<16;++i){if(!matrix[i].isDouble()||!std::isfinite(matrix[i].toDouble()))throw ExchangeError("Invalid new instance matrix number");instance.m_xform[i/4][i%4]=matrix[i].toDouble();}
        if(!instance.m_xform.IsValid()||!instance.m_xform.IsAffine()||!std::isfinite(instance.m_xform.Determinant())||instance.m_xform.Determinant()==0||!om9_3dm_overlay_allowed(3,2,true,true))throw ExchangeError("Invalid or singular new instance transform");instance.m_instance_definition_uuid=ON_UuidFromString(target.toLatin1().constData());
        ONX_ModelComponentIterator layers(model,ON_ModelComponent::Type::Layer);auto layer=ON_Layer::Cast(layers.FirstComponent());if(!layer)throw ExchangeError("New instance requires a document layer");ON_3dmObjectAttributes attributes;attributes.m_uuid=ON_UuidFromString(uuid.toLatin1().constData());attributes.m_layer_index=layer->Index();if(role=="definition-member")attributes.SetMode(ON::idef_object);
        auto added=model.AddModelGeometryComponent(&instance,&attributes);if(added.IsEmpty()||id(added.ModelComponent()->Id())!=uuid)throw ExchangeError("Cannot allocate new instance identity");
        records.append(QJsonObject{{"source_uuid",uuid},{"class_uuid",id(instance.ClassId()->Uuid())},{"class_name","ON_InstanceRef"},{"component_type",QString::fromWCharArray(ON_ModelComponent::ComponentTypeToString(ON_ModelComponent::Type::ModelGeometry).Array())},{"name",""},{"role",role},{"dependencies",QJsonArray{id(layer->Id()),target}},{"capability","retained"},{"geometry_crc",QString::number(instance.DataCRC(0))},{"wire_density",attributes.m_wire_density},{"instance_matrix",matrix},{"instance_definition_uuid",target},{"user_strings",QJsonArray{}},{"userdata",QJsonArray{}},{"attribute_user_strings",QJsonArray{}},{"attribute_userdata",QJsonArray{}}});
        row["action"]="instance";row.remove("role");if(role=="top-level")selected.append(row);else newMembers.append(row);
    }
    struct MemberCopyPlan{QString source;ON_Xform transform;QJsonObject overlay;std::unique_ptr<ON_Geometry> independent;bool followCanonical;QString role;};std::map<QString,MemberCopyPlan> memberCopies;std::map<QString,std::size_t> copySizes;std::size_t copyBytes=definitionCopyBytes;
    auto copySources=records;
    for(auto value:request["member_copies"].toArray()){
        if(!value.isObject())throw ExchangeError("Invalid member copy record");auto copy=value.toObject();auto sourceId=copy["source_uuid"].toString(),outputId=copy["output_uuid"].toString(),host=copy["host_id"].toString();QUuid parsed(outputId);
        if(copy["namespace"]!=source["namespace"]||parsed.isNull()||parsed.toString(QUuid::WithoutBraces)!=outputId||host.isEmpty()||!hostIds.insert(host).second||!occupiedIds.insert(outputId).second)throw ExchangeError("Invalid or colliding member copy identity");
        for(auto key:copy.keys())if(key!="namespace"&&key!="source_uuid"&&key!="output_uuid"&&key!="host_id"&&key!="member_matrix"&&key!="metadata"&&key!="independent_overlay"&&key!="follow_canonical"&&key!="role")throw ExchangeError("Unsupported member copy field");
        auto role=copy.value("role").toString("definition-member");if(copy.contains("role")&&!copy["role"].isString())throw ExchangeError("Invalid copied geometry role");if(role!="definition-member"&&role!="top-level")throw ExchangeError("Invalid copied geometry role");
        if(copy.contains("follow_canonical")&&!copy["follow_canonical"].isBool())throw ExchangeError("Invalid canonical-follow policy");bool followCanonical=copy["follow_canonical"].toBool(true);if(!followCanonical&&copy.contains("independent_overlay"))throw ExchangeError("Baseline member payload cannot also have an independent overlay");
        QJsonObject original;for(auto row:copySources)if(row.toObject()["source_uuid"]==sourceId){original=row.toObject();break;}if(original.isEmpty()||(original["role"]!="definition-member"&&original["role"]!="top-level"))throw ExchangeError("Member copy requires source geometry");safeUserdata(original);
        if(!om9_3dm_overlay_allowed(original["capability"]=="editable"?1:3,3,true,original["class_name"]=="ON_InstanceRef"))throw ExchangeError("Member copy policy rejected");
        for(auto key:{"user_strings","attribute_user_strings"})for(auto text:original[key].toArray())if(text.toObject()["value"].toString().contains(sourceId,Qt::CaseInsensitive))throw ExchangeError("Copied member user text requires identity remapping");
        auto values=copy["member_matrix"].toArray();if(values.size()!=16)throw ExchangeError("Member copy requires a4x4 matrix");ON_Xform transform;for(int i=0;i<16;++i){if(!values[i].isDouble()||!std::isfinite(values[i].toDouble()))throw ExchangeError("Invalid member copy matrix number");transform[i/4][i%4]=values[i].toDouble();}if(!transform.IsValid()||!transform.IsAffine()||!std::isfinite(transform.Determinant())||transform.Determinant()==0)throw ExchangeError("Invalid or singular member copy transform");
        auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(sourceId.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());if(!component||!component->Geometry(nullptr)||!component->Attributes(nullptr))throw ExchangeError("Missing member copy source");
        std::unique_ptr<ON_Geometry> independent;
        if(copy.contains("independent_overlay")){
            if(!copy["independent_overlay"].isObject())throw ExchangeError("Invalid independent member overlay");auto edit=copy["independent_overlay"].toObject();auto action=edit["action"].toString();
            for(auto key:edit.keys())if(key!="action"&&key!="brep"&&key!="vertices"&&key!="faces"&&key!="instance_matrix"&&key!="instance_definition_uuid"&&key!="hatch_fields"&&key!="text_dot"&&key!="point_cloud_fields"&&key!="point_cloud_sha256")throw ExchangeError("Unsupported independent member overlay field");
            validateNativeFields(edit);
            if(action=="replace"){
                if(edit.contains("instance_matrix")||edit.contains("instance_definition_uuid")||!om9_3dm_overlay_allowed(original["capability"]=="editable"?1:3,1,true,false))throw ExchangeError("Independent member replacement requires editable geometry");
                independent=replacementGeometry(edit);
                for(auto text:original["user_strings"].toArray()){auto entry=text.toObject();independent->SetUserString(entry["key"].toString().toStdWString().c_str(),entry["value"].toString().toStdWString().c_str());}
            }else if(action=="instance"){
                if(edit.contains("brep")||edit.contains("vertices")||edit.contains("faces"))throw ExchangeError("Ambiguous independent instance geometry");
                auto native=ON_InstanceRef::Cast(component->Geometry(nullptr));if(!om9_3dm_overlay_allowed(3,2,true,native!=nullptr))throw ExchangeError("Independent instance overlay requires native instance geometry");
                auto matrix=edit["instance_matrix"].toArray();if(matrix.size()!=16)throw ExchangeError("Independent instance requires a4x4 matrix");ON_Xform own;
                for(int i=0;i<16;++i){if(!matrix[i].isDouble()||!std::isfinite(matrix[i].toDouble()))throw ExchangeError("Invalid independent instance matrix number");own[i/4][i%4]=matrix[i].toDouble();}
                if(!own.IsValid()||!own.IsAffine()||!std::isfinite(own.Determinant())||own.Determinant()==0)throw ExchangeError("Invalid or singular independent instance matrix");
                independent.reset(native->Duplicate());ON_InstanceRef::Cast(independent.get())->m_xform=own;
                if(edit.contains("instance_definition_uuid"))changeInstanceDefinition(model,ON_InstanceRef::Cast(independent.get()),original,edit["instance_definition_uuid"]);
            }else if(action=="transform"){
                if(!((edit.size()==2&&(edit.contains("text_dot")||edit.contains("hatch_fields")))||(edit.size()==3&&edit.contains("point_cloud_fields")))||!om9_3dm_overlay_allowed(3,4,true,false))throw ExchangeError("Independent geometry requires known native current fields");independent.reset(component->Geometry(nullptr)->Duplicate());if(!independent)throw ExchangeError("Cannot clone independent native fields");if(edit.contains("hatch_fields"))applyHatchFields(*independent,edit["hatch_fields"],model);else if(edit.contains("text_dot"))applyTextDotFields(*independent,edit["text_dot"]);else applyPointCloudFields(*independent,std::filesystem::path(edit["point_cloud_fields"].toString().toStdWString()),edit["point_cloud_sha256"].toString());
            }else throw ExchangeError("Unsupported independent member overlay action");
            if(!independent)throw ExchangeError("Cannot clone independent copied member");transformNativeGeometry(*independent,transform,&model);
            original["class_name"]=independent->ClassId()->ClassName();original["class_uuid"]=id(independent->ClassId()->Uuid());original["geometry_crc"]=QString::number(independent->DataCRC(0));if(auto instance=ON_InstanceRef::Cast(independent.get()))original["instance_matrix"]=matrixJson(instance->m_xform);
        }
        if(!copySizes.contains(sourceId))copySizes[sourceId]=objectBytes(*component->Geometry(nullptr))+objectBytes(*component->Attributes(nullptr));auto payloadBytes=independent?objectBytes(*independent)+objectBytes(*component->Attributes(nullptr)):copySizes.at(sourceId);auto budget=om9_3dm_copy_budget(payloadBytes,1,copyBytes);if(budget<0)throw ExchangeError("Member copy data exceeds512MiB");copyBytes=budget;
        original["source_uuid"]=outputId;original["role"]=role;records.append(original);memberCopies.emplace(outputId,MemberCopyPlan{sourceId,transform,copy,std::move(independent),followCanonical,role});
    }
    manifest["records"]=records;
    if(QJsonDocument(manifest).toJson(QJsonDocument::Compact).size()>32*1024*1024)throw ExchangeError("Copied member manifest exceeds32MiB");
    for(auto& [uuid,copy]:memberCopies){auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(copy.source.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());auto attributes=*component->Attributes(nullptr);attributes.m_uuid=ON_UuidFromString(uuid.toLatin1().constData());attributes.SetMode(copy.role=="top-level"?ON::normal_object:ON::idef_object);auto added=model.AddModelGeometryComponent(copy.independent?copy.independent.get():component->Geometry(nullptr),&attributes);if(added.IsEmpty()||id(added.ModelComponent()->Id())!=uuid)throw ExchangeError("Cannot allocate copied member UUID");}
    std::vector<QJsonObject> rows;std::map<QString,std::size_t> indices;std::set<QString> geometryIds;std::map<QString,MeshData> replacementMeshes;
    for(auto value:manifest["records"].toArray())geometryIds.insert(value.toObject()["source_uuid"].toString());
    for(auto collection:{manifest["records"].toArray(),manifest["components"].toArray()})for(auto value:collection){auto row=value.toObject();auto uuid=row["source_uuid"].toString();if(indices.contains(uuid))throw ExchangeError("Duplicate source component identity");indices[uuid]=rows.size();rows.push_back(row);}
    // Reference overlays change the graph before reachability validates target
    // definition/member overlays. Unreachable edits still fail below.
    auto applyReferences=[&](const QJsonArray& entries){for(auto value:entries){auto overlay=value.toObject();if(!overlay.contains("instance_definition_uuid"))continue;auto uuid=overlay["source_uuid"].toString();if(overlay["action"]!="instance"||!geometryIds.contains(uuid))throw ExchangeError("Definition target requires an instance overlay");
        auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());safeUserdata(rows[indices.at(uuid)]);changeInstanceDefinition(model,component?ON_InstanceRef::Cast(component->ExclusiveGeometry()):nullptr,rows[indices.at(uuid)],overlay["instance_definition_uuid"]);
    }};
    applyReferences(selected);applyReferences(request["dependency_overlays"].toArray());
    for(auto& [uuid,copy]:memberCopies)if(!copy.independent&&copy.followCanonical){auto sourceRef=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(copy.source.toLatin1().constData()));auto sourceComponent=ON_ModelGeometryComponent::Cast(sourceRef.ModelComponent());if(auto sourceInstance=sourceComponent?ON_InstanceRef::Cast(sourceComponent->Geometry(nullptr)):nullptr){auto copyRef=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));auto copyComponent=ON_ModelGeometryComponent::Cast(copyRef.ModelComponent());changeInstanceDefinition(model,ON_InstanceRef::Cast(copyComponent->ExclusiveGeometry()),rows[indices.at(uuid)],id(sourceInstance->m_instance_definition_uuid));}}
    std::set<QString> definitionIds;
    for(auto value:definitionEntries){
        if(!value.isObject())throw ExchangeError("Invalid definition overlay");auto overlay=value.toObject();auto uuid=overlay["source_uuid"].toString(),host=overlay["host_id"].toString();
        if(overlay["namespace"]!=source["namespace"]||!indices.contains(uuid)||rows[indices.at(uuid)]["class_name"]!="ON_InstanceDefinition"||!definitionIds.insert(uuid).second||host.isEmpty()||!hostIds.insert(host).second||!overlay["member_uuids"].isArray())throw ExchangeError("Invalid definition overlay identity or membership");
        for(auto key:overlay.keys())if(key!="source_uuid"&&key!="namespace"&&key!="host_id"&&key!="name"&&key!="member_uuids")throw ExchangeError("Unsupported definition overlay field");
        auto& row=rows[indices.at(uuid)];safeUserdata(row);auto reference=model.ComponentFromId(ON_ModelComponent::Type::InstanceDefinition,ON_UuidFromString(uuid.toLatin1().constData()));auto definition=const_cast<ON_InstanceDefinition*>(ON_InstanceDefinition::Cast(reference.ModelComponent()));if(!definition)throw ExchangeError("Missing native definition");
        ON_SimpleArray<ON_UUID> members;QJsonArray dependencies;std::set<QString> unique;
        for(auto item:overlay["member_uuids"].toArray()){auto member=item.toString();if(!item.isString()||!indices.contains(member)||!geometryIds.contains(member)||rows[indices.at(member)]["role"]!="definition-member"||!unique.insert(member).second)throw ExchangeError("Invalid or duplicate definition member");members.Append(ON_UuidFromString(member.toLatin1().constData()));dependencies.append(member);}
        definition->SetInstanceGeometryIdList(members);row["member_uuids"]=overlay["member_uuids"];row["dependencies"]=dependencies;
        if(overlay.contains("name")){auto name=overlay["name"].toString();if(!overlay["name"].isString()||name.isEmpty()||name.size()>1024)throw ExchangeError("Invalid definition name");ONX_ModelComponentIterator definitions(model,ON_ModelComponent::Type::InstanceDefinition);for(auto component=definitions.FirstComponent();component;component=definitions.NextComponent())if(component->Id()!=definition->Id()&&QString::fromWCharArray(component->Name().Array()).compare(name,Qt::CaseInsensitive)==0)throw ExchangeError("Definition name collision");if(!definition->SetName(name.toStdWString().c_str()))throw ExchangeError("Invalid native definition name");row["name"]=name;}
    }
    // Dependency edits never become selection roots. They must already belong
    // to the selected native graph. Referenced top-level owners accept metadata
    // only; geometry edits need verified topology/domain-preserving overlays.
    std::set<QString> reachable,dependencyIds;std::vector<QString> pending;
    for(auto value:selected){auto uuid=value.toObject()["source_uuid"].toString();if(!indices.contains(uuid)||rows[indices.at(uuid)]["role"]!="top-level")throw ExchangeError("Selection requires source top-level geometry");pending.push_back(uuid);}
    while(!pending.empty()){auto uuid=pending.back();pending.pop_back();if(!reachable.insert(uuid).second)continue;if(!indices.contains(uuid))throw ExchangeError("Missing dependency overlay reference");for(auto dependency:rows[indices.at(uuid)]["dependencies"].toArray())pending.push_back(dependency.toString());}
    for(auto& uuid:definitionIds)if(!reachable.contains(uuid))throw ExchangeError("Definition overlay is not reachable from selected instances");
    std::set<QString> copySourceIds;for(auto& [uuid,copy]:memberCopies){if(!reachable.contains(uuid))throw ExchangeError("Member copy is not reachable from selected definitions");copySourceIds.insert(copy.source);}
    std::set<QString> nativeOwnerIds;
    for(auto& uuid:reachable)for(auto key:{"poly_edge_native","curve_on_surface_native"})for(auto owner:rows[indices.at(uuid)].value(key).toObject().value("object_references").toArray())nativeOwnerIds.insert(owner.toString());
    auto overlays=selected;
    for(auto value:request["dependency_overlays"].toArray()){
        auto overlay=value.toObject();auto uuid=overlay["source_uuid"].toString(),host=overlay["host_id"].toString(),action=overlay["action"].toString();
        const bool nativeOwner=indices.contains(uuid)&&geometryIds.contains(uuid)&&rows[indices.at(uuid)]["role"]=="top-level"&&nativeOwnerIds.contains(uuid);
        if(nativeOwner){if(action!="unchanged")throw ExchangeError("Native reference owner geometry overlay requires verified topology/domain mapping");for(auto key:overlay.keys())if(key!="namespace"&&key!="source_uuid"&&key!="host_id"&&key!="action"&&key!="metadata")throw ExchangeError("Unsupported native owner metadata overlay field");}
        if(overlay["namespace"]!=source["namespace"]||(!reachable.contains(uuid)&&!copySourceIds.contains(uuid))||!indices.contains(uuid)||(!nativeOwner&&rows[indices.at(uuid)]["role"]!="definition-member")||!dependencyIds.insert(uuid).second||host.isEmpty()||!hostIds.insert(host).second||(action!="unchanged"&&action!="replace"&&action!="instance"&&action!="transform"))throw ExchangeError("Invalid or unreachable definition-member overlay");
        overlays.append(overlay);
    }
    for(auto value:newMembers){auto uuid=value.toObject()["source_uuid"].toString();if(!reachable.contains(uuid)||!dependencyIds.insert(uuid).second)throw ExchangeError("New member is not reachable from selected definitions");overlays.append(value);}
    for(auto value:overlays){auto selection=value.toObject();if(selection["action"]!="replace")continue;auto uuid=selection["source_uuid"].toString();if(!geometryIds.contains(uuid))throw ExchangeError("Missing replacement source record");auto& row=rows[indices.at(uuid)];if(!om9_3dm_overlay_allowed(row["capability"]=="editable"?1:3,1,true,false))throw ExchangeError("Replacement requires editable geometry");safeUserdata(row);
        auto mesh=replacementGeometry(selection);if(auto nativeMesh=ON_Mesh::Cast(mesh.get()))replacementMeshes[uuid]=importMesh(*nativeMesh);
        for(auto text:row["user_strings"].toArray()){auto entry=text.toObject();mesh->SetUserString(entry["key"].toString().toStdWString().c_str(),entry["value"].toString().toStdWString().c_str());}
        auto sourceRef=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));auto sourceGeometry=ON_ModelGeometryComponent::Cast(sourceRef.ModelComponent());if(!sourceGeometry||!sourceGeometry->Attributes(nullptr))throw ExchangeError("Missing replacement attributes");auto attributes=*sourceGeometry->Attributes(nullptr);if(model.RemoveModelComponent(ON_ModelComponent::Type::ModelGeometry,sourceGeometry->Id()).IsEmpty())throw ExchangeError("Cannot replace source geometry");auto added=model.AddModelGeometryComponent(mesh.get(),&attributes);if(added.IsEmpty()||id(added.ModelComponent()->Id())!=uuid)throw ExchangeError("Replacement changed UUID");
        row["class_name"]=mesh->ClassId()->ClassName();row["class_uuid"]=id(mesh->ClassId()->Uuid());row["geometry_crc"]=QString::number(mesh->DataCRC(0));
    }
    std::vector<std::size_t> offsets{0},edges,roots;
    auto layerForPath=[&](const QJsonValue& value)->const ON_Layer*{
        if(!value.isString())throw ExchangeError("Layer path must be text");auto path=value.toString();auto names=path.split("::",Qt::KeepEmptyParts);if(path.size()>1024||names.size()>64||path.isEmpty())throw ExchangeError("Invalid or oversized layer path");
        ON_UUID parent=ON_nil_uuid;const ON_Layer* current=nullptr;
        for(auto& name:names){if(name.isEmpty()||name!=name.trimmed())throw ExchangeError("Invalid empty or padded layer name");current=nullptr;
            ONX_ModelComponentIterator layers(model,ON_ModelComponent::Type::Layer);for(auto component=layers.FirstComponent();component;component=layers.NextComponent()){auto layer=ON_Layer::Cast(component);if(layer&&layer->ParentId()==parent&&QString::fromWCharArray(layer->Name().Array()).compare(name,Qt::CaseInsensitive)==0){if(current)throw ExchangeError("Ambiguous layer path");current=layer;}}
            if(!current){ON_Layer layer;ON_UUID uuid;ON_CreateUuid(uuid);if(!layer.SetId(uuid)||!layer.SetName(name.toStdWString().c_str()))throw ExchangeError("Invalid native layer name");layer.SetParentLayerId(parent);auto added=model.AddModelComponent(layer);current=ON_Layer::Cast(added.ModelComponent());if(!current||current->Id()!=uuid||QString::fromWCharArray(current->Name().Array())!=name)throw ExchangeError("Cannot allocate requested layer");
                QJsonArray dependencies;if(parent!=ON_nil_uuid)dependencies.append(id(parent));ON_wString type=ON_ModelComponent::ComponentTypeToString(ON_ModelComponent::Type::Layer);
                QJsonObject row{{"source_uuid",id(uuid)},{"class_uuid",id(current->ClassId()->Uuid())},{"class_name","ON_Layer"},{"component_type",QString::fromWCharArray(type.Array())},{"name",name},{"role","top-level"},{"dependencies",dependencies},{"capability","retained"},{"user_strings",QJsonArray{}},{"userdata",QJsonArray{}},{"attribute_user_strings",QJsonArray{}},{"attribute_userdata",QJsonArray{}}};indices[id(uuid)]=rows.size();rows.push_back(row);
            }
            parent=current->Id();
        }
        return current;
    };
    auto applyMetadata=[&](const QJsonArray& entries){for(auto value:entries){auto selection=value.toObject();if(!selection.contains("metadata"))continue;if(!selection["metadata"].isObject())throw ExchangeError("Invalid preservation metadata overlay");auto metadata=selection["metadata"].toObject();
        for(auto key:metadata.keys())if(key!="name"&&key!="color"&&key!="visible"&&key!="locked"&&key!="layer")throw ExchangeError("Unsupported preservation metadata overlay");
        // Canonical roots carry own state in Rust, while flat UI projections
        // contain inherited state. Only their current name is a flat edit.
        if(canonical&&selection.contains("layer_object_id"))for(auto key:{"layer","color","locked","visible"})metadata.remove(key);
        auto uuid=selection["source_uuid"].toString();if(!geometryIds.contains(uuid))throw ExchangeError("Missing metadata source record");auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());auto attributes=component?component->ExclusiveAttributes():nullptr;if(!attributes)throw ExchangeError("Cannot update source attributes");
        if(metadata.contains("name")){if(!metadata["name"].isString())throw ExchangeError("Invalid source name");auto name=metadata["name"].toString().toStdWString();attributes->m_name=name.c_str();if(!const_cast<ON_ModelGeometryComponent*>(component)->SetName(name.c_str()))throw ExchangeError("Cannot update source component name");rows[indices.at(uuid)]["name"]=metadata["name"];}
        if(metadata.contains("layer")){auto layer=layerForPath(metadata["layer"]);attributes->m_layer_index=layer->Index();auto& row=rows[indices.at(uuid)];QJsonArray dependencies;dependencies.append(id(layer->Id()));for(auto dependency:row["dependencies"].toArray())if(rows[indices.at(dependency.toString())]["class_name"]!="ON_Layer")dependencies.append(dependency);row["dependencies"]=dependencies;}
        if(metadata.contains("color")){auto channels=metadata["color"].toArray();if(channels.size()!=3)throw ExchangeError("Invalid source color");int color[3];for(int i=0;i<3;++i){auto number=channels[i].toDouble(-1);if(!channels[i].isDouble()||!std::isfinite(number)||number<0||number>255||number!=std::floor(number))throw ExchangeError("Invalid source color channel");color[i]=static_cast<int>(number);}attributes->m_color=ON_Color(color[0],color[1],color[2]);attributes->SetColorSource(ON::color_from_object);}
        if(metadata.contains("visible")){if(!metadata["visible"].isBool())throw ExchangeError("Invalid source visibility");attributes->SetVisible(metadata["visible"].toBool());}
        if(metadata.contains("locked")){if(!metadata["locked"].isBool())throw ExchangeError("Invalid source lock");auto visible=attributes->IsVisible();if(!attributes->IsInstanceDefinitionObject())attributes->SetMode(metadata["locked"].toBool()?ON::locked_object:ON::normal_object);attributes->SetVisible(visible);attributes->SetUserString(L"OpenMatrix9.Locked",metadata["locked"].toBool()?L"1":L"0");auto& row=rows[indices.at(uuid)];QJsonArray strings;for(auto entry:row["attribute_user_strings"].toArray())if(entry.toObject()["key"]!="OpenMatrix9.Locked")strings.append(entry);strings.append(QJsonObject{{"key","OpenMatrix9.Locked"},{"value",metadata["locked"].toBool()?"1":"0"}});row["attribute_user_strings"]=strings;if(row["attribute_userdata"].toArray().isEmpty()){char uuid[37]{};ON_UuidToString(ON_UserStringList().ClassId()->Uuid(),uuid);row["attribute_userdata"]=QJsonArray{QJsonObject{{"class_uuid",uuid},{"class_name","ON_UserStringList"},{"capability","retained"}}};}}
    }};
    applyMetadata(overlays);
    std::set<QString> selectedIds;
    std::map<QString,ON_Xform> copiedMemberTransforms;std::map<QString,QJsonValue> copiedTextDotFields,copiedHatchFields;std::map<QString,QJsonObject> copiedPointCloudFields;
    for(auto value:overlays){auto row=value.toObject();auto uuid=row["source_uuid"].toString();auto action=row["action"].toString();if(row["namespace"]!=source["namespace"]||(action!="unchanged"&&action!="instance"&&action!="replace"&&action!="transform")||!geometryIds.contains(uuid)||!selectedIds.insert(uuid).second)throw ExchangeError("Invalid or unsupported source overlay");
        validateNativeFields(row);
        if(action=="transform"){
            for(auto key:row.keys())if(key!="namespace"&&key!="host_id"&&key!="source_uuid"&&key!="action"&&key!="metadata"&&key!="geometry_matrix"&&key!="hatch_fields"&&key!="text_dot"&&key!="point_cloud_fields"&&key!="point_cloud_sha256"&&!(canonical&&key=="layer_object_id"))throw ExchangeError("Ambiguous native geometry transform overlay field");
            auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());auto geometry=component?component->ExclusiveGeometry():nullptr;
            if(!geometry||!om9_3dm_overlay_allowed(rows[indices.at(uuid)]["capability"]=="editable"?1:3,4,true,ON_InstanceRef::Cast(geometry)!=nullptr))throw ExchangeError("Native geometry transform requires a supported non-instance payload");
            safeUserdata(rows[indices.at(uuid)]);auto matrix=row["geometry_matrix"].toArray();if(matrix.size()!=16)throw ExchangeError("Native geometry transform requires a4x4 matrix");ON_Xform transform;
            if(row.contains("text_dot")){if(memberCopies.contains(uuid))copiedTextDotFields.emplace(uuid,row["text_dot"]);else applyTextDotFields(*geometry,row["text_dot"]);}
            if(row.contains("hatch_fields")){if(memberCopies.contains(uuid))copiedHatchFields.emplace(uuid,row["hatch_fields"]);else applyHatchFields(*geometry,row["hatch_fields"],model);}
            if(row.contains("point_cloud_fields")){if(memberCopies.contains(uuid))copiedPointCloudFields.emplace(uuid,row);else applyPointCloudFields(*geometry,std::filesystem::path(row["point_cloud_fields"].toString().toStdWString()),row["point_cloud_sha256"].toString());}
            for(int i=0;i<16;++i){if(!matrix[i].isDouble()||!std::isfinite(matrix[i].toDouble()))throw ExchangeError("Invalid native geometry transform matrix number");transform[i/4][i%4]=matrix[i].toDouble();}
            // Canonical edits feed copies, while output-member deltas follow
            // the copy's own local matrix rather than preceding it.
            if(memberCopies.contains(uuid))copiedMemberTransforms.emplace(uuid,transform);
            else {transformNativeGeometry(*geometry,transform,&model);rows[indices.at(uuid)]["geometry_crc"]=QString::number(geometry->DataCRC(0));}
        }
        if(action=="instance"){
            auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());auto instance=component?ON_InstanceRef::Cast(component->ExclusiveGeometry()):nullptr;
            if(!om9_3dm_overlay_allowed(3,2,true,instance!=nullptr))throw ExchangeError("Instance overlay requires native instance geometry");
            auto matrix=row["instance_matrix"].toArray();if(matrix.size()!=16)throw ExchangeError("Instance overlay requires a4x4 matrix");ON_Xform transform;
            for(int i=0;i<16;++i){if(!matrix[i].isDouble()||!std::isfinite(matrix[i].toDouble()))throw ExchangeError("Invalid instance matrix number");transform[i/4][i%4]=matrix[i].toDouble();}
            if(!transform.IsValid()||!transform.IsAffine()||!std::isfinite(transform.Determinant())||transform.Determinant()==0)throw ExchangeError("Invalid or singular instance overlay transform");
            safeUserdata(rows[indices.at(uuid)]);instance->m_xform=transform;rows[indices.at(uuid)]["geometry_crc"]=QString::number(instance->DataCRC(0));rows[indices.at(uuid)]["instance_matrix"]=matrixJson(transform);
        }
        if(!dependencyIds.contains(uuid))roots.push_back(indices.at(uuid));}
    QJsonArray copyMetadata;
    copyBytes=currentDefinitionCopyBytes();copySizes.clear();for(auto& [uuid,copy]:memberCopies){auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString((copy.independent?uuid:copy.source).toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());if(!component||!component->Geometry(nullptr)||!component->Attributes(nullptr))throw ExchangeError("Missing edited member copy source");auto budgetKey=copy.independent?uuid:copy.source;if(!copySizes.contains(budgetKey))copySizes[budgetKey]=objectBytes(*component->Geometry(nullptr))+objectBytes(*component->Attributes(nullptr));auto budget=om9_3dm_copy_budget(copySizes.at(budgetKey),1,copyBytes);if(budget<0)throw ExchangeError("Edited member copy data exceeds512MiB");copyBytes=budget;}
    for(auto& [uuid,copy]:memberCopies){
        if(copy.independent){
            if(auto mesh=ON_Mesh::Cast(copy.independent.get()))replacementMeshes[uuid]=importMesh(*mesh);
            if(copy.overlay.contains("metadata")){auto metadata=copy.overlay;metadata["source_uuid"]=uuid;copyMetadata.append(metadata);}
            continue;
        }
        auto payloadId=copy.followCanonical?copy.source:uuid;auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(payloadId.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());if(!component||!component->Geometry(nullptr)||!component->Attributes(nullptr))throw ExchangeError("Missing edited member copy source");
        auto original=rows[indices.at(payloadId)];safeUserdata(original);for(auto key:{"user_strings","attribute_user_strings"})for(auto text:original[key].toArray())if(text.toObject()["value"].toString().contains(copy.source,Qt::CaseInsensitive))throw ExchangeError("Edited copied member user text requires identity remapping");
        std::unique_ptr<ON_Geometry> geometry(component->Geometry(nullptr)->Duplicate());if(!geometry)throw ExchangeError("Cannot clone copied native member");transformNativeGeometry(*geometry,copy.transform,&model);auto attributes=*component->Attributes(nullptr);attributes.m_uuid=ON_UuidFromString(uuid.toLatin1().constData());attributes.SetMode(copy.role=="top-level"?ON::normal_object:ON::idef_object);if(model.RemoveModelComponent(ON_ModelComponent::Type::ModelGeometry,attributes.m_uuid).IsEmpty())throw ExchangeError("Cannot refresh copied native member");auto added=model.AddModelGeometryComponent(geometry.get(),&attributes);if(added.IsEmpty()||id(added.ModelComponent()->Id())!=uuid)throw ExchangeError("Edited member copy changed UUID");
        original["source_uuid"]=uuid;original["role"]=copy.role;original["geometry_crc"]=QString::number(geometry->DataCRC(0));if(auto instance=ON_InstanceRef::Cast(geometry.get()))original["instance_matrix"]=matrixJson(instance->m_xform);if(auto mesh=ON_Mesh::Cast(geometry.get()))replacementMeshes[uuid]=importMesh(*mesh);rows[indices.at(uuid)]=original;
        if(copy.overlay.contains("metadata")){auto metadata=copy.overlay;metadata["source_uuid"]=uuid;copyMetadata.append(metadata);}
    }
    for(auto& [uuid,transform]:copiedMemberTransforms){auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());auto geometry=component?component->ExclusiveGeometry():nullptr;if(!geometry)throw ExchangeError("Missing final retained member transform payload");if(copiedTextDotFields.contains(uuid))applyTextDotFields(*geometry,copiedTextDotFields.at(uuid));if(copiedHatchFields.contains(uuid))applyHatchFields(*geometry,copiedHatchFields.at(uuid),model);if(copiedPointCloudFields.contains(uuid)){auto fields=copiedPointCloudFields.at(uuid);applyPointCloudFields(*geometry,std::filesystem::path(fields["point_cloud_fields"].toString().toStdWString()),fields["point_cloud_sha256"].toString());}transformNativeGeometry(*geometry,transform,&model);rows[indices.at(uuid)]["geometry_crc"]=QString::number(geometry->DataCRC(0));}
    applyMetadata(copyMetadata);
    // Account for transformed geometry and copy-specific metadata too, before
    // writing any destination bytes. This supplements the pre-clone budget.
    copyBytes=currentDefinitionCopyBytes();for(auto& [uuid,copy]:memberCopies){auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());auto budget=om9_3dm_copy_budget(objectBytes(*component->Geometry(nullptr))+objectBytes(*component->Attributes(nullptr)),1,copyBytes);if(budget<0)throw ExchangeError("Final member copy data exceeds512MiB");copyBytes=budget;}
    // Affine hatches own derived pattern identities. Register those components
    // and refresh each current native reference before selection closure. This
    // also covers copied canonical/independent members and repeated transforms.
    ONX_ModelComponentIterator patterns(model,ON_ModelComponent::Type::HatchPattern);
    for(auto component=patterns.FirstComponent();component;component=patterns.NextComponent())if(!indices.contains(id(component->Id()))){auto pattern=ON_HatchPattern::Cast(component);if(!pattern)throw ExchangeError("Invalid generated hatch pattern component");indices[id(pattern->Id())]=rows.size();rows.push_back(hatchPatternRecord(*pattern));}
    for(auto& row:rows)if(row["class_name"]=="ON_Hatch"){
        auto component=ON_ModelGeometryComponent::Cast(model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(row["source_uuid"].toString().toLatin1().constData())).ModelComponent());auto hatch=component?ON_Hatch::Cast(component->Geometry(nullptr)):nullptr;if(!hatch)throw ExchangeError("Missing current native hatch");QJsonArray dependencies;
        for(auto dependency:row["dependencies"].toArray()){auto found=indices.find(dependency.toString());if(found==indices.end()||rows[found->second]["class_name"]!="ON_HatchPattern")dependencies.append(dependency);}
        if(hatch->PatternIndex()>=0){auto pattern=model.ComponentFromIndex(ON_ModelComponent::Type::HatchPattern,hatch->PatternIndex()).ModelComponent();if(!pattern||!indices.contains(id(pattern->Id())))throw ExchangeError("Missing current native hatch pattern dependency");dependencies.append(id(pattern->Id()));}row["dependencies"]=dependencies;
    }
    std::optional<NativeLayerTable> retainedPalette;
    if(canonical){
        std::vector<NativeLayerObjectBinding> bindings;bindings.reserve(selected.size());
        for(auto value:selected){auto entry=value.toObject();if(!entry["layer_object_id"].isString()||entry["layer_object_id"].toString().isEmpty())throw ExchangeError("Canonical selected object requires its exact layer object binding");bindings.push_back({entry["source_uuid"].toString().toStdString(),entry["layer_object_id"].toString().toStdString()});}
        auto applied=applyRetainedLayerOverlay(model,layerSnapshot.get(),bindings);
        auto userStrings=[](const ON_Object& object){QJsonArray result;ON_ClassArray<ON_UserString> values;object.GetUserStrings(values);for(int i=0;i<values.Count();++i)result.append(QJsonObject{{"key",QString::fromWCharArray(values[i].m_key.Array())},{"value",QString::fromWCharArray(values[i].m_string_value.Array())}});return result;};
        auto userData=[](const ON_Object& object){QJsonArray result;for(auto data=object.FirstUserData();data;data=data->Next())result.append(QJsonObject{{"class_uuid",id(data->ClassId()->Uuid())},{"class_name",data->ClassId()->ClassName()},{"capability","retained"}});return result;};
        ONX_ModelComponentIterator layers(model,ON_ModelComponent::Type::Layer);
        for(auto component=layers.FirstComponent();component;component=layers.NextComponent()){
            auto layer=ON_Layer::Cast(component);if(!layer)throw ExchangeError("Invalid current retained layer");auto uuid=id(layer->Id());QJsonArray dependencies;
            if(layer->ParentId()!=ON_nil_uuid)dependencies.append(id(layer->ParentId()));
            if(layer->RenderMaterialIndex()>=0){auto material=model.ComponentFromIndex(ON_ModelComponent::Type::Material,layer->RenderMaterialIndex()).ModelComponent();if(!material)throw ExchangeError("Missing retained layer material");dependencies.append(id(material->Id()));}
            if(layer->LinetypeIndex()>=0){auto pattern=model.ComponentFromIndex(ON_ModelComponent::Type::LinePattern,layer->LinetypeIndex()).ModelComponent();if(!pattern)throw ExchangeError("Missing retained layer line pattern");dependencies.append(id(pattern->Id()));}
            QJsonObject row{{"source_uuid",uuid},{"class_uuid",id(layer->ClassId()->Uuid())},{"class_name","ON_Layer"},{"component_type",QString::fromWCharArray(ON_ModelComponent::ComponentTypeToString(ON_ModelComponent::Type::Layer).Array())},{"name",QString::fromWCharArray(layer->Name().Array())},{"role","top-level"},{"dependencies",dependencies},{"capability","retained"},{"user_strings",userStrings(*layer)},{"userdata",userData(*layer)},{"attribute_user_strings",QJsonArray{}},{"attribute_userdata",QJsonArray{}}};
            if(indices.contains(uuid))rows[indices.at(uuid)]=row;else{indices[uuid]=rows.size();rows.push_back(row);}
            // Empty layers belong to the session palette, so selection closure
            // retains them even when no selected geometry references them.
            roots.push_back(indices.at(uuid));
        }
        for(const auto& object:applied.objects){
            auto uuid=QString::fromStdString(object.id);auto component=ON_ModelGeometryComponent::Cast(model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(object.id.c_str())).ModelComponent());auto attributes=component?component->Attributes(nullptr):nullptr;if(!attributes)throw ExchangeError("Missing canonical retained attributes");
            auto& row=rows[indices.at(uuid)];QJsonArray dependencies;dependencies.append(QString::fromStdString(object.layerId));
            for(auto dependency:row["dependencies"].toArray())if(rows[indices.at(dependency.toString())]["class_name"]!="ON_Layer")dependencies.append(dependency);
            row["dependencies"]=dependencies;row["attribute_user_strings"]=userStrings(*attributes);row["attribute_userdata"]=userData(*attributes);
        }
        retainedPalette=readNativeLayerTable(model);
    }
    if(rows.size()>1000000)throw ExchangeError("Generated component count exceeds preservation limit");
    QJsonArray currentRows;for(auto& row:rows)currentRows.append(row);if(QJsonDocument(currentRows).toJson(QJsonDocument::Compact).size()>32*1024*1024)throw ExchangeError("Current generated component manifest exceeds32MiB");
    for(auto& row:rows){for(auto value:row["dependencies"].toArray()){auto found=indices.find(value.toString());if(found==indices.end())throw ExchangeError("Missing preservation dependency");edges.push_back(found->second);}offsets.push_back(edges.size());}
    auto settingRoot=[&](ON_ModelComponent::Type type,ON_UUID uuid,int index){auto reference=uuid==ON_nil_uuid?model.ComponentFromIndex(type,index):model.ComponentFromId(type,uuid);if(auto component=reference.ModelComponent()){auto found=indices.find(id(component->Id()));if(found!=indices.end())roots.push_back(found->second);}};
    settingRoot(ON_ModelComponent::Type::Layer,model.m_settings.CurrentLayerId(),model.m_settings.CurrentLayerIndex());
    settingRoot(ON_ModelComponent::Type::Material,model.m_settings.CurrentMaterialId(),model.m_settings.CurrentMaterialIndex());
    settingRoot(ON_ModelComponent::Type::LinePattern,model.m_settings.CurrentLinePatternId(),model.m_settings.CurrentLinePatternIndex());
    settingRoot(ON_ModelComponent::Type::DimStyle,model.m_settings.CurrentDimensionStyleId(),model.m_settings.CurrentDimensionStyleIndex());
    if(model.m_settings.CurrentDimensionStyleId()==ON_nil_uuid){ONX_ModelComponentIterator styles(model,ON_ModelComponent::Type::DimStyle);if(auto style=styles.FirstComponent()){roots.push_back(indices.at(id(style->Id())));model.m_settings.SetCurrentDimensionStyleId(style->Id());}}
    auto size=om9_3dm_dependency_closure(rows.size(),offsets.data(),edges.data(),edges.size(),roots.data(),roots.size(),nullptr,0);
    if(size<0)throw ExchangeError("Invalid or cyclic preservation dependency graph");
    std::vector<std::size_t> closure(size);if(om9_3dm_dependency_closure(rows.size(),offsets.data(),edges.data(),edges.size(),roots.data(),roots.size(),closure.data(),closure.size())!=size)throw ExchangeError("Cannot compute preservation closure");
    std::set<QString> keep;for(auto index:closure){safeUserdata(rows[index]);keep.insert(rows[index]["source_uuid"].toString());}
    for(auto& uuid:keep){auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());auto cloud=component?ON_PointCloud::Cast(component->Geometry(nullptr)):nullptr;if(cloud)validateRhino5PointCloud(*cloud);}
    for(auto index:closure)for(auto key:{"user_strings","attribute_user_strings"})for(auto text:rows[index][key].toArray())for(auto& [uuid,recordIndex]:indices)if(!keep.contains(uuid)&&text.toObject()["value"].toString().contains(uuid,Qt::CaseInsensitive))throw ExchangeError("User text may reference an omitted source component: "+uuid.toStdString());
    std::function<void(const ON_ModelGeometryComponent*,unsigned)> checkBlock;
    checkBlock=[&](const ON_ModelGeometryComponent* component,unsigned depth){if(!component||!component->Geometry(nullptr))throw ExchangeError("Missing block geometry");if(auto instance=ON_InstanceRef::Cast(component->Geometry(nullptr))){if(depth>=64)throw ExchangeError("Block nesting exceeds64 levels");auto transform=instance->m_xform;for(int i=0;i<3;++i)if(std::abs(transform[3][i])<=1e-12)transform[3][i]=0;if(std::abs(transform[3][3]-1)<=1e-12)transform[3][3]=1;if(!transform.IsValid()||!transform.IsAffine()||transform.Determinant()==0)throw ExchangeError("Invalid or singular block transform");auto reference=model.ComponentFromId(ON_ModelComponent::Type::InstanceDefinition,instance->m_instance_definition_uuid);auto definition=ON_InstanceDefinition::Cast(reference.ModelComponent());if(!definition)throw ExchangeError("Missing block definition");auto& members=definition->InstanceGeometryIdList();for(int i=0;i<members.Count();++i){auto member=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,members[i]);checkBlock(ON_ModelGeometryComponent::Cast(member.ModelComponent()),depth+1);}}};
    for(auto& uuid:selectedIds){auto component=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));if(!component.IsEmpty())checkBlock(ON_ModelGeometryComponent::Cast(component.ModelComponent()),0);}
    bool changedBlockGeometry=!definitionIds.empty()||!memberCopies.empty();for(auto overlay:overlays)if(overlay.toObject()["action"]=="replace"||overlay.toObject()["action"]=="instance"||overlay.toObject()["action"]=="transform")changedBlockGeometry=true;
    if(changedBlockGeometry){
        std::map<QString,ON_BoundingBox> definitionBounds;
        std::function<ON_BoundingBox(ON_Geometry*,unsigned)> bounds;
        bounds=[&](ON_Geometry* geometry,unsigned depth)->ON_BoundingBox{
            if(!geometry||depth>64)throw ExchangeError("Invalid block bounds dependency");
            if(auto instance=ON_InstanceRef::Cast(geometry)){
                auto uuid=id(instance->m_instance_definition_uuid);ON_BoundingBox box;
                auto cached=definitionBounds.find(uuid);
                if(cached!=definitionBounds.end())box=cached->second;
                else{
                    auto reference=model.ComponentFromId(ON_ModelComponent::Type::InstanceDefinition,instance->m_instance_definition_uuid);auto definition=const_cast<ON_InstanceDefinition*>(ON_InstanceDefinition::Cast(reference.ModelComponent()));if(!definition)throw ExchangeError("Missing definition for block bounds");
                    auto& members=definition->InstanceGeometryIdList();for(int i=0;i<members.Count();++i){auto member=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,members[i]);auto component=ON_ModelGeometryComponent::Cast(member.ModelComponent());if(!component)throw ExchangeError("Missing block bounds member");auto geometry=component->ExclusiveGeometry();auto memberBox=bounds(geometry,depth+1);if(!memberBox.IsValid()&&!ON_InstanceRef::Cast(geometry))throw ExchangeError("Invalid member bounds after overlay");if(memberBox.IsValid())box.Union(memberBox);}
                    definition->SetBoundingBox(box);definitionBounds[uuid]=box;
                }
                if(box.IsValid()&&!box.Transform(instance->m_xform))throw ExchangeError("Cannot transform block bounds");instance->m_bbox=box;return box;
            }
            return geometry->BoundingBox();
        };
        for(auto value:selected){auto uuid=value.toObject()["source_uuid"].toString();auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());bounds(component->ExclusiveGeometry(),0);}
        for(auto& row:rows)if(row["class_name"]=="ON_InstanceRef"&&keep.contains(row["source_uuid"].toString())){auto reference=model.ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(row["source_uuid"].toString().toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());row["geometry_crc"]=QString::number(component->Geometry(nullptr)->DataCRC(0));}
    }
    auto currentGraph=currentSurfaceReferenceGraph(model,&keep);
    std::map<QString,QByteArray> geometryDigests,attributeDigests;
    for(auto type:{ON_ModelComponent::Type::ModelGeometry,ON_ModelComponent::Type::RenderLight}){ONX_ModelComponentIterator it(model,type);for(auto component=it.FirstComponent();component;component=it.NextComponent())if(keep.contains(id(component->Id()))){auto geometry=ON_ModelGeometryComponent::Cast(component);auto attributes=geometry?geometry->Attributes(nullptr):nullptr;auto native=geometry?geometry->Geometry(nullptr):nullptr;if(!native||!attributes)throw ExchangeError("Invalid retained native geometry");if(attributes->m_rendering_attributes.m_materials.Count()||attributes->m_rendering_attributes.m_mappings.Count())throw ExchangeError("Object rendering/mapping dependencies are not implemented yet");
        if(!(ON_Point::Cast(native)||ON_TextDot::Cast(native)||ON_PointCloud::Cast(native)||ON_Hatch::Cast(native)||ON_Curve::Cast(native)||ON_Brep::Cast(native)||ON_Surface::Cast(native)||ON_Mesh::Cast(native)||ON_InstanceRef::Cast(native)||ON_Light::Cast(native)))throw ExchangeError("Unverified native dependency semantics for "+std::string(native->ClassId()->ClassName()));
        auto& row=rows[indices.at(id(component->Id()))];row.remove("text_dot");row.remove("point_cloud");auto facts=nativeGeometryFacts(*native);for(auto key:facts.keys())row[key]=facts[key];
        if(auto hatch=ON_Hatch::Cast(native)){row["hatch_current"]=hatchCurrentFields(*hatch,model);row["hatch_loop_current"]=hatchLoopFields(*hatch);row.remove("hatch_current_unavailable");}
        geometryDigests[id(component->Id())]=currentObjectDigest(*native,id(component->Id()),*currentGraph);
        attributeDigests[id(component->Id())]=objectDigest(*attributes);
    }}
    for(unsigned type=1;type<static_cast<unsigned>(ON_ModelComponent::Type::NumOf);++type){if(type==16)continue;auto kind=static_cast<ON_ModelComponent::Type>(type);std::vector<ON_UUID> remove;ONX_ModelComponentIterator iterator(model,kind);for(auto component=iterator.FirstComponent();component;component=iterator.NextComponent())if(!keep.contains(id(component->Id())))remove.push_back(component->Id());for(auto uuid:remove)if(model.RemoveModelComponent(kind,uuid).IsEmpty())throw ExchangeError("Cannot remove unselected source component");}
    auto staged=destination;staged+=L"."+QUuid::createUuid().toString(QUuid::WithoutBraces).toStdWString()+L".tmp.3dm";
    try{
        if(!writeSelectedModel(model,staged,purpose))throw ExchangeError("Cannot write selected native graph");
        auto checked=inspectArchive(staged);auto checkedGraph=currentSurfaceReferenceGraph(*checked.nativeModel);auto verified=checked.document;auto version=verified["source_version"].toInt();if(purpose==ArchivePurpose::GeometryStaging?version!=80:(version!=50&&version!=5))throw ExchangeError("Selected archive version does not match its purpose");
        if(!verified["issues"].toArray().isEmpty())throw ExchangeError("Written archive has unresolved references");
        if(retainedPalette)verifyRetainedPalette(*retainedPalette,*checked.nativeModel);
        std::map<QString,QJsonObject> output;for(auto collection:{verified["records"].toArray(),verified["components"].toArray()})for(auto value:collection){auto row=value.toObject();output[row["source_uuid"].toString()]=row;}
        if(output.size()!=keep.size()){std::string details;for(auto& [uuid,row]:output)if(!keep.contains(uuid))details+=" added "+uuid.toStdString()+" "+row["class_name"].toString().toStdString();for(auto& uuid:keep)if(!output.contains(uuid))details+=" missing "+uuid.toStdString();throw ExchangeError("Rhino5 write dropped or added source components:"+details);}
        for(auto index:closure){auto original=rows[index];auto found=output.find(original["source_uuid"].toString());if(found==output.end()||found->second!=original){std::string fields;if(found!=output.end()){for(auto key:original.keys())if(original[key]!=found->second[key])fields+=" "+key.toStdString();for(auto key:found->second.keys())if(!original.contains(key))fields+=" added:"+key.toStdString();}throw ExchangeError("Rhino5 write changed retained component semantics: "+original["source_uuid"].toString().toStdString()+fields);}}
        // Fresh meshes acquire derived archive caches on read; verify the requested
        // double-precision vertices and topology independently of those caches.
        for(auto& [uuid,expected]:replacementMeshes){if(!keep.contains(uuid))continue;auto reference=checked.nativeModel->ComponentFromId(ON_ModelComponent::Type::ModelGeometry,ON_UuidFromString(uuid.toLatin1().constData()));auto component=ON_ModelGeometryComponent::Cast(reference.ModelComponent());auto mesh=component?ON_Mesh::Cast(component->Geometry(nullptr)):nullptr;if(!mesh)throw ExchangeError("Replacement mesh missing after write");auto actual=importMesh(*mesh);if(actual.vertices!=expected.vertices||actual.faces!=expected.faces)throw ExchangeError("Replacement mesh precision or topology changed");geometryDigests[uuid]=objectDigest(*mesh);}
        for(auto type:{ON_ModelComponent::Type::ModelGeometry,ON_ModelComponent::Type::RenderLight}){ONX_ModelComponentIterator it(*checked.nativeModel,type);for(auto component=it.FirstComponent();component;component=it.NextComponent()){auto geometry=ON_ModelGeometryComponent::Cast(component);auto uuid=id(component->Id());if(!geometry||!geometry->Geometry(nullptr)||!geometryDigests.contains(uuid)||geometryDigests.at(uuid)!=currentObjectDigest(*geometry->Geometry(nullptr),uuid,*checkedGraph))throw ExchangeError("Rhino5 write changed native geometry payload: "+uuid.toStdString());if(!geometry->Attributes(nullptr)||attributeDigests.at(uuid)!=objectDigest(*geometry->Attributes(nullptr)))throw ExchangeError("Rhino5 write changed native object attributes: "+uuid.toStdString());}}
        QFile input(QString::fromStdWString(staged.wstring()));QSaveFile target(QString::fromStdWString(destination.wstring()));if(!input.open(QIODevice::ReadOnly)||!target.open(QIODevice::WriteOnly))throw ExchangeError("Cannot stage preserved output replacement");
        while(!input.atEnd()){auto bytes=input.read(1024*1024);if(bytes.isEmpty()&&input.error()!=QFile::NoError)throw ExchangeError("Cannot read staged output");if(target.write(bytes)!=bytes.size())throw ExchangeError("Cannot write output replacement");}
        input.close();if(!target.commit())throw ExchangeError("Cannot replace preserved output atomically");
        std::error_code cleanupError;std::filesystem::remove(staged,cleanupError);
    }catch(...){std::error_code cleanupError;std::filesystem::remove(staged,cleanupError);throw;}
}
void writePreservedArchive(const QJsonObject& request,const std::filesystem::path& destination){writeSelectedArchive(request,destination,ArchivePurpose::Rhino5Preservation);}
void writeGeometryStagingArchive(const QJsonObject& request,const std::filesystem::path& destination){writeSelectedArchive(request,destination,ArchivePurpose::GeometryStaging);}
}
