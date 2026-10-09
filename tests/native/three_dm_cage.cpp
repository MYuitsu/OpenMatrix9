// SPDX-License-Identifier: LGPL-2.1-or-later
#include "ThreeDmCage.h"
#include "ThreeDmArchive.h"
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <cmath>
#include <functional>
#include <fstream>
#include <iomanip>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
namespace {
void require(bool ok,const char* why){if(!ok)throw ExchangeError(why);}
void close(double a,double b){require(std::abs(a-b)<1e-8,"independent numeric expectation");}
std::string uuid(const ON_UUID& id){char s[37]{};ON_UuidToString(id,s);return s;}
std::string add(ONX_Model& model,const ON_Geometry& g,const ON_UUID& id=ON_nil_uuid,const wchar_t* name=L"Cage fixture"){
    ON_3dmObjectAttributes a;a.m_name=name;a.m_uuid=id;
    auto ref=model.AddModelGeometryComponent(&g,&a);require(!ref.IsEmpty(),"fixture add");return uuid(ref.ModelComponent()->Id());
}
void rejects(const std::function<void()>& operation,const char* expected){
    try{operation();}catch(const ExchangeError& e){require(std::string(e.what()).find(expected)!=std::string::npos,expected);return;}
    throw ExchangeError(std::string("Expected rejection: ")+expected);
}
ON_NurbsCage original(){return ON_NurbsCage(ON_BoundingBox(ON_3dPoint(10,20,30),ON_3dPoint(12,23,34)),2,2,2,2,2,2);}
ON_MorphControl morph(){
    ON_MorphControl c;c.m_varient=3;c.m_nurbs_cage=original();
    require(ON_GetCageXform(c.m_nurbs_cage,c.m_nurbs_cage0),"original reference fixture");
    require(c.m_nurbs_cage.Transform(ON_Xform::TranslationTransformation(4,0,0)),"current cage translation");
    c.m_sporh_tolerance=.002;c.m_sporh_bPreserveStructure=true;return c;
}
void initialize(ONX_Model& m,ON::LengthUnitSystem units=ON::LengthUnitSystem::Inches){
    m.m_settings.m_ModelUnitsAndTolerances.m_unit_system=ON_UnitSystem(units);
    m.m_settings.m_ModelUnitsAndTolerances.m_absolute_tolerance=.001;
    ON_Layer l;l.SetName(L"Cage test");require(!m.AddModelComponent(l).IsEmpty(),"fixture layer");
}
void write(ONX_Model& m,const std::filesystem::path& p){require(m.Write(p.c_str(),5,nullptr),"Rhino 5 fixture write");}
std::array<double,3> applyMatrix(const std::array<double,16>& matrix,const std::array<double,3>& point){
    std::array<double,3> result{};for(int r=0;r<3;++r){result[r]=matrix[r*4+3];for(int c=0;c<3;++c)result[r]+=matrix[r*4+c]*point[c];}return result;
}
void nativeFixture(const std::filesystem::path& dir,bool nonlinear){
    ONX_Model fixture;initialize(fixture,ON::LengthUnitSystem::Millimeters);auto control=morph();
    if(nonlinear){ON_3dPoint corner;require(control.m_nurbs_cage.GetCV(1,1,1,corner),"native fixture corner");
        corner.x+=.5;corner.z+=1;require(control.m_nurbs_cage.SetCV(1,1,1,corner),"native nonaffine current cage");}
    const std::array<std::array<double,3>,3> parameters{{{.25,.25,.5},{.75,.25,.5},{.25,.75,.5}}};
    std::array<ON_3dPoint,3> current{};ON_Mesh mesh;
    for(std::size_t i=0;i<parameters.size();++i){const auto& p=parameters[i];
        current[i]=control.m_nurbs_cage.PointAt(p[0],p[1],p[2]);
        mesh.m_V.Append(ON_3fPoint(current[i].x,current[i].y,current[i].z));}
    mesh.m_F.Append(ON_MeshFace{{0,1,2,2}});mesh.ComputeVertexNormals();
    require(mesh.IsValid(),"native captive mesh fixture");
    auto captive=add(fixture,mesh,ON_nil_uuid,L"CapturedMesh");
    auto unrelated=add(fixture,mesh,ON_nil_uuid,L"UnrelatedMesh"); // coincident, explicitly NOT captured
    control.m_captive_id.AddUuid(ON_UuidFromString(captive.c_str()));
    auto record=add(fixture,control,ON_nil_uuid,L"RhinoCageControl");
    const std::string stem=nonlinear?"cage-native-nonaffine":"cage-native";
    write(fixture,dir/(stem+".3dm"));auto decoded=cageArchiveRecord(dir/(stem+".3dm"),record);
    require(decoded.captiveSourceUuids==std::vector<std::string>{captive},"native fixture explicit captive UUID");
    auto currentArchive=readArchive(dir/(stem+".3dm"),0,true);bool found=false;
    for(const auto& item:currentArchive.items)if(item.sourceUuid==captive){found=true;const auto& saved=std::get<MeshData>(item.geometry);
        require(saved.vertices.size()==3,"native saved mesh vertex count");
        for(std::size_t i=0;i<current.size();++i){close(saved.vertices[i][0],current[i].x);close(saved.vertices[i][1],current[i].y);close(saved.vertices[i][2],current[i].z);}}
    require(found,"native fixture stored current geometry");
    std::ofstream manifest(dir/(stem+".ids.json"));manifest<<std::setprecision(17);
    manifest<<"{\"control\":\""<<record<<"\",\"captive\":\""<<captive<<"\",\"unrelated\":\""<<unrelated
        <<"\",\"scale_mm\":1,\"nonaffine_current\":"<<(nonlinear?"true":"false")<<",\"bind_parameters\":[";
    for(std::size_t i=0;i<parameters.size();++i){if(i)manifest<<',';const auto& p=parameters[i];manifest<<'['<<p[0]<<','<<p[1]<<','<<p[2]<<']';}
    manifest<<"],\"current_vertices_mm\":[";
    for(std::size_t i=0;i<current.size();++i){if(i)manifest<<',';const auto& p=current[i];manifest<<'['<<p.x<<','<<p.y<<','<<p.z<<']';}
    manifest<<"],\"control_points_mm\":[";
    for(std::size_t i=0;i<decoded.currentCage.points.size();++i){if(i)manifest<<',';const auto& p=decoded.currentCage.points[i];manifest<<'['<<p[0]<<','<<p[1]<<','<<p[2]<<']';}
    manifest<<"],\"original_world_mm_to_parameters\":[";
    for(std::size_t i=0;i<16;++i){if(i)manifest<<',';manifest<<decoded.originalReference->worldMmToParameters[i];}
    manifest<<"]}";require(manifest.good(),"native fixture manifest write");
}
}
int main(int argc,char** argv){try{
    ON::Begin();const auto dir=argc>1?std::filesystem::path(argv[1]):std::filesystem::temp_directory_path()/"om9-cage";
    std::filesystem::create_directories(dir);
    ONX_Model m;initialize(m);auto c=morph();require(c.m_nurbs_cage.MakeRational(),"rational cage fixture");
    // Weight two on the high corner, preserving its Euclidean CV.
    require(c.m_nurbs_cage.SetCV(1,1,1,ON_4dPoint(32,46,68,2)),"rational weighted CV");
    const ON_3dPoint currentOutput(100./9+4,195./9,290./9);
    auto captive=add(m,ON_Point(currentOutput));c.m_captive_id.AddUuid(ON_UuidFromString(captive.c_str()));
    auto unrelated=add(m,ON_Point(currentOutput));auto control=add(m,c);auto plain=add(m,original());
    const auto path=dir/"cage-rhino5.3dm";write(m,path);
    auto decoded=cageArchiveRecord(path,control);
    require(decoded.isMorphControl&&decoded.originalReference.has_value(),"typed morph-control/reference");
    require(decoded.sourceUuid==control&&decoded.sourceClass=="ON_MorphControl","source record identity");
    require(decoded.captiveSourceUuids==std::vector<std::string>{captive},"explicit IDs only; no nearby/bounding-box guess");
    require(decoded.captiveSourceUuids[0]!=unrelated,"unrelated coincident point excluded");
    close(decoded.scaleMm,25.4);close(decoded.toleranceMm,.001*25.4);close(decoded.morphToleranceMm,.002*25.4);
    require(decoded.preserveStructure&&!decoded.quickPreview,"morph options retained");
    require(decoded.currentCage.rational&&decoded.currentCage.counts==std::array<int,3>{2,2,2}
            &&decoded.currentCage.degrees==std::array<int,3>{1,1,1},"rational basis retained");
    for(const auto& knots:decoded.currentCage.fullKnots)require(knots==std::vector<double>{0,0,1,1},"full standard knot vector");
    close(decoded.currentCage.weights[7],2);close(decoded.currentCage.points[7][0],16*25.4);
    const auto& reference=*decoded.originalReference;
    auto param=applyMatrix(reference.worldMmToParameters,{11*25.4,21.5*25.4,32*25.4});
    for(auto p:param)close(p,.5);
    auto point=applyMatrix(reference.parametersToWorldMm,param);close(point[0],11*25.4);close(point[1],21.5*25.4);close(point[2],32*25.4);
    require(decoded.captiveGeometryState==CaptiveGeometryState::CurrentArchiveOutputOnly,"no invented undeformed captive snapshot");
    auto archive=readArchive(path,0,true);bool found=false;
    for(const auto& item:archive.items)if(item.sourceUuid==captive){found=true;
        auto p=BRep_Tool::Pnt(TopoDS::Vertex(std::get<TopoDS_Shape>(item.geometry)));
        close(p.X(),currentOutput.x*25.4);close(p.Y(),currentOutput.y*25.4);close(p.Z(),currentOutput.z*25.4);}
    require(found,"stored captive output reproduced");
    auto volume=cageArchiveRecord(path,plain);require(!volume.isMorphControl&&!volume.originalReference&&volume.captiveSourceUuids.empty(),"plain cage has no inferred original/captives");
    rejects([&]{cageArchiveRecord(path,"invalid");},"Invalid source UUID");
    rejects([&]{cageArchiveRecord(path,uuid(ON_CreateId()));},"absent");
    rejects([&]{cageArchiveRecord(path,captive);},"not a NURBS cage");
    auto scenario=[&](const char* name,const std::function<void(ONX_Model&,ON_MorphControl&,ON_UUID)>& change,const char* error){
        ONX_Model fixture;initialize(fixture);auto mc=morph();const auto self=ON_CreateId();change(fixture,mc,self);
        auto record=add(fixture,mc,self);auto file=dir/(std::string(name)+".3dm");write(fixture,file);
        rejects([&]{cageArchiveRecord(file,record);},error);
    };
    scenario("cage-missing",[](auto&,auto& mc,auto){mc.m_captive_id.AddUuid(ON_CreateId());},"Missing captive relationship");
    scenario("cage-self",[](auto&,auto& mc,auto self){mc.m_captive_id.AddUuid(self);},"cannot capture itself");
    scenario("cage-nil",[](auto&,auto& mc,auto){mc.m_captive_id.AddUuid(ON_nil_uuid);},"nil UUID");
    scenario("cage-duplicate",[](auto& fixture,auto& mc,auto){auto id=ON_UuidFromString(add(fixture,ON_Point(ON_3dPoint::Origin)).c_str());
        mc.m_captive_id.AddUuid(id,false);mc.m_captive_id.AddUuid(id,false);},"Duplicate captive relationship");
    scenario("cage-local",[](auto&,auto& mc,auto){require(mc.AddSphereLocalizer(ON_3dPoint::Origin,1,2),"localizer fixture");},"localizer attenuation cannot be discarded");
    scenario("cage-singular",[](auto&,auto& mc,auto){mc.m_nurbs_cage0[0][0]=0;},"singular original");
    scenario("cage-projective",[](auto&,auto& mc,auto){mc.m_nurbs_cage0[3][0]=.2;},"original cage reference");
    scenario("curve-control",[](auto&,auto& mc,auto){mc.m_varient=1;ON_LineCurve line(ON_3dPoint::Origin,ON_3dPoint(1,0,0));
        line.GetNurbForm(mc.m_nurbs_curve0);mc.m_nurbs_curve=mc.m_nurbs_curve0;},"Curve morph-control variant");
    scenario("surface-control",[](auto&,auto& mc,auto){mc.m_varient=2;std::unique_ptr<ON_NurbsSurface> surface(mc.m_nurbs_cage.IsoSurface(2,0));
        mc.m_nurbs_surface0=*surface;mc.m_nurbs_surface=*surface;},"Surface morph-control variant");
    scenario("cage-degree-limit",[](auto&,auto& mc,auto){
        mc.m_nurbs_cage=ON_NurbsCage(ON_BoundingBox(ON_3dPoint::Origin,ON_3dPoint(1,1,1)),34,2,2,34,2,2);
    },"count/degree exceeds decoder bounds");
    ONX_Model unitless;initialize(unitless,ON::LengthUnitSystem::None);auto empty=add(unitless,morph());auto customPath=dir/"cage-unitless.3dm";write(unitless,customPath);
    rejects([&]{cageArchiveRecord(customPath,empty);},"millimeters per file unit");
    auto custom=cageArchiveRecord(customPath,empty,2.5);close(custom.scaleMm,2.5);close(custom.currentCage.points[0][0],14*2.5);
    require(custom.captiveSourceUuids.empty(),"empty UUID table does not infer captives");
    ONX_Model domainModel;initialize(domainModel);auto domainControl=morph();
    auto domainOriginal=original();
    for(int k=0;k<domainOriginal.KnotCount(0);++k){
        require(domainOriginal.SetKnot(0,k,2+2*domainOriginal.Knot(0,k)),"original parameter domain fixture");
        require(domainControl.m_nurbs_cage.SetKnot(0,k,2+2*domainControl.m_nurbs_cage.Knot(0,k)),"current parameter domain fixture");
    }
    require(ON_GetCageXform(domainOriginal,domainControl.m_nurbs_cage0),"nonunit domain original map");
    auto domainId=add(domainModel,domainControl);auto domainPath=dir/"cage-parameter-domain.3dm";write(domainModel,domainPath);
    auto domainRecord=cageArchiveRecord(domainPath,domainId);
    require(domainRecord.currentCage.fullKnots[0]==std::vector<double>{2,2,4,4},"parameter knots are not length-scaled");
    close(applyMatrix(domainRecord.originalReference->worldMmToParameters,{11*25.4,21.5*25.4,32*25.4})[0],3);
    // Affine current cage: inverse-current binding keeps y unchanged. Reusing
    // original map on y adds the already applied translation a second time.
    auto affine=morph();const auto y=affine.m_nurbs_cage.PointAt(.5,.5,.5);
    const auto wrong=affine.m_nurbs_cage.PointAt(affine.m_nurbs_cage0*y);close(wrong.x,y.x+4);
    ON_Xform currentMap;require(ON_GetCageXform(affine.m_nurbs_cage,currentMap),"current inverse map");
    const auto unchanged=affine.m_nurbs_cage.PointAt(currentMap*y);close(unchanged.x,y.x);close(unchanged.y,y.y);close(unchanged.z,y.z);
    nativeFixture(dir,false);nativeFixture(dir,true);
    std::cout<<"3DM Cage PASS; Rhino 5 typed references, current outputs, explicit IDs, localizer rejection and units\n";return 0;
}catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}}
