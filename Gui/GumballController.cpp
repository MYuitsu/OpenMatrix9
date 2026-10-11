// SPDX-License-Identifier: LGPL-2.1-or-later
#include "GumballController.h"
#include "GumballFrame.h"
#include "GumballOverlay.h"
#include "GumballMeshComponents.h"
#include "GumballFrameObjects.h"
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
#include <QDialog>
#include <QFormLayout>
#include <QDoubleSpinBox>
#include <QDialogButtonBox>
#include <QColorDialog>
#include <QPushButton>
#include <QInputDialog>
#include <Gui/Navigation/NavigationStyle.h>
#include <App/DocumentObjectGroup.h>
#include <algorithm>
#include <limits>
#include <stdexcept>
namespace OpenMatrix9Gui {
namespace {
auto prefs(){return App::GetApplication().GetParameterGroupByPath("User parameter:BaseApp/Preferences/Mod/OpenMatrix9/Gumball");}
struct SettingField {const char* key;const char* label;double Om9GumballSettings::* member;};
const SettingField settingFields[]={
    {"MoveLength","Move length (px)",&Om9GumballSettings::move_length},
    {"MoveThickness","Move line width (px)",&Om9GumballSettings::move_thickness},
    {"RotateRadius","Rotate radius (px)",&Om9GumballSettings::rotate_radius},
    {"RotateThickness","Rotate line width (px)",&Om9GumballSettings::rotate_thickness},
    {"ArrowLength","Arrowhead length (px)",&Om9GumballSettings::arrow_length},
    {"ArrowWidth","Arrowhead width (px)",&Om9GumballSettings::arrow_width},
    {"ScaleSize","Scale square size (px)",&Om9GumballSettings::scale_size},
    {"OriginSize","Origin size (px)",&Om9GumballSettings::origin_size},
    {"PlaneSize","Plane square size (px)",&Om9GumballSettings::plane_size},
    {"PlaneOffset","Plane position (px)",&Om9GumballSettings::plane_offset},
    {"MenuSize","Menu ring size (px)",&Om9GumballSettings::menu_size},
    {"MenuX","Menu horizontal offset (px)",&Om9GumballSettings::menu_x},
    {"MenuY","Menu vertical offset (px)",&Om9GumballSettings::menu_y},
    {"PickRadius","Pick radius (px)",&Om9GumballSettings::pick_radius},
    {"DragStrength","Drag strength (%)",&Om9GumballSettings::strength}
};
QJsonObject settingsFacts(const Om9GumballSettings& s){
    QJsonObject result{{"snappy",bool(s.snappy)},{"orbit",bool(s.orbit)},{"strength",s.strength},{"scale_size",s.scale_size}};
    for(const auto& field:settingFields)result[field.key]=s.*field.member;
    QJsonArray colors;for(double value:s.colors)colors.append(value);result["colors"]=colors;return result;
}

bool parse(const QString& text,unsigned& kind,unsigned& value){const auto raw=text.toUtf8();return om9_gumball_command(reinterpret_cast<const unsigned char*>(raw.constData()),raw.size(),&kind,&value)==0;}
bool foreignActive(){return om9_curve_active()||SurfaceController::instance().active()||SolidController::instance().active()||EditController::instance().active()||HistoryController::instance().active()||CageController::instance().active()||CoreDistance::instance().active()||CorePictureFrame::instance().active()||CoreViewControls::instance().toolActive();}
QString keyOf(const std::vector<Gui::SelectionObject>& selection){QStringList keys;for(const auto& s:selection){QString row=QString::fromUtf8(s.getDocName())+"/"+QString::fromUtf8(s.getFeatName());for(const auto& sub:s.getSubNames())row+="/"+QString::fromStdString(sub);keys<<row;}keys.sort();return keys.join('|');}
std::vector<Gui::SelectionObject> nativeSelection(App::Document& doc){
    auto selected=Gui::Selection().getSelectionEx(doc.getName(),App::DocumentObject::getClassTypeId(),Gui::ResolveMode::NoResolve);
    // Preserve an explicitly selected whole parent alongside its child path.
    const auto count=selected.size();
    for(std::size_t i=0;i<count;++i)if(selected[i].hasSubNames()&&Gui::Selection().isSelected(doc.getName(),selected[i].getFeatName(),"",Gui::ResolveMode::NoResolve))
        selected.emplace_back(doc.getObject(selected[i].getFeatName()));
    return selected;
}
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
void GumballController::activate(){workbench=true;loadSettings();Om9GumballEvent enable{};enable.pressed=prefs()->GetBool("Enabled",false);event(enable);Om9GumballEvent align{};align.kind=1;align.key=unsigned(std::clamp(long(prefs()->GetInt("Alignment",0)),0L,2L));event(align);refresh();}
void GumballController::deactivate(){cancel();clearOverlays();workbench=false;inputs.reset();view=nullptr;selectionKey.clear();}
bool GumballController::active()const{const auto phase=snapshot().phase;return phase>=2&&phase<=5;}
void GumballController::releaseMouse(){if(mouseOwner){auto* owned=mouseOwner.data();mouseOwner=nullptr;if(QWidget::mouseGrabber()==owned)owned->releaseMouse();}mouseDown=false;}
void GumballController::clearPreviews(){for(auto& row:views)row.overlay->clearPreview();}
void GumballController::clearOverlays(){for(auto& row:views)row.overlay->detach();}
GumballOverlay* GumballController::overlayFor(Gui::View3DInventor* native)const{for(const auto& row:views)if(row.view==native)return row.overlay.get();return nullptr;}
void GumballController::syncViews(){
    auto* gui=Gui::Application::Instance?Gui::Application::Instance->activeDocument():nullptr;
    std::vector<Gui::View3DInventor*> live;
    if(gui){
#if defined(OM9_STOCK_FREECAD)
        const auto candidates=gui->getMDIViews();
#else
        const auto candidates=gui->getMDIViews(true);
#endif
        for(auto* candidate:candidates)
            if(auto* native=dynamic_cast<Gui::View3DInventor*>(candidate))live.push_back(native);
    }
    const bool lostOwner=active()&&(!view||std::find(live.begin(),live.end(),view.data())==live.end());
    if(lostOwner)cancel();
    std::erase_if(views,[&](const auto& row){return !row.view||std::find(live.begin(),live.end(),row.view.data())==live.end();});
    for(auto* native:live)if(!overlayFor(native))views.push_back({native,std::make_unique<GumballOverlay>()});
    overlay=overlayFor(view);
}
void GumballController::cancel(){
    Om9GumballEvent e{};e.kind=7;om9_gumball_event(session,&e);relocateInputs.clear();relocatePointCount=0;
    if(mouseDown)suppressRelease=true;releaseMouse();
    for(auto& row:views){row.overlay->closeEditor();row.overlay->clearPreview();if(row.view)row.view->getViewer()->viewport()->unsetCursor();}
    gestureCamera=QJsonArray();widgetRelocate=false;
}
bool GumballController::handles(std::size_t index){auto* name=om9_command_id(index);if(!name)return false;unsigned kind=0,value=0;return parse(QString::fromUtf8(name),kind,value);}
bool GumballController::matches(std::size_t index,const QString& name){unsigned a=0,av=0,b=0,bv=0;auto* id=om9_command_id(index);return id&&parse(QString::fromUtf8(id),a,av)&&parse(name,b,bv)&&a==b;}
bool GumballController::checked(std::size_t index){unsigned kind=0,value=0;auto* name=om9_command_id(index);return name&&parse(QString::fromUtf8(name),kind,value)&&kind==1&&instance().snapshot().enabled;}
bool GumballController::available(std::size_t index)const{
    if(!workbench||!handles(index)||!Gui::Application::Instance||Gui::Application::Instance->isClosing())return false;auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    if(!doc||!gui||gui->getInEdit()||!om9AlterView(doc))return false;unsigned kind=0,value=0;parse(QString::fromUtf8(om9_command_id(index)),kind,value);
    return kind!=3||(snapshot().enabled&&Gui::Selection().hasSelection(doc->getName())&&CoreLayers::selectionEditable());
}
bool GumballController::execute(std::size_t index){if(!available(index))return false;unsigned kind=0,value=0;parse(QString::fromUtf8(om9_command_id(index)),kind,value);applyCommand(kind,value);return false;}
void GumballController::refresh(){
    if(publishing)return;
    syncViews();
    if(!workbench||!snapshot().enabled){clearOverlays();return;}
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    auto* current=gui?dynamic_cast<Gui::View3DInventor*>(gui->getActiveView()):nullptr;
    if(!doc||!current||gui->getInEdit()||foreignActive()){if(active())cancel();clearOverlays();return;}
    auto selected=nativeSelection(*doc);
    const auto key=keyOf(selected);
    const bool changed=documentName!=QString::fromUtf8(doc->getName())||selectionKey!=key;
    const auto cp=captureCoordinateContext(*doc,current);
    if(active()){
        const bool navigation=view!=current||(inputs&&inputs->coordinates.frame!=cp.frame)||(!gestureCamera.isEmpty()&&overlay&&gestureCamera!=overlay->cameraState());
        if(changed||navigation){cancel();inputs.reset();if(changed)relocated=false;}
        else {try{if(inputs)verifyGumballInputs(*doc,*inputs);updateOverlay();}catch(const std::exception& e){cancel();message(QString::fromUtf8(e.what()));inputs.reset();clearOverlays();}return;}
    }
    if(inputs&&!changed){try{verifyGumballInputs(*doc,*inputs);}catch(...){inputs.reset();relocated=false;}}
    const bool contextChanged=view!=current||(inputs&&inputs->coordinates.frame!=cp.frame);
    view=current;overlay=overlayFor(current);
    if(!changed&&!contextChanged&&inputs){try{verifyGumballInputs(*doc,*inputs);updateOverlay();return;}catch(...){inputs.reset();relocated=false;}}
    if(changed)relocated=false;documentName=QString::fromUtf8(doc->getName());selectionKey=key;inputs.reset();
    if(selected.empty()){auto state=snapshot();om9_gumball_owner_frame(session,nullptr,state.frame,cp.frame.data());clearOverlays();return;}
    try{
        auto captured=captureGumballInputs(*doc,selected,cp);auto frame=captureGumballFrame(captured,current,snapshot().alignment);
        if(!frame.unresolved.isEmpty()){error=frame.unresolved;clearOverlays();return;}
        const bool stable=selected.size()==1&&captured.roots.size()==1&&captured.roots.front().subelement.empty()&&!doc->getObject(captured.roots.front().objectName.c_str())->getExtension<App::GroupExtension>();
        // Owner-frame observation restores relocated overrides. World/Object do
        // not gain a view-specific frame; idle CPlane observes the active plane.
        if(relocated&&!changed){const auto previous=snapshot();if(om9_gumball_frame(session,previous.frame,captured.coordinates.frame.data()))throw std::runtime_error("Cannot retain relocated Gumball frame");}
        else if(om9_gumball_owner_frame(session,stable?captured.sources.data():nullptr,frame.frame.data(),captured.coordinates.frame.data()))throw std::runtime_error("Cannot observe native gumball frame");
        inputs=std::move(captured);extentPoints.clear();error.clear();updateOverlay();
    }catch(const std::exception& e){error=QString::fromUtf8(e.what());clearOverlays();}
}
bool GumballController::applyCommand(unsigned kind,unsigned value){
    if(kind==1){cancel();const bool enabled=value==std::numeric_limits<unsigned>::max()||value==2?!snapshot().enabled:value==1;Om9GumballEvent e{};e.pressed=enabled;event(e);inputs.reset();relocated=false;persist();refresh();message(enabled?"Gumball On":"Gumball Off");return true;}
    if(kind==2){if(value==std::numeric_limits<unsigned>::max()){alignmentMenu();return true;}cancel();if(inputs){auto frame=captureGumballFrame(*inputs,view,value);if(!frame.unresolved.isEmpty()){message(frame.unresolved);return true;}if(om9_gumball_frame(session,frame.frame.data(),inputs->coordinates.frame.data()))return true;}Om9GumballEvent e{};e.kind=1;e.key=value;event(e);relocated=false;inputs.reset();persist();refresh();return true;}
    if(kind==3){CurveController::instance().cancel();refresh();if(!inputs){message("RelocateGumball: select an editable object and enable Gumball");return true;}Om9GumballEvent e{};e.kind=8;if(event(e)){relocateInputs.clear();relocatePointCount=0;message("RelocateGumball: new origin");}return true;}
    return false;
}
void GumballController::alignmentMenu(){if(popup)delete popup.data();popup=new QMenu(Gui::getMainWindow());popup->setObjectName("OM9GumballAlignmentMenu");const char* labels[]={"CPlane","Object","World"};for(unsigned i=0;i<3;++i){auto* action=popup->addAction(labels[i]);action->setCheckable(true);action->setChecked(snapshot().alignment==i);if(i==1&&inputs){const auto frame=captureGumballFrame(*inputs,view,i);if(!frame.unresolved.isEmpty()){action->setEnabled(false);action->setToolTip(frame.unresolved);}}connect(action,&QAction::triggered,this,[this,i]{applyCommand(2,i);});}popup->popup(QCursor::pos());}
void GumballController::loadSettings(){
    Om9GumballSettings current{};om9_gumball_settings(session,nullptr,&current);
    for(const auto& field:settingFields)current.*field.member=prefs()->GetFloat(field.key,current.*field.member);
    for(unsigned i=0;i<9;++i){auto key=QString("AxisColor%1").arg(i).toUtf8();current.colors[i]=prefs()->GetFloat(key.constData(),current.colors[i]);}
    current.snappy=prefs()->GetBool("Snappy",false);current.orbit=prefs()->GetBool("OrbitAroundPivot",false);
    if(om9_gumball_settings(session,&current,nullptr)){om9_gumball_restore_defaults(session);message("Invalid saved Gumball settings; using defaults");}
}
bool GumballController::applySettings(const Om9GumballSettings& current){
    if(om9_gumball_settings(session,&current,nullptr)){message("Gumball settings require finite values and a positive drag strength; finish the gesture first");return false;}
    for(const auto& field:settingFields)prefs()->SetFloat(field.key,current.*field.member);
    for(unsigned i=0;i<9;++i){auto key=QString("AxisColor%1").arg(i).toUtf8();prefs()->SetFloat(key.constData(),current.colors[i]);}
    prefs()->SetBool("Snappy",current.snappy!=0);prefs()->SetBool("OrbitAroundPivot",current.orbit!=0);updateOverlay();return true;
}
void GumballController::orbitAroundPivot(){
    if(!view||!inputs)return;auto* navigation=view->getViewer()->navigationStyle();if(!navigation)return;
    Om9GumballSettings current{};om9_gumball_settings(session,nullptr,&current);const auto state=snapshot();
    if(current.orbit)navigation->setRotationCenter(SbVec3f(float(state.frame[0]),float(state.frame[1]),float(state.frame[2])));
    else {
#if defined(OM9_STOCK_FREECAD)
        navigation->setRotationCenter(navigation->getFocalPoint());
#else
        navigation->setRotationCenter(view->getViewer()->getFocalPoint());
#endif
    }
}
void GumballController::settingsDialog(){
    if(active())return;
    if(auto* existing=Gui::getMainWindow()->findChild<QDialog*>("OM9GumballSettings")){existing->show();existing->raise();return;}
    auto* dialog=new QDialog(Gui::getMainWindow());dialog->setObjectName("OM9GumballSettings");dialog->setWindowTitle("Gumball settings");dialog->setAttribute(Qt::WA_DeleteOnClose);
    auto* form=new QFormLayout(dialog);auto value=std::make_shared<Om9GumballSettings>();om9_gumball_settings(session,nullptr,value.get());
    for(const auto& field:settingFields){
        auto* spin=new QDoubleSpinBox(dialog);spin->setObjectName(field.key);spin->setDecimals(3);
        spin->setRange(field.member==&Om9GumballSettings::strength?.001:field.member==&Om9GumballSettings::menu_x||field.member==&Om9GumballSettings::menu_y?-4096:0,field.member==&Om9GumballSettings::strength?1e9:4096);
        spin->setValue((*value).*field.member);form->addRow(field.label,spin);
        connect(spin,qOverload<double>(&QDoubleSpinBox::valueChanged),dialog,[value,member=field.member](double number){(*value).*member=number;});
    }
    const char* axes[]={"X axis color","Y axis color","Z axis color"};
    for(unsigned i=0;i<3;++i){
        auto* button=new QPushButton("Choose…",dialog);button->setObjectName(QString("AxisColor%1").arg(i));form->addRow(axes[i],button);
        connect(button,&QPushButton::clicked,dialog,[value,i,dialog]{auto color=QColorDialog::getColor(QColor::fromRgbF(value->colors[i*3],value->colors[i*3+1],value->colors[i*3+2]),dialog,"Axis color");if(color.isValid()){value->colors[i*3]=color.redF();value->colors[i*3+1]=color.greenF();value->colors[i*3+2]=color.blueF();}});
    }
    auto* buttons=new QDialogButtonBox(QDialogButtonBox::Ok|QDialogButtonBox::Cancel,dialog);form->addRow(buttons);
    connect(buttons,&QDialogButtonBox::accepted,dialog,[this,dialog,value]{if(applySettings(*value))dialog->accept();});
    connect(buttons,&QDialogButtonBox::rejected,dialog,&QDialog::reject);dialog->show();
}
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
    popup->addSeparator();Om9GumballSettings configuration{};om9_gumball_settings(session,nullptr,&configuration);
    for(unsigned i=0;i<2;++i){auto* action=popup->addAction(i?"Smooth dragging":"Snappy dragging");action->setCheckable(true);action->setChecked(configuration.snappy==(i?0u:1u));connect(action,&QAction::triggered,this,[this,i]{Om9GumballSettings s{};om9_gumball_settings(session,nullptr,&s);s.snappy=i?0:1;applySettings(s);});}
    auto* strength=popup->addAction("Drag strength…");connect(strength,&QAction::triggered,this,[this]{Om9GumballSettings s{};om9_gumball_settings(session,nullptr,&s);bool ok=false;const auto text=QInputDialog::getText(Gui::getMainWindow(),"Gumball drag strength","Percent (100 = normal)",QLineEdit::Normal,QString::number(s.strength,'g',12),&ok);if(ok){bool valid=false;s.strength=text.toDouble(&valid);if(!valid||!applySettings(s))message("Drag strength must be a finite positive percentage");}});
    auto* settings=popup->addAction("Settings…");connect(settings,&QAction::triggered,this,[this]{settingsDialog();});
    auto* orbit=popup->addAction("Rotate view around gumball");orbit->setCheckable(true);orbit->setChecked(configuration.orbit!=0);connect(orbit,&QAction::triggered,this,[this](bool on){Om9GumballSettings s{};om9_gumball_settings(session,nullptr,&s);s.orbit=on;if(applySettings(s))orbitAroundPivot();});
    auto* defaults=popup->addAction("Restore defaults");connect(defaults,&QAction::triggered,this,[this]{if(!om9_gumball_restore_defaults(session)){Om9GumballSettings s{};om9_gumball_settings(session,nullptr,&s);applySettings(s);orbitAroundPivot();}});
    connect(popup,&QMenu::aboutToHide,this,[this]{suppressRelease=false;});
    popup->popup(view->getViewer()->viewport()->mapToGlobal(position));
}
bool GumballController::submit(const QString& text){
    if(text.compare("GumballCamera",Qt::CaseInsensitive)==0||text.compare("GumballLight",Qt::CaseInsensitive)==0){try{cancel();const auto name=createGumballFrameObject(text.endsWith("Camera",Qt::CaseInsensitive)?"Camera":"Light");auto* doc=App::GetApplication().getActiveDocument();Gui::Selection().clearSelection();Gui::Selection().addSelection(doc->getName(),name.c_str());refresh();message("Created selectable camera/light frame");}catch(const std::exception& e){message(QString::fromUtf8(e.what()));}return true;}
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
void GumballController::updateOverlay(){
    if(!view||!inputs||!snapshot().enabled||foreignActive()){clearOverlays();return;}
    const auto state=snapshot();Om9GumballSettings settings{};if(om9_gumball_settings(session,nullptr,&settings))return;
    if(extentPoints.empty()||!std::equal(extentFrame.begin(),extentFrame.end(),state.frame)){
        extentPoints=gumballExtentPoints(*inputs,state.frame);std::copy(std::begin(state.frame),std::end(state.frame),extentFrame.begin());
    }
    if(state.phase<=1&&om9_gumball_extents(session,reinterpret_cast<const double(*)[3]>(extentPoints.data()),extentPoints.size()))throw std::runtime_error("Invalid Gumball extent observations");
    const auto component=gumballComponentHighlight(*inputs);
    for(auto& row:views)if(row.view){row.overlay->attach(row.view);row.overlay->render(state,session,settings,extentPoints);row.overlay->componentHighlight(component?&*component:nullptr);}
    overlay=overlayFor(view);
    if(settings.orbit)orbitAroundPivot();
}
void GumballController::updatePreview(){
    updateOverlay();if(widgetRelocate){clearPreviews();return;}
    try{auto* doc=App::GetApplication().getActiveDocument();if(!doc||!inputs)throw std::runtime_error("Missing gumball source");verifyGumballInputs(*doc,*inputs);auto staged=stageGumballBatch(*inputs,snapshot());const auto lines=gumballPreviewLines(staged);for(auto& row:views)row.overlay->preview(lines);}
    catch(const std::exception& e){clearPreviews();message(QString::fromUtf8(e.what()));}
    catch(const Base::Exception& e){clearPreviews();message(QString::fromUtf8(e.what()));}
    catch(...){clearPreviews();message("Gumball native preview failed");}
}
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
        std::vector<std::string> selectedOutputs;
        {
            QScopedValueRollback<bool> committing(publishing,true);
            selectedOutputs=publishGumballBatch(*doc,captured,plan,std::move(staged));
        }
        Om9GumballEvent done{};done.kind=10;
        if(!event(done))throw std::runtime_error("Gumball commit acknowledgement failed");
        {QScopedValueRollback<bool> selecting(publishing,true);Gui::Selection().clearSelection();for(const auto& name:selectedOutputs)Gui::Selection().addSelection(doc->getName(),name.c_str());}
        for(std::size_t i=0;i<om9_command_count();++i)if(QString::fromUtf8(om9_command_id(i))=="OM9_Gumball"){om9_sidebar_record_execution(i,true);break;}
        if(overlay)overlay->closeEditor();clearPreviews();inputs.reset();selectionKey.clear();relocated=false;refresh();message("Gumball transform committed");
    }catch(const std::exception& e){cancel();message(QString::fromUtf8(e.what()));}
    catch(const Base::Exception& e){cancel();message(QString::fromUtf8(e.what()));}
    catch(...){cancel();message("Gumball native commit failed; transaction aborted");}
}
bool GumballController::processEvent(QObject* receiver,QEvent* e){
    if(!workbench||publishing)return false;
    auto* widget=qobject_cast<QWidget*>(receiver);
    Gui::View3DInventor* receivingView=nullptr;
    for(const auto& row:views)if(row.view&&widget&&(widget==row.view->getViewer()||row.view->getViewer()->isAncestorOf(widget))){receivingView=row.view;break;}
    const bool ownEditor=overlay&&overlay->editor()&&receiver==overlay->editor();
    if(receivingView&&e->type()==QEvent::MouseButtonPress&&!ownEditor){
        if(active()&&receivingView!=view){cancel();}
        Gui::getMainWindow()->setActiveWindow(receivingView);
        // Resolve the source view in this event, before the native handle pick.
        if(!active()){refresh();view=receivingView;overlay=overlayFor(receivingView);}
    }
    const bool viewport=receivingView&&receivingView==view&&overlay;
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
    // Mesh component picking also works with an empty selection, when there
    // is intentionally no Gumball overlay attached to this viewport yet.
    if(receivingView&&snapshot().enabled&&!active()&&!foreignActive()&&e->type()==QEvent::MouseButtonPress){
        auto* mouse=static_cast<QMouseEvent*>(e);
        if(mouse->button()==Qt::LeftButton&&(mouse->modifiers()&Qt::ControlModifier)&&(mouse->modifiers()&Qt::ShiftModifier)){
            const auto point=receivingView->getViewer()->viewport()->mapFrom(widget,mouse->position().toPoint());
            if(!overlay||!overlay->hit(point)){
                Om9GumballSettings settings{};om9_gumball_settings(session,nullptr,&settings);
                if(const auto component=pickGumballMeshComponent(receivingView,point,settings.pick_radius)){
                    Gui::Selection().clearSelection();Gui::Selection().addSelection(component->documentName.c_str(),component->objectName.c_str(),component->subelement.c_str());refresh();suppressRelease=true;return true;
                }
            }
        }
    }
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
        if(mouse->button()!=Qt::LeftButton)return false;refresh();
        if(!overlay||!inputs||active())return false;const auto candidate=overlay->hit(pos);
        if(!candidate)return false;
        if(candidate->handle==13){suppressRelease=true;widgetMenu(pos);return true;}
        try{
            auto* doc=App::GetApplication().getActiveDocument();
            if(!doc||!om9AlterDocument(doc))return false;
            const std::string name=doc->getName(),uid=doc->Uid.getValueStr();
            const auto observed=*inputs;const auto ownerView=view;const auto camera=overlay->cameraState();
            const auto initial=snapshot();const auto selectedKey=keyOf(nativeSelection(*doc));
            bool settled=false;
            {QScopedValueRollback<bool> admitting(publishing,true);settled=settleGumballInputs(*doc,observed);}
            auto* current=App::GetApplication().getDocument(name.c_str());
            auto* gui=Gui::Application::Instance?Gui::Application::Instance->activeDocument():nullptr;
            const auto now=snapshot();
            if(current!=doc||!current||current->Uid.getValueStr()!=uid||App::GetApplication().getActiveDocument()!=current||
               !workbench||!now.enabled||now.phase!=initial.phase||now.generation!=initial.generation||active()||foreignActive()||
               !ownerView||view!=ownerView||!gui||gui->getDocument()!=current||gui->getInEdit()||gui->getActiveView()!=ownerView||
               keyOf(nativeSelection(*current))!=selectedKey||!overlay||overlay->cameraState()!=camera)
                throw std::runtime_error("Gumball document, selection or interaction changed during gesture admission");
            const auto coordinates=captureCoordinateContext(*current,ownerView);
            if(coordinates.frame!=observed.coordinates.frame||coordinates.frameRevision!=observed.coordinates.frameRevision||
               coordinates.logicalViewSlot!=observed.coordinates.logicalViewSlot)
                throw std::runtime_error("Gumball coordinate context changed during gesture admission");
            if(settled){inputs.reset();refresh();}
        }catch(const std::exception& ex){cancel();inputs.reset();clearOverlays();message(QString::fromUtf8(ex.what()));return true;}
        // The first hit only qualified a Gumball press. Native recompute can
        // replace Shape/extent observations, so pick again from the fresh frame.
        if(!overlay||!inputs||active())return false;auto picked=overlay->hit(pos);
        if(!picked||!inputs||active())return false;
        if(picked->handle==13){suppressRelease=true;widgetMenu(pos);return true;}
        try{auto* doc=App::GetApplication().getActiveDocument();if(!doc||!om9AlterDocument(doc))return false;verifyGumballInputs(*doc,*inputs);Om9GumballBegin begin{};begin.document=inputs->coordinates.documentOwner;begin.view_generation=std::max(std::uint64_t(1),inputs->coordinates.frameRevision);begin.unit_generation=1;begin.mm_per_unit=unitInputScale(*doc);auto state=snapshot();std::copy(std::begin(state.frame),std::end(state.frame),begin.frame);std::copy(inputs->coordinates.frame.begin(),inputs->coordinates.frame.end(),begin.cplane);std::copy(picked->origin.begin(),picked->origin.end(),begin.ray_origin);std::copy(picked->direction.begin(),picked->direction.end(),begin.ray_direction);begin.handle=picked->handle;begin.modifiers=(mouse->modifiers()&Qt::ShiftModifier?1:0)|(mouse->modifiers()&Qt::ControlModifier?2:0)|(mouse->modifiers()&Qt::AltModifier?4:0);if(om9_gumball_begin(session,&begin,inputs->sources.data(),inputs->sources.size()))throw std::runtime_error("Cannot begin gumball gesture");gestureCamera=overlay->cameraState();mouseOwner=view->getViewer()->viewport();mouseDown=true;moved=false;widgetRelocate=begin.modifiers&2;pressPosition=pos;mouseOwner->grabMouse();mouseOwner->setCursor(snapshot().copy?Qt::DragCopyCursor:Qt::CrossCursor);message("Gumball: drag or click to enter a value");return true;}catch(const std::exception& ex){cancel();message(QString::fromUtf8(ex.what()));return true;}
    }
    if(!mouseDown)return false;
    if(e->type()==QEvent::MouseMove){if((pos-pressPosition).manhattanLength()<3&&!moved)return true;moved=true;auto ray=overlay->ray(pos);Om9GumballEvent motion{};motion.kind=2;std::copy(ray.origin.begin(),ray.origin.end(),motion.ray_origin);std::copy(ray.direction.begin(),ray.direction.end(),motion.ray_direction);Om9GumballSettings settings{};om9_gumball_settings(session,nullptr,&settings);Base::Vector3d snapped;if(settings.snappy&&CoreSnaps::pick(view,pos,snapped)){motion.kind=13;motion.world_point[0]=snapped.x;motion.world_point[1]=snapped.y;motion.world_point[2]=snapped.z;}if(event(motion))updatePreview();return true;}
    if(mouse->button()!=Qt::LeftButton)return true;
    releaseMouse();if(view)view->getViewer()->viewport()->unsetCursor();
    if(!moved){auto pick=overlay->ray(pos);pick.handle=snapshot().handle;if(widgetRelocate&&pick.handle>=10)pick.handle-=10;auto* editor=overlay->numericEditor(pick,snapshot().copy);if(editor)editor->setProperty("om9GumballEditor",true);return true;}
    // The release event is authoritative even if a Ctrl key-up was delivered
    // to another focused widget while this viewport owned the mouse grab.
    Om9GumballEvent control{};control.kind=3;control.key=2;control.pressed=bool(mouse->modifiers()&Qt::ControlModifier);event(control);
    Om9GumballEvent release{};release.kind=5;if(event(release)){if(snapshot().phase==5)finishGesture();else{if(widgetRelocate)relocated=true;clearPreviews();updateOverlay();}}return true;
}
QJsonObject GumballController::state(){
    refresh();auto s=snapshot();QJsonArray frame,rows;for(double v:s.frame)frame.append(v);
    for(const auto& row:views)if(row.view){auto facts=row.overlay->diagnostics();facts["slot"]=row.view->property("om9ViewSlot").toInt();facts["frame"]=frame;rows.append(facts);}
    Om9GumballSettings configuration{};om9_gumball_settings(session,nullptr,&configuration);QJsonArray orbit;
    if(view&&view->getViewer()->navigationStyle()){SbBool valid=false;auto center=view->getViewer()->navigationStyle()->getRotationCenter(valid);if(valid)orbit=QJsonArray{center[0],center[1],center[2]};}
    return {{"source_slot",view?view->property("om9ViewSlot").toInt():-1},{"settings",settingsFacts(configuration)},{"orbit_center",orbit},{"enabled",bool(s.enabled)},{"phase",int(s.phase)},{"alignment",int(s.alignment)},{"frame",frame},{"workbench_active",workbench},{"has_selection",bool(inputs)},{"unresolved",error},{"suspended",foreignActive()},{"overlay",overlay?overlay->diagnostics():QJsonObject()},{"views",rows}};
}
QJsonArray GumballController::handlePositions()const{return overlay?overlay->handlePositions():QJsonArray();}
void GumballController::addMethods(PyObject* module){static PyMethodDef methods[]={{"gumballState",stateAPI,METH_NOARGS,"Read gumball command/session observations."},{"gumballHandles",handlesAPI,METH_NOARGS,"Read native handle positions for pointer targeting."},{nullptr,nullptr,0,nullptr}};PyModule_AddFunctions(module,methods);}
}
