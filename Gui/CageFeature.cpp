// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CageFeature.h"
#include "CageRust.h"
#include "CageGeometry.h"
#include "HistoryFeature.h"
#include "RetainedArchive.h"
#include "ThreeDmCage.h"
#include "CurveGeometry.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObjectPy.h>
#include <App/GeoFeature.h>
#include <App/GeoFeatureGroupExtension.h>
#include <Base/Exception.h>
#include <Base/Interpreter.h>
#include <Gui/Application.h>
#include <Gui/Control.h>
#include <Gui/Document.h>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_GTrsf.hxx>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <algorithm>
#include <cmath>
#include <memory>
#include <set>
#include <stdexcept>
extern "C" bool om9_retained_uuid_normalize(const char*,char*,std::size_t);
namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::CageControl,Part::Feature)
PROPERTY_SOURCE(OpenMatrix9Gui::CageBinding,App::DocumentObject)
namespace {
struct Writing {bool& flag;bool old;explicit Writing(bool& f):flag(f),old(f){flag=true;}~Writing(){flag=old;}};
std::vector<App::DocumentObject*> placementAncestors(App::DocumentObject* source){std::vector<App::DocumentObject*> out;std::set<App::DocumentObject*> seen;for(auto* p=source?App::GeoFeatureGroupExtension::getGroupOfObject(source):nullptr;p;p=App::GeoFeatureGroupExtension::getGroupOfObject(p)){if(!seen.insert(p).second||out.size()>=64)throw std::runtime_error("Invalid cage placement hierarchy");out.push_back(p);}return out;}
bool includes(const std::vector<App::DocumentObject*>& objects,const App::DocumentObject& object){return std::find(objects.begin(),objects.end(),&object)!=objects.end();}
using RustCage=std::unique_ptr<Om9CageHandle,decltype(&om9_cage_destroy)>;
App::DocumentObject* native(PyObject* p){
    if(!PyObject_TypeCheck(p,&App::DocumentObjectPy::Type))throw std::runtime_error("Expected native document object");
    auto* wrapper=static_cast<App::DocumentObjectPy*>(p);if(!wrapper->Base::PyObjectBase::isValid())throw std::runtime_error("Document object no longer exists");auto* o=wrapper->getDocumentObjectPtr();
    if(!o||o->isRemoving()||!o->getDocument()||!o->getDocument()->containsObject(o))throw std::runtime_error("Document object no longer exists");return o;
}
void editable(App::Document& d){
    auto* g=Gui::Application::Instance->activeDocument();
    if(App::GetApplication().getActiveDocument()!=&d||!g||g->getDocument()!=&d||g->getInEdit()||g->isAboutToClose()||!Gui::Control().isAllowedAlterDocument(&d))
        throw std::runtime_error("The active document is not editable");
}
Base::Matrix4D frame(App::DocumentObject* o){return App::GeoFeature::getGlobalPlacement(o).toMatrix();}
std::array<double,16> array(const Base::Matrix4D& m){std::array<double,16> out{};for(int i=0;i<4;++i)for(int j=0;j<4;++j)out[i*4+j]=m[i][j];return out;}
std::vector<double> flat(const std::vector<Base::Vector3d>& v){std::vector<double> out;out.reserve(v.size()*3);for(auto p:v){out.push_back(p.x);out.push_back(p.y);out.push_back(p.z);}return out;}
std::vector<Base::Vector3d> vectors(const std::vector<double>& v){std::vector<Base::Vector3d> out;for(std::size_t i=0;i<v.size();i+=3)out.emplace_back(v[i],v[i+1],v[i+2]);return out;}
RustCage cageHandle(CageControl& c,const CageBinding* binding=nullptr){
    if(binding&&std::string(binding->Region.getValue())!="Global"&&std::string(binding->Region.getValue())!="Local")throw std::runtime_error("Unsupported cage region");
    Om9CageDescriptor desc{};auto count=c.Counts.getValues(),degree=c.Degrees.getValues();
    if(count.size()!=3||degree.size()!=3)throw std::runtime_error("Cage counts/degrees require three entries");
    const std::vector<double>* knots[]={&c.UKnots.getValues(),&c.VKnots.getValues(),&c.WKnots.getValues()};
    for(int a=0;a<3;++a){if(count[a]<2||degree[a]<1)throw std::runtime_error("Invalid cage count/degree");desc.counts[a]=std::size_t(count[a]);desc.degrees[a]=std::size_t(degree[a]);desc.knots[a]=knots[a]->data();desc.knot_lengths[a]=knots[a]->size();}
    const auto& points=c.ControlPoints.getValues();const auto pose=array(frame(&c));
    auto coordinates=flat(points);desc.points=coordinates.data();desc.point_count=points.size();desc.weights=c.Weights.getValues().data();desc.weight_count=c.Weights.getValues().size();
    auto reference=array(c.ReferenceFrame.getValue());std::copy(reference.begin(),reference.end(),desc.world_to_parameter);
    if(binding&&binding->Region.getValue()==std::string("Local")){
        desc.region=1;auto lo=binding->LocalMin.getValue(),hi=binding->LocalMax.getValue();for(int i=0;i<3;++i){desc.local_min[i]=lo[i];desc.local_max[i]=hi[i];}desc.falloff=binding->Falloff.getValue();
    }
    Om9CageHandle* raw=nullptr;if(om9_cage_create_placed(&desc,pose.data(),&raw)!=0)throw std::runtime_error("Invalid rational NURBS cage, placement or region");return RustCage(raw,om9_cage_destroy);
}
std::vector<Base::Vector3d> batch(const RustCage& handle,const std::vector<Base::Vector3d>& points,bool inverse){
    auto input=flat(points);std::vector<double> out(input.size());auto status=inverse?om9_cage_inverse(handle.get(),input.data(),points.size(),out.data(),points.size()):om9_cage_evaluate(handle.get(),input.data(),points.size(),out.data(),points.size());
    if(status!=0)throw std::runtime_error(inverse?"Cage capture inverse failed, is ambiguous or outside supported bind domain":"Cage evaluation failed");return vectors(out);
}
std::optional<std::array<double,16>> affineForBinding(const RustCage& current,CageControl& control){
    std::array<double,16> m{};if(om9_cage_affine(current.get(),m.data())!=0)return {};
    // Rust's transform maps reference world coordinates to edited world. Native
    // captures use that reference. Archive rebinds need a different bind inverse;
    // the geometry path below proves it from original and mapped sample poles.
    return m;
}
TopoDS_Shape worldShape(Part::Feature& source){
    if(source.Shape.getShape().isNull())throw std::runtime_error("Empty captive shape");
    auto parent=frame(&source)*source.Placement.getValue().inverse().toMatrix();
    auto local=source.Shape.getShape();auto transform=parent*local.getTransform();local.setTransform(Base::Matrix4D());
    Part::TopoShape shape;shape.makeGTransform(local,transform,nullptr,true);return shape.getShape();
}
std::vector<App::DocumentObject*> objects(PyObject* list){
    CurvePyRef seq(PySequence_Fast(list,"Expected a sequence of objects"));if(!seq.value)throw std::runtime_error("Expected object sequence");
    std::vector<App::DocumentObject*> result;std::set<App::DocumentObject*> seen;
    for(Py_ssize_t i=0;i<PySequence_Fast_GET_SIZE(seq.value);++i){auto* o=native(PySequence_Fast_GET_ITEM(seq.value,i));if(!seen.insert(o).second)throw std::runtime_error("Duplicate capture input");result.push_back(o);}return result;
}
void setMesh(App::DocumentObject& object,const std::vector<Base::Vector3d>& vertices,const std::vector<long>& triangles){
    if(triangles.size()%3!=0)throw std::runtime_error("Invalid saved mesh facet cardinality");
    CurvePyRef module(PyImport_ImportModule("Mesh")),faces(PyList_New(triangles.size()/3));
    for(std::size_t i=0;i<triangles.size();i+=3){CurvePyRef face(PyList_New(3));for(int j=0;j<3;++j){auto index=triangles[i+j];if(index<0||std::size_t(index)>=vertices.size())throw std::runtime_error("Invalid saved mesh topology");auto p=vertices[index];PyList_SET_ITEM(face.value,j,Py_BuildValue("(ddd)",p.x,p.y,p.z));}PyList_SET_ITEM(faces.value,i/3,face.value);face.value=nullptr;}
    CurvePyRef mesh(module.value?PyObject_CallMethod(module.value,"Mesh","O",faces.value):nullptr),target(object.getPyObject()),placement(PyObject_GetAttrString(target.value,"Placement"));
    if(!mesh.value||!placement.value||PyObject_SetAttrString(mesh.value,"Placement",placement.value)!=0||PyObject_SetAttrString(target.value,"Mesh",mesh.value)!=0)throw std::runtime_error("Cannot set native deformed mesh");
}
void applyShape(App::DocumentObject& object,const TopoDS_Shape& world){
    auto inverse=frame(&object);inverse.inverseGauss();Part::TopoShape original(world),value;auto transform=inverse*original.getTransform();original.setTransform(Base::Matrix4D());value.makeGTransform(original,transform,nullptr,true);auto* part=dynamic_cast<Part::Feature*>(&object);if(!part)throw std::runtime_error("Captive is no longer a native shape");
    // Part::Feature synchronizes Shape's location to Placement. Keep the object's
    // existing placement/group and let native Part apply it exactly once.
    // Strip the input Location and compose it into the transform before copying:
    // OCCT can preserve that Location even with copy=true.
    auto placement=part->Placement.getValue();value.setTransform(placement.toMatrix());part->Shape.setValue(value);
}
void applyMesh(App::DocumentObject& object,std::vector<Base::Vector3d> world,const std::vector<long>& triangles){auto inverse=frame(&object);inverse.inverseGauss();for(auto& p:world)inverse.multVec(p,p);setMesh(object,world,triangles);}
std::vector<CageBinding*> bindingsFor(App::DocumentObject* o){std::vector<CageBinding*> out;if(!o||!o->getDocument())return out;for(auto* candidate:o->getDocument()->getObjects())if(auto* b=dynamic_cast<CageBinding*>(candidate);b&&b->Active.getValue()&&(candidate==o||b->Captive.getValue()==o))out.push_back(b);return out;}
struct Prepared {App::DocumentObject* object;bool mesh=false;TopoDS_Shape shape;std::vector<Base::Vector3d> vertices,parameters;std::vector<long> faces;};
Prepared prepare(App::DocumentObject* o,CageControl& control,const RustCage& handle){
    if(o==&control||o->getDocument()!=control.getDocument()||!o->isValid()||!bindingsFor(o).empty())throw std::runtime_error("Invalid, foreign or already captured object");
    if(hasRecordedHistory(o))throw std::runtime_error("Clear existing object History before capturing it in a cage");
    Prepared p{o};if(auto* shape=dynamic_cast<Part::Feature*>(o)){p.shape=worldShape(*shape);if(p.shape.IsNull())throw std::runtime_error("Empty captive shape");p.vertices=cageShapeControlPoints(p.shape);
        if(p.vertices.empty()&&!affineForBinding(handle,control))throw std::runtime_error("This BRep topology supports affine cage deformation only");
    }else{
        CurvePyRef obj(o->getPyObject()),mesh(PyObject_GetAttrString(obj.value,"Mesh"));if(!mesh.value){PyErr_Clear();throw std::runtime_error("Cage capture requires a native Part or Mesh object");}
        CurvePyRef topology(PyObject_GetAttrString(mesh.value,"Topology"));if(!topology.value||PyTuple_Size(topology.value)!=2)throw std::runtime_error("Cannot read native mesh topology");p.mesh=true;
        auto* points=PyTuple_GetItem(topology.value,0);auto* faces=PyTuple_GetItem(topology.value,1);auto* own=o->getPropertyByName<App::PropertyPlacement>("Placement");if(!own)throw std::runtime_error("Native mesh has no placement");auto transform=frame(o)*own->getValue().inverse().toMatrix();
        for(Py_ssize_t i=0;i<PySequence_Size(points);++i){CurvePyRef item(PySequence_GetItem(points,i));Base::Vector3d v;for(int j=0;j<3;++j){CurvePyRef n(PySequence_GetItem(item.value,j));v[j]=PyFloat_AsDouble(n.value);}if(PyErr_Occurred())throw std::runtime_error("Invalid mesh vertex");transform.multVec(v,v);p.vertices.push_back(v);}
        for(Py_ssize_t i=0;i<PySequence_Size(faces);++i){CurvePyRef face(PySequence_GetItem(faces,i));if(PySequence_Size(face.value)!=3)throw std::runtime_error("Only native triangular mesh captives are supported");for(int j=0;j<3;++j){CurvePyRef index(PySequence_GetItem(face.value,j));auto k=PyLong_AsLong(index.value);if(k<0||std::size_t(k)>=p.vertices.size()||PyErr_Occurred())throw std::runtime_error("Invalid mesh facet");p.faces.push_back(k);}}
        if(p.vertices.empty()||p.faces.empty())throw std::runtime_error("Empty mesh captive");
    }
    if(!p.vertices.empty())p.parameters=batch(handle,p.vertices,true);return p;
}
PyObject* insertBindings(CageControl& c,const std::vector<Prepared>& prepared,const RustCage& handle,const char* region,double falloff){
    auto& d=*c.getDocument();CurvePyRef out(PyList_New(prepared.size()));
    Base::BoundBox3d bounds;auto global=frame(&c);for(auto point:c.ControlPoints.getValues()){global.multVec(point,point);bounds.Add(point);}
    auto bindAffine=affineForBinding(handle,c);Base::Matrix4D bindFrame;
    if(bindAffine){Base::Matrix4D map;for(int i=0;i<4;++i)for(int j=0;j<4;++j)map[i][j]=(*bindAffine)[i*4+j];map.inverseGauss();bindFrame=c.ReferenceFrame.getValue()*map;}
    for(std::size_t i=0;i<prepared.size();++i){const auto& p=prepared[i];auto* b=dynamic_cast<CageBinding*>(d.addObject("OpenMatrix9Gui::CageBinding","CageBinding"));b->Control.setValue(&c);b->Captive.setValue(p.object);b->MeshSource.setValue(p.mesh);b->SourceShape.setValue(Part::TopoShape(p.shape));b->SourceVertices.setValues(p.vertices);b->BoundParameters.setValues(p.parameters);b->SourceTriangles.setValues(p.faces);b->BoundFrame.setValue(bindFrame);b->AffineBind.setValue(bool(bindAffine));b->Region.setValue(region);b->Falloff.setValue(falloff);b->LocalMin.setValue(Base::Vector3d(bounds.MinX,bounds.MinY,bounds.MinZ));b->LocalMax.setValue(Base::Vector3d(bounds.MaxX,bounds.MaxY,bounds.MaxZ));b->Active.setValue(true);PyList_SET_ITEM(out.value,i,b->getPyObject());}
    auto* result=out.value;out.value=nullptr;return result;
}
PyObject* capture(PyObject*,PyObject* args){
    PyObject *controlObject,*items;const char* region="Global";double falloff=0;
    if(!PyArg_ParseTuple(args,"OO|sd",&controlObject,&items,&region,&falloff))return nullptr;
    try{
        auto* c=dynamic_cast<CageControl*>(native(controlObject));if(!c)throw std::runtime_error("Expected an OpenMatrix9 native cage");auto& d=*c->getDocument();editable(d);
        if(std::string(region)!="Global"&&std::string(region)!="Local")throw std::runtime_error("Supported cage regions: Global or Local box");if(!std::isfinite(falloff)||falloff<0)throw std::runtime_error("Falloff must be finite and nonnegative");
        auto handle=cageHandle(*c);auto selected=objects(items);if(selected.empty())throw std::runtime_error("Select captive objects");std::vector<Prepared> prepared;for(auto* o:selected){auto p=prepare(o,*c,handle);if(std::string(region)=="Local"&&!p.mesh&&p.vertices.empty())throw std::runtime_error("Local cage deformation requires a mesh, single edge or rectangular face");prepared.push_back(std::move(p));}
        // Fully decode and invert before any native transaction or source write.
        d.openTransaction("Cage Edit capture");try{CurvePyRef out(insertBindings(*c,prepared,handle,region,falloff));
            d.recompute();for(Py_ssize_t i=0;i<PyList_Size(out.value);++i){auto* b=native(PyList_GetItem(out.value,i));if(b->isError()){const char* error=d.getErrorDescription(b);throw std::runtime_error(std::string("Cage binding recompute failed: ")+(error?error:"unknown recompute error"));}}d.commitTransaction();auto* result=out.value;out.value=nullptr;return result;
        }catch(...){d.abortTransaction();throw;}
    }catch(const Base::Exception& e){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}catch(const std::exception& e){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}catch(...){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,"Native cage operation failed");return nullptr;}
}
PyObject* restoreArchive(PyObject*,PyObject* args){PyObject* source;if(!PyArg_ParseTuple(args,"O",&source))return nullptr;
    try{auto* original=native(source);auto& d=*original->getDocument();editable(d);const auto verified=verifiedRetainedArchive(source);auto decoded=ThreeDm::cageArchiveRecord(verified.archiveBytes,verified.sourceUuid,verified.scaleMm);
        if(std::abs(decoded.scaleMm-verified.scaleMm)>1e-10*std::max(1.,verified.scaleMm))throw std::runtime_error("Archive cage units mismatch");
        std::vector<App::DocumentObject*> captives;
        for(const auto& id:decoded.captiveSourceUuids){App::DocumentObject* match=nullptr;for(auto* o:d.getObjects()){
            auto* space=o->getPropertyByName<App::PropertyString>("OM9ImportNamespace");auto* uuid=o->getPropertyByName<App::PropertyString>("OM9SourceUUID");
            char canonical[37]{};
            if(space&&uuid&&space->getStrValue()==verified.importNamespace&&om9_retained_uuid_normalize(uuid->getValue(),canonical,sizeof canonical)&&std::string(canonical)==id){if(match)throw std::runtime_error("Ambiguous captive UUID in import namespace");match=o;}}
            if(!match||match==original)throw std::runtime_error("Cage captive is missing from the original import namespace");captives.push_back(match);
        }
        Base::Matrix4D parent=frame(original);std::vector<Base::Vector3d> points;std::vector<double> weights;const auto& data=decoded.currentCage;
        for(int w=0;w<data.counts[2];++w)for(int v=0;v<data.counts[1];++v)for(int u=0;u<data.counts[0];++u){auto i=std::size_t((u*data.counts[1]+v)*data.counts[2]+w);const auto p=data.points.at(i);points.emplace_back(p[0],p[1],p[2]);weights.push_back(data.weights.at(i));}
        Base::Matrix4D reference;
        if(decoded.originalReference){const auto& m=decoded.originalReference->worldMmToParameters;for(int i=0;i<4;++i)for(int j=0;j<4;++j)reference[i][j]=m[i*4+j];auto inverse=parent;inverse.inverseGauss();reference=reference*inverse;
            for(int axis=0;axis<3;++axis){auto start=data.fullKnots[axis][data.degrees[axis]],end=data.fullKnots[axis][data.counts[axis]];for(int j=0;j<4;++j)reference[axis][j]/=(end-start);reference[axis][3]-=start/(end-start);}}
        d.openTransaction("Restore Rhino cage relationships");try{
            auto* c=dynamic_cast<CageControl*>(d.addObject("OpenMatrix9Gui::CageControl","Cage"));c->Counts.setValues(std::vector<long>(data.counts.begin(),data.counts.end()));c->Degrees.setValues(std::vector<long>(data.degrees.begin(),data.degrees.end()));c->ControlPoints.setValues(points);c->Weights.setValues(weights);c->UKnots.setValues(data.fullKnots[0]);c->VKnots.setValues(data.fullKnots[1]);c->WKnots.setValues(data.fullKnots[2]);c->ReferenceFrame.setValue(reference);
            c->Placement.setValue(Base::Placement(parent));
            QJsonObject metadata{{"source_record_uuid",QString::fromStdString(verified.sourceUuid)},{"import_namespace",QString::fromStdString(verified.importNamespace)},{"source_class",QString::fromStdString(decoded.sourceClass)},{"bind_geometry","current_archive_output"},{"preserve_structure",decoded.preserveStructure},{"quick_preview",decoded.quickPreview},{"scale_mm",decoded.scaleMm}};
            c->SourceMetadata.setValue(QJsonDocument(metadata).toJson(QJsonDocument::Compact).constData());auto handle=cageHandle(*c);std::vector<Prepared> prepared;for(auto* captive:captives)prepared.push_back(prepare(captive,*c,handle));CurvePyRef created(insertBindings(*c,prepared,handle,"Global",0));
            d.recompute();if(c->isError())throw std::runtime_error("Archive cage reconstruction failed");for(auto* o:d.getObjects())if(auto* b=dynamic_cast<CageBinding*>(o);b&&b->Control.getValue()==c&&b->isError())throw std::runtime_error("Archive captive reconstruction failed");d.commitTransaction();return c->getPyObject();
        }catch(...){d.abortTransaction();throw;}
    }catch(const Base::Exception& e){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}catch(const std::exception& e){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}catch(...){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,"Native cage operation failed");return nullptr;}
}
PyObject* release(PyObject*,PyObject* args){PyObject* items;if(!PyArg_ParseTuple(args,"O",&items))return nullptr;try{auto selected=objects(items);if(selected.empty())throw std::runtime_error("Select captives to release");auto& d=*selected[0]->getDocument();editable(d);std::vector<CageBinding*> bindings;for(auto* o:selected){if(o->getDocument()!=&d)throw std::runtime_error("Foreign captive");for(auto* b:bindingsFor(o))if(std::find(bindings.begin(),bindings.end(),b)==bindings.end())bindings.push_back(b);}if(bindings.empty())throw std::runtime_error("Selected objects have no active cage binding");d.openTransaction("Release From Cage");try{for(auto* b:bindings)b->detach();d.commitTransaction();}catch(...){d.abortTransaction();throw;}Py_RETURN_NONE;}catch(const Base::Exception& e){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}catch(const std::exception& e){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}catch(...){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,"Native cage release failed");return nullptr;}}
std::array<double,3> xyz(PyObject* values){CurvePyRef list(PySequence_Fast(values,"Expected three coordinates"));if(!list.value||PySequence_Fast_GET_SIZE(list.value)!=3)throw std::runtime_error("Expected three coordinates");std::array<double,3> p{};for(int i=0;i<3;++i){p[i]=PyFloat_AsDouble(PySequence_Fast_GET_ITEM(list.value,i));if(!std::isfinite(p[i])||PyErr_Occurred())throw std::runtime_error("Coordinates must be finite");}return p;}
PyObject* create(PyObject*,PyObject* args){const char* document;PyObject *minimum,*maximum,*countObject=nullptr,*degreeObject=nullptr;
    if(!PyArg_ParseTuple(args,"sOO|OO",&document,&minimum,&maximum,&countObject,&degreeObject))return nullptr;
    try{auto* d=App::GetApplication().getDocument(document);if(!d)throw std::runtime_error("Missing active document");editable(*d);auto lo=xyz(minimum),hi=xyz(maximum);std::array<double,3> counts{2,2,2},degrees{1,1,1};if(countObject)counts=xyz(countObject);if(degreeObject)degrees=xyz(degreeObject);
        std::array<std::size_t,3> nativeCounts{},nativeDegrees{};
        if(om9_cage_box_parameters(counts.data(),degrees.data(),nativeCounts.data(),nativeDegrees.data())!=0)throw std::runtime_error("Invalid cage integer counts/degrees or point budget");
        std::vector<long> n(nativeCounts.begin(),nativeCounts.end()),p(nativeDegrees.begin(),nativeDegrees.end());std::array<std::vector<double>,3> k;
        const auto size=nativeCounts[0]*nativeCounts[1]*nativeCounts[2];std::vector<double> coordinates(size*3);std::array<double,16> matrix{};
        for(int i=0;i<3;++i)k[i].resize(nativeCounts[i]+nativeDegrees[i]+1);
        if(om9_cage_box_fill(nativeCounts.data(),nativeDegrees.data(),lo.data(),hi.data(),coordinates.data(),size,k[0].data(),k[0].size(),k[1].data(),k[1].size(),k[2].data(),k[2].size(),matrix.data())!=0)throw std::runtime_error("Cage requires a finite positive box");
        auto points=vectors(coordinates);Base::Matrix4D reference;for(int i=0;i<4;++i)for(int j=0;j<4;++j)reference[i][j]=matrix[i*4+j];
        d->openTransaction("Create Cage");try{auto* c=dynamic_cast<CageControl*>(d->addObject("OpenMatrix9Gui::CageControl","Cage"));c->Counts.setValues(n);c->Degrees.setValues(p);c->UKnots.setValues(k[0]);c->VKnots.setValues(k[1]);c->WKnots.setValues(k[2]);c->ControlPoints.setValues(points);c->Weights.setValues(std::vector<double>(size,1));c->ReferenceFrame.setValue(reference);d->recompute();if(c->isError())throw std::runtime_error("Cannot build cage");d->commitTransaction();return c->getPyObject();}catch(...){d->abortTransaction();throw;}
    }catch(const Base::Exception& e){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}catch(const std::exception& e){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}catch(...){PyErr_Clear();PyErr_SetString(PyExc_RuntimeError,"Native cage operation failed");return nullptr;}
}
}
CageControl::CageControl(){
    ADD_PROPERTY(ControlPoints,(std::vector<Base::Vector3d>()));ADD_PROPERTY(Counts,(std::vector<long>()));ADD_PROPERTY(Degrees,(std::vector<long>()));ADD_PROPERTY(UKnots,(std::vector<double>()));ADD_PROPERTY(VKnots,(std::vector<double>()));ADD_PROPERTY(WKnots,(std::vector<double>()));ADD_PROPERTY(Weights,(std::vector<double>()));ADD_PROPERTY(ReferenceFrame,(Base::Matrix4D()));ADD_PROPERTY(OM9FeatureId,("OM9-TRANSFORM-041"));ADD_PROPERTY(SourceMetadata,(""));OM9FeatureId.setReadOnly(true);
}
short CageControl::mustExecute()const{return ControlPoints.isTouched()||Counts.isTouched()||Degrees.isTouched()||Weights.isTouched()||UKnots.isTouched()||VKnots.isTouched()||WKnots.isTouched()?1:Part::Feature::mustExecute();}
App::DocumentObjectExecReturn* CageControl::execute(){try{auto handle=cageHandle(*this);auto n=Counts.getValues();const auto& points=ControlPoints.getValues();TopoDS_Compound lattice;BRep_Builder builder;builder.MakeCompound(lattice);
    auto index=[&](long u,long v,long w){return std::size_t((w*n[1]+v)*n[0]+u);};
    for(long w=0;w<n[2];++w)for(long v=0;v<n[1];++v)for(long u=0;u<n[0];++u){auto a=points[index(u,v,w)];long q[3]={u,v,w};for(int axis=0;axis<3;++axis)if(q[axis]+1<n[axis]){auto next=q[axis];++q[axis];auto b=points[index(q[0],q[1],q[2])];q[axis]=next;if((a-b).Length()>1e-12)builder.Add(lattice,BRepBuilderAPI_MakeEdge(gp_Pnt(a.x,a.y,a.z),gp_Pnt(b.x,b.y,b.z)).Edge());}}
    auto placement=Placement.getValue();Part::TopoShape value(lattice);value.setTransform(placement.toMatrix());Shape.setValue(value);return Part::Feature::execute();
    }catch(const Base::Exception& e){Shape.setValue(Part::TopoShape());return new App::DocumentObjectExecReturn(e.what(),this);}catch(const std::exception& e){Shape.setValue(Part::TopoShape());return new App::DocumentObjectExecReturn(e.what(),this);}catch(...){Shape.setValue(Part::TopoShape());return new App::DocumentObjectExecReturn("Native cage control rebuild failed",this);}}
