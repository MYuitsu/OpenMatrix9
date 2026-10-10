#include <Python.h>
#include "SnapObjectInfo.h"
#include "CoreSnapGeometry.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/GeoFeature.h>
#include <App/PropertyStandard.h>
#include <App/PropertyGeo.h>
#include <Mod/Part/App/PropertyTopoShape.h>
#include <BRep_Tool.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
extern "C" bool om9_modeling_snap_allowed(unsigned,bool,bool,unsigned);
namespace OpenMatrix9Gui {
SnapObjectInfo classifySnapObject(const App::DocumentObject* object) {
    SnapObjectInfo info;
    if(!object)return info;
    auto* source=object->getLinkedObject(true);
    if(!source)return info;
    info.resolved_member=source;
    if(auto* cap=source->getPropertyByName<App::PropertyString>("OM9Capability")) {
        const std::string value=cap->getValue();
        info.preview=value=="retained"||value=="display-retained"||value=="incompatible";
    }
    if(source->getPropertyByName("OM9CloudPoints")) {info.kind=5;info.representation="native-cloud";return info;}
    if(source->getTypeId().isDerivedFrom(Base::Type::fromName("Mesh::Feature"))) {info.kind=4;info.representation="native-mesh";return info;}
    auto* property=source->getPropertyByName<Part::PropertyPartShape>("Shape");
    if(!property||property->getValue().IsNull())return info;
    info.shape=property->getValue().Located(TopLoc_Location());
    const auto type=info.shape.ShapeType();
    info.kind=type==TopAbs_VERTEX?1:(type==TopAbs_EDGE||type==TopAbs_WIRE?2:3);
    info.native_cad=true;info.representation="native-cad";
    info.display_mesh_state="absent";
    TopExp_Explorer faces(info.shape,TopAbs_FACE);
    if(faces.More()) {TopLoc_Location location;info.display_mesh_state=BRep_Tool::Triangulation(TopoDS::Face(faces.Current()),location).IsNull()?"unknown":"present";}
    Base::Matrix4D local;
    object->getLinkedObject(true,&local,true);
    Base::Placement parent;
    if(auto* placement=object->getPropertyByName<App::PropertyPlacement>("Placement"))
        parent=App::GeoFeature::getGlobalPlacement(object)*placement->getValue().inverse();
    info.global_transform=parent.toMatrix()*local;
    return info;
}
}
namespace {
App::DocumentObject* findObject(const char* docName,const char* name) {
    auto* doc=App::GetApplication().getDocument(docName);
    auto* object=doc?doc->getObject(name):nullptr;
    if(!object)PyErr_SetString(PyExc_ValueError,"Snap object is not in the named document");
    return object;
}
PyObject* encode(const QJsonObject& result) {
    const auto bytes=QJsonDocument(result).toJson(QJsonDocument::Compact);
    return PyUnicode_FromStringAndSize(bytes.constData(),bytes.size());
}
PyObject* classify(PyObject*,PyObject* args) {
    const char *doc,*name;if(!PyArg_ParseTuple(args,"ss",&doc,&name))return nullptr;
    auto* object=findObject(doc,name);if(!object)return nullptr;
    try {
        const auto info=OpenMatrix9Gui::classifySnapObject(object);unsigned modes=0;
        for(unsigned mode:{2u,4u,8u})if(om9_modeling_snap_allowed(info.kind,info.native_cad,info.preview,mode))modes|=mode;
        return encode({{"kind",int(info.kind)},{"native_cad",info.native_cad},{"preview",info.preview},
            {"representation",QString::fromStdString(info.representation)},{"display_mesh_state",QString::fromStdString(info.display_mesh_state)},
            {"eligible_modes",int(modes)}});
    }catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
PyObject* candidates(PyObject*,PyObject* args) {
    const char *doc,*name;unsigned mode,budget;
    if(!PyArg_ParseTuple(args,"ssII",&doc,&name,&mode,&budget))return nullptr;
    if(!budget||budget>2048){PyErr_SetString(PyExc_ValueError,"Candidate budget must be 1..2048");return nullptr;}
    auto* object=findObject(doc,name);if(!object)return nullptr;
    try {
        const auto result=OpenMatrix9Gui::boundedSnapCandidates(object,mode,budget);
        QJsonArray points;for(const auto& p:result.points)points.append(QJsonArray{p.x,p.y,p.z});
        return encode({{"points",points},{"complete",result.complete},{"visited_topology",int(result.visited_topology)},
            {"read_mesh_vertices",0},{"read_cloud_points",0}});
    }catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
}
}
void AddModelingSnapMethods(PyObject* module) {
    static PyMethodDef methods[]={{"classifySnap3dm",classify,METH_VARARGS,"Classify native geometry without point-array reads."},
        {"snapObjectCandidates3dm",candidates,METH_VARARGS,"Bounded native CAD topology candidates."},{nullptr,nullptr,0,nullptr}};
    PyModule_AddFunctions(module,methods);
}
