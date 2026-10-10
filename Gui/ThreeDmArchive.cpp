#include "ThreeDmArchive.h"
#include "ThreeDmThreadPool.h"
#include "ThreeDmPointCloud.h"
#include "ThreeDmCurveOnSurface.h"
#include "opennurbs_polyedgecurve.h"
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <TopoDS.hxx>
#include <BRep_Tool.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <TopExp_Explorer.hxx>
#include <map>
#include <cmath>
#include <functional>
#include <chrono>
#include <set>
#include <cstdlib>
namespace OpenMatrix9Gui::ThreeDm {
void applyExtendedLayerOverlay(ExchangeModel& model,std::uint64_t sourceSnapshot){
    auto view=[](const std::string& value)->Om9LayerByteView{return {reinterpret_cast<const unsigned char*>(value.data()),value.size()};};
    std::vector<Om9LayerGeometryBindingView> facts;facts.reserve(model.items.size());
    for(const auto& item:model.items){
        if(!item.ownState)throw ExchangeError("Prepared geometry has no physical layer identity");
        facts.push_back({view(item.ownState->id),view(item.transferObjectId)});
    }
    std::uint64_t handle=0;
    const auto status=om9_layer_clipboard_bind_objects(sourceSnapshot,facts.data(),facts.size(),&handle);
    if(status)throw ExchangeError("Extended geometry-to-layer binding rejected: Rust code "+std::to_string(status));
    NativeLayerSnapshot bound(handle);
    auto table=nativeLayerTableFromSnapshot(bound.get());
    auto context=nativeLayerContextFromSnapshot(bound.get());
    auto objects=nativeObjectLayersFromSnapshot(bound.get());
    std::map<std::string,std::size_t> objectIndex;
    std::map<std::string,std::string> layerPaths;
    for(std::size_t i=0;i<objects.size();++i)objectIndex.emplace(objects[i].id,i);
    for(const auto& layer:table.rows)layerPaths.emplace(layer.id,layer.path);
    struct Projection {NativeObjectLayerRow own;std::string path;Om9LayerEffective effective;};
    std::vector<Projection> projections;projections.reserve(model.items.size());
    for(const auto& item:model.items){
        auto own=std::move(objects.at(objectIndex.at(item.ownState->id)));
        Om9LayerEffective effective{};
        const auto error=om9_layer_snapshot_effective(bound.get(),view(own.id),&effective);
        if(error)throw ExchangeError("Extended layer projection rejected: Rust code "+std::to_string(error));
        auto path=layerPaths.at(own.layerId);
        projections.push_back({std::move(own),std::move(path),effective});
    }
    // All bindings/projections are validated and allocated before this detached
    // metadata commit. No shape serialization, point-array copy or remeshing.
    model.layerTable=std::move(table);
    model.layerContext=std::move(context);
    for(std::size_t i=0;i<model.items.size();++i){
        auto& item=model.items[i];auto& projection=projections[i];
        item.ownState=std::move(projection.own);item.layer=std::move(projection.path);
        item.locked=projection.effective.locked!=0;item.visible=projection.effective.visible!=0;
        item.color={projection.effective.rgb[0],projection.effective.rgb[1],projection.effective.rgb[2]};
    }
}
bool writeModelRhino5(const ONX_Model& model,const std::filesystem::path& path,ON_TextLog* log){
    // RDK3 cannot carry RDK4 embedded resources. Refuse before opening output;
    // retaining the source snapshot is safer than a silent resource downgrade.
    if(model.ActiveComponentCount(ON_ModelComponent::Type::EmbeddedFile)>0)
        throw ExchangeError("Rhino 5 RDK3 cannot preserve embedded resources; export is refused without changing the target");
    ONX_ModelComponentIterator objects(model,ON_ModelComponent::Type::ModelGeometry);
    for(auto component=objects.FirstComponent();component;component=objects.NextComponent()){
        auto object=ON_ModelGeometryComponent::Cast(component);auto brep=object?ON_Brep::Cast(object->Geometry(nullptr)):nullptr;
        if(auto geometry=object?object->Geometry(nullptr):nullptr)if(geometry->ClassId()==&ON_CLASS_RTTI(ON_PolyEdgeCurve)||geometry->ClassId()==&ON_CLASS_RTTI(ON_PolyEdgeSegment))
            throw ExchangeError("Rhino 5 leaves standalone PolyEdge references unresolved and invalid; native owner graph remains retained in the project; export refused without changing the target");
        if(auto curve=object?ON_Curve::Cast(object->Geometry(nullptr)):nullptr;curve&&hasPolyEdgeReference(*curve))
            throw ExchangeError("Rhino 5 leaves nested PolyEdge references unresolved and loses child class, metadata or reversal; native owner graph remains retained in the project; export refused without changing the target");
        if(auto curve=object?ON_CurveOnSurface::Cast(object->Geometry(nullptr)):nullptr)validateCurveOnSurfaceRhino5(*curve);
        // Actual Rhino5 insertion normalizes definition members as well as roots.
        // Retained UUID/class/history graphs cannot be rewritten transparently.
        if(brep){
            int direction;
            try{direction=resolvedBrepOrientation(*brep,model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance);}
            catch(const ExchangeError& error){char uuid[37]{};ON_UuidToString(component->Id(),uuid);throw ExchangeError(std::string("Cannot resolve BRep solid orientation for Rhino5 object ")+uuid+": "+error.what()+"; preservation refused without changing the target");}
            if(direction==-1)throw ExchangeError("Rhino 5 would reverse an inward solid, including a definition member. Use explicit Geometry only export; preservation export refused without changing the target");
        }
    }
    return model.Write(path.c_str(),5,log);
}
static void initialize(){static const bool initialized=[](){ON::Begin();return true;}();(void)initialized;}
static std::string utf8(const ON_wString& s){ON_String value(s);return value.IsEmpty()?std::string{}:std::string(value.Array());}
static constexpr const wchar_t* signedSolidTag=L"OpenMatrix9.InwardSolidInstance";
static constexpr const wchar_t* transferObjectTag=L"OpenMatrix9.LayerObjectId";
static void addGeometryForRhinoDocument(ONX_Model& model,ON_Geometry& geometry,const ON_3dmObjectAttributes& attributes){
    auto brep=ON_Brep::Cast(&geometry);
    if(!brep||brep->SolidOrientation()!=-1){if(model.AddModelGeometryComponent(&geometry,&attributes).IsEmpty())throw ExchangeError("Cannot add exported object");return;}
    // Rhino document insertion flips a flat inward BRep outward. Represent its
    // signed physical placement with an outward member and a reflected instance.
    // The two reflections preserve position and surface/edge geometry exactly.
    ON_Xform reflection=ON_Xform::IdentityTransformation;reflection[0][0]=-1;
    auto worldBounds=brep->BoundingBox();transformNativeGeometry(*brep,reflection);
    if(brep->SolidOrientation()!=1)throw ExchangeError("Cannot prepare outward signed-solid member");
    auto memberAttributes=attributes;memberAttributes.m_uuid=ON_nil_uuid;memberAttributes.SetMode(ON::idef_object);memberAttributes.SetVisible(true);memberAttributes.SetColorSource(ON::color_from_parent);memberAttributes.SetUserString(L"OpenMatrix9.Locked",nullptr);
    memberAttributes.SetUserString(transferObjectTag,nullptr);
    auto member=model.AddModelGeometryComponent(brep,&memberAttributes);if(member.IsEmpty())throw ExchangeError("Cannot add signed-solid member");
    ON_InstanceDefinition definition;ON_UUID definitionId;ON_CreateUuid(definitionId);definition.SetId(definitionId);char name[37]{};ON_UuidToString(definitionId,name);definition.SetName(ON_wString((std::string("OM9_Inward_")+name).c_str()));definition.SetInstanceDefinitionType(ON_InstanceDefinition::IDEF_UPDATE_TYPE::Static);definition.SetUnitSystem(ON::LengthUnitSystem::Millimeters);definition.SetBoundingBox(brep->BoundingBox());definition.AddInstanceGeometryId(member.ModelComponent()->Id());
    auto added=model.AddModelComponent(definition);if(added.IsEmpty())throw ExchangeError("Cannot add signed-solid definition");
    ON_InstanceRef instance;instance.m_instance_definition_uuid=added.ModelComponent()->Id();instance.m_xform=reflection;instance.m_bbox=worldBounds;
    auto placementAttributes=attributes;placementAttributes.SetUserString(signedSolidTag,L"1");if(model.AddModelGeometryComponent(&instance,&placementAttributes).IsEmpty())throw ExchangeError("Cannot add signed-solid placement");
}
static ExchangeModel readArchiveImpl(const std::filesystem::path& path,double custom,bool preserve,bool definitionMembersOnly,const std::string& sourceRoot,const std::set<std::string>& subset,bool modelSpaceOnly,bool deferred=false){
    std::lock_guard sdkLock(sdkArchiveMutex());
    if(std::filesystem::file_size(path)>512ULL*1024*1024)throw ExchangeError("3DM archive exceeds the 512 MiB import limit");
    initialize();ONX_Model model;ON_wString log;ON_TextLog errors(log);if(!model.Read(path.c_str(),&errors))throw ExchangeError("Cannot read 3DM archive: "+utf8(log));
    ExchangeModel result;
    try { result.layerTable=readNativeLayerTable(model); }
    catch(const std::exception& error){throw ExchangeError(std::string("Invalid 3DM layer table: ")+error.what());}
    const auto units=model.m_settings.m_ModelUnitsAndTolerances.m_unit_system.UnitSystem();
    double factor=(units==ON::LengthUnitSystem::None||units==ON::LengthUnitSystem::CustomUnits)?custom:ON::UnitScale(units,ON::LengthUnitSystem::Millimeters);
    if(!std::isfinite(factor)||factor<=0)throw ExchangeError("3DM file has unitless/custom units; specify millimeters per file unit");result.scaleMm=factor;result.tolerance=std::max(1e-7,model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance*factor);
    std::map<int,const ON_Layer*> layers;ONX_ModelComponentIterator layerIt(model,ON_ModelComponent::Type::Layer);for(auto c=layerIt.FirstComponent();c;c=layerIt.NextComponent())if(auto layer=ON_Layer::Cast(c))layers[layer->Index()]=layer;
    auto layerPath=[&](const ON_Layer* l){std::vector<std::string> names;std::size_t depth=0;while(l){if(++depth>layers.size())throw ExchangeError("Cyclic layer ancestry");names.push_back(utf8(l->Name()));auto parent=l->ParentId();const ON_Layer* next=nullptr;if(parent!=ON_nil_uuid)for(auto [i,p]:layers)if(p->Id()==parent){next=p;break;}l=next;}std::string value;for(auto i=names.rbegin();i!=names.rend();++i){if(!value.empty())value+="::";value+=*i;}return value;};
    std::vector<const ON_ModelGeometryComponent*> objects;
    std::vector<const ON_InstanceDefinition*> definitions;
    ONX_ModelComponentIterator defIt(model,ON_ModelComponent::Type::InstanceDefinition);
    for(auto c=defIt.FirstComponent();c;c=defIt.NextComponent())if(auto d=ON_InstanceDefinition::Cast(c))definitions.push_back(d);
    ONX_ModelComponentIterator it(model,ON_ModelComponent::Type::ModelGeometry);
    for(auto c=it.FirstComponent();c;c=it.NextComponent())if(auto component=ON_ModelGeometryComponent::Cast(c))objects.push_back(component);
    if(preserve){ONX_ModelComponentIterator lights(model,ON_ModelComponent::Type::RenderLight);for(auto c=lights.FirstComponent();c;c=lights.NextComponent())if(auto component=ON_ModelGeometryComponent::Cast(c))objects.push_back(component);}
    auto findObject=[&](const ON_UUID& id)->const ON_ModelGeometryComponent*{for(auto object:objects)if(object->Id()==id)return object;return nullptr;};
    // Pinned IsInstanceGeometryId incorrectly returns true for an empty list.
    // Check the actual UUID list so empty definitions do not hide placements.
    auto definitionMember=[&](const ON_UUID& id){for(auto d:definitions){auto& ids=d->InstanceGeometryIdList();for(int i=0;i<ids.Count();++i)if(ids[i]==id)return true;}return false;};
    std::vector<ON_UUID> ancestry;
    bool emptyEmbeddedBlock=false;
    std::string rootUuid;std::set<std::string> matchedRoots;
    std::function<void(const ON_ModelGeometryComponent*,const ON_Xform&,const ExchangeItem*,bool)> expand;
    expand=[&](const ON_ModelGeometryComponent* component,const ON_Xform& transform,const ExchangeItem* parent,bool signedMember){
        auto started=std::chrono::steady_clock::now();
        auto g=component->Geometry(nullptr);auto a=component->Attributes(nullptr);
        if(!g||!a)throw ExchangeError("Invalid geometry component");
        const std::string geometryType=g->ClassId()->ClassName();
        ExchangeItem item;item.wireDensity=a->m_wire_density;item.name=utf8(a->m_name);auto found=layers.find(a->m_layer_index);const ON_Layer* layer=found==layers.end()?nullptr:found->second;item.layer=layerPath(layer);item.visible=a->IsVisible()&&(!layer||layer->IsVisible());item.locked=a->Mode()==ON::locked_object|| (layer&&layer->IsLocked());auto color=a->ColorSource()==ON::color_from_layer&&layer?layer->Color():a->m_color;item.color={color.Red(),color.Green(),color.Blue()};
        for(const ON_Layer* ancestor=layer;ancestor;){item.visible=item.visible&&ancestor->IsVisible();item.locked=item.locked||ancestor->IsLocked();const auto parent=ancestor->ParentId();ancestor=nullptr;if(parent!=ON_nil_uuid)for(auto [i,p]:layers)if(p->Id()==parent){ancestor=p;break;}}
        ON_wString lockedMetadata; if(a->GetUserString(L"OpenMatrix9.Locked",lockedMetadata)&&lockedMetadata==L"1")item.locked=true;
        ON_wString transferMetadata;if(a->GetUserString(transferObjectTag,transferMetadata))item.transferObjectId=utf8(transferMetadata);

        char sourceId[37]{};ON_UuidToString(component->Id(),sourceId);item.sourceUuid=sourceId;item.sourceClass=geometryType;item.sourceRootUuid=rootUuid;
        if(!layer)throw ExchangeError("Object references missing layer: "+item.sourceUuid);
        NativeObjectLayerRow own;own.id=item.sourceUuid;
        char layerId[37]{};ON_UuidToString(layer->Id(),layerId);own.layerId=layerId;
        own.locked=a->Mode()==ON::locked_object||(lockedMetadata==L"1");own.visible=a->IsVisible();
        own.rgb={static_cast<unsigned char>(a->m_color.Red()),static_cast<unsigned char>(a->m_color.Green()),static_cast<unsigned char>(a->m_color.Blue())};
        if(a->ColorSource()==ON::color_from_layer)own.colorSource=1;
        else if(a->ColorSource()==ON::color_from_object)own.colorSource=2;
        else own.colorSource=3; // Native instance-context source, not a portable ByLayer/ByObject claim.
        item.ownState=own;
        if(parent){
            // A flattened physical placement is a new native object. Keep its
            // bounded UUID separate from original member/root provenance.
            ON_UUID physicalId;if(!ON_CreateUuid(physicalId))throw ExchangeError("Cannot allocate expanded native placement identity");
            char text[37]{};ON_UuidToString(physicalId,text);item.ownState->id=text;
        }
        if(preserve&&(ON_InstanceRef::Cast(g)||ON_CurveOnSurface::Cast(g)||!(ON_Point::Cast(g)||ON_Curve::Cast(g)||ON_Brep::Cast(g)||ON_Mesh::Cast(g)||ON_Surface::Cast(g)))){
            item.retained=true;result.items.push_back(std::move(item));return;
        }
        if(parent){
            item.visible=item.visible&&parent->visible;item.locked=item.locked||parent->locked;
            if(a->ColorSource()==ON::color_from_parent)item.color=parent->color;
            item.name=signedMember?parent->name:parent->name+"/"+(item.name.empty()?g->ClassId()->ClassName():item.name);
            // Expanded instances are physical output objects. The member's own
            // flags remain separate from layer inheritance. Native ByParent
            // color is resolved by the existing expansion context.
            if(parent->ownState){
                item.ownState->locked=item.ownState->locked||parent->ownState->locked;
                item.ownState->visible=item.ownState->visible&&parent->ownState->visible;
                if(signedMember){item.ownState=parent->ownState;item.layer=parent->layer;item.transferObjectId=parent->transferObjectId;}
            }
            if(item.ownState->colorSource==3){
                item.ownState->colorSource=2;
                item.ownState->rgb={static_cast<unsigned char>(item.color[0]),static_cast<unsigned char>(item.color[1]),static_cast<unsigned char>(item.color[2])};
            }
        }
        try {
            if(auto instance=ON_InstanceRef::Cast(g)){
                if(ancestry.size()>=64)throw ExchangeError("Block nesting exceeds 64 levels");
                auto id=instance->m_instance_definition_uuid;
                for(auto ancestor:ancestry)if(ancestor==id)throw ExchangeError("Cyclic block definition");
                const ON_InstanceDefinition* definition=nullptr;for(auto d:definitions)if(d->Id()==id){definition=d;break;}
                if(!definition)throw ExchangeError("Missing block definition (external linked blocks require embedded geometry)");
                auto local=instance->m_xform;
                // Rhino archives can contain roundoff in an otherwise affine bottom row.
                if(local.IsValid()&&std::abs(local[3][0])<=1e-12&&std::abs(local[3][1])<=1e-12&&std::abs(local[3][2])<=1e-12&&std::abs(local[3][3]-1)<=1e-12){
                    local[3][0]=local[3][1]=local[3][2]=0;local[3][3]=1;
                }
                auto combined=transform*local;
                if(!combined.IsValid()||!combined.IsAffine()||combined.Determinant()==0)throw ExchangeError("Invalid or singular block transform");
                if(definition->InstanceGeometryIdList().Count()==0){auto file=definition->LinkedFileReference();if(!file.FullPath().IsEmpty()||!file.RelativePath().IsEmpty())throw ExchangeError("External-only block contains no embedded geometry");emptyEmbeddedBlock=true;return;}
                const auto& members=definition->InstanceGeometryIdList();
                ON_wString signedTag;bool signedPlacement=false;
                if(members.Count()==1&&a->GetUserString(signedSolidTag,signedTag)&&signedTag==L"1"){
                    auto child=findObject(members[0]);auto solid=child?ON_Brep::Cast(child->Geometry(nullptr)):nullptr;signedPlacement=solid&&solid->IsSolid()&&solid->SolidOrientation()==1;
                }
                if(item.name.empty()&&!signedPlacement)item.name=utf8(definition->Name());
                ancestry.push_back(id);
                for(int memberIndex=0;memberIndex<members.Count();++memberIndex){const auto& objectId=members[memberIndex];
                    auto child=findObject(objectId);if(!child)throw ExchangeError("Missing block member geometry");
                    expand(child,combined,&item,signedPlacement);
                }
                ancestry.pop_back();return;
            }
            std::unique_ptr<ON_Geometry> transformed;
            if(!transform.IsIdentity()){
                transformed.reset(ON_Geometry::Cast(g->Duplicate()));
                if(!transformed)throw ExchangeError("Cannot clone block geometry for transform");
                transformNativeGeometry(*transformed,transform);
                g=transformed.get();
            }
            if(g->ClassId()==&ON_CLASS_RTTI(ON_PolyEdgeCurve)||g->ClassId()==&ON_CLASS_RTTI(ON_PolyEdgeSegment))throw GeometryRepresentationUnavailable("Standalone PolyEdge owner graph is retained; no independent editable CAD representation is established");
            if(auto curve=ON_Curve::Cast(g);curve&&hasPolyEdgeReference(*curve))throw GeometryRepresentationUnavailable("Nested PolyEdge owner graph is retained; no independent editable CAD representation is established");
            if(auto p=ON_Point::Cast(g))item.geometry=BRepBuilderAPI_MakeVertex(gp_Pnt(p->point.x,p->point.y,p->point.z)).Shape();
            else if(auto curve=ON_Curve::Cast(g))item.geometry=importCurve(*curve,result.tolerance/factor);
            else if(auto brep=ON_Brep::Cast(g)){
                if(deferred)result.pendingBreps.push_back({result.items.size(),prepareBrepAssembly(*brep,result.tolerance/factor)});
                else item.geometry=importBrep(*brep,result.tolerance/factor);
            }
            else if(auto mesh=ON_Mesh::Cast(g))item.geometry=importMesh(*mesh);
            else if(auto cloud=ON_PointCloud::Cast(g)){validatePointCloud(*cloud);item.geometry=*cloud;}
            else if(auto surface=ON_Surface::Cast(g)){std::unique_ptr<ON_Brep> brep(surface->BrepForm());if(!brep)throw ExchangeError("Cannot convert surface/extrusion to BRep");if(deferred)result.pendingBreps.push_back({result.items.size(),prepareBrepAssembly(*brep,result.tolerance/factor)});
                else item.geometry=importBrep(*brep,result.tolerance/factor);}
            else throw ExchangeError("Unsupported 3DM object type (annotations and plugin objects are not converted)");
            if(auto s=std::get_if<TopoDS_Shape>(&item.geometry)){if(!s->IsNull()&&factor!=1.0){gp_Trsf transform;transform.SetScale(gp_Pnt(0,0,0),factor);*s=BRepBuilderAPI_Transform(*s,transform,true).Shape();}}
            else if(auto mesh=std::get_if<MeshData>(&item.geometry)){for(auto& p:mesh->vertices)for(auto& v:p)v*=factor;}
            else if(factor!=1.0)transformNativeGeometry(std::get<ON_PointCloud>(item.geometry),ON_Xform(factor));

        }catch(const GeometryRepresentationUnavailable& e){
            if(!preserve)throw ExchangeError("Object '"+item.name+"' ["+geometryType+"]: "+e.what());
            item.retained=true;item.representationIssue=e.what();
        }catch(const ExchangeError& e){throw ExchangeError("Object '"+item.name+"' ["+geometryType+"]: "+e.what());}
        item.conversionSeconds=std::chrono::duration<double>(std::chrono::steady_clock::now()-started).count();
        result.items.push_back(std::move(item));if(result.items.size()>1000000)throw ExchangeError("3DM object limit exceeded");
    };
    auto selected=[&](const ON_ModelGeometryComponent* object){auto attributes=object->Attributes(nullptr);if(!attributes)throw ExchangeError("Missing object attributes");
        if(modelSpaceOnly&&attributes->m_space==ON::page_space)return false;
        bool member=attributes->IsInstanceDefinitionObject()||definitionMember(object->Id());
        char uuid[37]{};ON_UuidToString(object->Id(),uuid);
        if(!subset.empty()&&!subset.contains(uuid))return false;
        if(!sourceRoot.empty())return sourceRoot==uuid;
        return definitionMembersOnly?member:!member;
    };
    for(auto object:objects){if(!selected(object))continue;
        char id[37]{};ON_UuidToString(object->Id(),id);rootUuid=id;matchedRoots.insert(rootUuid);
        expand(object,ON_Xform::IdentityTransformation,nullptr,false);
    }
    if(!subset.empty()&&matchedRoots!=subset)throw ExchangeError("Missing selected conversion root");
    if(result.items.empty()&&!preserve&&!emptyEmbeddedBlock&&result.layerTable.rows.empty())throw ExchangeError("3DM archive has no supported geometry or palette");return result;
}
ExchangeModel readArchive(const std::filesystem::path& path,double custom,bool preserve,bool definitionMembersOnly,const std::string& sourceRoot,bool modelSpaceOnly){
    return readArchiveImpl(path,custom,preserve,definitionMembersOnly,sourceRoot,{},modelSpaceOnly);
}
ExchangeModel readArchiveSubset(const std::filesystem::path& path,double custom,bool members,const std::set<std::string>& subset,bool preserve){
    if(subset.empty())throw ExchangeError("Empty conversion subset");
    return readArchiveImpl(path,custom,preserve,members,"",subset,!preserve);
}
ExchangeModel readWorkingArchiveDeferred(const std::filesystem::path& path,double scale){return readArchiveImpl(path,scale,false,false,"",{},true,true);}
void finishDeferredBrep(ExchangeModel& model,size_t index){
    auto& pending=model.pendingBreps.at(index);auto& item=model.items.at(pending.item);
    try{auto shape=assembleBrep(pending.assembly);
        if(model.scaleMm!=1){gp_Trsf scale;scale.SetScale(gp_Pnt(0,0,0),model.scaleMm);shape=BRepBuilderAPI_Transform(shape,scale,true).Shape();}
        item.geometry=std::move(shape);
    }catch(const Standard_Failure& error){throw ExchangeError("Object '"+item.name+"' ["+item.sourceClass+"]: "+error.GetMessageString());}
    catch(const ExchangeError& error){throw ExchangeError("Object '"+item.name+"' ["+item.sourceClass+"]: "+error.what());}
}
void writeArchive5(const ExchangeModel& source,const std::filesystem::path& path){
    std::lock_guard sdkLock(sdkArchiveMutex());
    const bool canonical=!source.layerTable.rows.empty();
    if(canonical){
        std::vector<NativeObjectLayerRow> objects;objects.reserve(source.items.size());
        for(std::size_t i=0;i<source.items.size();++i){
            if(!source.items[i].ownState)throw ExchangeError("Canonical layer export is missing own object state");
            objects.push_back(*source.items[i].ownState);
        }
        try { auto validated=createNativeLayerSnapshot(source.layerTable,objects,"archive-export",0); }
        catch(const std::exception& error){throw ExchangeError(std::string("Invalid export layer metadata: ")+error.what());}
    }
    for(const auto& item:source.items)if(item.retained)throw ExchangeError("Legacy geometry export cannot preserve source-retained records");
    // Preflight every cloud before opening the output. Modern reread alone
    // cannot detect fields discarded by the independent Rhino5-era reader.
    for(const auto& item:source.items)if(auto cloud=std::get_if<ON_PointCloud>(&item.geometry))validateRhino5PointCloud(*cloud);
    ExchangeModel input=source;
    if(!std::isfinite(input.tolerance)||input.tolerance<=0)throw ExchangeError("Invalid export tolerance");
    for(const auto& item:input.items)if(auto shape=std::get_if<TopoDS_Shape>(&item.geometry)){
        for(TopExp_Explorer it(*shape,TopAbs_VERTEX);it.More();it.Next())input.tolerance=std::max(input.tolerance,BRep_Tool::Tolerance(TopoDS::Vertex(it.Current())));
        for(TopExp_Explorer it(*shape,TopAbs_EDGE);it.More();it.Next())input.tolerance=std::max(input.tolerance,BRep_Tool::Tolerance(TopoDS::Edge(it.Current())));
        for(TopExp_Explorer it(*shape,TopAbs_FACE);it.More();it.Next())input.tolerance=std::max(input.tolerance,BRep_Tool::Tolerance(TopoDS::Face(it.Current())));
    }
    initialize();if(input.items.empty()&&!canonical)throw ExchangeError("No geometry or palette to export");ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance=input.tolerance;model.m_properties.m_Application.m_application_name=L"OpenMatrix9";
    std::map<std::string,int> layers;
    if(canonical){
        try{layers=writeNativeLayerTable(model,input.layerTable);}
        catch(const std::exception& error){throw ExchangeError(std::string("Cannot write canonical layer table: ")+error.what());}
    }
    auto createLayer=[&](const std::string& full,const std::array<int,3>& color){
        std::string path;ON_UUID parent=ON_nil_uuid;int index=-1;std::size_t start=0;
        const std::string name=full.empty()?"Default":full;
        do {auto end=name.find("::",start);auto segment=name.substr(start,end==std::string::npos?end:end-start);if(segment.empty())throw ExchangeError("Empty layer name");
            if(!path.empty())path+="::";path+=segment;auto found=layers.find(path);
            if(found!=layers.end()){index=found->second;auto ref=model.ComponentFromIndex(ON_ModelComponent::Type::Layer,index);parent=ref.ModelComponent()->Id();}
            else{ON_Layer layer;layer.SetName(ON_wString(segment.c_str()));layer.SetParentId(parent);layer.SetColor(ON_Color(color[0],color[1],color[2]));auto ref=model.AddModelComponent(layer);auto l=ON_Layer::Cast(ref.ModelComponent());if(!l)throw ExchangeError("Cannot create output layer");index=l->Index();parent=l->Id();layers[path]=index;}
            if(end==std::string::npos)break;start=end+2;
        }while(true);return index;
    };
    for(const auto& item:input.items){const int li=canonical?layers.at(item.ownState->layerId):createLayer(item.layer,item.color);
        ON_3dmObjectAttributes attributes;attributes.m_name=ON_wString(item.name.c_str());attributes.m_layer_index=li;
        const auto color=canonical?std::array<int,3>{item.ownState->rgb[0],item.ownState->rgb[1],item.ownState->rgb[2]}:item.color;
        const bool ownLocked=canonical?item.ownState->locked:item.locked,ownVisible=canonical?item.ownState->visible:item.visible;
        attributes.m_color=ON_Color(color[0],color[1],color[2]);attributes.SetColorSource(canonical&&item.ownState->colorSource==1?ON::color_from_layer:ON::color_from_object);attributes.SetMode(ownLocked?ON::locked_object:ON::normal_object);attributes.SetVisible(ownVisible);if(ownLocked)attributes.SetUserString(L"OpenMatrix9.Locked",L"1");
        if(canonical)attributes.SetUserString(transferObjectTag,ON_wString(item.ownState->id.c_str()));
        std::unique_ptr<ON_Geometry> geometry;
        if(auto s=std::get_if<TopoDS_Shape>(&item.geometry)){
            if(s->ShapeType()==TopAbs_VERTEX){auto p=BRep_Tool::Pnt(TopoDS::Vertex(*s));geometry=std::make_unique<ON_Point>(p.X(),p.Y(),p.Z());}
            else if(s->ShapeType()==TopAbs_EDGE)geometry=exportCurve(TopoDS::Edge(*s),input.tolerance);
            else if(s->ShapeType()==TopAbs_WIRE){auto poly=std::make_unique<ON_PolyCurve>();for(BRepTools_WireExplorer edges(TopoDS::Wire(*s));edges.More();edges.Next()){auto curve=exportCurve(edges.Current(),input.tolerance);if(!poly->Append(curve.release()))throw ExchangeError("Cannot append wire curve");}if(!poly->IsValid())throw ExchangeError("Invalid wire curve");geometry=std::move(poly);}
            else {try{geometry=exportBrep(*s,input.tolerance);}catch(const ExchangeError& e){throw ExchangeError("Export object '"+item.name+"' on layer '"+item.layer+"': "+e.what());}}
        }else if(auto mesh=std::get_if<MeshData>(&item.geometry))geometry=exportMesh(*mesh);
        else {auto cloud=std::make_unique<ON_PointCloud>(std::get<ON_PointCloud>(item.geometry));cloud->PurgeUserData();geometry=std::move(cloud);}
        addGeometryForRhinoDocument(model,*geometry,attributes);
    }
    ON_wString log;ON_TextLog errors(log);if(!writeModelRhino5(model,path,&errors))throw ExchangeError("Cannot write Rhino 5 archive: "+utf8(log));
    ONX_Model check;if(!check.Read(path.c_str(),&errors)||(check.m_3dm_file_version!=5&&check.m_3dm_file_version!=50))throw ExchangeError("Rhino 5 output validation failed");
    // Validate converted geometry too before the host replaces any destination.
    const auto verified=readArchive(path,1.0);if(verified.items.size()!=input.items.size())throw ExchangeError("Rhino 5 output object count changed");
    for(std::size_t i=0;i<input.items.size();++i)if(auto cloud=std::get_if<ON_PointCloud>(&input.items[i].geometry)){
        auto reread=std::get_if<ON_PointCloud>(&verified.items[i].geometry);
        if(!reread||pointCloudFields(*reread)!=pointCloudFields(*cloud))throw ExchangeError("Rhino5 output PointCloud current fields changed");
    }
}
}
