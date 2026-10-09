// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CageController.h"
#include "CageFeature.h"
#include "CurveController.h"
#include "CoreKeyboard.h"
#include "EditGeometry.h"
#include "RustBridge.h"
#include <App/Document.h>
#include <App/GeoFeature.h>
#include <Base/Interpreter.h>
#include <Gui/Application.h>
#include <Gui/Control.h>
#include <Gui/Document.h>
#include <Gui/MainWindow.h>
#include <Gui/Selection/Selection.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <QApplication>
#include <QComboBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFormLayout>
#include <QKeyEvent>
#include <QLabel>
#include <QLineEdit>
#include <QPushButton>
#include <QTimer>
#include <QVBoxLayout>
#include <array>
#include <sstream>
#include <stdexcept>
extern "C" {
unsigned om9_cage_command_kind(const char*);
const char* om9_cage_command_caption(unsigned);
bool om9_cage_command_available(bool,bool,bool,bool);
void* om9_cage_session_new(unsigned);
void om9_cage_session_free(void*);
int om9_cage_session_add(void*,const char*,const char*,const double*,const double*,unsigned);
int om9_cage_session_advance(void*);
int om9_cage_session_prepare(void*);
int om9_cage_session_option(void*,const char*,const char*);
int om9_cage_session_verify(void*,const char*,const char*);
const char* om9_cage_session_error(const void*);
unsigned om9_cage_session_phase(const void*);
std::size_t om9_cage_session_count(const void*);
const char* om9_cage_session_name(const void*,std::size_t);
const char* om9_cage_session_control(const void*);
unsigned om9_cage_session_values(const void*,double*);
}
namespace OpenMatrix9Gui {
namespace {
// API error text is preserved; no command strings are ever evaluated.
std::string pythonError(){PyObject *type=nullptr,*value=nullptr,*trace=nullptr;PyErr_Fetch(&type,&value,&trace);PyErr_NormalizeException(&type,&value,&trace);PyObject* text=value?PyObject_Str(value):nullptr;const char* p=text?PyUnicode_AsUTF8(text):nullptr;std::string message=p?p:"Native Cage API failed";Py_XDECREF(text);Py_XDECREF(type);Py_XDECREF(value);Py_XDECREF(trace);PyErr_Clear();return message;}
struct Ref{PyObject* value;explicit Ref(PyObject* v):value(v){if(!v)throw std::runtime_error(pythonError());}~Ref(){Py_DECREF(value);}Ref(const Ref&)=delete;};
std::string text(PyObject* v){const char* p=PyUnicode_AsUTF8(v);if(!p)throw std::runtime_error(pythonError());return p;}
double number(PyObject* p,const char* key){Ref v(PyObject_GetAttrString(p,key));double d=PyFloat_AsDouble(v.value);if(PyErr_Occurred())throw std::runtime_error(pythonError());return d;}
bool nativeControl(App::DocumentObject* o){return dynamic_cast<CageControl*>(o)!=nullptr;}
bool retainedControl(App::DocumentObject* o){auto* p=o?o->getPropertyByName<App::PropertyString>("OM9SourceClass"):nullptr;return p&&(p->getStrValue()=="ON_MorphControl"||p->getStrValue()=="ON_NurbsCage");}
struct Snapshot{std::string signature;std::array<double,3> lo{},hi{};};
Snapshot snapshot(App::DocumentObject* o,bool retained=false){
    if(!o||!o->getDocument()||!o->getDocument()->containsObject(o))throw std::runtime_error("Cage input was deleted; cancel and reselect");
    Ref object(o->getPyObject());Snapshot s;std::ostringstream frame;frame<<o->getID()<<':'<<std::hexfloat;
    auto global=App::GeoFeature::getGlobalPlacement(o).toMatrix();for(int r=0;r<4;++r)for(int c=0;c<4;++c)frame<<';'<<global[r][c];
    if(retained){for(const char* key:{"OM9SourceUUID","OM9SourceClass","OM9ImportNamespace","OM9Capability"}){Ref p(PyObject_GetAttrString(object.value,key)),v(PyObject_Repr(p.value));frame<<'|'<<text(v.value);}s.signature=frame.str();return s;}
    if(PyObject_HasAttrString(object.value,"Mesh")){
        Ref original(PyObject_GetAttrString(object.value,"Mesh")),copy(PyObject_CallMethod(original.value,"copy",nullptr));
        s.signature=editMeshSignature(original.value)+frame.str();Ref placement(PyObject_CallMethod(object.value,"getGlobalPlacement",nullptr));
        if(PyObject_SetAttrString(copy.value,"Placement",placement.value)<0)throw std::runtime_error(pythonError());
        Ref bbox(PyObject_GetAttrString(copy.value,"BoundBox"));const char* lows[]={"XMin","YMin","ZMin"};const char* highs[]={"XMax","YMax","ZMax"};for(int a=0;a<3;++a){s.lo[a]=number(bbox.value,lows[a]);s.hi[a]=number(bbox.value,highs[a]);}
    }else{
        auto input=nativeShapeInput(*o->getDocument(),o->getNameInDocument());s.signature=input.signature+frame.str();Ref bbox(PyObject_GetAttrString(input.shape->value,"BoundBox"));const char* lows[]={"XMin","YMin","ZMin"};const char* highs[]={"XMax","YMax","ZMax"};for(int a=0;a<3;++a){s.lo[a]=number(bbox.value,lows[a]);s.hi[a]=number(bbox.value,highs[a]);}
    }
    // Control data, including rational weights and full knots, is independent
    // of its displayed lattice. Exact scalar/vector property serialization
    // catches modifications which leave the visible lattice unchanged.
    if(auto* cage=dynamic_cast<CageControl*>(o)){std::ostringstream data;data<<std::hexfloat;for(auto p:cage->ControlPoints.getValues())data<<'|'<<p.x<<','<<p.y<<','<<p.z;for(auto v:cage->Counts.getValues())data<<'|'<<v;for(auto v:cage->Degrees.getValues())data<<'|'<<v;for(auto* p:{&cage->UKnots,&cage->VKnots,&cage->WKnots,&cage->Weights})for(auto v:p->getValues())data<<'|'<<v;auto m=cage->ReferenceFrame.getValue();for(int r=0;r<4;++r)for(int c=0;c<4;++c)data<<'|'<<m[r][c];s.signature+=data.str();}
    return s;
}
}
CageController& CageController::instance(){static auto* c=new CageController;return *c;}
CageController::CageController():QObject(qApp){
    qApp->installEventFilter(this);
    deleted=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& d){if(document==&d)cancel();});
    changed=App::GetApplication().signalActiveDocument.connect([this](const App::Document& d){if(active()&&document!=&d)cancel();});
    auto* timer=new QTimer(this);timer->setInterval(150);connect(timer,&QTimer::timeout,this,[this]{if(active()&&!valid())cancel();});timer->start();
}
bool CageController::handles(std::size_t i){return om9_cage_command_kind(om9_command_id(i))!=0;}
bool CageController::matches(std::size_t i,const QString& s){return handles(i)&&om9_cage_command_kind(s.toUtf8().constData())==om9_cage_command_kind(om9_command_id(i));}
bool CageController::available(std::size_t i)const{auto* d=App::GetApplication().getActiveDocument();auto* g=Gui::Application::Instance->activeDocument();return handles(i)&&om9_cage_command_available(d!=nullptr,g!=nullptr,g&&!g->getInEdit()&&!g->isAboutToClose(),d&&Gui::Control().isAllowedAlterDocument(d));}
bool CageController::active()const{return document&&session;}
bool CageController::valid()const{auto* g=Gui::Application::Instance->activeDocument();return document&&document==App::GetApplication().getActiveDocument()&&g&&g->getDocument()==document&&!g->getInEdit()&&!g->isAboutToClose()&&Gui::Control().isAllowedAlterDocument(document);}
void CageController::prompt(const QString& s){CurveController::instance().setPrompt(s);CurveController::instance().logMessage(s);}
void CageController::check(int result)const{if(result<0)throw std::runtime_error(om9_cage_session_error(session));}
void CageController::closeDialog(){if(dialog){auto* d=dialog.data();dialog=nullptr;status=nullptr;d->hide();d->deleteLater();}}
void CageController::cancel(){closeDialog();om9_cage_session_free(session);session=nullptr;document=nullptr;kind=0;}
void CageController::add(const QString& name,unsigned role){
    auto* o=document->getObject(name.toUtf8().constData());if(role==1&&!nativeControl(o))throw std::runtime_error("Select one existing OpenMatrix9 native 3D cage; use Restore=name for retained 3DM controls");if(role==2&&!retainedControl(o))throw std::runtime_error("Restore requires an existing retained ON_MorphControl or ON_NurbsCage");
    if(role==0&&(nativeControl(o)||dynamic_cast<CageBinding*>(o)))throw std::runtime_error("Select whole captive/source geometry; controls and binding storage are separate roles");
    auto s=snapshot(o,role==2);check(om9_cage_session_add(session,name.toUtf8().constData(),s.signature.c_str(),s.lo.data(),s.hi.data(),role));
}
void CageController::selection(){for(const auto& selected:Gui::Selection().getSelection(document->getName())){if(selected.SubName&&*selected.SubName)throw std::runtime_error("Cage commands require whole objects");auto* o=document->getObject(selected.FeatName);if(kind==2&&nativeControl(o))continue;add(QString::fromUtf8(selected.FeatName));}}
bool CageController::start(std::size_t i){if(!available(i))return false;cancel();CurveController::instance().cancel();document=App::GetApplication().getActiveDocument();command=i;kind=om9_cage_command_kind(om9_command_id(i));session=om9_cage_session_new(kind);
    try{Base::PyGILStateLocker lock;selection();prompt(QString::fromUtf8(om9_cage_command_caption(kind))+": select whole "+(kind==1?"source objects for World BoundingBox":"captives")+" or type object names; Enter / Cancel");}catch(const std::exception& e){prompt(QString::fromUtf8(om9_cage_command_caption(kind))+": "+e.what());cancel();}return false;
}
void CageController::chooseControl(){
    closeDialog();dialog=new QDialog(Gui::getMainWindow());dialog->setObjectName("OM9CageChooseControl");dialog->setWindowTitle("Cage Edit — choose existing control");auto* layout=new QVBoxLayout(dialog);auto* role=new QLabel("Captives selected. Choose one existing native 3D cage, or explicitly restore a retained 3DM cage and its archived relationships.",dialog);role->setWordWrap(true);layout->addWidget(role);auto* controls=new QComboBox(dialog);controls->setObjectName("OM9CageControls");for(auto* o:document->getObjects())if(nativeControl(o)||retainedControl(o))controls->addItem(QString::fromUtf8(o->getNameInDocument())+(nativeControl(o)?" — native 3D cage":" — retained 3DM control (Restore)"),QString::fromUtf8(o->getNameInDocument()));layout->addWidget(controls);
    auto* buttons=new QDialogButtonBox(QDialogButtonBox::Ok|QDialogButtonBox::Cancel,dialog);layout->addWidget(buttons);connect(buttons,&QDialogButtonBox::accepted,this,[this,controls]{if(!active()||!valid()){cancel();return;}try{Base::PyGILStateLocker lock;auto name=controls->currentData().toString();auto* o=document->getObject(name.toUtf8().constData());add(name,nativeControl(o)?1:2);options();}catch(const std::exception& e){prompt(QString("CageEdit: ")+e.what());}});connect(buttons,&QDialogButtonBox::rejected,this,[this]{cancel();prompt("Command:");});connect(dialog,&QDialog::rejected,this,[this]{cancel();prompt("Command:");});dialog->show();prompt("CageEdit: choose existing native cage by name / Restore=retainedName / Cancel");
}
void CageController::options(){
    closeDialog();dialog=new QDialog(Gui::getMainWindow());dialog->setObjectName("OM9CageOptions");dialog->setWindowTitle(QString::fromUtf8(om9_cage_command_caption(kind)));auto* layout=new QVBoxLayout(dialog);auto* form=new QFormLayout;layout->addLayout(form);double values[13];unsigned flags=om9_cage_session_values(session,values);
    QStringList names;for(std::size_t i=0;i<om9_cage_session_count(session);++i)names<<QString::fromUtf8(om9_cage_session_name(session,i));auto* roles=new QLabel((kind==1?"BoundingBox sources: ":"Captives: ")+names.join(", "),dialog);roles->setWordWrap(true);layout->addWidget(roles);
    if(kind==1){form->addRow("Coordinates",new QLabel("World",dialog));form->addRow("Shape",new QLabel("BoundingBox — positive X/Y/Z extents required",dialog));const char* fields[]={"UCount","VCount","WCount","UDegree","VDegree","WDegree"};for(int i=0;i<6;++i){auto* edit=new QLineEdit(QString::number(values[i+6],'g',15),dialog);edit->setObjectName(QString("OM9Cage")+fields[i]);form->addRow(fields[i],edit);}}
    else{auto* control=om9_cage_session_control(session);form->addRow("Control",new QLabel(QString::fromUtf8(control?control:""),dialog));if(flags&2){auto* restore=new QLabel("Explicit Restore: the verified archive's own captive relationships will be restored. The preceding captive selection is not added to that archive record.",dialog);restore->setWordWrap(true);layout->addWidget(restore);}else{auto* region=new QComboBox(dialog);region->setObjectName("OM9CageRegion");region->addItems({"Global","Local"});region->setCurrentIndex(flags&1?1:0);form->addRow("Region",region);auto* falloff=new QLineEdit(QString::number(values[12],'g',15),dialog);falloff->setObjectName("OM9CageFalloff");form->addRow("Falloff (mm)",falloff);auto* limits=new QLabel("Local uses the current cage world bounding box. Only existing native 3D cages are supported.",dialog);limits->setWordWrap(true);layout->addWidget(limits);}}
    status=new QLabel("OK commits the native Cage service. Cancel creates no document objects.",dialog);status->setObjectName("OM9CageStatus");status->setWordWrap(true);layout->addWidget(status);auto* buttons=new QDialogButtonBox(QDialogButtonBox::Ok|QDialogButtonBox::Cancel,dialog);layout->addWidget(buttons);connect(buttons,&QDialogButtonBox::accepted,this,[this]{commit();});connect(buttons,&QDialogButtonBox::rejected,this,[this]{cancel();prompt("Command:");});connect(dialog,&QDialog::rejected,this,[this]{cancel();prompt("Command:");});dialog->show();prompt(QString::fromUtf8(om9_cage_command_caption(kind))+": supported options shown; OK / Cancel");
}
void CageController::syncOptions(){if(!dialog)return;const char* fields[]={"UCount","VCount","WCount","UDegree","VDegree","WDegree"};double values[13];om9_cage_session_values(session,values);for(int i=0;i<6;++i)if(auto* edit=dialog->findChild<QLineEdit*>(QString("OM9Cage")+fields[i]))edit->setText(QString::number(values[i+6],'g',15));if(auto* edit=dialog->findChild<QLineEdit*>("OM9CageFalloff"))edit->setText(QString::number(values[12],'g',15));if(auto* combo=dialog->findChild<QComboBox*>("OM9CageRegion")){double data[13];combo->setCurrentIndex(om9_cage_session_values(session,data)&1?1:0);}}
void CageController::commit(){
    if(!active()||!valid()){cancel();return;}try{Base::PyGILStateLocker lock;
        if(om9_cage_session_phase(session)==3&&dialog){if(kind==1){const char* fields[]={"UCount","VCount","WCount","UDegree","VDegree","WDegree"};QStringList values;for(const char* key:fields)values<<dialog->findChild<QLineEdit*>(QString("OM9Cage")+key)->text().trimmed();check(om9_cage_session_option(session,"Parameters",values.join(',').toUtf8().constData()));}if(auto* edit=dialog->findChild<QLineEdit*>("OM9CageFalloff"))check(om9_cage_session_option(session,"Falloff",edit->text().trimmed().toUtf8().constData()));if(auto* region=dialog->findChild<QComboBox*>("OM9CageRegion"))check(om9_cage_session_option(session,"Region",region->currentText().toUtf8().constData()));}
        double values[13];unsigned flags=om9_cage_session_values(session,values);Ref captives(PyList_New(0));for(std::size_t i=0;i<om9_cage_session_count(session);++i){const char* name=om9_cage_session_name(session,i);auto* o=document->getObject(name);auto current=snapshot(o);check(om9_cage_session_verify(session,name,current.signature.c_str()));Ref object(o->getPyObject());if(PyList_Append(captives.value,object.value)<0)throw std::runtime_error(pythonError());}
        const char* controlName=om9_cage_session_control(session);App::DocumentObject* control=controlName?document->getObject(controlName):nullptr;if(controlName){auto current=snapshot(control,flags&2);check(om9_cage_session_verify(session,controlName,current.signature.c_str()));}
        check(om9_cage_session_prepare(session));Ref api(PyImport_ImportModule("OpenMatrix9Gui"));
        if(kind==1){Ref lo(Py_BuildValue("(ddd)",values[0],values[1],values[2])),hi(Py_BuildValue("(ddd)",values[3],values[4],values[5])),counts(Py_BuildValue("(ddd)",values[6],values[7],values[8])),degrees(Py_BuildValue("(ddd)",values[9],values[10],values[11]));Ref result(PyObject_CallMethod(api.value,"createCage","sOOOO",document->getName(),lo.value,hi.value,counts.value,degrees.value));}
        else if(kind==2){if(!control)throw std::runtime_error("Control was deleted; cancel and reselect");Ref object(control->getPyObject());if(flags&2){Ref result(PyObject_CallMethod(api.value,"restore3dmCage","O",object.value));}else{Ref result(PyObject_CallMethod(api.value,"captureCage","OOsd",object.value,captives.value,flags&1?"Local":"Global",values[12]));}}
        else{Ref result(PyObject_CallMethod(api.value,"releaseFromCage","O",captives.value));}
        check(om9_cage_session_advance(session));om9_sidebar_record_execution(command,true);QString caption=QString::fromUtf8(om9_cage_command_caption(kind));cancel();prompt(caption+" completed. Command:");
    }catch(const std::exception& e){if(status)status->setText(e.what());prompt(QString::fromUtf8(om9_cage_command_caption(kind))+": "+e.what());}
}
void CageController::submit(const QString& value){
    if(!active())return;if(!valid()){cancel();return;}auto s=value.trimmed();if(s.compare("Cancel",Qt::CaseInsensitive)==0||s.compare("Esc",Qt::CaseInsensitive)==0){cancel();prompt("Command:");return;}
    try{Base::PyGILStateLocker lock;auto phase=om9_cage_session_phase(session);const int equal=s.indexOf('=');if(equal>0){auto key=s.left(equal).trimmed();auto val=s.mid(equal+1).trimmed();if(phase==2&&key.compare("Restore",Qt::CaseInsensitive)==0){add(val,2);options();return;}check(om9_cage_session_option(session,key.toUtf8().constData(),val.toUtf8().constData()));syncOptions();return;}
        if(s.isEmpty()||s.compare("OK",Qt::CaseInsensitive)==0){if(phase==1){selection();check(om9_cage_session_advance(session));phase=om9_cage_session_phase(session);if(phase==4){commit();return;}if(phase==3){options();return;}QStringList controls;for(const auto& selected:Gui::Selection().getSelection(document->getName()))if(nativeControl(document->getObject(selected.FeatName))){if(selected.SubName&&*selected.SubName)throw std::runtime_error("Choose a whole cage control");controls<<QString::fromUtf8(selected.FeatName);}if(controls.size()==1){add(controls.front(),1);options();return;}chooseControl();if(controls.size()>1)throw std::runtime_error("Choose one control; multiple native cages were preselected");return;}if(phase==2){throw std::runtime_error("Choose one existing native cage name or Restore=retainedName");}commit();return;}
        if(phase==1){add(s);return;}if(phase==2){add(s,1);options();return;}throw std::runtime_error("Use supported options, OK or Cancel");
    }catch(const std::exception& e){if(status)status->setText(e.what());prompt(QString::fromUtf8(om9_cage_command_caption(kind))+": "+e.what());}
}
bool CageController::eventFilter(QObject* watched,QEvent* event){if(!active()||event->type()!=QEvent::KeyPress||!CoreKeyboard::inputContext(watched))return false;auto* key=static_cast<QKeyEvent*>(event);if(key->key()==Qt::Key_Escape){cancel();prompt("Command:");return true;}if(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter){auto* g=Gui::Application::Instance->activeDocument();auto* view=g?dynamic_cast<Gui::View3DInventor*>(g->getActiveView()):nullptr;auto* w=qobject_cast<QWidget*>(watched);if(view&&w&&(w==view->getViewer()||view->getViewer()->isAncestorOf(w))){submit({});return true;}}return false;}
}