CageBinding::CageBinding(){
    ADD_PROPERTY_TYPE(ControlPlacementSources,(nullptr),"OpenMatrix9",App::Prop_Hidden,"Native control ancestor frames without container cycles");
    ADD_PROPERTY_TYPE(CaptivePlacementSources,(nullptr),"OpenMatrix9",App::Prop_Hidden,"Native captive ancestor frames for independent-edit policy");
    ADD_PROPERTY(Control,(nullptr));ADD_PROPERTY(Captive,(nullptr));ADD_PROPERTY(Active,(false));ADD_PROPERTY(MeshSource,(false));ADD_PROPERTY(SourceShape,(Part::TopoShape()));ADD_PROPERTY(SourceVertices,(std::vector<Base::Vector3d>()));ADD_PROPERTY(BoundParameters,(std::vector<Base::Vector3d>()));ADD_PROPERTY(SourceTriangles,(std::vector<long>()));ADD_PROPERTY(BoundFrame,(Base::Matrix4D()));ADD_PROPERTY(AffineBind,(false));ADD_PROPERTY(Region,("Global"));ADD_PROPERTY(LocalMin,(Base::Vector3d()));ADD_PROPERTY(LocalMax,(Base::Vector3d(1,1,1)));ADD_PROPERTY(Falloff,(0));ADD_PROPERTY(OM9FeatureId,("OM9-TRANSFORM-020"));
    for(auto* p:std::initializer_list<App::Property*>{&Control,&Captive,&Active,&MeshSource,&SourceShape,&SourceVertices,&BoundParameters,&SourceTriangles,&OM9FeatureId,&BoundFrame,&AffineBind})p->setReadOnly(true);
}
CageBinding::~CageBinding(){changed.disconnect();beforeChanged.disconnect();}
void CageBinding::refreshPlacements(){auto control=placementAncestors(Control.getValue()),captive=placementAncestors(Captive.getValue());Writing internal(writing);if(control!=ControlPlacementSources.getValues())ControlPlacementSources.setValues(control);if(captive!=CaptivePlacementSources.getValues())CaptivePlacementSources.setValues(captive);}
void CageBinding::onSettingDocument(){App::DocumentObject::onSettingDocument();changed.disconnect();beforeChanged.disconnect();if(auto* d=getDocument()){
    beforeChanged=d->signalBeforeChangeObject.connect([this](const App::DocumentObject& object,const App::Property& prop){
        if(writing||!Active.getValue()||!getDocument()||getDocument()->testStatus(App::Document::Restoring)||getDocument()->isPerformingTransaction()||isRemoving())return;
        const bool direct=&object==Captive.getValue()&&!object.isRecomputing()&&(object.getPropertyByName("Shape")==&prop||object.getPropertyByName("Mesh")==&prop||object.getPropertyByName("Placement")==&prop);
        const bool ancestor=object.getPropertyByName("Placement")==&prop&&includes(CaptivePlacementSources.getValues(),object)&&!includes(ControlPlacementSources.getValues(),object);
        if((direct||ancestor)&&historyEditLocked(Captive.getValue()))throw Base::RuntimeError("History Lock prevents captive geometry edits; edit the cage or release the captive");
    });
    changed=d->signalChangedObject.connect([this](const App::DocumentObject& object,const App::Property& prop){
    if(writing||!Active.getValue()||!getDocument()||getDocument()->testStatus(App::Document::Restoring)||getDocument()->isPerformingTransaction()||isRemoving())return;
    if(object.getPropertyByName("Group")==&prop){refreshPlacements();touch();}
    const bool controlFrame=object.getPropertyByName("Placement")==&prop&&includes(ControlPlacementSources.getValues(),object);
    const bool captiveFrame=object.getPropertyByName("Placement")==&prop&&includes(CaptivePlacementSources.getValues(),object);
    if(&object==Control.getValue()||controlFrame)touch();
    if((captiveFrame&&!controlFrame)||(&object==Captive.getValue()&&!object.isRecomputing()&&(object.getPropertyByName("Shape")==&prop||object.getPropertyByName("Mesh")==&prop||object.getPropertyByName("Placement")==&prop))){warnBrokenHistory(Captive.getValue());detach();}
    if(&object==Control.getValue())if(auto* c=dynamic_cast<const CageControl*>(&object);c&&&prop==&c->Shape&&c->Shape.getShape().isNull())clearInvalidGeometry();
});}}
void CageBinding::unsetupObject(){changed.disconnect();beforeChanged.disconnect();App::DocumentObject::unsetupObject();}
void CageBinding::onUndoRedoFinished(){App::DocumentObject::onUndoRedoFinished();refreshPlacements();if(Active.getValue())touch();}
void CageBinding::onDocumentRestored(){Writing internal(writing);App::DocumentObject::onDocumentRestored();refreshPlacements();}
void CageBinding::onChanged(const App::Property* p){App::DocumentObject::onChanged(p);if(writing||isRestoring()||isRemoving()||!getDocument()||getDocument()->testStatus(App::Document::Restoring)||getDocument()->isPerformingTransaction())return;if(p==&Control||p==&Captive){refreshPlacements();if(Active.getValue())touch();}}
void CageBinding::onLostLinkToObject(App::DocumentObject* object){const bool lostControl=Control.getValue()==object,lostCaptive=Captive.getValue()==object;App::DocumentObject::onLostLinkToObject(object);if(Active.getValue()&&(lostControl||lostCaptive)){if(lostControl)clearInvalidGeometry();touch();setError();}}
void CageBinding::detach(){Writing internal(writing);Active.setValue(false);Control.setValue(nullptr);ControlPlacementSources.setValues({});CaptivePlacementSources.setValues({});}
void CageBinding::clearInvalidGeometry(){if(auto* target=Captive.getValue()){Writing internal(writing);if(auto* p=dynamic_cast<Part::Feature*>(target))p->Shape.setValue(Part::TopoShape());else if(MeshSource.getValue()){Base::PyGILStateLocker lock;setMesh(*target,{},{});}}}
short CageBinding::mustExecute()const{return Active.getValue()&&isTouched()?1:App::DocumentObject::mustExecute();}
App::DocumentObjectExecReturn* CageBinding::execute(){
    if(!Active.getValue())return App::DocumentObject::StdReturn;
    auto* control=dynamic_cast<CageControl*>(Control.getValue());auto* target=Captive.getValue();if(!control||!target){if(target)clearInvalidGeometry();return new App::DocumentObjectExecReturn("Missing cage control or captive; Undo restores the relationship",this);}
    const char* stage="validate cage";
    try{Base::PyGILStateLocker lock;Writing internal(writing);if(control->getDocument()!=getDocument()||target->getDocument()!=getDocument()||!control->isValid())throw std::runtime_error("Missing/invalid cage control");auto handle=cageHandle(*control,this);
        auto original=flat(SourceVertices.getValues()),parameters=flat(BoundParameters.getValues());if(original.size()!=parameters.size())throw std::runtime_error("Invalid saved cage binding");std::vector<double> result(original.size());
        stage="evaluate cage coordinates";if(om9_cage_apply(handle.get(),original.data(),parameters.data(),original.size()/3,result.data(),result.size()/3)!=0)throw std::runtime_error("Cage binding evaluation failed");auto mapped=vectors(result);
        auto affine=affineForBinding(handle,*control);
        if(!AffineBind.getValue())affine.reset();
        else if(affine){Base::Matrix4D edited;for(int i=0;i<4;++i)for(int j=0;j<4;++j)edited[i][j]=(*affine)[i*4+j];auto inverse=control->ReferenceFrame.getValue();inverse.inverseGauss();affine=array(edited*inverse*BoundFrame.getValue());}
        stage="write native cage geometry";if(MeshSource.getValue())applyMesh(*target,mapped,SourceTriangles.getValues());
        else {stage="read native BRep snapshot";auto source=SourceShape.getValue();stage="deform native BRep";auto shape=cageDeformShape(source,mapped,affine);stage="assign native BRep";applyShape(*target,shape);}
        return App::DocumentObject::StdReturn;
    }catch(const Base::Exception& e){try{clearInvalidGeometry();}catch(...){}return new App::DocumentObjectExecReturn(std::string(stage)+": "+e.what(),this);
    }catch(const std::exception& e){try{clearInvalidGeometry();}catch(...){}return new App::DocumentObjectExecReturn(e.what(),this);
    }catch(...){try{clearInvalidGeometry();}catch(...){}return new App::DocumentObjectExecReturn(std::string(stage)+": native cage rebuild failed",this);}
}
void initializeCageTypes(){CageControl::init();CageBinding::init();}
void AddCageMethods(PyObject* module){static PyMethodDef methods[]={{"createCage",create,METH_VARARGS,"Create an independent rational box cage."},{"captureCage",capture,METH_VARARGS,"Atomically bind native captives to the current cage."},{"releaseFromCage",release,METH_VARARGS,"Detach selected captives while preserving current geometry."},{"restore3dmCage",restoreArchive,METH_VARARGS,"Rebind explicit Rhino cage captives from a verified current archive snapshot."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
bool cageHistoryRecorded(App::DocumentObject* o){if(auto* b=dynamic_cast<CageBinding*>(o))return b->Active.getValue()&&b->Control.getValue()&&b->Captive.getValue();return !bindingsFor(o).empty();}
std::vector<App::DocumentObject*> cageHistoryParents(App::DocumentObject* o){std::vector<App::DocumentObject*> out;for(auto* b:bindingsFor(o))if(auto* c=b->Control.getValue())out.push_back(c);return out;}
void detachCageHistory(App::DocumentObject* o){for(auto* b:bindingsFor(o))b->detach();}
bool isCageStorageObject(App::DocumentObject* o){return dynamic_cast<CageBinding*>(o)!=nullptr;}
}
