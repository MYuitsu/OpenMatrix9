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
#include <App/PropertyStandard.h>
#include <App/Material.h>
#include <Gui/Control.h>
#include <Gui/ViewProviderGeometryObject.h>
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
#include <TopoDS_Iterator.hxx>
#include <TopExp_Explorer.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <QApplication>
#include <QThread>
#include <QUuid>
#include <fastsignals/signal.h>
#include <algorithm>
#include <set>
#include <map>
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
bool samePoints(const Points::PointKernel& actual,const Points::PointKernel& expected){
    if(actual.getBasicPoints()!=expected.getBasicPoints())return false;
    const auto a=actual.getTransform(),b=expected.getTransform();
    for(unsigned r=0;r<4;++r)for(unsigned c=0;c<4;++c)if(a[r][c]!=b[r][c])return false;
    return true;
}
App::PropertyString* ownedString(App::DocumentObject& object,const char* name,const char* documentation="",short flags=0){
    // App::Link forwards unknown getPropertyByName() lookups to its definition.
    // Gesture provenance and copy identity belong to this occurrence itself.
    auto* property=dynamic_cast<App::PropertyString*>(object.getDynamicPropertyByName(name));
    if(!property)property=dynamic_cast<App::PropertyString*>(object.addDynamicProperty("App::PropertyString",name,"Gumball",documentation,flags));
    if(!property||property->getContainer()!=&object)throw std::runtime_error("Cannot store occurrence-owned gumball metadata");
    return property;
}
Om9GumballSource source(const ObjectSnapshot& s){Om9GumballSource v{};v.document=s.stamp.document;v.object=s.stamp.object;v.generation=s.stamp.generation;v.revision=s.stamp.revision;v.epoch=s.stamp.epoch;std::copy(s.world.begin(),s.world.end(),v.world);return v;}
struct Session {std::uint64_t id=om9_gumball_create();Session(){if(!id)throw std::runtime_error("Gumball session budget exhausted");Om9GumballEvent e{};e.pressed=1;if(om9_gumball_event(id,&e))throw std::runtime_error("Cannot enable gumball session");}~Session(){om9_gumball_destroy(id);}};
// Observe synchronous native callbacks while publishing. Only the exact native
// property setter owned by this adapter may write a captured source at a time.
class GumballPublicationGuard {
    App::Document& doc;const GumballInputs& inputs;std::string documentName,bad,field;
    std::set<std::uint64_t> watched,written;std::uint64_t writer=0;bool recomputing=false;
    std::map<std::uint64_t,TopoDS_Shape> expectedShapes;
    std::map<std::uint64_t,QJsonObject> expectedDiscrete;
    fastsignals::scoped_connection changed,removed;
    void remember(const App::DocumentObject& object){
        const auto id=std::uint64_t(object.getID());
        if(const auto* part=dynamic_cast<const Part::Feature*>(&object))expectedShapes[id]=part->Shape.getValue();
        else if(dynamic_cast<const Mesh::Feature*>(&object)||dynamic_cast<const Points::Feature*>(&object))expectedDiscrete[id]=discreteSnapshot(const_cast<App::DocumentObject&>(object),dynamic_cast<const Mesh::Feature*>(&object)!=nullptr);
    }
    bool unchangedGeometry(const App::DocumentObject& object,const std::string& key)const{
        const auto id=std::uint64_t(object.getID());
        if(key=="Shape"){
            auto found=expectedShapes.find(id);const auto* part=dynamic_cast<const Part::Feature*>(&object);
            if(found==expectedShapes.end()||!part)return false;
            const auto& expected=found->second;const auto& actual=part->Shape.getValue();
            if(!expected.IsPartner(actual)||expected.Orientation()!=actual.Orientation())return false;
            // Recompute builds a new TopLoc datum for the same Placement.
            // TopLoc identity is therefore stricter than unchanged geometry.
            const auto a=expected.Location().Transformation(),b=actual.Location().Transformation();
            for(int r=1;r<=3;++r)for(int c=1;c<=4;++c)if(a.Value(r,c)!=b.Value(r,c))return false;
            return true;
        }
        if(key=="Mesh"||key=="Points"){auto found=expectedDiscrete.find(id);return found!=expectedDiscrete.end()&&found->second==discreteSnapshot(const_cast<App::DocumentObject&>(object),key=="Mesh");}
        return false;
    }
public:
    GumballPublicationGuard(App::Document& d,const GumballInputs& values):doc(d),inputs(values),documentName(d.getName()){
        for(const auto& w:inputs.witnesses){watched.insert(w.stamp.object);for(const auto& a:w.ancestors)watched.insert(a.object);}
        for(const auto& d:inputs.definitions){watched.insert(d.snapshot.stamp.object);for(const auto& a:d.snapshot.ancestors)watched.insert(a.object);}
        for(auto* object:doc.getObjects())if(watched.contains(std::uint64_t(object->getID())))remember(*object);
        changed=App::GetApplication().signalChangedObject.connect([this](const App::DocumentObject& o,const App::Property& p){
            if(o.getDocument()!=&doc)return;
            const auto id=std::uint64_t(o.getID());if(!watched.contains(id))return;
            const char* raw=o.getPropertyName(&p);const std::string key=raw?raw:"";
            if(key=="_GroupTouched"||key=="_LinkTouched")return;
            const bool geometryField=field=="Placement"||field=="LinkPlacement"||field=="Shape"||field=="Mesh"||field=="Points";
            const bool geometryKey=key=="Placement"||key=="LinkPlacement"||key=="Shape"||key=="Mesh"||key=="Points";
            if(id==writer&&(key==field||(geometryField&&geometryKey))){
                // Placement may synchronize the native geometry location, but
                // cannot legitimately replace its captured topology/vertices.
                if((field=="Placement"||field=="LinkPlacement")&&key=="Shape"){
                    const auto* part=dynamic_cast<const Part::Feature*>(&o);auto old=expectedShapes.find(id);
                    if(part&&old!=expectedShapes.end()&&!old->second.IsPartner(part->Shape.getValue())){bad=std::string(o.getNameInDocument())+"."+key;return;}
                }
                if((field=="Placement"||field=="LinkPlacement")&&(key=="Mesh"||key=="Points")){
                    auto old=expectedDiscrete.find(id);
                    if(old!=expectedDiscrete.end()&&old->second["geometry_data"]!=discreteSnapshot(const_cast<App::DocumentObject&>(o),key=="Mesh")["geometry_data"]){bad=std::string(o.getNameInDocument())+"."+key;return;}
                }
                written.insert(id);return;
            }
            try{if(unchangedGeometry(o,key))return;}catch(...){}
            // Native parametric execute() owns regeneration. Its other input
            // properties stay guarded while its Placement is the only edit.
            if(recomputing&&written.contains(id)&&key=="Shape"&&o.isRecomputing()&&std::string(o.getTypeId().getName())!="Part::Feature")return;
            bad=std::string(o.getNameInDocument())+"."+key;
        });
        removed=App::GetApplication().signalDeletedObject.connect([this](const App::DocumentObject& o){if(o.getDocument()==&doc&&watched.contains(std::uint64_t(o.getID())))bad=o.getNameInDocument();});
    }
    bool live()const{return App::GetApplication().getDocument(documentName.c_str())==&doc;}
    void check(){if(!live()||App::GetApplication().getActiveDocument()!=&doc)throw std::runtime_error("Gumball document changed during publication");if(!bad.empty())throw std::runtime_error("Gumball source changed during publication: "+bad);verifyCoordinateContext(doc,inputs.coordinates);for(const auto& w:inputs.witnesses){auto now=captureObject(doc,w.objectName);if(now.stamp.document!=w.stamp.document||now.stamp.object!=w.stamp.object||now.stamp.generation!=w.stamp.generation||now.stamp.epoch!=w.stamp.epoch||!CoreLayers::objectEditable(doc.getObject(w.objectName.c_str())))throw std::runtime_error("Gumball source identity or permission changed during publication");}for(const auto& d:inputs.definitions){verifyObject(doc,d.snapshot);if(sourceAttributes(*doc.getObject(d.snapshot.objectName.c_str()),d.snapshot)!=d.attributes)throw std::runtime_error("Gumball Link definition attributes changed during publication");}}
    void start(App::DocumentObject* object,const char* key){check();writer=std::uint64_t(object->getID());field=key;}
    void finish(){if(!live())check();for(auto* object:doc.getObjects())if(std::uint64_t(object->getID())==writer){remember(*object);break;}writer=0;field.clear();check();}
    void recompute(bool enabled){recomputing=enabled;}
};
std::vector<std::uint64_t> selectedRoots(const std::vector<Om9GumballSource>& list,const CoordinateContext& cp){
    Session session;Om9GumballBegin b{};b.document=cp.documentOwner;b.view_generation=cp.frameRevision?cp.frameRevision:1;b.unit_generation=1;b.mm_per_unit=1;std::copy(std::begin(identityFrame),std::end(identityFrame),b.frame);std::copy(cp.frame.begin(),cp.frame.end(),b.cplane);b.ray_origin[2]=10;b.ray_direction[2]=-1;
    if(om9_gumball_begin(session.id,&b,list.data(),list.size()))throw std::runtime_error("Invalid gumball selection ownership");
    std::size_t count=0;if(om9_gumball_sources(session.id,nullptr,0,&count))throw std::runtime_error("Cannot normalize gumball selection");std::vector<Om9GumballSource> roots(count);
    if(om9_gumball_sources(session.id,roots.data(),roots.size(),&count))throw std::runtime_error("Cannot capture gumball selection roots");std::vector<std::uint64_t> result;for(const auto& root:roots)result.push_back(root.object);return result;
}
void captureDefinition(App::Document& doc,App::DocumentObject& object,GumballInputs& out,std::set<std::uint64_t>& visited,unsigned depth){
    if(object.getDocument()!=&doc)throw std::runtime_error("UnsupportedOwner: external-document Link definitions require a multi-document publication adapter");
    if(depth>64||out.definitions.size()>=16384)throw std::runtime_error("Gumball Link definition exceeds dependency budget");
    if(!visited.insert(std::uint64_t(object.getID())).second)return;
    if(std::any_of(out.definitions.begin(),out.definitions.end(),[&](const auto& d){return d.snapshot.stamp.object==std::uint64_t(object.getID());}))return;
    auto snapshot=captureObject(doc,object.getNameInDocument());out.definitions.push_back({snapshot,sourceAttributes(object,snapshot)});
    if(auto* group=object.getExtension<App::GroupExtension>())for(auto* child:group->Group.getValues())if(child)captureDefinition(doc,*child,out,visited,depth+1);
    auto* linked=object.getLinkedObject(false);if(linked&&linked!=&object)captureDefinition(doc,*linked,out,visited,depth+1);
}
void collect(App::Document& doc,App::DocumentObject& object,std::uint64_t root,GumballInputs& out,GumballNativeData& data,std::set<std::string>& visited,unsigned depth){
    if(depth>64||visited.size()>=16384||!visited.insert(object.getNameInDocument()).second)throw std::runtime_error("Gumball ownership tree is cyclic or exceeds input budget");
    if(!CoreLayers::objectEditable(&object))throw std::runtime_error("Gumball source is hidden, locked or not selectable");
    auto guard=captureObject(doc,object.getNameInDocument());out.witnesses.push_back(guard);
    auto* linked=object.getLinkedObject(false);if(linked&&linked!=&object){std::set<std::uint64_t> dependencies;captureDefinition(doc,*linked,out,dependencies,0);}
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
    std::erase_if(inputs.definitions,[&](const auto& d){return std::any_of(inputs.witnesses.begin(),inputs.witnesses.end(),[&](const auto& w){return w.stamp.object==d.snapshot.stamp.object;});});
    if(data->items.empty())throw std::runtime_error("Gumball selection has no native geometry");inputs.native=std::move(data);verifyGumballInputs(doc,inputs);return inputs;
}
void verifyGumballInputs(App::Document& doc,const GumballInputs& inputs){guiThread();verifyCoordinateContext(doc,inputs.coordinates);for(const auto& root:inputs.roots)verifyObject(doc,root);for(const auto& witness:inputs.witnesses){verifyObject(doc,witness);if(!CoreLayers::objectEditable(doc.getObject(witness.objectName.c_str())))throw std::runtime_error("Gumball source became locked, hidden or unselectable");}for(const auto& d:inputs.definitions){verifyObject(doc,d.snapshot);if(sourceAttributes(*doc.getObject(d.snapshot.objectName.c_str()),d.snapshot)!=d.attributes)throw std::runtime_error("Gumball Link definition attributes changed");}if(inputs.native)for(const auto& item:inputs.native->items){auto* object=doc.getObject(item.snapshot.objectName.c_str());if(sourceAttributes(*object,item.snapshot)!=item.attributes)throw std::runtime_error("Gumball source appearance or attributes changed");}}
GumballStagedBatch stageGumballBatch(const GumballInputs& inputs,const Om9GumballSnapshot& plan){
    guiThread();if(!inputs.native||plan.phase<2||plan.phase>5)throw std::runtime_error("No active gumball geometry plan");auto data=std::make_shared<GumballNativeData>(*inputs.native);const auto affine=matrix(plan.matrix);QJsonArray rows;
    for(auto& item:data->items){
        if(auto* shape=std::get_if<Part::TopoShape>(&item.geometry)){
            if(plan.operation==3){
                // Boolean planar faces may be wrapped in an exact single-child
                // compound. Unwrap that native wrapper, keeping the source.
                for(unsigned depth=0;(shape->getShape().ShapeType()==TopAbs_COMPOUND||shape->getShape().ShapeType()==TopAbs_SHELL)&&depth<64;++depth){
                    TopoDS_Iterator children(shape->getShape());if(!children.More())break;
                    auto child=children.Value();children.Next();if(children.More())break;
                    *shape=Part::TopoShape(child);
                }
            }
            if(plan.operation==3){if(shape->getShape().ShapeType()!=TopAbs_EDGE&&shape->getShape().ShapeType()!=TopAbs_WIRE&&shape->getShape().ShapeType()!=TopAbs_FACE)throw std::runtime_error("Gumball extrusion requires a planar curve or face");if(!shape->isPlanar())throw std::runtime_error("Gumball extrusion requires planar input");const gp_Vec displacement(plan.matrix[3],plan.matrix[7],plan.matrix[11]);if(displacement.SquareMagnitude()==0)throw std::runtime_error("Gumball extrusion distance is zero");*shape=Part::TopoShape(BRepPrimAPI_MakePrism(shape->getShape(),displacement,true,true).Shape());}
            else if(plan.operation==2)*shape=Part::TopoShape(shape->transformGShape(affine,true));else shape->transformShape(affine,true);
            ThreeDm::InputTree::validate(shape->getShape());if(!BRepCheck_Analyzer(shape->getShape()).IsValid())throw std::runtime_error("Gumball produced invalid native BRep");
        }else {if(plan.operation==3)throw std::runtime_error("Gumball extrusion requires native planar BRep geometry");std::visit([&](auto& geometry){geometry.transformGeometry(affine);},item.geometry);}
        rows.append(geometryFacts(item));
    }
    QJsonArray roots;for(const auto& root:inputs.roots)roots.append(QString::fromStdString(root.objectName));return {data,{{"selected_roots",roots},{"staged",rows}}};
}
QJsonObject gumballInputFacts(const GumballInputs& inputs){QJsonArray rows;for(const auto& item:inputs.native->items)rows.append(geometryFacts(item));return {{"items",rows}};}
std::vector<std::vector<Base::Vector3d>> gumballPreviewLines(const GumballStagedBatch& batch){
    std::vector<std::vector<Base::Vector3d>> lines;std::size_t count=0;
    for(const auto& item:batch.native->items){
        if(const auto* shape=std::get_if<Part::TopoShape>(&item.geometry)){
            for(TopExp_Explorer edges(shape->getShape(),TopAbs_EDGE);edges.More();edges.Next()){
                if((count+=33)>100000)throw std::runtime_error("Gumball native preview exceeds vertex budget");
                BRepAdaptor_Curve curve(TopoDS::Edge(edges.Current()));auto first=curve.FirstParameter(),last=curve.LastParameter();std::vector<Base::Vector3d> points;
                for(unsigned i=0;i<33;++i){auto p=curve.Value(first+(last-first)*i/32.);points.emplace_back(p.X(),p.Y(),p.Z());}lines.push_back(std::move(points));
            }
        }else if(const auto* mesh=std::get_if<Mesh::MeshObject>(&item.geometry)){
            if((count+=mesh->countPoints())>100000)throw std::runtime_error("Gumball mesh preview exceeds vertex budget");
            for(unsigned i=0;i<mesh->countPoints();++i){auto p=mesh->getPoint(i);lines.push_back({p});}
        }else{const auto& points=std::get<Points::PointKernel>(item.geometry);if((count+=points.size())>100000)throw std::runtime_error("Gumball point preview exceeds vertex budget");for(unsigned i=0;i<points.size();++i){auto p=points.getPoint(i);lines.push_back({p});}}
    }return lines;
}
void publishGumballBatch(App::Document& doc,const GumballInputs& inputs,const Om9GumballSnapshot& plan,GumballStagedBatch&& batch){
    guiThread();verifyGumballInputs(doc,inputs);if(plan.phase!=5||!batch.native||batch.report.empty()||!Gui::Control().isAllowedAlterDocument(&doc))throw std::runtime_error("Gumball publication is not allowed");
    // Shape assignment is durable only for plain native geometry owners. Rigid
    // Placement edits preserve parametric owners and individual Link occurrences.
    if(plan.operation>=2)for(const auto& item:batch.native->items){auto* object=doc.getObject(item.snapshot.objectName.c_str());const auto type=std::string(object->getTypeId().getName());if(plan.operation==2&&type!="Part::Feature"&&type!="Mesh::Feature"&&type!="Points::Feature")throw std::runtime_error("UnsupportedOwner: affine scale requires a plain BRep, Mesh or Points owner; parametric/Link geometry cannot be overwritten");}
    for(const auto& root:inputs.roots){auto* object=doc.getObject(root.objectName.c_str());if(plan.operation<2&&!object->getPropertyByName<App::PropertyPlacement>("Placement"))throw std::runtime_error("UnsupportedOwner: source has no editable native Placement");}
    if(plan.operation==3)for(const auto& root:inputs.roots)if(doc.getObject(root.objectName.c_str())->getExtension<App::GroupExtension>())throw std::runtime_error("UnsupportedOwner: select planar curves or faces directly for extrusion");
    const auto affine=matrix(plan.matrix);std::map<std::string,App::DocumentObject*> outputs;GumballPublicationGuard guard(doc,inputs);
    doc.openTransaction(plan.operation==3?"Gumball Extrude":plan.copy?"Gumball Copy":"Gumball Transform");
    try{
        if(plan.copy&&plan.operation!=3){
            std::vector<App::DocumentObject*> originals;for(const auto& witness:inputs.witnesses)originals.push_back(doc.getObject(witness.objectName.c_str()));
            const auto copied=doc.copyObject(originals,false,false);guard.check();if(copied.size()!=originals.size())throw std::runtime_error("Gumball copy ownership mismatch");
            for(unsigned i=0;i<copied.size();++i){outputs[originals[i]->getNameInDocument()]=copied[i];auto* id=ownedString(*copied[i],"OM9GumballIdentity");id->setValue(QUuid::createUuid().toString(QUuid::WithoutBraces).toUtf8().constData());}
            for(unsigned i=0;i<copied.size();++i){if(auto* group=copied[i]->getExtension<App::GroupExtension>()){std::vector<App::DocumentObject*> members;for(auto* old:originals[i]->getExtension<App::GroupExtension>()->Group.getValues()){auto found=outputs.find(old->getNameInDocument());if(found==outputs.end())throw std::runtime_error("Gumball copied group has an uncaptured member");members.push_back(found->second);}group->Group.setValues(members);}}
            // A copied child stays in its original containing group when that
            // group was not selected. Copied groups receive their private members.
            for(const auto& root:inputs.roots){auto* original=doc.getObject(root.objectName.c_str());auto* output=outputs.at(root.objectName);for(auto* candidate:doc.getObjects()){if(outputs.contains(candidate->getNameInDocument()))continue;if(auto* group=candidate->getExtension<App::GroupExtension>()){auto members=group->Group.getValues();if(std::find(members.begin(),members.end(),original)!=members.end()){members.push_back(output);guard.start(candidate,"Group");group->Group.setValues(members);guard.finish();}}}}
            for(unsigned i=0;i<copied.size();++i)CoreLayers::inheritLayer(*copied[i],*originals[i]);
        }
        if(plan.operation<2){
            for(const auto& root:inputs.roots){guard.check();auto* original=doc.getObject(root.objectName.c_str());auto* output=plan.copy?outputs.at(root.objectName):original;auto* placement=output->getPropertyByName<App::PropertyPlacement>("Placement");const auto global=matrix(root.world.data());auto parent=global*original->getPropertyByName<App::PropertyPlacement>("Placement")->getValue().inverse().toMatrix();parent.inverse();Base::Placement local;local.fromMatrix(parent*affine*global);guard.start(output,"Placement");placement->setValue(local);guard.finish();if(placement->getValue()!=local)throw std::runtime_error("Gumball output Placement changed during publication");}
        }else for(auto& item:batch.native->items){
            guard.check();auto* original=doc.getObject(item.snapshot.objectName.c_str());App::DocumentObject* output=plan.copy&&plan.operation!=3?outputs.at(item.snapshot.objectName):original;
            auto global=App::GeoFeature::getGlobalPlacement(original);auto* placement=original->getPropertyByName<App::PropertyPlacement>("Placement");auto parent=placement?global.toMatrix()*placement->getValue().inverse().toMatrix():Base::Matrix4D();parent.inverse();
            if(plan.operation==3){
                output=doc.addObject("Part::Feature","GumballExtrusion");outputs[item.snapshot.objectName]=output;output->Label.setValue((std::string(original->Label.getValue())+" extrusion").c_str());CoreLayers::inheritLayer(*output,*original);
                auto* from=dynamic_cast<Gui::ViewProviderGeometryObject*>(Gui::Application::Instance->getViewProvider(original));auto* to=dynamic_cast<Gui::ViewProviderGeometryObject*>(Gui::Application::Instance->getViewProvider(output));if(from&&to)to->ShapeAppearance.setValues(from->ShapeAppearance.getValues());
                for(auto* candidate:doc.getObjects())if(auto* group=candidate->getExtension<App::GroupExtension>()){auto members=group->Group.getValues();if(std::find(members.begin(),members.end(),original)!=members.end()){members.push_back(output);guard.start(candidate,"Group");group->Group.setValues(members);guard.finish();}}
                CoreLayers::inheritLayer(*output,*original);
            }
            if(auto* shape=std::get_if<Part::TopoShape>(&item.geometry)){shape->transformShape(parent,true);guard.start(output,"Shape");dynamic_cast<Part::Feature*>(output)->Shape.setValue(*shape);guard.finish();if(!dynamic_cast<Part::Feature*>(output)->Shape.getValue().IsEqual(shape->getShape()))throw std::runtime_error("Gumball BRep changed during publication");}
            else if(auto* mesh=std::get_if<Mesh::MeshObject>(&item.geometry)){mesh->transformGeometry(parent);const auto expected=meshSnapshot(*mesh);guard.start(output,"Mesh");dynamic_cast<Mesh::Feature*>(output)->Mesh.setValue(*mesh);if(discreteSnapshot(*output,true)!=expected)throw std::runtime_error("Gumball Mesh changed during publication");guard.finish();}
            else{auto& points=std::get<Points::PointKernel>(item.geometry);points.transformGeometry(parent);guard.start(output,"Points");auto* native=dynamic_cast<Points::Feature*>(output);native->Points.setValue(points);if(!samePoints(native->Points.getValue(),points))throw std::runtime_error("Gumball Points changed during publication");guard.finish();}
        }
        guard.recompute(true);doc.recompute();guard.recompute(false);guard.check();for(const auto& root:inputs.roots){auto* object=plan.copy&&plan.operation!=3?outputs.at(root.objectName):doc.getObject(root.objectName.c_str());if(object&&object->isError())throw std::runtime_error("Gumball native owner recompute failed");}
        for(const auto& root:inputs.roots){auto* output=plan.copy||plan.operation==3?outputs.at(root.objectName):doc.getObject(root.objectName.c_str());guard.start(output,"OM9GumballReport");auto* report=ownedString(*output,"OM9GumballReport","Owned Rust transform/input provenance",App::Prop_ReadOnly);report->setValue(batch.report.c_str());guard.finish();guard.start(output,"OM9UnitContext");if(!inputs.coordinates.unitContext.empty())ownedString(*output,"OM9UnitContext","Input units/tolerances snapshot",App::Prop_ReadOnly);addUnitContextProvenance(*output,inputs.coordinates.unitContext);guard.finish();}
        for(const auto& item:inputs.native->items)if(sourceAttributes(*doc.getObject(item.snapshot.objectName.c_str()),item.snapshot)!=item.attributes)throw std::runtime_error("Gumball source attributes changed during publication");guard.check();
        if(batch.beforeCommit)batch.beforeCommit();
        doc.commitTransaction();
    }catch(...){if(guard.live())doc.abortTransaction();throw;}
}
QJsonObject gumballObjectFrameFacts(const GumballInputs& inputs){
    if(inputs.roots.size()!=1)throw std::runtime_error("Object alignment requires one unambiguous object frame");
    const auto& root=inputs.roots.front();Base::Vector3d x(root.world[0],root.world[4],root.world[8]),normal(root.world[2],root.world[6],root.world[10]);
    if(!inputs.native->items.empty())if(const auto* shape=std::get_if<Part::TopoShape>(&inputs.native->items.front().geometry)){
        auto sub=root.subelement.empty()?shape->getShape():shape->getSubShape(root.subelement.c_str());
        // A wire/shell with one native child has the same unambiguous tangent
        // or surface frame as that child; do not substitute the Placement axes.
        for(unsigned depth=0;depth<64&&(sub.ShapeType()==TopAbs_COMPOUND||sub.ShapeType()==TopAbs_WIRE||sub.ShapeType()==TopAbs_SHELL);++depth){TopoDS_Iterator children(sub);if(!children.More())break;const auto child=children.Value();children.Next();if(children.More())break;sub=child;}
        if(sub.ShapeType()==TopAbs_FACE){BRepAdaptor_Surface surface(TopoDS::Face(sub));double u=(surface.FirstUParameter()+surface.LastUParameter())/2,v=(surface.FirstVParameter()+surface.LastVParameter())/2;gp_Pnt p;gp_Vec du,dv;surface.D1(u,v,p,du,dv);x=Base::Vector3d(du.X(),du.Y(),du.Z());auto n=du.Crossed(dv);normal=Base::Vector3d(n.X(),n.Y(),n.Z());}
        else if(sub.ShapeType()==TopAbs_EDGE){BRepAdaptor_Curve curve(TopoDS::Edge(sub));gp_Pnt p;gp_Vec tangent,second;curve.D2((curve.FirstParameter()+curve.LastParameter())/2,p,tangent,second);x=Base::Vector3d(tangent.X(),tangent.Y(),tangent.Z());auto n=tangent.Crossed(second);if(n.SquareMagnitude()>0)normal=Base::Vector3d(n.X(),n.Y(),n.Z());}
        else if(!root.subelement.empty())throw std::runtime_error("UnsupportedFrame: selected BRep component has no native tangent or surface frame; use whole-object, CPlane or World alignment");
    }else if(!root.subelement.empty())throw std::runtime_error("UnsupportedFrame: native Mesh/Points component tangent frame is unavailable; use whole-object, CPlane or World alignment");
    return {{"x",QJsonArray{x.x,x.y,x.z}},{"normal",QJsonArray{normal.x,normal.y,normal.z}}};
}
void addGumballGeometryMethods(PyObject* module){static PyMethodDef methods[]={{"gumballGeometryFacts",factsAPI,METH_VARARGS,"Read private native staging facts; no document mutation or UI proof."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
}
