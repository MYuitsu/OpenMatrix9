// SPDX-License-Identifier: LGPL-2.1-or-later
// Spec: OM9-SOLID-012 (Box), OM9-SOLID-014 (Sphere).
#include "SolidController.h"
#include "CoreLayers.h"
#include "RustBridge.h"
#include "CoreGeometryContract.h"
#include "CoreUnits.h"
#include "CurveController.h"
#include "CoreWorkspace.h"
#include "CoreDistance.h"
#include "CorePictureFrame.h"
#include "CoreViewControls.h"
#include "CoreSnaps.h"
#include "CoreKeyboard.h"
#include "CoreMouse.h"
#include <Base/Interpreter.h>
#include <Base/Console.h>
#include <Base/Exception.h>
#include <App/Application.h>
#include <App/Document.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Mod/Part/App/TopoShapePy.h>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <Inventor/SoDB.h>
#include <Inventor/SoInput.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <Inventor/nodes/SoCoordinate3.h>
#include <Inventor/nodes/SoLineSet.h>
#include <Inventor/nodes/SoPointSet.h>
#include <Inventor/nodes/SoBaseColor.h>
#include <Inventor/nodes/SoDrawStyle.h>
#include <Inventor/nodes/SoCamera.h>
#include <QApplication>
#include <QMouseEvent>
#include <QKeyEvent>
#include <QTimer>
#include <QToolButton>
#include <cmath>
#include <cstring>
#include <stdexcept>
#include <sstream>
#include <iomanip>
#include <array>
#include <algorithm>
namespace {
Gui::View3DInventor* activeView(){auto* gui=Gui::Application::Instance->activeDocument();return gui?dynamic_cast<Gui::View3DInventor*>(gui->getActiveView()):nullptr;}
Gui::View3DInventor* containing(QObject* object){
    auto* widget=qobject_cast<QWidget*>(object);auto* gui=Gui::Application::Instance->activeDocument();if(!widget||!gui)return nullptr;
    for(auto* mdi:gui->getMDIViews())if(auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);view&&(widget==view->getViewer()||view->getViewer()->isAncestorOf(widget)))return view;
    return nullptr;
}
std::string pythonError(){
    PyObject *type=nullptr,*value=nullptr,*trace=nullptr;PyErr_Fetch(&type,&value,&trace);PyErr_NormalizeException(&type,&value,&trace);
    PyObject* text=value?PyObject_Str(value):nullptr;const char* bytes=text?PyUnicode_AsUTF8(text):nullptr;
    std::string message=bytes?bytes:"Native solid adapter failed";Py_XDECREF(text);Py_XDECREF(type);Py_XDECREF(value);Py_XDECREF(trace);PyErr_Clear();return message;
}
struct Ref {
    PyObject* p;explicit Ref(PyObject* p):p(p){if(!p)throw std::runtime_error(pythonError());}
    ~Ref(){Py_DECREF(p);}Ref(const Ref&)=delete;
};
void set(PyObject* object,const char* name,PyObject* value){if(PyObject_SetAttrString(object,name,value)<0)throw std::runtime_error(pythonError());}
void addStringProperty(PyObject* object,const char* name,const char* text){
    Ref added(PyObject_CallMethod(object,"addProperty","sss","App::PropertyString",name,"OpenMatrix9"));Ref value(PyUnicode_FromString(text));set(object,name,value.p);
}
bool flag(PyObject* object,const char* method){Ref v(PyObject_CallMethod(object,method,nullptr));const int result=PyObject_IsTrue(v.p);if(result<0)throw std::runtime_error(pythonError());return result==1;}
// Fingerprint the same solver topology before/after callbacks. Native copies
// exclude display meshes; Checked is a mutable OCCT cache bit, not geometry.
PyObject* solidSolverFingerprint(PyObject* shape){
    Ref copy(PyObject_CallMethod(shape,"copy",nullptr));
    try{
        auto native=static_cast<Part::TopoShapePy*>(copy.p)->getTopoShapePtr()->getShape();
        TopTools_IndexedMapOfShape topology;TopExp::MapShapes(native,topology);
        for(int i=1;i<=topology.Extent();++i){auto item=topology(i);item.Checked(false);}
        Part::TopoShape owned(native);std::ostringstream out;owned.exportBrep(out);
        return PyUnicode_FromString(out.str().c_str());
    }catch(const Standard_Failure& error){throw std::runtime_error(error.GetMessageString());}
}
std::array<double,16> solidPlacement(PyObject* placement){
    Ref matrix(PyObject_CallMethod(placement,"toMatrix",nullptr)),values(PyObject_GetAttrString(matrix.p,"A")),sequence(PySequence_Fast(values.p,"Native solid Placement requires 16 values"));
    if(PySequence_Fast_GET_SIZE(sequence.p)!=16)throw std::runtime_error("Native solid Placement requires 16 values");
    std::array<double,16> out;
    for(std::size_t i=0;i<out.size();++i){out[i]=PyFloat_AsDouble(PySequence_Fast_GET_ITEM(sequence.p,i));if(PyErr_Occurred())throw std::runtime_error(pythonError());}
    return out;
}
void verifySolidPlacement(const std::array<double,16>& expected,PyObject* placement,const char* frame,const std::string& context){
    const auto actual=solidPlacement(placement);Om9SolidPublicationFrame input{};
    std::copy(expected.begin(),expected.end(),input.expected);std::copy(actual.begin(),actual.end(),input.actual);
    const auto status=om9_solid_publication_frame(input);if(!status)return;
    const char* category=status==1?"NonFiniteGeometry: ":status==2?"UnsupportedTransform: ":"StaleGeometry: ";
    throw std::runtime_error(std::string(category)+context+"Solid output "+frame+" Placement changed or is invalid before publication");
}
std::string solidInputIdentity(const std::map<std::size_t,OpenMatrix9Gui::SolidCurveReference>& refs,
                              const std::optional<OpenMatrix9Gui::SolidCurveReference>& path){
    std::string identity;
    const auto append=[&](const auto& ref){if(!identity.empty())identity+=", ";identity+=ref.input.name;if(!ref.sub.empty())identity+="."+ref.sub;};
    for(const auto& [_,ref]:refs)append(ref);
    if(path)append(*path);
    return identity.empty()?"typed construction points":identity;
}
void commitSolidPublication(App::Document& doc,const std::string& context){
    try{doc.commitTransaction();}
    catch(const Base::Exception& error){throw std::runtime_error(context+error.what());}
    catch(const std::exception& error){throw std::runtime_error(context+error.what());}
    catch(...){throw std::runtime_error(context+"native commit failed");}
}
std::string solidConstructionEvidence(std::uint64_t generation,const std::string& context){
    static_assert(sizeof(Om9SolidConstructionSummary)==16&&alignof(Om9SolidConstructionSummary)==8);
    const auto query=om9_solid_construction_json(generation,nullptr,0);
    if(query.status||query.reserved||!query.needed||query.needed>om9_prepared_result_limit())
        throw std::runtime_error(std::string(query.status==5?"StaleGeometry: ":"InvalidBasis: ")+context+"Rust construction capture rejected");
    std::string evidence(query.needed,'\0');
    const auto copied=om9_solid_construction_json(generation,reinterpret_cast<unsigned char*>(evidence.data()),evidence.size());
    if(copied.status||copied.reserved||copied.needed!=query.needed)
        throw std::runtime_error(std::string(copied.status==5?"StaleGeometry: ":"KernelFailure: ")+context+"Rust construction capture changed before copy");
    return evidence;
}
// Typed Python API calls invoke the host Part kernel without executing CMD text.
PyObject* buildSolid(unsigned kind,const double* b){
    Ref part(PyImport_ImportModule("Part")),app(PyImport_ImportModule("FreeCAD"));
    PyObject* shape=nullptr;
    if(kind==1){
        Ref box(PyObject_CallMethod(part.p,"makeBox","ddd",b[12],b[13],b[14]));Ref matrix(PyObject_CallMethod(app.p,"Matrix",nullptr));
        for(unsigned row=0;row<3;++row)for(unsigned col=0;col<4;++col){const auto key="A"+std::to_string(row+1)+std::to_string(col+1);Ref value(PyFloat_FromDouble(col<3?b[3+col*3+row]:b[row]));set(matrix.p,key.c_str(),value.p);}
        Ref transform(PyObject_CallMethod(box.p,"transformShape","O",matrix.p));shape=box.p;Py_INCREF(shape);
    }else if(kind==2){Ref center(PyObject_CallMethod(app.p,"Vector","ddd",b[0],b[1],b[2]));shape=PyObject_CallMethod(part.p,"makeSphere","dO",b[12],center.p);}
    else throw std::runtime_error("Unsupported solid command");
    Ref result(shape);Ref solids(PyObject_GetAttrString(result.p,"Solids")),volume(PyObject_GetAttrString(result.p,"Volume"));
    const double v=PyFloat_AsDouble(volume.p);if(PyErr_Occurred())throw std::runtime_error(pythonError());
    if(flag(result.p,"isNull")||!flag(result.p,"isValid")||!flag(result.p,"isClosed")||PySequence_Size(solids.p)!=1||!std::isfinite(v)||v<=0)throw std::runtime_error("Kernel did not produce one valid closed solid");
    Py_INCREF(result.p);return result.p;
}
}
namespace OpenMatrix9Gui {
SolidController& SolidController::instance(){static auto* controller=new SolidController;return *controller;}
SolidController::SolidController():QObject(qApp){
    deleteConnection=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(document==&doc)cancel();});
    activeConnection=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(active()&&document!=&doc)cancel();});
    auto* timer=new QTimer(this);timer->setInterval(100);connect(timer,&QTimer::timeout,this,[this]{if(active()&&!valid())cancel();});timer->start();
}
bool SolidController::handles(std::size_t i){return om9_native_solid_declared(om9_solid_kind(om9_command_id(i)));}
bool SolidController::matches(std::size_t i,const QString& name){return handles(i)&&om9_solid_kind(name.toUtf8().constData())==om9_solid_kind(om9_command_id(i));}
void SolidController::activate(){enabled=true;qApp->installEventFilter(this);}
void SolidController::deactivate(){enabled=false;cancel();}
bool SolidController::active()const{return document&&om9_solid_phase()!=0;}
bool SolidController::available(std::size_t i)const{
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    return enabled&&handles(i)&&doc&&gui&&!gui->isAboutToClose()&&!gui->getInEdit()&&activeView()&&Gui::Control().isAllowedAlterDocument(doc);
}
bool SolidController::valid()const{return document==App::GetApplication().getActiveDocument()&&available(command);}
void SolidController::frame(Gui::View3DInventor* view){
    const auto p=CoreWorkspace::instance().plane(view);const auto origin=p.getPosition();double b[12]={origin.x,origin.y,origin.z};
    for(unsigned i=0;i<3;++i){Base::Vector3d axis;axis[i]=1.;axis=p.getRotation().multVec(axis);for(unsigned j=0;j<3;++j)b[3+i*3+j]=axis[j];}
    if(!om9_solid_frame(b))throw std::runtime_error("Invalid construction plane");
}
void SolidController::prompt(){
    auto& cmd=CurveController::instance();
    if(optionBridge.active()){
        if(optionBridge.syncSolid()){cmd.renderOptions(optionBridge,{});return;}
        cancel();cmd.setPrompt("Solid session changed; restart the command. Command: ");return;
    }
    char b[2048]={};om9_solid_message(b,sizeof(b));cmd.setPrompt(QString::fromUtf8(b));cmd.logMessage(QString::fromUtf8(b));
}
bool SolidController::start(std::size_t i,bool recordInvocation){
    if(!available(i))return false;
    CoreDistance::instance().cancel();CorePictureFrame::instance().cancel();CoreViewControls::instance().cancel();CurveController::instance().cancel();cancel();
    try {
    // Reject corrupt saved frame metadata before creating a Rust session.
    CoreWorkspace::instance().plane(activeView());
    document=App::GetApplication().getActiveDocument();command=i;kind=om9_solid_kind(om9_command_id(i));
    unitContext=unitContextSnapshot(*document);
    if(!om9_solid_start(om9_command_id(i))){document=nullptr;return false;}
    inputEffect=0;
    if(!optionBridge.open(kind==1?14:15,unitInputScale(*document))||!optionBridge.syncSolid())throw std::runtime_error("Cannot open solid command options");
    if(recordInvocation)CurveController::instance().logInvocation(kind==1?"Box":"Sphere");
    frame(activeView());qApp->installEventFilter(this);CoreMouse::instance().prioritize();prompt();
    CurveController::instance().focusInput([generation=om9_solid_generation()]{return om9_solid_phase()!=0&&om9_solid_generation()==generation;});return false; // history on commit only
    }catch(const std::exception& error){cancel();CoreSnaps::clearTransient();CurveController::instance().setPrompt(QString::fromUtf8(error.what()));return false;}
}
void SolidController::clearPreview(){for(const auto& [root,node]:previews){if(root->findChild(node)>=0)root->removeChild(node);root->unref();}previews.clear();}
void SolidController::cancel(){optionBridge.close();if(document)CoreSnaps::clearTransient();clearPreview();om9_solid_cancel();document=nullptr;unitContext.clear();Base::PyGILStateLocker lock;tangentReferences.clear();pathReference.reset();fitSources.clear();}
bool SolidController::optionEvent(unsigned event,const QString& text,double number){
    if(!active()||!optionBridge.active())return false;
    if(!valid()||!optionBridge.syncSolid()){cancel();CurveController::instance().setPrompt("Solid session changed; restart the command. Command: ");return true;}
    auto& cmd=CurveController::instance();optionBridge.draft(cmd.draftInput());
    const auto outcome=optionBridge.event(event,text,number);
    if(outcome==0&&event!=2)return false;
    if(outcome==4){cmd.logMessage("Error: "+optionBridge.error());cmd.renderOptions(optionBridge,{});return true;}
    if(outcome!=0&&outcome!=3){cmd.renderOptions(optionBridge,{});return true;}
    const auto nativeInput=outcome==3?optionBridge.action():text;
    const auto beforePrompt=cmd.promptInput();inputEffect=0;
    submit(nativeInput);
    if(inputEffect){
        const auto afterPrompt=cmd.promptInput();
        // Preserve the prompt that admitted the input in the shared transcript.
        // The native solver has already accepted it; rejected drafts stay live.
        cmd.setPrompt(beforePrompt);
        if(event==1)cmd.setDraftInput(nativeInput);
        cmd.consumeOptionInput(2);
        if(active()){
            prompt();
            // A native solve may accept a parameter but require an explicit
            // solution branch. Keep that diagnostic in the CMD transcript.
            if(afterPrompt!=cmd.promptInput())cmd.logMessage("Error: "+afterPrompt);
        }else cmd.setPrompt(afterPrompt);
    }
    return true;
}
void SolidController::reference(const QString& name,const SolidPoint& pick){
    frame(activeView());Base::PyGILStateLocker lock;auto native=solidCurveReference(*document,name.toStdString(),"Sphere",true);const unsigned mode=om9_solid_reference_mode();
    if(mode==1){pathReference=std::move(native);result(om9_solid_reference(pick[0],pick[1],pick[2],0,0,1));}
    else if(mode==3){const auto located=solidOnCurve(native,pick);const auto i=om9_solid_constraints(nullptr);tangentReferences[i]=std::move(native);result(om9_solid_reference(located.first[0],located.first[1],located.first[2],0,0,1));}
    else throw std::runtime_error("No native curve selection is expected");
}
void SolidController::solveTangent(int solution){
    Base::PyGILStateLocker lock;verifySolidReferences(*document,tangentReferences,pathReference);double b[12],frameData[12];const auto count=om9_solid_constraints(b);if(!om9_solid_reference_frame(frameData))throw std::runtime_error("Missing tangent construction plane");
    const auto [center,radius]=solidTangentSphere(tangentReferences,b,count,frameData,om9_solid_tangent_radius(),solution);result(om9_solid_resolve(center[0],center[1],center[2],radius));
}
void SolidController::submit(const QString& text){
    if(!active())return;if(!valid()){cancel();return;}
    // Keep shared Osnap/Ortho controls usable during point entry.
    if(CoreSnaps::submit(text)||CoreKeyboard::instance().submit(text)){inputEffect=1;return;}
    try {frame(activeView());}
    catch(const std::exception& error){cancel();CurveController::instance().setPrompt(QString::fromUtf8(error.what()));return;}
    try{
        const auto t=text.trimmed();
        if((om9_solid_reference_mode()==1&&t.startsWith("Path=",Qt::CaseInsensitive))||(om9_solid_reference_mode()==3&&t.startsWith("Curve=",Qt::CaseInsensitive))){
            const auto value=t.section('=',1);const auto split=value.indexOf('@');const auto name=split<0?value:value.left(split);SolidPoint pick{0,0,0};
            if(split>=0){const auto coords=value.mid(split+1).split(',');if(coords.size()!=3)throw std::runtime_error("Curve pick syntax: Curve=Object.EdgeN@worldX,worldY,worldZ");for(int i=0;i<3;++i){bool ok=false;pick[i]=coords[i].toDouble(&ok);if(!ok||!std::isfinite(pick[i]))throw std::runtime_error("Curve pick coordinates must be finite");}}
            reference(name,pick);return;
        }
        if(om9_solid_reference_mode()==2&&t.startsWith("OnCurve=",Qt::CaseInsensitive)){if(!pathReference)throw std::runtime_error("Reselect the curve");bool ok=false;const auto fraction=t.section('=',1).toDouble(&ok);if(!ok)throw std::runtime_error("OnCurve requires a fraction within 0..1");Base::PyGILStateLocker lock;verifySolidReferences(*document,tangentReferences,pathReference);const auto [p,n]=solidOnCurve(*pathReference,{0,0,0},fraction);result(om9_solid_reference(p[0],p[1],p[2],n[0],n[1],n[2]));return;}
        if(om9_solid_phase()==21&&t.startsWith("Solution=",Qt::CaseInsensitive)){bool ok=false;const auto value=t.section('=',1).toInt(&ok);if(!ok||value<1)throw std::runtime_error("Solution index starts at 1");solveTangent(value-1);return;}
        if(om9_solid_reference_mode()==4 && ((t.isEmpty()&&om9_solid_point_count()==0)||t.compare("Selection",Qt::CaseInsensitive)==0)){
            Base::PyGILStateLocker lock;auto inputs=captureSolidFitSelection(*document,om9_solid_point_count());std::vector<double> flat;
            for(const auto& input:inputs)for(const auto& p:input.points)flat.insert(flat.end(),p.begin(),p.end());
            // Include all previously retained batches before either state accepts
            // this selection. Undo/cancel erase records and release their budget.
            validateSolidFitBudget(fitSources,inputs);
            // Reserve the guard store before Rust accepts an atomic point batch.
            fitSources.reserve(fitSources.size()+inputs.size());
            if(!flat.empty()){
                if(om9_solid_fit_points(flat.data(),flat.size()/3)!=1)throw std::runtime_error("Invalid selected fit point batch");
                for(auto& input:inputs)fitSources.push_back(std::move(input));
            }
            if(!t.isEmpty()){if(!flat.empty())inputEffect=1;prompt();return;}
        }
        const auto previousPoints=om9_solid_point_count();
        const auto effect=om9_solid_input(t.toUtf8().constData());
        if(std::strcmp(om9_solid_mode_name(),"FitPoints")!=0)fitSources.clear();
        else {const auto count=om9_solid_point_count();std::erase_if(fitSources,[&](const auto& input){return input.end>count;});}
        CoreSnaps::acceptedPoint(effect!=0&&effect!=3&&om9_solid_point_count()>previousPoints);
        result(effect);
        {Base::PyGILStateLocker lock;const auto count=om9_solid_constraints(nullptr);for(auto it=tangentReferences.begin();it!=tangentReferences.end();)if(it->first>=count)it=tangentReferences.erase(it);else ++it;if(om9_solid_phase()==16)pathReference.reset();}
    }catch(const std::exception& error){clearPreview();CurveController::instance().setPrompt(QString::fromUtf8(error.what()));}
}
void SolidController::result(unsigned effect){
    inputEffect=effect;
    clearPreview();
    if(effect==1&&om9_solid_phase()==21){try{solveTangent();}catch(const std::exception& error){CurveController::instance().setPrompt(QString::fromUtf8(error.what()));}return;}
    if(effect==2){
        const auto generation=om9_solid_generation();const auto executionCommand=command;
        try{commit();om9_sidebar_record_execution(executionCommand,true);if(om9_solid_generation()==generation)cancel();CurveController::instance().setPrompt(kind==1?"Command: ":"Solid created. Command:");}
        catch(const std::exception& error){
            inputEffect=0;auto& cmd=CurveController::instance();
            const auto diagnostic=QString::fromUtf8(error.what());
            if(om9_solid_generation()==generation){cancel();cmd.setPrompt(diagnostic);}
            else {
                // The failed publication belongs to the old session. Keep its
                // diagnostic in history and restore the callback's new phase.
                cmd.logMessage(diagnostic);cmd.setDraftInput({});
                if(active())prompt(); // Preserve a newer non-solid/idle prompt.
            }
            Base::Console().warning("OpenMatrix9 solid: %s\n",error.what());
        }
    }else if(effect==3){cancel();CurveController::instance().setPrompt(kind==1?"Command: ":"Solid command cancelled. Command:");}
    else {
        if(effect==0&&optionBridge.active()){char message[2048]={};om9_solid_message(message,sizeof(message));CurveController::instance().logMessage("Error: "+QString::fromUtf8(message));}
        prompt();
    }
}
void SolidController::recordAcceptedPointPhase(unsigned effect){
    if(optionBridge.active()&&(effect==1||effect==2)){
        auto& cmd=CurveController::instance();cmd.logMessage(cmd.promptInput());
    }
}
void SolidController::preview(const double* point,bool ortho){
    clearPreview();if(om9_native_solid_capability(1,4)!=4)return;
    if(kind==1){
        double edges[72],marks[9];const auto count=om9_solid_wire_preview(edges,marks,point[0],point[1],point[2],ortho);if(!count||count>12)return;
        auto* gui=Gui::Application::Instance->activeDocument();for(auto* mdi:gui->getMDIViews()){
            auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);if(!view)continue;auto* root=dynamic_cast<SoSeparator*>(view->getViewer()->getSceneGraph());if(!root)continue;
            auto* node=new SoSeparator;node->setName("OM9SolidPreview");auto* skip=new SoPickStyle;skip->style=SoPickStyle::UNPICKABLE;node->addChild(skip);
            auto* color=new SoBaseColor;color->rgb.setValue(166.f/255,104.f/255,209.f/255);node->addChild(color);
            auto* style=new SoDrawStyle;style->lineWidth=1.5f;style->pointSize=6.f;node->addChild(style);
            auto* coordinates=new SoCoordinate3;for(unsigned i=0;i<count*2;++i)coordinates->point.set1Value(i,edges[i*3],edges[i*3+1],edges[i*3+2]);node->addChild(coordinates);
            auto* lines=new SoLineSet;for(unsigned i=0;i<count;++i)lines->numVertices.set1Value(i,2);node->addChild(lines);
            auto* markers=new SoCoordinate3;for(unsigned i=0;i<3;++i)markers->point.set1Value(i,marks[i*3],marks[i*3+1],marks[i*3+2]);node->addChild(markers);
            auto* points=new SoPointSet;points->numPoints=3;node->addChild(points);
            root->ref();root->addChild(node);previews.emplace_back(root,node);view->getViewer()->redraw();
        }
        return;
    }
    double b[15];if(!om9_solid_geometry(b,true,point[0],point[1],point[2],ortho))return;
    try{
        Base::PyGILStateLocker lock;Ref shape(buildSolid(kind,b)),inventor(PyObject_CallMethod(shape.p,"writeInventor",nullptr));const char* text=PyUnicode_AsUTF8(inventor.p);if(!text)throw std::runtime_error(pythonError());
        auto* gui=Gui::Application::Instance->activeDocument();for(auto* mdi:gui->getMDIViews()){
            auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);if(!view)continue;auto* root=dynamic_cast<SoSeparator*>(view->getViewer()->getSceneGraph());if(!root)continue;
            SoInput input;input.setBuffer(text,std::strlen(text));auto* mesh=SoDB::readAll(&input);if(!mesh)throw std::runtime_error("Cannot render solid preview");
            auto* node=new SoSeparator;node->setName("OM9SolidPreview");auto* skip=new SoPickStyle;skip->style=SoPickStyle::UNPICKABLE;node->addChild(skip);node->addChild(mesh);root->ref();root->addChild(node);previews.emplace_back(root,node);view->getViewer()->redraw();
        }
    }catch(const std::exception&){clearPreview();}
}
void SolidController::commit(){
    const char* name=kind==1?"Box":"Sphere";
    if(!valid())throw std::runtime_error(std::string("StaleGeometry: ")+name+" input typed construction points: Document is no longer editable");validateCurrentSolidCapability(1,3,name,"typed construction points");double b[15];if(!om9_solid_geometry(b,false,0,0,0,false))throw std::runtime_error("No solid output");
    Base::PyGILStateLocker lock;
    // Keep request/source names and owned native captures stable across callbacks.
    auto* const target=document;const auto refs=tangentReferences;const auto path=pathReference;const auto fit=fitSources;
    const auto generation=om9_solid_generation();
    const auto units=unitContext;
    auto identity=solidInputIdentity(refs,path);for(const auto& input:fit)identity+="; "+input.source.objectName+(input.source.subelement.empty()?"":"."+input.source.subelement);
    const auto staleContext=std::string("StaleGeometry: ")+name+" input "+identity+": ";
    const auto commitContext=std::string("KernelFailure: ")+name+" input "+identity+": ";
    const auto verify=[&]{
        if(om9_solid_generation()!=generation||document!=target||!valid())throw std::runtime_error(staleContext+"Solid session changed before publication");
        try{
            if(unitContextSnapshot(*target)!=units)throw std::runtime_error("Solid unit context changed during command");
            verifySolidReferences(*target,refs,path);
            verifySolidFitSources(*target,fit);
        }
        catch(const std::exception& error){throw std::runtime_error(staleContext+error.what());}
    };
    verify();
    const auto construction=solidConstructionEvidence(generation,std::string(name)+" input "+identity+": ");
    const auto evidence=captureSolidEvidence(*target,refs,path,fit,construction,units);verify();Ref shape(buildSolid(kind,b));
    const auto outputName=target->getUniqueObjectName(name);
    const auto geometryResult=solidGeometryResult(*target,shape.p,evidence,outputName);verify();
    Ref expectedPlacement(PyObject_GetAttrString(shape.p,"Placement"));const auto expectedFrame=solidPlacement(expectedPlacement.p);
    Ref expectedBrep(solidSolverFingerprint(shape.p)),expectedOrientation(PyObject_GetAttrString(shape.p,"Orientation"));
    const char* feature=kind==1?"OM9-SOLID-012":"OM9-SOLID-014";
    std::ostringstream settings;settings<<std::setprecision(17)<<"mm;mode="<<om9_solid_mode_name()<<";origin="<<b[0]<<","<<b[1]<<","<<b[2]<<";dimensions="<<b[12]<<","<<b[13]<<","<<b[14]<<";axes=";for(unsigned i=3;i<12;++i)settings<<b[i]<<",";if(path)settings<<";path="<<path->input.name<<"."<<path->sub;for(const auto& [i,ref]:refs)settings<<";tangent"<<i<<"="<<ref.input.name<<"."<<ref.sub;
    const auto layer=CoreLayers::captureCircle(*target);verify();
    if(!target->openTransactionPreservingRedo(name))throw std::runtime_error(staleContext+"Solid publication transaction could not be opened");
    try{
        verify();Ref doc(target->getPyObject()),object(PyObject_CallMethod(doc.p,"addObject","ss","Part::Feature",outputName.c_str()));verify();set(object.p,"Shape",shape.p);
        addStringProperty(object.p,"OM9FeatureId",feature);addStringProperty(object.p,"OM9Command",name);
        addStringProperty(object.p,"OM9UnitContext",units.c_str());
        addStringProperty(object.p,"OM9SolidConstruction",construction.c_str());
        PyObject* constructionAttribute=PyObject_GetAttrString(object.p,"OM9SolidConstruction");
        if(!constructionAttribute)throw std::runtime_error(staleContext+"Solid output metadata unavailable before publication: OM9SolidConstruction: "+pythonError());
        Ref constructionValue(constructionAttribute);
        Ref readonly(PyObject_CallMethod(object.p,"setEditorMode","si","OM9SolidConstruction",1));
        addStringProperty(object.p,"OM9GeometryResult",geometryResult.c_str());
        PyObject* resultAttribute=PyObject_GetAttrString(object.p,"OM9GeometryResult");
        if(!resultAttribute)throw std::runtime_error(staleContext+"Solid output metadata unavailable before publication: OM9GeometryResult: "+pythonError());
        Ref resultValue(resultAttribute),resultReadonly(PyObject_CallMethod(object.p,"setEditorMode","si","OM9GeometryResult",1));
        addStringProperty(object.p,"SolidParameters",settings.str().c_str());
        Ref view(PyObject_GetAttrString(object.p,"ViewObject")),color(Py_BuildValue("(ddd)",166./255.,104./255.,209./255.));set(view.p,"ShapeColor",color.p);set(view.p,"LineColor",color.p);
        if(layer.index){Ref outputName(PyObject_GetAttrString(object.p,"Name"));auto* native=target->getObject(PyUnicode_AsUTF8(outputName.p));if(!native)throw std::runtime_error("Solid output identity missing");CoreLayers::applyCircle(*target,*native,layer);}
        target->recompute();verify();
        Ref current(PyObject_CallMethod(doc.p,"getObject","s",outputName.c_str()));
        if(current.p!=object.p)throw std::runtime_error(staleContext+"Solid output identity changed before publication");
        Ref localPlacement(PyObject_GetAttrString(object.p,"Placement")),worldPlacement(PyObject_CallMethod(object.p,"getGlobalPlacement",nullptr));
        const auto outputContext=std::string(name)+" input "+identity+": ";
        verifySolidPlacement(expectedFrame,localPlacement.p,"local",outputContext);verifySolidPlacement(expectedFrame,worldPlacement.p,"world",outputContext);
        Ref actualShape(PyObject_GetAttrString(object.p,"Shape")),shapePlacement(PyObject_GetAttrString(actualShape.p,"Placement"));
        verifySolidPlacement(expectedFrame,shapePlacement.p,"shape",outputContext);
        Ref partner(PyObject_CallMethod(actualShape.p,"isPartner","O",shape.p)),orientation(PyObject_GetAttrString(actualShape.p,"Orientation")),currentSolverBrep(solidSolverFingerprint(shape.p));
        const int samePartner=PyObject_IsTrue(partner.p),sameOrientation=PyObject_RichCompareBool(expectedOrientation.p,orientation.p,Py_EQ),sameBrep=PyObject_RichCompareBool(expectedBrep.p,currentSolverBrep.p,Py_EQ);
        if(samePartner<0||sameOrientation<0||sameBrep<0)throw std::runtime_error(commitContext+pythonError());
        if(!samePartner||!sameOrientation||!sameBrep)throw std::runtime_error(staleContext+"Solid output geometry or orientation changed before publication");
        const auto metadataMatches=[&](const char* property,const char* expected){
            PyObject* attribute=PyObject_GetAttrString(object.p,property);
            if(!attribute)throw std::runtime_error(staleContext+"Solid output metadata unavailable before publication: "+property+": "+pythonError());
            Ref value(attribute);
            const char* text=PyUnicode_AsUTF8(value.p);
            if(!text)throw std::runtime_error(commitContext+pythonError());
            return std::strcmp(text,expected)==0;
        };
        if(!metadataMatches("OM9FeatureId",feature)||!metadataMatches("OM9Command",name)||!metadataMatches("SolidParameters",settings.str().c_str())
            ||!metadataMatches("OM9SolidConstruction",construction.c_str())||!metadataMatches("OM9GeometryResult",geometryResult.c_str())||!metadataMatches("OM9UnitContext",units.c_str()))throw std::runtime_error(staleContext+"Solid output metadata changed before publication");
        verify();commitSolidPublication(*target,commitContext);
    }catch(...){target->abortTransaction();throw;}
}
bool SolidController::eventFilter(QObject* watched,QEvent* event){
    if(event->type()==QEvent::MouseButtonRelease&&releaseTarget==watched&&static_cast<QMouseEvent*>(event)->button()==releaseButton){releaseTarget=nullptr;return true;}
    if(auto* button=qobject_cast<QToolButton*>(watched);button&&button->property("om9Command").isValid()){
        const auto i=std::size_t(button->property("om9Command").toULongLong());
        if(handles(i)&&event->type()==QEvent::ContextMenu)return true;
        if(handles(i)&&event->type()==QEvent::MouseButtonPress&&static_cast<QMouseEvent*>(event)->button()==Qt::RightButton&&button->isEnabled()&&available(i)){
            start(i);submit(kind==1?"3Point":"2Point");releaseTarget=watched;releaseButton=Qt::RightButton;return true;
        }
    }
    if(!active()||Gui::Application::Instance->isClosing())return false;if(!valid()){cancel();return false;}
    auto* view=containing(watched);
    if(event->type()==QEvent::KeyPress&&CoreKeyboard::inputContext(watched)){
        auto* key=static_cast<QKeyEvent*>(event);
        if(key->key()==Qt::Key_Escape){CurveController::instance().cancelInput();CurveController::instance().setPrompt(kind==1?"Command: ":"Solid command cancelled. Command:");return true;}
        if(key->key()==Qt::Key_F4&&CoreKeyboard::pointKeyAllowed(watched,key)){
            try {frame(activeView());const auto p=CoreWorkspace::instance().plane(activeView()).getPosition();const auto effect=om9_solid_point(p.x,p.y,p.z,false);recordAcceptedPointPhase(effect);CoreSnaps::acceptedPoint(effect==1||effect==2);result(effect);}
            catch(const std::exception& error){cancel();CurveController::instance().setPrompt(QString::fromUtf8(error.what()));}
            return true;
        }
        if(view&&(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter||key->key()==Qt::Key_Space)){CurveController::instance().acceptInput();return true;}
    }
    const bool hover=event->type()==QEvent::MouseMove;
    if(!view||(!hover&&event->type()!=QEvent::MouseButtonPress))return false;
    auto* mouse=static_cast<QMouseEvent*>(event);if(!hover&&mouse->button()!=Qt::LeftButton)return false;
    if(mouse->modifiers()&(Qt::ControlModifier|Qt::AltModifier|Qt::MetaModifier))return false;
    if(!hover){releaseTarget=watched;releaseButton=Qt::LeftButton;}
    auto* viewer=view->getViewer();auto* widget=qobject_cast<QWidget*>(watched);const auto pos=viewer->viewport()->mapFrom(widget,mouse->position().toPoint());
    try {frame(view);}
    catch(const std::exception& error){cancel();CurveController::instance().setPrompt(QString::fromUtf8(error.what()));return !hover;}
    try{
        if(!hover&&(om9_solid_reference_mode()==1||om9_solid_reference_mode()==3)){
            Base::PyGILStateLocker lock;Ref viewObject(view->getPyObject());const auto pixel=viewer->fromQPoint(pos);Ref info(PyObject_CallMethod(viewObject.p,"getObjectInfo","((ii))",int(pixel[0]),int(pixel[1])));
            if(!PyDict_Check(info.p))throw std::runtime_error("Pick a native curve edge");auto* object=PyDict_GetItemString(info.p,"Object");auto* component=PyDict_GetItemString(info.p,"Component");if(!object||!component)throw std::runtime_error("Pick a native curve edge");
            const char* name=PyUnicode_AsUTF8(object);const char* edge=PyUnicode_AsUTF8(component);if(!name||!edge)throw std::runtime_error("Invalid native curve pick");SolidPoint pick;const char* axes[]={"x","y","z"};for(unsigned i=0;i<3;++i){auto* value=PyDict_GetItemString(info.p,axes[i]);if(!value)throw std::runtime_error("No world curve pick point");pick[i]=PyFloat_AsDouble(value);}reference(QString::fromUtf8(name)+"."+QString::fromUtf8(edge),pick);return true;
        }
        if(hover&&(om9_solid_reference_mode()==1||om9_solid_reference_mode()==3))return false;
        if(om9_solid_reference_mode()==2){
            if(hover)return false;
            if(!pathReference)throw std::runtime_error("Reselect native curve");
            Base::PyGILStateLocker lock;verifySolidReferences(*document,tangentReferences,pathReference);
            Ref viewObject(view->getPyObject());const auto pixel=viewer->fromQPoint(pos);Ref picks(PyObject_CallMethod(viewObject.p,"getObjectsInfo","((ii))",int(pixel[0]),int(pixel[1])));
            if(PyList_Check(picks.p))for(Py_ssize_t i=0;i<PyList_Size(picks.p);++i){auto* info=PyList_GetItem(picks.p,i);auto* object=PyDict_GetItemString(info,"Object");auto* component=PyDict_GetItemString(info,"Component");
                if(!object||!component||pathReference->input.name!=PyUnicode_AsUTF8(object)||pathReference->sub!=PyUnicode_AsUTF8(component))continue;
                SolidPoint seed;const char* keys[]={"x","y","z"};for(unsigned j=0;j<3;++j){auto* value=PyDict_GetItemString(info,keys[j]);if(!value)throw std::runtime_error("No world curve pick point");seed[j]=PyFloat_AsDouble(value);}
                const auto [c,n]=solidOnCurve(*pathReference,seed);result(om9_solid_reference(c[0],c[1],c[2],n[0],n[1],n[2]));return true;
            }
            throw std::runtime_error("Pick a center on the selected path, or enter OnCurve=fraction");
        }
        Base::Vector3d p;const auto pixel=viewer->fromQPoint(pos);
        if(!CoreSnaps::pick(view,pos,p)){
            SbVec3f nearPoint,farPoint;viewer->projectPointToLine(pixel,nearPoint,farPoint);Base::Vector3d a(nearPoint[0],nearPoint[1],nearPoint[2]),d(farPoint[0]-nearPoint[0],farPoint[1]-nearPoint[1],farPoint[2]-nearPoint[2]);
            double axis[6];
            if(om9_solid_height_axis(axis)){
                // Closest point on the height axis to the mouse ray.
                const Base::Vector3d origin(axis[0],axis[1],axis[2]),n(axis[3],axis[4],axis[5]);const auto delta=a-origin;const double dn=d.Dot(n),dd=d.Dot(d),den=dd-dn*dn;
                if(std::abs(den)<1e-8*dd){
                    // A top-view ray cannot determine height along its own axis.
                    // Use screen-up drag from the base corner, as in the reference.
                    if(std::abs(dn)<1e-12)throw std::runtime_error("Cannot project Height; switch view or enter a value");
                    const auto hit=a+d*((origin-a).Dot(n)/dn);SbVec3f up;
                    viewer->getCamera()->orientation.getValue().multVec(SbVec3f(0,1,0),up);
                    const double h[3]={hit.x,hit.y,hit.z},u[3]={up[0],up[1],up[2]};double result[3];
                    if(!om9_solid_height_drag(result,h,u))throw std::runtime_error("Cannot drag Height in this view");
                    p=Base::Vector3d(result[0],result[1],result[2]);
                }else{const double height=(dd*delta.Dot(n)-dn*delta.Dot(d))/den;p=origin+n*height;}
            }else{
                double special[6];
                if(om9_solid_pick_plane(special)){const Base::Vector3d origin(special[0],special[1],special[2]),normal(special[3],special[4],special[5]);if(std::abs(d.Dot(normal))<=1e-6*d.Length())throw std::runtime_error("View is parallel to construction plane; switch view or enter coordinates");p=a+d*((origin-a).Dot(normal)/d.Dot(normal));}
                else{
                const auto plane=CoreWorkspace::instance().plane(view);const auto n=plane.getRotation().multVec(Base::Vector3d(0,0,1));
                if(std::abs(d.Dot(n))<=1e-6*d.Length())throw std::runtime_error("View is parallel to CPlane; enter coordinates or switch view");
                const auto hit=viewer->getPointOnXYPlaneOfPlacement(pixel,plane);p=Base::Vector3d(hit[0],hit[1],hit[2]);
                }
            }
        }
        const bool ortho=om9_ortho_active(mouse->modifiers().testFlag(Qt::ShiftModifier));const double point[3]={p.x,p.y,p.z};
        if(hover)preview(point,ortho);else {const auto effect=om9_solid_point(p.x,p.y,p.z,ortho);recordAcceptedPointPhase(effect);CoreSnaps::acceptedPoint(effect==1||effect==2);result(effect);}
    }catch(const std::exception& error){clearPreview();if(!hover)CurveController::instance().setPrompt(QString::fromUtf8(error.what()));}
    return !hover;
}
}
