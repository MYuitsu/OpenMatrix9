#include "ThreeDmArchive.h"
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
static void require(bool ok,const char* text){if(!ok)throw ExchangeError(text);}
int main(){try{ON::Begin();ONX_Model m;m.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);
ON_Layer layer;layer.SetName(L"Blocks");m.AddModelComponent(layer);
ON_3dmObjectAttributes a;a.SetMode(ON::idef_object);a.SetColorSource(ON::color_from_parent);a.m_name=L"Point";
ON_Point p(1,2,3);auto pr=m.AddModelGeometryComponent(&p,&a);
ON_InstanceDefinition inner;inner.SetName(L"Inner");inner.AddInstanceGeometryId(pr.ModelComponent()->Id());auto ir=m.AddModelComponent(inner);
ON_InstanceRef nested;nested.m_instance_definition_uuid=ir.ModelComponent()->Id();nested.m_xform=ON_Xform::TranslationTransformation(ON_3dVector(10,0,0));auto nr=m.AddModelGeometryComponent(&nested,&a);
ON_InstanceDefinition outer;outer.SetName(L"Outer");outer.AddInstanceGeometryId(nr.ModelComponent()->Id());auto outerRef=m.AddModelComponent(outer);
ON_InstanceRef instance;instance.m_instance_definition_uuid=outerRef.ModelComponent()->Id();instance.m_xform=ON_Xform::TranslationTransformation(ON_3dVector(0,20,0));instance.m_xform[3][0]=1e-18;
ON_3dmObjectAttributes top;top.m_name=L"Placement";top.m_color=ON_Color(11,22,33);top.SetColorSource(ON::color_from_object);top.SetMode(ON::locked_object);m.AddModelGeometryComponent(&instance,&top);
instance.m_xform=ON_Xform::IdentityTransformation;instance.m_xform[0][0]=-2;instance.m_xform[1][1]=3;instance.m_xform[2][2]=4;m.AddModelGeometryComponent(&instance,&top);
auto path=std::filesystem::temp_directory_path()/"om9-nested-blocks.3dm";require(m.Write(path.c_str(),5,nullptr),"fixture write");auto result=readArchive(path);require(result.items.size()==2,"definition geometry must not duplicate instances");
auto point=BRep_Tool::Pnt(TopoDS::Vertex(std::get<TopoDS_Shape>(result.items[0].geometry)));require(point.Distance(gp_Pnt(11,22,3))<1e-8,"nested transform composition");
point=BRep_Tool::Pnt(TopoDS::Vertex(std::get<TopoDS_Shape>(result.items[1].geometry)));require(point.Distance(gp_Pnt(-22,6,12))<1e-8,"nonuniform reflected transform");
for(auto& item:result.items)require(item.locked&&item.color==std::array<int,3>{11,22,33},"parent color and lock");
auto output=path.parent_path()/"om9-nested-blocks-roundtrip.3dm";writeArchive5(result,output);require(readArchive(output).items.size()==2,"flattened roundtrip");
auto rejects=[&](ONX_Model& invalid,const char* filename,const char* message){auto file=path.parent_path()/filename;require(invalid.Write(file.c_str(),5,nullptr),"invalid fixture write");bool rejected=false;try{readArchive(file);}catch(const ExchangeError& e){rejected=std::string(e.what()).find(message)!=std::string::npos;}require(rejected,message);};
ONX_Model missing;missing.AddModelComponent(layer);ON_InstanceRef missingRef;ON_CreateUuid(missingRef.m_instance_definition_uuid);missing.AddModelGeometryComponent(&missingRef,&top);rejects(missing,"om9-missing-block.3dm","Missing block definition");
ONX_Model cyclic;cyclic.AddModelComponent(layer);ON_UUID cycleId;ON_CreateUuid(cycleId);ON_InstanceRef cycleRef;cycleRef.m_instance_definition_uuid=cycleId;auto cycleMember=cyclic.AddModelGeometryComponent(&cycleRef,&a);ON_InstanceDefinition cycleDefinition;cycleDefinition.SetName(L"Cycle");cycleDefinition.SetId(cycleId);cycleDefinition.AddInstanceGeometryId(cycleMember.ModelComponent()->Id());cyclic.AddModelComponent(cycleDefinition);cyclic.AddModelGeometryComponent(&cycleRef,&top);rejects(cyclic,"om9-cycle-block.3dm","Cyclic block definition");
std::cout<<"nested blocks PASS\n";return 0;}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
