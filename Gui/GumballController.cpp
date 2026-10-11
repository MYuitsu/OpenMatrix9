// SPDX-License-Identifier: LGPL-2.1-or-later
#include "GumballController.h"
#include "GumballFrame.h"
#include "GumballOverlay.h"
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
#include <Base/Exception.h>
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
#include <QLineEdit>
#include <QJsonDocument>
#include <QScopedValueRollback>
#include <App/DocumentObjectGroup.h>
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
GumballController::GumballController():QObject(qApp),session(om9_gumball_create()),overlay(std::make_unique<GumballOverlay>()){
    auto* timer=new QTimer(this);timer->setInterval(100);connect(timer,&QTimer::timeout,this,[this]{if(workbench)refresh();});timer->start();
    deleteConnection=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(documentName==QString::fromUtf8(doc.getName())){cancel();inputs.reset();view=nullptr;documentName.clear();selectionKey.clear();}});
    activeConnection=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(documentName!=QString::fromUtf8(doc.getName())){cancel();inputs.reset();view=nullptr;selectionKey.clear();}});
}
Om9GumballSnapshot GumballController::snapshot()const{Om9GumballSnapshot s{};om9_gumball_snapshot(session,&s);return s;}
bool GumballController::event(const Om9GumballEvent& e){if(om9_gumball_event(session,&e)){message("Gumball: invalid input for the current phase");return false;}return true;}
void GumballController::message(const QString& text){error=text;CurveController::instance().setPrompt(text);CurveController::instance().logMessage(text);}
void GumballController::persist(){auto s=snapshot();prefs()->SetBool("Enabled",s.enabled!=0);prefs()->SetInt("Alignment",s.alignment);}
void GumballController::activate(){workbench=true;Om9GumballEvent enable{};enable.pressed=prefs()->GetBool("Enabled",false);event(enable);Om9GumballEvent align{};align.kind=1;align.key=unsigned(std::clamp(long(prefs()->GetInt("Alignment",0)),0L,2L));event(align);refresh();}
void GumballController::deactivate(){cancel();overlay->detach();workbench=false;inputs.reset();view=nullptr;selectionKey.clear();}
bool GumballController::active()const{const auto phase=snapshot().phase;return phase>=2&&phase<=5;}
void GumballController::releaseMouse(){if(mouseOwner){auto* owned=mouseOwner.data();mouseOwner=nullptr;if(QWidget::mouseGrabber()==owned)owned->releaseMouse();}mouseDown=false;}
void GumballController::cancel(){Om9GumballEvent e{};e.kind=7;om9_gumball_event(session,&e);relocateInputs.clear();relocatePointCount=0;if(mouseDown)suppressRelease=true;releaseMouse();overlay->closeEditor();overlay->clearPreview();if(view){view->getViewer()->viewport()->unsetCursor();overlay->render(snapshot());}}
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
    // A setter emits synchronous callbacks while source stamps are changing.
    // Read-only observations must not recapture or free that publication.
    if(publishing)return;
    if(!workbench||!snapshot().enabled){overlay->detach();return;}
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();auto* current=gui?dynamic_cast<Gui::View3DInventor*>(gui->getActiveView()):nullptr;
    if(!doc||!current||gui->getInEdit()||foreignActive()){if(active())cancel();overlay->detach();return;}
    const auto selected=Gui::Selection().getSelectionEx(doc->getName(),App::DocumentObject::getClassTypeId(),Gui::ResolveMode::NoResolve);const auto key=keyOf(selected);
    const bool changed=documentName!=QString::fromUtf8(doc->getName())||selectionKey!=key||view!=current;
    if(active()){
        if(changed){cancel();inputs.reset();relocated=false;}else{try{if(inputs)verifyGumballInputs(*doc,*inputs);updateOverlay();}catch(const std::exception& e){cancel();message(QString::fromUtf8(e.what()));inputs.reset();overlay->detach();}return;}
    }
    if(!changed&&inputs){try{verifyGumballInputs(*doc,*inputs);updateOverlay();return;}catch(...){inputs.reset();}}
    if(changed)relocated=false;documentName=QString::fromUtf8(doc->getName());selectionKey=key;view=current;inputs.reset();
    if(selected.empty()){auto state=snapshot();const auto cp=captureCoordinateContext(*doc,current);om9_gumball_owner_frame(session,nullptr,state.frame,cp.frame.data());overlay->detach();return;}
    try{
        auto captured=captureGumballInputs(*doc,selected,captureCoordinateContext(*doc,current));auto frame=captureGumballFrame(captured,current,snapshot().alignment);
        if(!frame.unresolved.isEmpty()){error=frame.unresolved;overlay->detach();return;}
        if(!relocated){
            const bool stable=selected.size()==1&&captured.roots.size()==1&&!doc->getObject(captured.roots.front().objectName.c_str())->getExtension<App::GroupExtension>();
            if(om9_gumball_owner_frame(session,stable?captured.sources.data():nullptr,frame.frame.data(),captured.coordinates.frame.data()))throw std::runtime_error("Cannot observe native gumball frame");
        }
        inputs=std::move(captured);error.clear();updateOverlay();
    }catch(const std::exception& e){error=QString::fromUtf8(e.what());overlay->detach();}
}
bool GumballController::applyCommand(unsigned kind,unsigned value){
    if(kind==1){cancel();const bool enabled=value==std::numeric_limits<unsigned>::max()||value==2?!snapshot().enabled:value==1;Om9GumballEvent e{};e.pressed=enabled;event(e);inputs.reset();relocated=false;persist();refresh();message(enabled?"Gumball On":"Gumball Off");return true;}
    if(kind==2){if(value==std::numeric_limits<unsigned>::max()){alignmentMenu();return true;}cancel();if(inputs){auto frame=captureGumballFrame(*inputs,view,value);if(!frame.unresolved.isEmpty()){message(frame.unresolved);return true;}if(om9_gumball_frame(session,frame.frame.data(),inputs->coordinates.frame.data()))return true;}Om9GumballEvent e{};e.kind=1;e.key=value;event(e);relocated=false;inputs.reset();persist();refresh();return true;}
    if(kind==3){CurveController::instance().cancel();refresh();if(!inputs){message("RelocateGumball: select an editable object and enable Gumball");return true;}Om9GumballEvent e{};e.kind=8;if(event(e)){relocateInputs.clear();relocatePointCount=0;message("RelocateGumball: new origin");}return true;}
    return false;
}
void GumballController::alignmentMenu(){if(popup)delete popup.data();popup=new QMenu(Gui::getMainWindow());popup->setObjectName("OM9GumballAlignmentMenu");const char* labels[]={"CPlane","Object","World"};for(unsigned i=0;i<3;++i){auto* action=popup->addAction(labels[i]);action->setCheckable(true);action->setChecked(snapshot().alignment==i);if(i==1&&inputs){const auto frame=captureGumballFrame(*inputs,view,i);if(!frame.unresolved.isEmpty()){action->setEnabled(false);action->setToolTip(frame.unresolved);}}connect(action,&QAction::triggered,this,[this,i]{applyCommand(2,i);});}popup->popup(QCursor::pos());}
void GumballController::widgetMenu(const QPoint& position){
    if(!view||!inputs||active())return;
    if(popup)delete popup.data();
    popup=new QMenu(Gui::getMainWindow());popup->setObjectName("OM9GumballWidgetMenu");
    auto* relocate=popup->addAction("Relocate gumball");
    connect(relocate,&QAction::triggered,this,[this]{applyCommand(3,0);});
    auto* reset=popup->addAction("Reset gumball");
    connect(reset,&QAction::triggered,this,[this]{applyCommand(2,snapshot().alignment);});
    popup->addSeparator();
    const char* labels[]={"Align to CPlane","Align to Object","Align to World"};
    for(unsigned i=0;i<3;++i){
        auto* action=popup->addAction(labels[i]);action->setCheckable(true);action->setChecked(snapshot().alignment==i);
        if(i==1){const auto frame=captureGumballFrame(*inputs,view,i);if(!frame.unresolved.isEmpty()){action->setEnabled(false);action->setToolTip(frame.unresolved);}}
        connect(action,&QAction::triggered,this,[this,i]{applyCommand(2,i);});
    }
    popup->popup(view->getViewer()->viewport()->mapToGlobal(position));
}
bool GumballController::submit(const QString& text){
    unsigned kind=0,value=0;const auto raw=text.toUtf8();const auto status=om9_gumball_command(reinterpret_cast<const unsigned char*>(raw.constData()),raw.size(),&kind,&value);
    if(status==0)return applyCommand(kind,value);if(status==2){message("Gumball: expected On/Off/Toggle or CPlane/Object/World");return true;}
    if(!active())return false;
    if(text.compare("Undo",Qt::CaseInsensitive)==0&&snapshot().phase!=4){Om9GumballEvent undo{};undo.kind=6;if(event(undo))updatePreview();return true;}
    // A declared foreign command ends this interaction, then uses its own router.
    for(std::size_t i=0;i<om9_command_count();++i)if(text.compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0||EditController::matches(i,text)||SurfaceController::matches(i,text)||SolidController::matches(i,text)){cancel();return false;}
    try{
        auto* doc=App::GetApplication().getActiveDocument();if(!doc||!inputs)throw std::runtime_error("Gumball document is missing");verifyGumballInputs(*doc,*inputs);
        if(snapshot().phase==4){Om9GumballEvent e{};if(text.trimmed().isEmpty())e.kind=4;else{auto point=parseCoordinateInput(inputs->coordinates,text);e.kind=9;e.world_point[0]=point.point.x;e.world_point[1]=point.point.y;e.world_point[2]=point.point.z;if(!event(e))return true;relocateInputs.push_back(std::move(point));++relocatePointCount;if(snapshot().phase==1){relocated=true;message("Gumball relocated");}else message(relocatePointCount==1?"RelocateGumball: X direction":"RelocateGumball: Y direction or Enter");return true;}if(event(e)){relocated=true;message("Gumball relocated");}return true;}
        if(!text.trimmed().isEmpty()&&om9_gumball_input(session,reinterpret_cast<const unsigned char*>(raw.constData()),raw.size())){message("Invalid gumball value or dimension");return true;}Om9GumballEvent e{};e.kind=4;if(event(e)){if(snapshot().phase==5)finishGesture();else{if(widgetRelocate&&snapshot().phase==1){relocated=true;overlay->closeEditor();}updatePreview();}}return true;
    }catch(const std::exception& e){message(QString::fromUtf8(e.what()));return true;}
}
void GumballController::updateOverlay(){if(view&&inputs&&snapshot().enabled&&!foreignActive()){overlay->attach(view);overlay->render(snapshot());}else overlay->detach();}
void GumballController::updatePreview(){updateOverlay();if(widgetRelocate){overlay->clearPreview();return;}try{auto* doc=App::GetApplication().getActiveDocument();if(!doc||!inputs)throw std::runtime_error("Missing gumball source");verifyGumballInputs(*doc,*inputs);auto staged=stageGumballBatch(*inputs,snapshot());overlay->preview(gumballPreviewLines(staged));}catch(const std::exception& e){overlay->clearPreview();message(QString::fromUtf8(e.what()));}catch(const Base::Exception& e){overlay->clearPreview();message(QString::fromUtf8(e.what()));}catch(...){overlay->clearPreview();message("Gumball native preview failed");}}
void GumballController::finishGesture(){
    try{
        refresh();auto* doc=App::GetApplication().getActiveDocument();
        if(!doc||!inputs||snapshot().phase!=5)throw std::runtime_error("Gumball source or selection changed before commit");
        // UI/document callbacks may clear the controller's selection. The
        // publisher retains all captured value records and private geometry.
        const auto captured=*inputs;verifyGumballInputs(*doc,captured);
        const auto plan=snapshot();auto staged=stageGumballBatch(captured,plan);
        std::size_t needed=0;
        if(om9_gumball_report(session,nullptr,0,&needed)||needed>4*1024*1024)throw std::runtime_error("Cannot read Rust gumball provenance");
        staged.report.resize(needed);
        if(om9_gumball_report(session,reinterpret_cast<unsigned char*>(staged.report.data()),needed,&needed))throw std::runtime_error("Cannot copy Rust gumball provenance");
        staged.beforeCommit=[this,generation=plan.generation]{
            const auto current=snapshot();
            if(!workbench||current.phase!=5||current.generation!=generation)throw std::runtime_error("Gumball interaction cancelled during publication");
        };
        {
            QScopedValueRollback<bool> committing(publishing,true);
            publishGumballBatch(*doc,captured,plan,std::move(staged));
        }
        Om9GumballEvent done{};done.kind=10;
        if(!event(done))throw std::runtime_error("Gumball commit acknowledgement failed");
        for(std::size_t i=0;i<om9_command_count();++i)if(QString::fromUtf8(om9_command_id(i))=="OM9_Gumball"){om9_sidebar_record_execution(i,true);break;}
        overlay->closeEditor();overlay->clearPreview();inputs.reset();selectionKey.clear();relocated=false;refresh();message("Gumball transform committed");
    }catch(const std::exception& e){cancel();message(QString::fromUtf8(e.what()));}
    catch(const Base::Exception& e){cancel();message(QString::fromUtf8(e.what()));}
    catch(...){cancel();message("Gumball native commit failed; transaction aborted");}
}
bool GumballController::processEvent(QObject* receiver,QEvent* e){
    if(!workbench)return false;
    auto* widget=qobject_cast<QWidget*>(receiver);
    const bool viewport=view&&widget&&(widget==view->getViewer()||view->getViewer()->isAncestorOf(widget));
    const bool ownEditor=overlay->editor()&&receiver==overlay->editor();
    // A menu popup can own the release of the viewport press that opened it.
    if(e->type()==QEvent::MouseButtonRelease&&suppressRelease&&(viewport||receiver==popup.data())&&static_cast<QMouseEvent*>(e)->button()==Qt::LeftButton){suppressRelease=false;if(viewport)return true;}
    if(active()&&(e->type()==QEvent::ApplicationDeactivate||(e->type()==QEvent::UngrabMouse&&mouseOwner&&receiver==mouseOwner))){cancel();return false;}
    const bool keyboardOwner=viewport||ownEditor||receiver->objectName()=="OM9CommandTranscript";
    if(active()&&keyboardOwner&&(e->type()==QEvent::KeyPress||e->type()==QEvent::KeyRelease||e->type()==QEvent::ShortcutOverride)){
        auto* key=static_cast<QKeyEvent*>(e);if(QApplication::activeModalWidget()||QApplication::activePopupWidget())return false;
        if(e->type()==QEvent::ShortcutOverride){if(ownEditor&&(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter||key->key()==Qt::Key_Escape)){e->accept();return true;}return false;}
        if(key->key()==Qt::Key_Escape&&e->type()==QEvent::KeyPress){if(ownEditor){overlay->closeEditor();if(view)view->getViewer()->viewport()->setFocus();}else cancel();return true;}
        if(!ownEditor&&e->type()==QEvent::KeyPress&&key->key()==Qt::Key_Z&&(key->modifiers()&Qt::ControlModifier)&&snapshot().phase!=4){Om9GumballEvent undo{};undo.kind=6;if(event(undo))updatePreview();return true;}
        unsigned modifier=key->key()==Qt::Key_Shift?1:key->key()==Qt::Key_Control?2:key->key()==Qt::Key_Alt?4:0;
        if(modifier&&snapshot().phase>=2&&snapshot().phase<=3){Om9GumballEvent change{};change.kind=3;change.key=modifier;change.pressed=e->type()==QEvent::KeyPress;change.auto_repeat=key->isAutoRepeat();event(change);if(mouseOwner)mouseOwner->setCursor(snapshot().copy?Qt::DragCopyCursor:Qt::CrossCursor);updatePreview();return !ownEditor;}
        if(e->type()==QEvent::KeyPress&&(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter)&&(ownEditor||viewport)){submit(ownEditor?overlay->editor()->text():QString());return true;}
    }
    if(ownEditor)return false;
    if(active()&&viewport&&(e->type()==QEvent::Wheel||(e->type()==QEvent::MouseButtonPress&&static_cast<QMouseEvent*>(e)->button()!=Qt::LeftButton))){cancel();return false;}
    if(snapshot().phase==4&&viewport&&e->type()==QEvent::MouseButtonPress){
        auto* mouse=static_cast<QMouseEvent*>(e);
        if(mouse->button()!=Qt::LeftButton)return false;
        suppressRelease=true;const auto pos=view->getViewer()->viewport()->mapFrom(widget,mouse->position().toPoint());
        Om9GumballEvent point{};Base::Vector3d snapped;
        if(CoreSnaps::pick(view,pos,snapped)){
            point.kind=9;point.world_point[0]=snapped.x;point.world_point[1]=snapped.y;point.world_point[2]=snapped.z;
        }else{
            // Native camera observations, Rust CPlane intersection/validation.
            const auto ray=overlay->ray(pos);point.kind=12;
            std::copy(ray.origin.begin(),ray.origin.end(),point.ray_origin);
            std::copy(ray.direction.begin(),ray.direction.end(),point.ray_direction);
        }
        if(event(point)){
            ++relocatePointCount;
            if(snapshot().phase==1){relocated=true;message("Gumball relocated");}
            else message(relocatePointCount==1?"RelocateGumball: X direction":"RelocateGumball: Y direction or Enter");
        }
        return true;
    }
    if(!viewport||!snapshot().enabled||foreignActive())return false;
    if(e->type()!=QEvent::MouseButtonPress&&e->type()!=QEvent::MouseMove&&e->type()!=QEvent::MouseButtonRelease)return false;
    auto* mouse=static_cast<QMouseEvent*>(e);const auto pos=view->getViewer()->viewport()->mapFrom(widget,mouse->position().toPoint());
    if(e->type()==QEvent::MouseButtonPress){
        if(mouse->button()!=Qt::LeftButton)return false;refresh();auto picked=overlay->hit(pos);if(!picked||!inputs||active())return false;
        if(picked->handle==13){suppressRelease=true;widgetMenu(pos);return true;}
        try{auto* doc=App::GetApplication().getActiveDocument();if(!doc||!Gui::Control().isAllowedAlterDocument(doc))return false;verifyGumballInputs(*doc,*inputs);Om9GumballBegin begin{};begin.document=inputs->coordinates.documentOwner;begin.view_generation=std::max(std::uint64_t(1),inputs->coordinates.frameRevision);begin.unit_generation=1;begin.mm_per_unit=unitInputScale(*doc);auto state=snapshot();std::copy(std::begin(state.frame),std::end(state.frame),begin.frame);std::copy(inputs->coordinates.frame.begin(),inputs->coordinates.frame.end(),begin.cplane);std::copy(picked->origin.begin(),picked->origin.end(),begin.ray_origin);std::copy(picked->direction.begin(),picked->direction.end(),begin.ray_direction);begin.handle=picked->handle;begin.modifiers=(mouse->modifiers()&Qt::ShiftModifier?1:0)|(mouse->modifiers()&Qt::ControlModifier?2:0)|(mouse->modifiers()&Qt::AltModifier?4:0);if(om9_gumball_begin(session,&begin,inputs->sources.data(),inputs->sources.size()))throw std::runtime_error("Cannot begin gumball gesture");mouseOwner=view->getViewer()->viewport();mouseDown=true;moved=false;widgetRelocate=begin.modifiers&2;pressPosition=pos;mouseOwner->grabMouse();mouseOwner->setCursor(snapshot().copy?Qt::DragCopyCursor:Qt::CrossCursor);message("Gumball: drag or click to enter a value");return true;}catch(const std::exception& ex){cancel();message(QString::fromUtf8(ex.what()));return true;}
    }
    if(!mouseDown)return false;
    if(e->type()==QEvent::MouseMove){if((pos-pressPosition).manhattanLength()<3&&!moved)return true;moved=true;auto ray=overlay->ray(pos);Om9GumballEvent motion{};motion.kind=2;std::copy(ray.origin.begin(),ray.origin.end(),motion.ray_origin);std::copy(ray.direction.begin(),ray.direction.end(),motion.ray_direction);if(event(motion))updatePreview();return true;}
    if(mouse->button()!=Qt::LeftButton)return true;
    releaseMouse();if(view)view->getViewer()->viewport()->unsetCursor();
    if(!moved){auto pick=overlay->ray(pos);pick.handle=snapshot().handle;if(widgetRelocate&&pick.handle>=10)pick.handle-=10;auto* editor=overlay->numericEditor(pick,snapshot().copy);if(editor)editor->setProperty("om9GumballEditor",true);return true;}
    // The release event is authoritative even if a Ctrl key-up was delivered
    // to another focused widget while this viewport owned the mouse grab.
    Om9GumballEvent control{};control.kind=3;control.key=2;control.pressed=bool(mouse->modifiers()&Qt::ControlModifier);event(control);
    Om9GumballEvent release{};release.kind=5;if(event(release)){if(snapshot().phase==5)finishGesture();else{if(widgetRelocate)relocated=true;overlay->clearPreview();updateOverlay();}}return true;
}
QJsonObject GumballController::state(){refresh();auto s=snapshot();QJsonArray frame;for(double v:s.frame)frame.append(v);return {{"enabled",bool(s.enabled)},{"phase",int(s.phase)},{"alignment",int(s.alignment)},{"frame",frame},{"workbench_active",workbench},{"has_selection",bool(inputs)},{"unresolved",error},{"suspended",foreignActive()},{"overlay",overlay->diagnostics()}};}
QJsonArray GumballController::handlePositions()const{return overlay->handlePositions();}
void GumballController::addMethods(PyObject* module){static PyMethodDef methods[]={{"gumballState",stateAPI,METH_NOARGS,"Read gumball command/session observations."},{"gumballHandles",handlesAPI,METH_NOARGS,"Read native handle positions for pointer targeting."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
}
