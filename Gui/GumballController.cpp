// SPDX-License-Identifier: LGPL-2.1-or-later
#include "GumballController.h"
#include "GumballFrame.h"
#include "RustBridge.h"
#include "CoreLayers.h"
#include "CoreSnaps.h"
#include "CoreUnits.h"
#include "CurveController.h"
#include "SurfaceController.h"
#include "SolidController.h"
#include "EditController.h"
#include "HistoryController.h"
#include "CageController.h"
#include "CoreDistance.h"
#include "CorePictureFrame.h"
#include "CoreViewControls.h"
#include <Python.h>
#include <App/Application.h>
#include <App/Document.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/MainWindow.h>
#include <Gui/Selection/Selection.h>
#include <Gui/Selection/SelectionObject.h>
#include <QApplication>
#include <QTimer>
#include <QMenu>
#include <QCursor>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QJsonDocument>
#include <algorithm>
#include <limits>
namespace OpenMatrix9Gui {
namespace {
auto prefs(){return App::GetApplication().GetParameterGroupByPath("User parameter:BaseApp/Preferences/Mod/OpenMatrix9/Gumball");}
bool parse(const QString& text,unsigned& kind,unsigned& value){const auto raw=text.toUtf8();return om9_gumball_command(reinterpret_cast<const unsigned char*>(raw.constData()),raw.size(),&kind,&value)==0;}
bool foreignActive(){return om9_curve_active()||SurfaceController::instance().active()||SolidController::instance().active()||EditController::instance().active()||HistoryController::instance().active()||CageController::instance().active()||CoreDistance::instance().active()||CorePictureFrame::instance().active()||CoreViewControls::instance().toolActive();}
QString keyOf(const std::vector<Gui::SelectionObject>& selection){QStringList keys;for(const auto& s:selection){QString row=QString::fromUtf8(s.getDocName())+"/"+QString::fromUtf8(s.getFeatName());for(const auto& sub:s.getSubNames())row+="/"+QString::fromStdString(sub);keys<<row;}keys.sort();return keys.join('|');}
PyObject* stateAPI(PyObject*,PyObject*){try{auto json=QJsonDocument(GumballController::instance().state()).toJson(QJsonDocument::Compact);PyObject* module=PyImport_ImportModule("json");if(!module)return nullptr;auto* value=PyObject_CallMethod(module,"loads","s",json.constData());Py_DECREF(module);return value;}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}}
PyObject* handlesAPI(PyObject*,PyObject*){auto json=QJsonDocument(GumballController::instance().handlePositions()).toJson(QJsonDocument::Compact);PyObject* module=PyImport_ImportModule("json");if(!module)return nullptr;auto* value=PyObject_CallMethod(module,"loads","s",json.constData());Py_DECREF(module);return value;}
}
GumballController& GumballController::instance(){static auto* controller=new GumballController;return *controller;}
GumballController::GumballController():QObject(qApp),session(om9_gumball_create()){
    auto* timer=new QTimer(this);timer->setInterval(100);connect(timer,&QTimer::timeout,this,[this]{if(workbench)refresh();});timer->start();
    deleteConnection=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(documentName==QString::fromUtf8(doc.getName())){cancel();inputs.reset();view=nullptr;documentName.clear();selectionKey.clear();}});
    activeConnection=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(documentName!=QString::fromUtf8(doc.getName())){cancel();inputs.reset();view=nullptr;selectionKey.clear();}});
}
Om9GumballSnapshot GumballController::snapshot()const{Om9GumballSnapshot s{};om9_gumball_snapshot(session,&s);return s;}
bool GumballController::event(const Om9GumballEvent& e){if(om9_gumball_event(session,&e)){message("Gumball: invalid input for the current phase");return false;}return true;}
void GumballController::message(const QString& text){error=text;CurveController::instance().setPrompt(text);CurveController::instance().logMessage(text);}
void GumballController::persist(){auto s=snapshot();prefs()->SetBool("Enabled",s.enabled!=0);prefs()->SetInt("Alignment",s.alignment);}
void GumballController::activate(){workbench=true;Om9GumballEvent enable{};enable.pressed=prefs()->GetBool("Enabled",false);event(enable);Om9GumballEvent align{};align.kind=1;align.key=unsigned(std::clamp(long(prefs()->GetInt("Alignment",0)),0L,2L));event(align);refresh();}
void GumballController::deactivate(){cancel();workbench=false;inputs.reset();view=nullptr;selectionKey.clear();}
bool GumballController::active()const{const auto phase=snapshot().phase;return phase>=2&&phase<=5;}
void GumballController::cancel(){Om9GumballEvent e{};e.kind=7;om9_gumball_event(session,&e);relocateInputs.clear();}
bool GumballController::handles(std::size_t index){auto* name=om9_command_id(index);if(!name)return false;unsigned kind=0,value=0;return parse(QString::fromUtf8(name),kind,value);}
bool GumballController::matches(std::size_t index,const QString& name){unsigned a=0,av=0,b=0,bv=0;auto* id=om9_command_id(index);return id&&parse(QString::fromUtf8(id),a,av)&&parse(name,b,bv)&&a==b;}
bool GumballController::checked(std::size_t index){unsigned kind=0,value=0;auto* name=om9_command_id(index);return name&&parse(QString::fromUtf8(name),kind,value)&&kind==1&&instance().snapshot().enabled;}
bool GumballController::available(std::size_t index)const{
    if(!workbench||!handles(index)||!Gui::Application::Instance||Gui::Application::Instance->isClosing())return false;auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    if(!doc||!gui||gui->getInEdit()||!Gui::Control().isAllowedAlterView(doc))return false;unsigned kind=0,value=0;parse(QString::fromUtf8(om9_command_id(index)),kind,value);
    return kind!=3||(snapshot().enabled&&Gui::Selection().hasSelection(doc->getName())&&CoreLayers::selectionEditable());
}
bool GumballController::execute(std::size_t index){if(!available(index))return false;unsigned kind=0,value=0;parse(QString::fromUtf8(om9_command_id(index)),kind,value);applyCommand(kind,value);return false;}
void GumballController::refresh(){
    if(!workbench||!snapshot().enabled)return;
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();auto* current=gui?dynamic_cast<Gui::View3DInventor*>(gui->getActiveView()):nullptr;
    if(!doc||!current||gui->getInEdit()||foreignActive()){if(active())cancel();return;}
    const auto selected=Gui::Selection().getSelectionEx(doc->getName(),App::DocumentObject::getClassTypeId(),Gui::ResolveMode::NoResolve);const auto key=keyOf(selected);
    const bool changed=documentName!=QString::fromUtf8(doc->getName())||selectionKey!=key||view!=current;
    if(active()){
        if(changed){cancel();inputs.reset();relocated=false;}else{try{if(inputs)verifyGumballInputs(*doc,*inputs);}catch(const std::exception& e){cancel();message(QString::fromUtf8(e.what()));inputs.reset();}return;}
    }
    if(!changed&&inputs){try{verifyGumballInputs(*doc,*inputs);return;}catch(...){inputs.reset();}}
    if(changed)relocated=false;documentName=QString::fromUtf8(doc->getName());selectionKey=key;view=current;inputs.reset();
    if(selected.empty())return;
    try{
        auto captured=captureGumballInputs(*doc,selected,captureCoordinateContext(*doc,current));auto frame=captureGumballFrame(captured,current,snapshot().alignment);
        if(!frame.unresolved.isEmpty()){error=frame.unresolved;return;}
        if(!relocated&&om9_gumball_frame(session,frame.frame.data(),captured.coordinates.frame.data()))throw std::runtime_error("Cannot observe native gumball frame");
        inputs=std::move(captured);error.clear();
    }catch(const std::exception& e){error=QString::fromUtf8(e.what());}
}
bool GumballController::applyCommand(unsigned kind,unsigned value){
    if(kind==1){cancel();const bool enabled=value==std::numeric_limits<unsigned>::max()||value==2?!snapshot().enabled:value==1;Om9GumballEvent e{};e.pressed=enabled;event(e);inputs.reset();relocated=false;persist();refresh();message(enabled?"Gumball On":"Gumball Off");return true;}
    if(kind==2){if(value==std::numeric_limits<unsigned>::max()){alignmentMenu();return true;}cancel();if(inputs){auto frame=captureGumballFrame(*inputs,view,value);if(!frame.unresolved.isEmpty()){message(frame.unresolved);return true;}if(om9_gumball_frame(session,frame.frame.data(),inputs->coordinates.frame.data()))return true;}Om9GumballEvent e{};e.kind=1;e.key=value;event(e);relocated=false;inputs.reset();persist();refresh();return true;}
    if(kind==3){CurveController::instance().cancel();refresh();if(!inputs){message("RelocateGumball: select an editable object and enable Gumball");return true;}Om9GumballEvent e{};e.kind=8;if(event(e)){relocateInputs.clear();message("RelocateGumball: new origin");}return true;}
    return false;
}
void GumballController::alignmentMenu(){if(popup)delete popup.data();popup=new QMenu(Gui::getMainWindow());popup->setObjectName("OM9GumballAlignmentMenu");const char* labels[]={"CPlane","Object","World"};for(unsigned i=0;i<3;++i){auto* action=popup->addAction(labels[i]);action->setCheckable(true);action->setChecked(snapshot().alignment==i);if(i==1&&inputs){const auto frame=captureGumballFrame(*inputs,view,i);if(!frame.unresolved.isEmpty()){action->setEnabled(false);action->setToolTip(frame.unresolved);}}connect(action,&QAction::triggered,this,[this,i]{applyCommand(2,i);});}popup->popup(QCursor::pos());}
bool GumballController::submit(const QString& text){
    unsigned kind=0,value=0;const auto raw=text.toUtf8();const auto status=om9_gumball_command(reinterpret_cast<const unsigned char*>(raw.constData()),raw.size(),&kind,&value);
    if(status==0)return applyCommand(kind,value);if(status==2){message("Gumball: expected On/Off/Toggle or CPlane/Object/World");return true;}
    if(!active())return false;
    // A declared foreign command ends this interaction, then uses its own router.
    for(std::size_t i=0;i<om9_command_count();++i)if(text.compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0||EditController::matches(i,text)||SurfaceController::matches(i,text)||SolidController::matches(i,text)){cancel();return false;}
    try{
        auto* doc=App::GetApplication().getActiveDocument();if(!doc||!inputs)throw std::runtime_error("Gumball document is missing");verifyGumballInputs(*doc,*inputs);
        if(snapshot().phase==4){Om9GumballEvent e{};if(text.trimmed().isEmpty())e.kind=4;else{auto point=parseCoordinateInput(inputs->coordinates,text);e.kind=9;e.world_point[0]=point.point.x;e.world_point[1]=point.point.y;e.world_point[2]=point.point.z;if(!event(e))return true;relocateInputs.push_back(std::move(point));if(snapshot().phase==1){relocated=true;message("Gumball relocated");}else message(relocateInputs.size()==1?"RelocateGumball: X direction":"RelocateGumball: Y direction or Enter");return true;}if(event(e)){relocated=true;message("Gumball relocated");}return true;}
        if(text.trimmed().isEmpty()){Om9GumballEvent e{};e.kind=4;event(e);}else if(om9_gumball_input(session,reinterpret_cast<const unsigned char*>(raw.constData()),raw.size()))message("Invalid gumball value or dimension");return true;
    }catch(const std::exception& e){message(QString::fromUtf8(e.what()));return true;}
}
bool GumballController::processEvent(QObject* receiver,QEvent* e){
    if(!workbench||!active())return false;
    if(e->type()==QEvent::KeyPress&&static_cast<QKeyEvent*>(e)->key()==Qt::Key_Escape){cancel();return true;}
    if(snapshot().phase==4&&view&&e->type()==QEvent::MouseButtonPress){auto* widget=qobject_cast<QWidget*>(receiver);auto* m=static_cast<QMouseEvent*>(e);if(m->button()==Qt::LeftButton&&widget&&(widget==view->getViewer()||view->getViewer()->isAncestorOf(widget))){Base::Vector3d p;auto pos=view->getViewer()->viewport()->mapFrom(widget,m->position().toPoint());if(CoreSnaps::pick(view,pos,p)){Om9GumballEvent point{};point.kind=9;point.world_point[0]=p.x;point.world_point[1]=p.y;point.world_point[2]=p.z;event(point);if(snapshot().phase==1)relocated=true;}return true;}}
    return false;
}
QJsonObject GumballController::state(){refresh();auto s=snapshot();QJsonArray frame;for(double v:s.frame)frame.append(v);return {{"enabled",bool(s.enabled)},{"phase",int(s.phase)},{"alignment",int(s.alignment)},{"frame",frame},{"workbench_active",workbench},{"has_selection",bool(inputs)},{"unresolved",error}};}
QJsonArray GumballController::handlePositions()const{return {};}
void GumballController::addMethods(PyObject* module){static PyMethodDef methods[]={{"gumballState",stateAPI,METH_NOARGS,"Read gumball command/session observations."},{"gumballHandles",handlesAPI,METH_NOARGS,"Read native handle positions for pointer targeting."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
}
