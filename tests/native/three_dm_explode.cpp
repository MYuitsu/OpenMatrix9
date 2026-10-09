#include "ThreeDmExplode.h"
#include <BRepGProp.hxx>
#include <BRep_Tool.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <TopoDS.hxx>
#include <TopExp_Explorer.hxx>
#include <GProp_GProps.hxx>
#include <fstream>
#include <iostream>
#include <functional>
#include <cmath>
using namespace OpenMatrix9Gui::ThreeDm;
namespace {
void require(bool ok,const char* message){if(!ok)throw ExchangeError(message);}
void close(double a,double b){require(std::abs(a-b)<1e-6,"independent numerical expectation");}
std::string id(const ON_UUID& value){char s[37]{};ON_UuidToString(value,s);return s;}
std::string add(ONX_Model& m,const ON_Geometry& g,const wchar_t* name,bool member=false){
    ON_3dmObjectAttributes a;a.m_name=name;if(member)a.SetMode(ON::idef_object);
    a.SetColorSource(ON::color_from_parent);a.m_color=ON_Color(23,45,67);
    auto ref=m.AddModelGeometryComponent(&g,&a);require(!ref.IsEmpty(),"fixture add");return id(ref.ModelComponent()->Id());
}
std::string definition(ONX_Model& m,const wchar_t* name,const std::string& member){
    ON_InstanceDefinition d;d.SetName(name);d.AddInstanceGeometryId(ON_UuidFromString(member.c_str()));
    auto ref=m.AddModelComponent(d);require(!ref.IsEmpty(),"definition add");return id(ref.ModelComponent()->Id());
}
void rejects(const std::function<void()>& operation,const char* match){bool caught=false;try{operation();}catch(const ExchangeError& e){caught=std::string(e.what()).find(match)!=std::string::npos;}require(caught,match);}
const TopoDS_Shape& shape(const ExplodeComponent& c){return std::get<TopoDS_Shape>(c.geometry);}
double length(const TopoDS_Shape& s){GProp_GProps value;BRepGProp::LinearProperties(s,value);return value.Mass();}
}
int main(int argc,char** argv){try{
    ON::Begin();const auto dir=argc>1?std::filesystem::path(argv[1]):std::filesystem::temp_directory_path()/"om9-explode";
    std::filesystem::create_directories(dir);
    ONX_Model m;m.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Inches);
    ON_Layer layer;layer.SetName(L"Explode");m.AddModelComponent(layer);
    ON_DimStyle style;style.SetName(L"ExplodeStyle");style.SetTextHeight(.25);style.SetExtOffset(0);style.SetExtExtension(0);style.SetDimExtension(0);
    style.SetDimTextLocation(ON_DimStyle::TextLocation::AboveDimLine);style.SetArrowSize(.1);style.SetDimScale(1);
    auto styleRef=m.AddModelComponent(style);auto savedStyle=ON_DimStyle::Cast(styleRef.ModelComponent());require(savedStyle,"fixture dimstyle");
    ON_LineCurve memberLine(ON_3dPoint(1,2,3),ON_3dPoint(4,2,3));auto member=add(m,memberLine,L"Member",true);
    auto inner=definition(m,L"Inner",member);
    ON_InstanceRef nested;nested.m_instance_definition_uuid=ON_UuidFromString(inner.c_str());nested.m_xform=ON_Xform::TranslationTransformation(ON_3dVector(10,0,0));
    auto nestedId=add(m,nested,L"Nested",true);auto outer=definition(m,L"Outer",nestedId);
    ON_InstanceRef block;block.m_instance_definition_uuid=ON_UuidFromString(outer.c_str());block.m_xform=ON_Xform::IdentityTransformation;
    block.m_xform[0][0]=-2;block.m_xform[1][1]=3;block.m_xform[2][2]=4;block.m_xform[1][3]=20;
    auto blockId=add(m,block,L"Block");
    ON_Text text;ON_Plane textPlane=ON_Plane::World_xy;textPlane.origin=ON_3dPoint(2,5,7);textPlane.UpdateEquation();
    require(text.Create(L"I",savedStyle,textPlane),"text fixture");auto textId=add(m,text,L"Text");
    ON_DimLinear linear;require(linear.Create(ON::AnnotationType::Aligned,savedStyle->Id(),ON_Plane::World_xy,ON_3dVector::XAxis,ON_3dPoint(0,0,0),ON_3dPoint(10,0,0),ON_3dPoint(0,3,0)),"linear fixture");
    linear.SetUserText(L"Length <> in");auto dimensionId=add(m,linear,L"Dimension");
    ON_NurbsCage cage(ON_BoundingBox(ON_3dPoint(0,0,0),ON_3dPoint(2,3,4)),3,2,2,3,2,2);
    require(cage.SetCV(1,1,1,ON_3dPoint(1.75,3.5,5)),"deformed cage CV");auto cageId=add(m,cage,L"Cage");
    ON_MorphControl control;control.m_varient=1;memberLine.GetNurbForm(control.m_nurbs_curve);control.m_nurbs_curve0=control.m_nurbs_curve;
    auto curveControlId=add(m,control,L"CurveControl");
    control.m_varient=2;std::unique_ptr<ON_NurbsSurface> controlSurface(cage.IsoSurface(2,0));control.m_nurbs_surface=*controlSurface;control.m_nurbs_surface0=*controlSurface;
    auto surfaceControlId=add(m,control,L"SurfaceControl");
    control.m_varient=3;control.m_nurbs_cage=cage;auto cageControlId=add(m,control,L"CageControl");
    auto path=dir/"explode-special.3dm";require(m.Write(path.c_str(),8,nullptr),"special fixture write");
    {std::ofstream manifest(dir/"explode-special.ids.json");manifest<<"{\"block\":\""<<blockId<<"\",\"text\":\""<<textId<<"\",\"dimension\":\""<<dimensionId<<"\",\"cage\":\""<<cageId<<"\",\"curve_control\":\""<<curveControlId<<"\",\"surface_control\":\""<<surfaceControlId<<"\",\"cage_control\":\""<<cageControlId<<"\"}";}
    auto exploded=explodeArchiveRecord(path,blockId);require(exploded.flattenedNestedBlocks,"nested policy");require(exploded.components.size()==1,"nested member count");
    close(length(shape(exploded.components[0])),6*25.4);
    double first,last;auto native=BRep_Tool::Curve(TopoDS::Edge(shape(exploded.components[0])),first,last);
    auto p=native->Value(first),q=native->Value(last);close(p.X(),-22*25.4);close(p.Y(),26*25.4);close(p.Z(),12*25.4);close(q.X(),-28*25.4);
    require(exploded.sourceUuid==blockId&&exploded.components[0].sourceUuid==member,"source identity proof");
    require(exploded.components[0].color==std::array<int,3>{23,45,67},"inherited block color");
    auto volume=explodeArchiveRecord(path,cageId);require(volume.components.size()==26,"six boundaries plus twenty cage lattice edges");
    for(std::size_t i=0;i<6;++i)require(volume.components[i].role=="boundary-surface"&&BRepCheck_Analyzer(shape(volume.components[i])).IsValid(),"valid exact cage boundary");
    auto descriptor=volume.components[0].cage;require(descriptor&&descriptor->counts==std::array<int,3>{3,2,2}&&descriptor->degrees==std::array<int,3>{2,1,1},"cage basis descriptor");
    require(descriptor->points.size()==12&&descriptor->knots[0].size()==4,"cage CV and compact knot counts");
    close(descriptor->points[7][0],1.75*25.4);close(descriptor->points[7][1],3.5*25.4);close(descriptor->points[7][2],5*25.4);
    auto boundary=BRep_Tool::Surface(TopoDS::Face(shape(volume.components[5])));auto surfacePoint=boundary->Value(.5,1);
    close(surfacePoint.X(),1.375*25.4);close(surfacePoint.Y(),3.25*25.4);close(surfacePoint.Z(),4.5*25.4);
    auto curveControl=explodeArchiveRecord(path,curveControlId);require(curveControl.components.size()==1&&curveControl.components[0].role=="control-curve","curve control decode");close(length(shape(curveControl.components[0])),3*25.4);
    auto surface=explodeArchiveRecord(path,surfaceControlId);require(surface.components.size()==1&&surface.components[0].role=="control-surface","surface control decode");
    GProp_GProps area;BRepGProp::SurfaceProperties(shape(surface.components[0]),area);close(area.Mass(),6*25.4*25.4);
    auto cageControl=explodeArchiveRecord(path,cageControlId);require(cageControl.components.size()==26,"morph cage variant decode");
    auto dimension=explodeArchiveRecord(path,dimensionId);require(dimension.components.size()==6,"three display lines, two arrows and editable text");
    close(length(shape(dimension.components[0])),3*25.4);close(length(shape(dimension.components[1])),3*25.4);close(length(shape(dimension.components[2])),10*25.4);
    const auto& label=std::get<ExplodeText>(dimension.components.back().geometry);require(label.text.find("Length 10")!=std::string::npos&&label.text.find("in")!=std::string::npos,"dimension label evaluated and preserved");close(label.heightMm,.25*25.4);close(label.worldTransform[0],25.4);close(label.worldTransform[3],5*25.4);require(!label.fontFamily.empty(),"dimension font metadata");
    // A platform lacking glyph services must refuse text atomically. Native
    // curves are otherwise verified against the font's actual source plane.
    bool textContours=false;try{auto outlines=explodeArchiveRecord(path,textId);require(!outlines.components.empty(),"glyph curve count");
        for(const auto& c:outlines.components){require(c.role=="text-contour","text curve role");for(TopExp_Explorer it(shape(c),TopAbs_VERTEX);it.More();it.Next())close(BRep_Tool::Pnt(TopoDS::Vertex(it.Current())).Z(),7*25.4);}textContours=true;
    }catch(const ExchangeError& e){require(std::string(e.what()).find("font")!=std::string::npos,"only absent font services may reject text");std::cout<<"Text contours unavailable: "<<e.what()<<'\n';}
    rejects([&]{explodeArchiveRecord(path,"not-a-uuid");},"Invalid source UUID");
    rejects([&]{explodeArchiveRecord(path,id(ON_CreateId()));},"absent");
    ONX_Model angularModel;angularModel.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);angularModel.AddModelComponent(layer);
    ON_DimStyle angularStyle=style;angularStyle.SetId(ON_CreateId());angularStyle.SetDimTextOrientation(ON::TextOrientation::InPlane);
    angularStyle.SetDimTextAngleStyle(ON_DimStyle::ContentAngleStyle::Aligned);
    auto angularStyleRef=angularModel.AddModelComponent(angularStyle);
    ON_DimAngular angle;require(angle.Create(angularStyleRef.ModelComponent()->Id(),ON_Plane::World_xy,ON_3dVector::XAxis,ON_3dPoint(0,0,0),ON_3dPoint(1,0,0),ON_3dPoint(0,1,0),ON_3dPoint(3/std::sqrt(2.0),3/std::sqrt(2.0),0)),"angular fixture");
    auto angleId=add(angularModel,angle,L"Angular");auto angularPath=dir/"explode-angular.3dm";require(angularModel.Write(angularPath.c_str(),8,nullptr),"angular write");
    auto angular=explodeArchiveRecord(angularPath,angleId);require(angular.components.size()==6,"angular extension lines, arc, arrows and text");
    close(length(shape(angular.components[0])),2);close(length(shape(angular.components[1])),2);close(length(shape(angular.components[2])),3*ON_PI/2);
    require(std::get<ExplodeText>(angular.components.back().geometry).text.find("90")!=std::string::npos,"angular label remains evaluated text");
    ONX_Model missing;missing.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);missing.AddModelComponent(layer);ON_InstanceRef missingBlock;ON_CreateUuid(missingBlock.m_instance_definition_uuid);auto missingId=add(missing,missingBlock,L"Missing");
    auto missingPath=dir/"explode-missing.3dm";require(missing.Write(missingPath.c_str(),8,nullptr),"missing fixture write");
    rejects([&]{explodeArchiveRecord(missingPath,missingId);},"Missing block definition");
    ONX_Model cycle;cycle.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(ON::LengthUnitSystem::Millimeters);cycle.AddModelComponent(layer);ON_UUID cycleUuid;ON_CreateUuid(cycleUuid);ON_InstanceRef cycleRef;cycleRef.m_instance_definition_uuid=cycleUuid;
    auto cycleMember=add(cycle,cycleRef,L"CycleMember",true);ON_InstanceDefinition cyclic;cyclic.SetId(cycleUuid);cyclic.SetName(L"Cycle");cyclic.AddInstanceGeometryId(ON_UuidFromString(cycleMember.c_str()));cycle.AddModelComponent(cyclic);auto cycleId=add(cycle,cycleRef,L"Cycle");
    auto cyclePath=dir/"explode-cycle.3dm";require(cycle.Write(cyclePath.c_str(),8,nullptr),"cycle write");rejects([&]{explodeArchiveRecord(cyclePath,cycleId);},"Cyclic block definition");
    std::cout<<"3DM Explode PASS; text contours "<<(textContours?"available":"safely rejected")<<"; fixtures "<<dir.string()<<'\n';return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
