#include "ModelingCurveEditor.h"
#include "CurveGeometry.h"
#include "SnapObjectInfo.h"
#include "CurveBasisTransfer.h"
#include "ModelingCurveEditorDialog.h"
#include "LayerDocumentAdapter.h"
#include <Base/Interpreter.h>
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <Mod/Part/App/PropertyTopoShape.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <Gui/Selection/Selection.h>
#include <BRepAdaptor_Curve.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepBuilderAPI_GTransform.hxx>
#include <TopTools_ListOfShape.hxx>
#include <set>
#include <BRepCheck_Analyzer.hxx>
#include <BRep_Tool.hxx>
#include <GeomConvert.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_TrimmedCurve.hxx>
#include <NCollection_Array1.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <QJsonDocument>
#include <QJsonArray>
#include <QCryptographicHash>
#include <cmath>
#include <stdexcept>
namespace {
using OpenMatrix9Gui::phase2Require;
struct Basis {App::DocumentObject* object;Part::PropertyPartShape* property;Handle(Geom_BSplineCurve) curve;double first,last;TopoDS_Edge edge;std::uint64_t layerGeneration;};
Basis readBasis(App::Document& doc,const std::string& name) {
    const auto generation=OpenMatrix9Gui::layerMutationGeneration(doc,{name},1);
    auto* object=doc.getObject(name.c_str());if(!object||object->getLinkedObject(true)!=object)throw std::runtime_error("Select one owning editable native curve, not a Link");
    const auto info=OpenMatrix9Gui::classifySnapObject(object);if(!info.native_cad||info.preview)throw std::runtime_error("Selection is not independently editable CAD");
    auto* property=object->getPropertyByName<Part::PropertyPartShape>("Shape");
    TopExp_Explorer faces(info.shape,TopAbs_FACE);if(faces.More())throw std::runtime_error("Surface/solid CV editing is outside this curve editor");
    TopExp_Explorer edges(info.shape,TopAbs_EDGE);if(!edges.More())throw std::runtime_error("Select a single native curve edge");
    const auto edge=TopoDS::Edge(edges.Current());edges.Next();if(edges.More())throw std::runtime_error("Select a single native curve edge, not a multi-edge wire");
    double first,last;auto geometry=BRep_Tool::Curve(edge,first,last);
    if(geometry.IsNull()||!std::isfinite(first)||!std::isfinite(last)||last<=first)throw std::runtime_error("Curve has an invalid parameter domain");
    auto curve=Handle(Geom_BSplineCurve)::DownCast(geometry);
    if(curve.IsNull()){curve=GeomConvert::CurveToBSplineCurve(new Geom_TrimmedCurve(geometry,first,last));first=curve->FirstParameter();last=curve->LastParameter();}
    if(curve.IsNull()||curve->NbPoles()>4096||curve->NbKnots()>4096)throw std::runtime_error("Selected curve exceeds editor basis budget");
    return {object,property,curve,first,last,edge,generation};
}
QJsonObject values(const Basis& basis) {
    QJsonArray poles,weights,knots,multiplicities;
    for(int i=1;i<=basis.curve->NbPoles();++i) {const auto p=basis.curve->Pole(i);poles.append(QJsonArray{p.X(),p.Y(),p.Z()});weights.append(basis.curve->Weight(i));}
    for(int i=1;i<=basis.curve->NbKnots();++i) {knots.append(basis.curve->Knot(i));multiplicities.append(basis.curve->Multiplicity(i));}
    QJsonObject output{{"degree",basis.curve->Degree()},{"periodic",bool(basis.curve->IsPeriodic())},{"poles",poles},{"weights",weights},{"knots",knots},{"multiplicities",multiplicities},{"first",basis.first},{"last",basis.last}};
    const auto info=OpenMatrix9Gui::classifySnapObject(basis.object);
    QJsonArray placement;for(int i=0;i<4;++i)for(int j=0;j<4;++j)placement.append(info.global_transform[i][j]);
    QJsonObject signature=output;signature["placement"]=placement;signature["object_identity"]=QString::number(reinterpret_cast<quintptr>(basis.object));
    signature["document_identity"]=QString::number(reinterpret_cast<quintptr>(basis.object->getDocument()));
    signature["document_uuid"]=QString::fromStdString(basis.object->getDocument()->Uid.getValueStr());
    signature["layer_generation"]=QString::number(basis.layerGeneration);
    signature["object_id"]=QString::number(basis.object->getID());
    signature["edge_orientation"]=int(basis.edge.Orientation());
    output["signature"]=QString::fromLatin1(QCryptographicHash::hash(QJsonDocument(signature).toJson(QJsonDocument::Compact),QCryptographicHash::Sha256).toHex());
    return output;
}
bool ready(App::Document& doc) {auto* gui=Gui::Application::Instance->activeDocument();return &doc==App::GetApplication().getActiveDocument()&&gui&&!gui->getInEdit()&&!gui->isAboutToClose()&&Gui::Control().isAllowedAlterDocument(&doc);}
std::string joinCurves(App::Document& doc,const std::vector<std::string>& names) {
    if(!ready(doc))throw std::runtime_error("Join needs an editable active project");phase2Require(om9_phase2_join_options(names.size(),1));
    OpenMatrix9Gui::requireLayerGeometryEditable(doc);
    OpenMatrix9Gui::layerMutationGeneration(doc,names,1);
    std::vector<std::uint64_t> tokens;for(const auto& name:names){auto* object=doc.getObject(name.c_str());tokens.push_back(object?std::uint64_t(object->getID())+1:0);}phase2Require(om9_phase2_join_selection(tokens.data(),tokens.size(),1));
    TopTools_ListOfShape edges;std::size_t count=0;
    for(const auto& name:names) {
        const auto info=OpenMatrix9Gui::classifySnapObject(doc.getObject(name.c_str()));
        if(!info.native_cad||info.preview||info.kind!=2)throw std::runtime_error("Join supports native curves only");
        if(BRep_Tool::IsClosed(info.shape))throw std::runtime_error("Join requires open curves with free endpoints; a closed curve has no endpoint to join");
        gp_GTrsf transform;for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)transform.SetValue(i,j,info.global_transform[i-1][j-1]);
        const auto world=BRepBuilderAPI_GTransform(info.shape,transform,true).Shape();
        for(TopExp_Explorer explorer(world,TopAbs_EDGE);explorer.More();explorer.Next()) {
            if(BRep_Tool::IsClosed(explorer.Current()))throw std::runtime_error("Join cannot include a closed curve segment");
            phase2Require(om9_phase2_join_options(names.size(),++count));edges.Append(explorer.Current());
        }
    }
    BRepBuilderAPI_MakeWire builder;builder.Add(edges);
    if(builder.Error()!=BRepBuilderAPI_WireDone||!builder.IsDone())throw std::runtime_error("Selected curves do not form one connected wire");
    const auto wire=builder.Wire();
    if(!BRepCheck_Analyzer(wire).IsValid())throw std::runtime_error("Join could not construct a valid native wire");
    OpenMatrix9Gui::validateCurveWire(wire,count);
    OpenMatrix9Gui::layerMutationGeneration(doc,names,1);
    const auto transaction=doc.openTransaction("Join curves");
    try {OpenMatrix9Gui::LayerGeometryTransaction layers(doc,transaction);OpenMatrix9Gui::layerMutationGeneration(doc,names,1);auto* object=doc.addObject("Part::Feature","JoinedCurve");object->getPropertyByName<Part::PropertyPartShape>("Shape")->setValue(wire);doc.recompute();const std::string name=object->getNameInDocument();layers.finish();doc.commitTransaction();return name;}
    catch(...){if(OpenMatrix9Gui::ownsLayerGeometryTransaction(doc,transaction))doc.abortTransaction();throw;}
}
}
namespace OpenMatrix9Gui {
QJsonObject readModelingCurveBasis(App::Document& doc,const std::string& name) {return values(readBasis(doc,name));}
CurveEditResult editModelingCurve(App::Document& doc,const std::string& name,const CurveEditRequest& request) {
    if(!ready(doc))throw std::runtime_error("The active project is not editable");
    requireLayerGeometryEditable(doc);
    const auto original=readBasis(doc,name);const auto current=values(original);const auto& input=request.basis;
    const auto signature=input.value("signature").toString().toUtf8(),currentSignature=current.value("signature").toString().toUtf8();
    const Om9WitnessInput initial{std::uint64_t(reinterpret_cast<std::uintptr_t>(&doc)),std::uint64_t(original.object->getID())+1,0,reinterpret_cast<const std::uint8_t*>(signature.constData()),std::size_t(signature.size())};
    auto now=initial;now.signature=reinterpret_cast<const std::uint8_t*>(currentSignature.constData());now.signature_len=currentSignature.size();
    CurveBasisTransfer decoded(input);auto raw=decoded.input();const auto session=om9_phase2_session_create(&initial,&raw);phase2Require(session!=0);
    struct Drop {std::uint64_t h;~Drop(){om9_phase2_session_drop(h);}} drop{session};
    phase2Require(om9_phase2_session_check(session,&now),session);const CurveBasisTransfer owned(session);
    const int degree=int(owned.degree),count=int(owned.poles.size()/3),kcount=int(owned.knots.size());
    NCollection_Array1<gp_Pnt> nativePoles(1,count);NCollection_Array1<double> nativeWeights(1,count),nativeKnots(1,kcount);NCollection_Array1<int> nativeMults(1,kcount);
    for(int i=0;i<count;++i) {nativePoles.SetValue(i+1,gp_Pnt(owned.poles[i*3],owned.poles[i*3+1],owned.poles[i*3+2]));nativeWeights.SetValue(i+1,owned.weights[i]);}
    for(int i=0;i<kcount;++i){nativeKnots.SetValue(i+1,owned.knots[i]);nativeMults.SetValue(i+1,int(owned.mults[i]));}
    Handle(Geom_BSplineCurve) curve=new Geom_BSplineCurve(nativePoles,nativeWeights,nativeKnots,nativeMults,degree,bool(owned.periodic));
    const double first=owned.first,last=owned.last;
    if(last<=first||first<curve->FirstParameter()-1e-12||last>curve->LastParameter()+1e-12)throw std::runtime_error("Curve trim domain is outside the edited basis");
    BRepBuilderAPI_MakeEdge builder(curve,first,last);if(!builder.IsDone())throw std::runtime_error("Edited curve cannot form a native edge");
    auto shape=builder.Edge();shape.Orientation(original.edge.Orientation());shape.Location(original.property->getValue().Location());
    if(!BRepCheck_Analyzer(shape).IsValid())throw std::runtime_error("Edited curve is not valid CAD");
    const auto transaction=doc.openTransaction("Edit curve CV");
    try {
        LayerGeometryTransaction layers(doc,transaction);
        const auto commitBasis=readBasis(doc,name);
        const auto commitSignature=values(commitBasis).value("signature").toString().toUtf8();
        const Om9WitnessInput commitWitness{std::uint64_t(reinterpret_cast<std::uintptr_t>(&doc)),std::uint64_t(commitBasis.object->getID())+1,0,reinterpret_cast<const std::uint8_t*>(commitSignature.constData()),std::size_t(commitSignature.size())};
        phase2Require(om9_phase2_session_check(session,&commitWitness),session);
        commitBasis.property->setValue(shape);doc.recompute();layers.finish();doc.commitTransaction();
    }
    catch(...){if(ownsLayerGeometryTransaction(doc,transaction))doc.abortTransaction();throw;}
    return {true};
}
bool modelingCurveEditorAvailable() {
    auto* doc=App::GetApplication().getActiveDocument();if(!doc||!ready(*doc))return false;
    const auto selected=Gui::Selection().getSelection(doc->getName());if(selected.size()!=1||(selected.front().SubName&&*selected.front().SubName))return false;
    try{requireLayerGeometryEditable(*doc);layerMutationGeneration(*doc,{selected.front().pObject->getNameInDocument()},1);}catch(const std::exception&){return false;}
    const auto info=classifySnapObject(selected.front().pObject);return info.native_cad&&!info.preview&&info.kind==2;
}
bool startModelingCurveEditor() {
    if(!modelingCurveEditorAvailable())return false;
    try {auto* doc=App::GetApplication().getActiveDocument();const auto selected=Gui::Selection().getSelection(doc->getName());showModelingCurveEditor(*doc,selected.front().pObject->getNameInDocument());return true;}
    catch(const Standard_Failure& e){Base::Console().error("PointsOn: %s\n",e.GetMessageString());return false;}
    catch(const std::exception& e){Base::Console().error("PointsOn: %s\n",e.what());return false;}
}
bool modelingCurveJoinAvailable() {
    auto* doc=App::GetApplication().getActiveDocument();if(!doc||!ready(*doc))return false;
    const auto selected=Gui::Selection().getSelection(doc->getName());if(!om9_phase2_join_options(selected.size(),1))return false;
    for(const auto& entry:selected){if(entry.SubName&&*entry.SubName)return false;const auto info=classifySnapObject(entry.pObject);if(!info.native_cad||info.preview||info.kind!=2)return false;}
    std::vector<std::string> names;names.reserve(selected.size());for(const auto& entry:selected)names.emplace_back(entry.FeatName);
    try{requireLayerGeometryEditable(*doc);layerMutationGeneration(*doc,names,1);}catch(const std::exception&){return false;}
    return true;
}
bool startModelingCurveJoin() {
    if(!modelingCurveJoinAvailable())return false;
    auto* doc=App::GetApplication().getActiveDocument();std::vector<std::string> names;
    for(const auto& entry:Gui::Selection().getSelection(doc->getName()))names.emplace_back(entry.FeatName);
    try{joinCurves(*doc,names);return true;}catch(const std::exception& e){Base::Console().error("Join curves: %s\n",e.what());return false;}
}
}
namespace {
PyObject* ownership(PyObject*,PyObject*) {return PyUnicode_FromString("{\"curve\":\"rust\",\"session\":\"rust\",\"snap\":\"rust\",\"rebuild\":\"rust\",\"native_bridge\":\"FreeCAD/Qt/OCCT/openNURBS\"}");}
PyObject* basis(PyObject*,PyObject* args) {
    const char *docName,*name;if(!PyArg_ParseTuple(args,"ss",&docName,&name))return nullptr;
    auto* doc=App::GetApplication().getDocument(docName);if(!doc){PyErr_SetString(PyExc_ValueError,"Missing curve document");return nullptr;}
    try {const auto bytes=QJsonDocument(values(readBasis(*doc,name))).toJson(QJsonDocument::Compact);return PyUnicode_FromStringAndSize(bytes.constData(),bytes.size());}
    catch(const Standard_Failure& e){PyErr_SetString(PyExc_ValueError,e.GetMessageString());return nullptr;}
    catch(const std::exception& e){PyErr_SetString(PyExc_ValueError,e.what());return nullptr;}
}
PyObject* edit(PyObject*,PyObject* args) {
    const char *docName,*name,*json;if(!PyArg_ParseTuple(args,"sss",&docName,&name,&json))return nullptr;
    auto* doc=App::GetApplication().getDocument(docName);if(!doc){PyErr_SetString(PyExc_ValueError,"Missing curve document");return nullptr;}
    QJsonParseError error;const auto document=QJsonDocument::fromJson(json,&error);
    if(error.error!=QJsonParseError::NoError||!document.isObject()){PyErr_SetString(PyExc_ValueError,"Invalid typed curve request");return nullptr;}
    try {OpenMatrix9Gui::editModelingCurve(*doc,name,{document.object()});Py_RETURN_NONE;}
    catch(const Standard_Failure& e){PyErr_SetString(PyExc_ValueError,e.GetMessageString());return nullptr;}
    catch(const std::exception& e){PyErr_SetString(PyExc_ValueError,e.what());return nullptr;}
}
PyObject* join(PyObject*,PyObject* args) {
    const char *docName;PyObject* input;if(!PyArg_ParseTuple(args,"sO",&docName,&input))return nullptr;
    auto* doc=App::GetApplication().getDocument(docName);if(!doc){PyErr_SetString(PyExc_ValueError,"Missing curve document");return nullptr;}
    const auto count=PySequence_Size(input);if(count<0)return nullptr;if(!om9_phase2_join_options(std::size_t(count),1)){PyErr_SetString(PyExc_ValueError,OpenMatrix9Gui::phase2Error().c_str());return nullptr;}
    std::vector<std::string> names;
    for(Py_ssize_t i=0;i<count;++i){auto* item=PySequence_GetItem(input,i);if(!item)return nullptr;const char* value=PyUnicode_AsUTF8(item);if(!value){Py_DECREF(item);return nullptr;}names.emplace_back(value);Py_DECREF(item);}
    try{const auto name=joinCurves(*doc,names);return PyUnicode_FromString(name.c_str());}
    catch(const Standard_Failure& e){PyErr_SetString(PyExc_ValueError,e.GetMessageString());return nullptr;}
    catch(const std::exception& e){PyErr_SetString(PyExc_ValueError,e.what());return nullptr;}
}
}
void AddModelingCurveMethods(PyObject* module) {static PyMethodDef methods[]={{"phase2Ownership3dm",ownership,METH_NOARGS,"Phase2 portable owners and required native bridge."},{"curveBasis3dm",basis,METH_VARARGS,"Read typed basis of the explicit selected owning curve."},{"editModelingCurve3dm",edit,METH_VARARGS,"Validate an owned typed curve basis and commit one Undo transaction."},{"joinModelingCurves3dm",join,METH_VARARGS,"Join selected curves as an independent native wire."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
