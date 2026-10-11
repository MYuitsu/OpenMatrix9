// SPDX-License-Identifier: LGPL-2.1-or-later
// GUI-thread kernel observations/copies. Gesture policy and transforms are Rust.
#include "GumballGeometry.h"
#include "GumballFrame.h"
#include "CoreLayers.h"
#include "CoreUnits.h"
#include "CoreSourceAttributes.h"
#include "CoreDiscreteSnapshot.h"
#include "NativeInputTree.h"
#include <Python.h>
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObjectGroup.h>
#include <App/GeoFeature.h>
#include <App/GeoFeatureGroupExtension.h>
#include <App/PropertyGeo.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/View3DInventor.h>
#include <Gui/Selection/SelectionObject.h>
#include <Mod/Part/App/PartFeature.h>
#include <Mod/Part/App/TopoShape.h>
#include <Mod/Mesh/App/Mesh.h>
#include <Mod/Mesh/App/MeshFeature.h>
#include <Mod/Points/App/Points.h>
#include <Mod/Points/App/PointsFeature.h>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <GProp_GProps.hxx>
#include <BRep_Tool.hxx>
#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <TopoDS.hxx>
#include <TopExp_Explorer.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <QApplication>
#include <QThread>
#include <algorithm>
#include <set>
#include <variant>
#include <stdexcept>
namespace OpenMatrix9Gui {
struct GumballNativeItem {
    ObjectSnapshot snapshot;
    std::uint64_t root;
    QJsonObject attributes;
    std::variant<Part::TopoShape,Mesh::MeshObject,Points::PointKernel> geometry;
};
struct GumballNativeData {std::vector<GumballNativeItem> items;};
namespace {
constexpr double identityFrame[12]={0,0,0,1,0,0,0,1,0,0,0,1};
Base::Matrix4D matrix(const double* values){Base::Matrix4D m;for(unsigned r=0;r<4;++r)for(unsigned c=0;c<4;++c)m[r][c]=values[4*r+c];return m;}
void guiThread(){if(!qApp||QThread::currentThread()!=qApp->thread())throw std::runtime_error("Gumball requires the GUI thread");}
Om9GumballSource source(const ObjectSnapshot& s){Om9GumballSource v{};v.document=s.stamp.document;v.object=s.stamp.object;v.generation=s.stamp.generation;v.revision=s.stamp.revision;v.epoch=s.stamp.epoch;std::copy(s.world.begin(),s.world.end(),v.world);return v;}
struct Session {std::uint64_t id=om9_gumball_create();Session(){if(!id)throw std::runtime_error("Gumball session budget exhausted");Om9GumballEvent e{};e.pressed=1;if(om9_gumball_event(id,&e))throw std::runtime_error("Cannot enable gumball session");}~Session(){om9_gumball_destroy(id);}};
std::vector<std::uint64_t> selectedRoots(const std::vector<Om9GumballSource>& list,const CoordinateContext& cp){
    Session session;Om9GumballBegin b{};b.document=cp.documentOwner;b.view_generation=cp.frameRevision?cp.frameRevision:1;b.unit_generation=1;b.mm_per_unit=1;std::copy(std::begin(identityFrame),std::end(identityFrame),b.frame);std::copy(cp.frame.begin(),cp.frame.end(),b.cplane);b.ray_origin[2]=10;b.ray_direction[2]=-1;
    if(om9_gumball_begin(session.id,&b,list.data(),list.size()))throw std::runtime_error("Invalid gumball selection ownership");
    std::size_t count=0;if(om9_gumball_sources(session.id,nullptr,0,&count))throw std::runtime_error("Cannot normalize gumball selection");std::vector<Om9GumballSource> roots(count);
    if(om9_gumball_sources(session.id,roots.data(),roots.size(),&count))throw std::runtime_error("Cannot capture gumball selection roots");std::vector<std::uint64_t> result;for(const auto& root:roots)result.push_back(root.object);return result;
}
void collect(App::Document& doc,App::DocumentObject& object,std::uint64_t root,GumballInputs& out,GumballNativeData& data,std::set<std::string>& visited,unsigned depth){
    if(depth>64||visited.size()>=16384||!visited.insert(object.getNameInDocument()).second)throw std::runtime_error("Gumball ownership tree is cyclic or exceeds input budget");
    if(!CoreLayers::objectEditable(&object))throw std::runtime_error("Gumball source is hidden, locked or not selectable");
    auto guard=captureObject(doc,object.getNameInDocument());out.witnesses.push_back(guard);
    if(auto* group=object.getExtension<App::GroupExtension>();group&&!object.isDerivedFrom(Base::Type::fromName("App::Link"))){for(auto* child:group->Group.getValues())if(child)collect(doc,*child,root,out,data,visited,depth+1);return;}
    const auto global=App::GeoFeature::getGlobalPlacement(&object);auto* place=object.getPropertyByName<App::PropertyPlacement>("Placement");
    const auto parent=place?global.toMatrix()*place->getValue().inverse().toMatrix():Base::Matrix4D();
    GumballNativeItem item{guard,root,sourceAttributes(object,guard),Part::TopoShape()};
    if(auto* mesh=dynamic_cast<Mesh::Feature*>(&object)){
        meshInputBudget(mesh->Mesh.getValue());meshSnapshot(mesh->Mesh.getValue());
        auto owned=mesh->Mesh.getValue();owned.transformGeometry(parent);item.geometry=std::move(owned);
    }else if(auto* points=dynamic_cast<Points::Feature*>(&object)){
        discreteSnapshot(object,false);auto owned=points->Points.getValue();owned.transformGeometry(parent);item.geometry=std::move(owned);
    }else{
        auto shape=Part::Feature::getTopoShape(&object,Part::ShapeOption::ResolveLink|Part::ShapeOption::Transform);
        if(shape.isNull())throw std::runtime_error("UnsupportedRepresentation: gumball has no native geometry adapter for "+guard.objectName);
        ThreeDm::InputTree::validate(shape.getShape());shape=Part::TopoShape(BRepBuilderAPI_Copy(shape.getShape(),true,false).Shape());shape.transformShape(parent,true);
        item.geometry=std::move(shape);
    }
    data.items.push_back(std::move(item));
}
QJsonObject geometryFacts(const GumballNativeItem& item){
    QJsonObject row{{"name",QString::fromStdString(item.snapshot.objectName)}};Base::BoundBox3d box;
    if(auto* shape=std::get_if<Part::TopoShape>(&item.geometry)){
        box=shape->getBoundBoxOptimal();row["representation"]="BRep";row["valid"]=BRepCheck_Analyzer(shape->getShape()).IsValid();GProp_GProps props;BRepGProp::VolumeProperties(shape->getShape(),props);row["volume"]=props.Mass();
        TopExp_Explorer edge(shape->getShape(),TopAbs_EDGE);if(edge.More()){double first,last;auto curve=BRep_Tool::Curve(TopoDS::Edge(edge.Current()),first,last);if(!curve.IsNull()){row["curve_wrapper"]=curve->DynamicType()->Name();for(unsigned depth=0;depth<64;++depth){auto trim=Handle(Geom_TrimmedCurve)::DownCast(curve);if(trim.IsNull())break;curve=trim->BasisCurve();}row["curve_type"]=curve->DynamicType()->Name();}}
    }else if(auto* mesh=std::get_if<Mesh::MeshObject>(&item.geometry)){box=mesh->getBoundBox();row["representation"]="Mesh";row["points"]=double(mesh->countPoints());}
    else {const auto& points=std::get<Points::PointKernel>(item.geometry);box=points.getBoundBox();row["representation"]="Points";row["points"]=double(points.size());}
    if(box.IsValid()){row["xmin"]=box.MinX;row["ymin"]=box.MinY;row["zmin"]=box.MinZ;row["bounds_x"]=box.MaxX-box.MinX;row["bounds_y"]=box.MaxY-box.MinY;row["bounds_z"]=box.MaxZ-box.MinZ;}return row;
}
PyObject* factsAPI(PyObject*,PyObject* args){
    const char *document,*handle,*value,*alignment;PyObject* names;
    if(!PyArg_ParseTuple(args,"sOsss",&document,&names,&handle,&value,&alignment))return nullptr;
    try{
        guiThread();auto* doc=App::GetApplication().getDocument(document);if(!doc)throw std::runtime_error("Gumball document is missing");
        if(!PyList_Check(names)||PyList_Size(names)<1||PyList_Size(names)>16384)throw std::runtime_error("Invalid gumball selection list");
        std::vector<Gui::SelectionObject> selected;for(Py_ssize_t i=0;i<PyList_Size(names);++i){const char* name=PyUnicode_AsUTF8(PyList_GetItem(names,i));if(!name)throw std::runtime_error("Invalid gumball object name");auto* o=doc->getObject(name);if(!o)throw std::runtime_error("Gumball source is missing");selected.emplace_back(o);}
        auto* view=dynamic_cast<Gui::View3DInventor*>(Gui::Application::Instance->activeDocument()?Gui::Application::Instance->activeDocument()->getActiveView():nullptr);
        auto cp=captureCoordinateContext(*doc,view);auto inputs=captureGumballInputs(*doc,selected,cp);Session session;
        Om9GumballBegin b{};b.document=cp.documentOwner;b.view_generation=cp.frameRevision?cp.frameRevision:1;b.unit_generation=1;b.mm_per_unit=unitInputScale(*doc);b.ray_origin[0]=1;b.ray_origin[1]=1;b.ray_origin[2]=10;b.ray_direction[2]=-1;
        std::copy(cp.frame.begin(),cp.frame.end(),b.cplane);
        const std::string align=alignment;unsigned mode=align=="World"?2:align=="Object"?1:align=="CPlane"?0:99;
        if(mode==99)throw std::runtime_error("Invalid gumball alignment");auto frame=captureGumballFrame(inputs,view,mode);if(!frame.unresolved.isEmpty())throw std::runtime_error(frame.unresolved.toStdString());
        // Geometry-only fixture uses World origin for literal affine comparisons.
        if(mode==2)std::copy(std::begin(identityFrame),std::end(identityFrame),b.frame);else std::copy(frame.frame.begin(),frame.frame.end(),b.frame);
        const std::string key=handle;bool extrude=key=="ExtrudeZ";
        if(key=="MoveX")b.handle=0;else if(key=="MoveY")b.handle=1;else if(key=="MoveZ"||extrude)b.handle=2;else if(key=="ScaleX")b.handle=10;else if(key=="RotateZ")b.handle=9;else throw std::runtime_error("Invalid gumball fixture handle");
        if(extrude)b.ray_direction[1]=-1;
        if(om9_gumball_begin(session.id,&b,inputs.sources.data(),inputs.sources.size()))throw std::runtime_error("Cannot begin native gumball staging");
        if(extrude){Om9GumballEvent move{};move.kind=2;std::copy(std::begin(b.ray_origin),std::end(b.ray_origin),move.ray_origin);std::copy(std::begin(b.ray_direction),std::end(b.ray_direction),move.ray_direction);if(om9_gumball_event(session.id,&move))throw std::runtime_error("Cannot observe extrusion motion");Om9GumballEvent ctrl{};ctrl.kind=3;ctrl.key=2;ctrl.pressed=1;if(om9_gumball_event(session.id,&ctrl))throw std::runtime_error("Cannot observe extrusion modifier");}
        if(om9_gumball_input(session.id,reinterpret_cast<const unsigned char*>(value),std::char_traits<char>::length(value)))throw std::runtime_error("Invalid native gumball numeric input");
        Om9GumballSnapshot snap{};if(om9_gumball_snapshot(session.id,&snap))throw std::runtime_error("Cannot read gumball staging plan");
        auto staged=stageGumballBatch(inputs,snap);verifyGumballInputs(*doc,inputs);QJsonArray observedFrame;for(double v:frame.frame)observedFrame.append(v);staged.facts["frame"]=observedFrame;auto json=QJsonDocument(staged.facts).toJson(QJsonDocument::Compact);
        PyObject* module=PyImport_ImportModule("json");if(!module)return nullptr;PyObject* result=PyObject_CallMethod(module,"loads","s",json.constData());Py_DECREF(module);return result;
    }catch(const Base::Exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}catch(const Standard_Failure& e){PyErr_SetString(PyExc_RuntimeError,e.GetMessageString());}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());}return nullptr;
}
}
GumballInputs captureGumballInputs(App::Document& doc,const std::vector<Gui::SelectionObject>& selection,const CoordinateContext& cp){
    guiThread();verifyCoordinateContext(doc,cp);GumballInputs inputs;inputs.coordinates=cp;std::vector<ObjectSnapshot> raw;std::set<std::uint64_t> ids;
    for(const auto& s:selection){auto* object=doc.getObject(s.getFeatName());if(!object||object->getDocument()!=&doc||!CoreLayers::objectEditable(object))throw std::runtime_error("Select editable objects in the active document");auto snapshot=captureObject(doc,object->getNameInDocument(),s.getSubNames().empty()?std::string():s.getSubNames().front());if(ids.insert(snapshot.stamp.object).second)raw.push_back(std::move(snapshot));}
    for(const auto& snap:raw){auto row=source(snap);for(const auto& parent:snap.ancestors)if(ids.contains(parent.object)){row.parent=parent.object;break;}inputs.sources.push_back(row);}
    const auto roots=selectedRoots(inputs.sources,cp);auto data=std::make_shared<GumballNativeData>();std::set<std::string> visited;
    for(const auto id:roots){const auto found=std::find_if(raw.begin(),raw.end(),[id](const auto& s){return s.stamp.object==id;});inputs.roots.push_back(*found);collect(doc,*doc.getObject(found->objectName.c_str()),id,inputs,*data,visited,0);}
    if(data->items.empty())throw std::runtime_error("Gumball selection has no native geometry");inputs.native=std::move(data);verifyGumballInputs(doc,inputs);return inputs;
}
void verifyGumballInputs(App::Document& doc,const GumballInputs& inputs){guiThread();verifyCoordinateContext(doc,inputs.coordinates);for(const auto& root:inputs.roots)verifyObject(doc,root);for(const auto& witness:inputs.witnesses){verifyObject(doc,witness);if(!CoreLayers::objectEditable(doc.getObject(witness.objectName.c_str())))throw std::runtime_error("Gumball source became locked, hidden or unselectable");}}
GumballStagedBatch stageGumballBatch(const GumballInputs& inputs,const Om9GumballSnapshot& plan){
    guiThread();if(!inputs.native||plan.phase<2||plan.phase>5)throw std::runtime_error("No active gumball geometry plan");auto data=std::make_shared<GumballNativeData>(*inputs.native);const auto affine=matrix(plan.matrix);QJsonArray rows;
    for(auto& item:data->items){
        if(auto* shape=std::get_if<Part::TopoShape>(&item.geometry)){
            if(plan.operation==3){if(shape->getShape().ShapeType()!=TopAbs_EDGE&&shape->getShape().ShapeType()!=TopAbs_WIRE&&shape->getShape().ShapeType()!=TopAbs_FACE)throw std::runtime_error("Gumball extrusion requires a planar curve or face");if(!shape->isPlanar())throw std::runtime_error("Gumball extrusion requires planar input");const gp_Vec displacement(plan.matrix[3],plan.matrix[7],plan.matrix[11]);if(displacement.SquareMagnitude()==0)throw std::runtime_error("Gumball extrusion distance is zero");*shape=Part::TopoShape(BRepPrimAPI_MakePrism(shape->getShape(),displacement,true,true).Shape());}
            else if(plan.operation==2)*shape=Part::TopoShape(shape->transformGShape(affine,true));else shape->transformShape(affine,true);
            ThreeDm::InputTree::validate(shape->getShape());if(!BRepCheck_Analyzer(shape->getShape()).IsValid())throw std::runtime_error("Gumball produced invalid native BRep");
        }else {if(plan.operation==3)throw std::runtime_error("Gumball extrusion requires native planar BRep geometry");std::visit([&](auto& geometry){geometry.transformGeometry(affine);},item.geometry);}
        rows.append(geometryFacts(item));
    }
    QJsonArray roots;for(const auto& root:inputs.roots)roots.append(QString::fromStdString(root.objectName));return {data,{{"selected_roots",roots},{"staged",rows}}};
}
QJsonObject gumballInputFacts(const GumballInputs& inputs){QJsonArray rows;for(const auto& item:inputs.native->items)rows.append(geometryFacts(item));return {{"items",rows}};}
QJsonObject gumballObjectFrameFacts(const GumballInputs& inputs){
    if(inputs.roots.size()!=1)throw std::runtime_error("Object alignment requires one unambiguous object frame");
    const auto& root=inputs.roots.front();Base::Vector3d x(root.world[0],root.world[4],root.world[8]),normal(root.world[2],root.world[6],root.world[10]);
    if(!inputs.native->items.empty())if(const auto* shape=std::get_if<Part::TopoShape>(&inputs.native->items.front().geometry)){
        auto sub=root.subelement.empty()?shape->getShape():shape->getSubShape(root.subelement.c_str());
        if(sub.ShapeType()==TopAbs_FACE){BRepAdaptor_Surface surface(TopoDS::Face(sub));double u=(surface.FirstUParameter()+surface.LastUParameter())/2,v=(surface.FirstVParameter()+surface.LastVParameter())/2;gp_Pnt p;gp_Vec du,dv;surface.D1(u,v,p,du,dv);x=Base::Vector3d(du.X(),du.Y(),du.Z());auto n=du.Crossed(dv);normal=Base::Vector3d(n.X(),n.Y(),n.Z());}
        else if(sub.ShapeType()==TopAbs_EDGE){BRepAdaptor_Curve curve(TopoDS::Edge(sub));gp_Pnt p;gp_Vec tangent,second;curve.D2((curve.FirstParameter()+curve.LastParameter())/2,p,tangent,second);x=Base::Vector3d(tangent.X(),tangent.Y(),tangent.Z());auto n=tangent.Crossed(second);if(n.SquareMagnitude()>0)normal=Base::Vector3d(n.X(),n.Y(),n.Z());}
    }
    return {{"x",QJsonArray{x.x,x.y,x.z}},{"normal",QJsonArray{normal.x,normal.y,normal.z}}};
}
void addGumballGeometryMethods(PyObject* module){static PyMethodDef methods[]={{"gumballGeometryFacts",factsAPI,METH_VARARGS,"Read private native staging facts; no document mutation or UI proof."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
}
