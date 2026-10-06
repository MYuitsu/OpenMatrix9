#include "ThreeDmArchive.h"
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <TopoDS.hxx>
#include <BRep_Tool.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <TopExp_Explorer.hxx>
#include <map>
#include <cmath>
#include <functional>
namespace OpenMatrix9Gui::ThreeDm {
static void initialize(){static const bool initialized=[](){ON::Begin();return true;}();(void)initialized;}
static std::string utf8(const ON_wString& s){ON_String value(s);return value.IsEmpty()?std::string{}:std::string(value.Array());}
ExchangeModel readArchive(const std::filesystem::path& path,double custom,bool preserve){
    if(std::filesystem::file_size(path)>512ULL*1024*1024)throw ExchangeError("3DM archive exceeds the 512 MiB import limit");
    initialize();ONX_Model model;ON_wString log;ON_TextLog errors(log);if(!model.Read(path.c_str(),&errors))throw ExchangeError("Cannot read 3DM archive: "+utf8(log));
    ExchangeModel result;const auto units=model.m_settings.m_ModelUnitsAndTolerances.m_unit_system.UnitSystem();
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
    auto definitionMember=[&](const ON_UUID& id){for(auto d:definitions)if(d->IsInstanceGeometryId(id))return true;return false;};
    std::vector<ON_UUID> ancestry;
    std::function<void(const ON_ModelGeometryComponent*,const ON_Xform&,const ExchangeItem*)> expand;
    expand=[&](const ON_ModelGeometryComponent* component,const ON_Xform& transform,const ExchangeItem* parent){
        auto g=component->Geometry(nullptr);auto a=component->Attributes(nullptr);
        if(!g||!a)throw ExchangeError("Invalid geometry component");
        const std::string geometryType=g->ClassId()->ClassName();
        ExchangeItem item;item.name=utf8(a->m_name);auto found=layers.find(a->m_layer_index);const ON_Layer* layer=found==layers.end()?nullptr:found->second;item.layer=layerPath(layer);item.visible=a->IsVisible()&&(!layer||layer->IsVisible());item.locked=a->Mode()==ON::locked_object|| (layer&&layer->IsLocked());auto color=a->ColorSource()==ON::color_from_layer&&layer?layer->Color():a->m_color;item.color={color.Red(),color.Green(),color.Blue()};
        for(const ON_Layer* ancestor=layer;ancestor;){item.visible=item.visible&&ancestor->IsVisible();item.locked=item.locked||ancestor->IsLocked();const auto parent=ancestor->ParentId();ancestor=nullptr;if(parent!=ON_nil_uuid)for(auto [i,p]:layers)if(p->Id()==parent){ancestor=p;break;}}
        ON_wString lockedMetadata; if(a->GetUserString(L"OpenMatrix9.Locked",lockedMetadata)&&lockedMetadata==L"1")item.locked=true;

        char sourceId[37]{};ON_UuidToString(component->Id(),sourceId);item.sourceUuid=sourceId;item.sourceClass=geometryType;
        if(preserve&&(ON_InstanceRef::Cast(g)||!(ON_Point::Cast(g)||ON_Curve::Cast(g)||ON_Brep::Cast(g)||ON_Mesh::Cast(g)||ON_Surface::Cast(g)))){
            item.retained=true;result.items.push_back(std::move(item));return;
        }
        if(parent){
            item.visible=item.visible&&parent->visible;item.locked=item.locked||parent->locked;
            if(a->ColorSource()==ON::color_from_parent)item.color=parent->color;
            item.name=parent->name+"/"+(item.name.empty()?g->ClassId()->ClassName():item.name);
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
                if(definition->InstanceGeometryIdList().Count()==0)throw ExchangeError("Block definition contains no embedded geometry");
                if(item.name.empty())item.name=utf8(definition->Name());
                ancestry.push_back(id);
                const auto& members=definition->InstanceGeometryIdList();
                for(int memberIndex=0;memberIndex<members.Count();++memberIndex){const auto& objectId=members[memberIndex];
                    auto child=findObject(objectId);if(!child)throw ExchangeError("Missing block member geometry");
                    expand(child,combined,&item);
                }
                ancestry.pop_back();return;
            }
            std::unique_ptr<ON_Geometry> transformed;
            if(!transform.IsIdentity()){
                transformed.reset(ON_Geometry::Cast(g->Duplicate()));
                if(!transformed||!transformed->Transform(transform))throw ExchangeError("Cannot apply block transform to geometry");
                g=transformed.get();
            }
            if(auto p=ON_Point::Cast(g))item.geometry=BRepBuilderAPI_MakeVertex(gp_Pnt(p->point.x,p->point.y,p->point.z)).Shape();
            else if(auto curve=ON_Curve::Cast(g))item.geometry=importCurve(*curve,result.tolerance/factor);
            else if(auto brep=ON_Brep::Cast(g))item.geometry=importBrep(*brep,result.tolerance/factor);
            else if(auto mesh=ON_Mesh::Cast(g))item.geometry=importMesh(*mesh);
            else if(auto surface=ON_Surface::Cast(g)){std::unique_ptr<ON_Brep> brep(surface->BrepForm());if(!brep)throw ExchangeError("Cannot convert surface/extrusion to BRep");item.geometry=importBrep(*brep,result.tolerance/factor);}
            else throw ExchangeError("Unsupported 3DM object type (annotations and plugin objects are not converted)");
            if(auto s=std::get_if<TopoDS_Shape>(&item.geometry)){if(factor!=1.0){gp_Trsf transform;transform.SetScale(gp_Pnt(0,0,0),factor);*s=BRepBuilderAPI_Transform(*s,transform,true).Shape();}}else for(auto& p:std::get<MeshData>(item.geometry).vertices)for(auto& v:p)v*=factor;

        }catch(const ExchangeError& e){throw ExchangeError("Object '"+item.name+"' ["+geometryType+"]: "+e.what());}
        result.items.push_back(std::move(item));if(result.items.size()>1000000)throw ExchangeError("3DM object limit exceeded");
    };
    for(auto object:objects){auto attributes=object->Attributes(nullptr);if(!attributes)throw ExchangeError("Missing object attributes");
        if(attributes->IsInstanceDefinitionObject()||definitionMember(object->Id()))continue;
        expand(object,ON_Xform::IdentityTransformation,nullptr);
    }
    if(result.items.empty()&&!preserve)throw ExchangeError("3DM archive has no supported geometry");return result;
}
void writeArchive5(const ExchangeModel& source,const std::filesystem::path& path){
    for(const auto& item:source.items)if(item.retained)throw ExchangeError("Legacy geometry export cannot preserve source-retained records");
    ExchangeModel input=source;
    if(!std::isfinite(input.tolerance)||input.tolerance<=0)throw ExchangeError("Invalid export tolerance");
    for(const auto& item:input.items)if(auto shape=std::get_if<TopoDS_Shape>(&item.geometry)){
        for(TopExp_Explorer it(*shape,TopAbs_VERTEX);it.More();it.Next())input.tolerance=std::max(input.tolerance,BRep_Tool::Tolerance(TopoDS::Vertex(it.Current())));
        for(TopExp_Explorer it(*shape,TopAbs_EDGE);it.More();it.Next())input.tolerance=std::max(input.tolerance,BRep_Tool::Tolerance(TopoDS::Edge(it.Current())));
        for(TopExp_Explorer it(*shape,TopAbs_FACE);it.More();it.Next())input.tolerance=std::max(input.tolerance,BRep_Tool::Tolerance(TopoDS::Face(it.Current())));
    }
    initialize();if(input.items.empty())throw ExchangeError("No geometry to export");ONX_Model model;model.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);model.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance=input.tolerance;model.m_properties.m_Application.m_application_name=L"OpenMatrix9";
    std::map<std::string,int> layers;
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
    for(const auto& item:input.items){const int li=createLayer(item.layer,item.color);
        ON_3dmObjectAttributes attributes;attributes.m_name=ON_wString(item.name.c_str());attributes.m_layer_index=li;attributes.m_color=ON_Color(item.color[0],item.color[1],item.color[2]);attributes.SetColorSource(ON::color_from_object);attributes.SetMode(item.locked?ON::locked_object:ON::normal_object);attributes.SetVisible(item.visible);if(item.locked)attributes.SetUserString(L"OpenMatrix9.Locked",L"1");
        std::unique_ptr<ON_Geometry> geometry;
        if(auto s=std::get_if<TopoDS_Shape>(&item.geometry)){
            if(s->ShapeType()==TopAbs_VERTEX){auto p=BRep_Tool::Pnt(TopoDS::Vertex(*s));geometry=std::make_unique<ON_Point>(p.X(),p.Y(),p.Z());}
            else if(s->ShapeType()==TopAbs_EDGE)geometry=exportCurve(TopoDS::Edge(*s),input.tolerance);
            else if(s->ShapeType()==TopAbs_WIRE){auto poly=std::make_unique<ON_PolyCurve>();for(BRepTools_WireExplorer edges(TopoDS::Wire(*s));edges.More();edges.Next()){auto curve=exportCurve(edges.Current(),input.tolerance);if(!poly->Append(curve.release()))throw ExchangeError("Cannot append wire curve");}if(!poly->IsValid())throw ExchangeError("Invalid wire curve");geometry=std::move(poly);}
            else {try{geometry=exportBrep(*s,input.tolerance);}catch(const ExchangeError& e){throw ExchangeError("Export object '"+item.name+"' on layer '"+item.layer+"': "+e.what());}}
        }else geometry=exportMesh(std::get<MeshData>(item.geometry));
        if(model.AddModelGeometryComponent(geometry.get(),&attributes).IsEmpty())throw ExchangeError("Cannot add exported object");
    }
    ON_wString log;ON_TextLog errors(log);if(!model.Write(path.c_str(),5,&errors))throw ExchangeError("Cannot write Rhino 5 archive: "+utf8(log));
    ONX_Model check;if(!check.Read(path.c_str(),&errors)||(check.m_3dm_file_version!=5&&check.m_3dm_file_version!=50))throw ExchangeError("Rhino 5 output validation failed");
    // Validate converted geometry too before the host replaces any destination.
    const auto verified=readArchive(path,1.0);if(verified.items.size()!=input.items.size())throw ExchangeError("Rhino 5 output object count changed");
}
}
