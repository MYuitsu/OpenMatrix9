// SPDX-License-Identifier: LGPL-2.1-or-later
#include "SurfaceController.h"
#include "RustBridge.h"
#include "CurveController.h"
#include "CoreDistance.h"
#include "CorePictureFrame.h"
#include "CoreViewControls.h"
#include "CoreKeyboard.h"
#include <Base/Interpreter.h>
#include <Base/Console.h>
#include <App/Application.h>
#include <App/Document.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <Gui/MainWindow.h>
#include <Gui/Selection/Selection.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <Inventor/SoDB.h>
#include <Inventor/SoInput.h>
#include <Inventor/SoPickedPoint.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <QApplication>
#include <QDialog>
#include <QDialogButtonBox>
#include <QPushButton>
#include <QLabel>
#include <QCheckBox>
#include <QComboBox>
#include <QDoubleSpinBox>
#include <QFormLayout>
#include <QVBoxLayout>
#include <QTableWidget>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QTimer>
#include <memory>
#include <cstring>
#include <stdexcept>
namespace {
QString message(){char text[2048]={};om9_surface_message(text,sizeof(text));return QString::fromUtf8(text);}
Gui::View3DInventor* containing(QObject* object){
    auto* widget=qobject_cast<QWidget*>(object);auto* gui=Gui::Application::Instance->activeDocument();if(!widget||!gui)return nullptr;
    for(auto* mdi:gui->getMDIViews())if(auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);view&&(widget==view->getViewer()||view->getViewer()->isAncestorOf(widget)))return view;
    return nullptr;
}
struct ShapeRef { PyObject* p; explicit ShapeRef(PyObject* p):p(p){}~ShapeRef(){Py_XDECREF(p);} };
}
namespace OpenMatrix9Gui {
SurfaceController& SurfaceController::instance(){static auto* instance=new SurfaceController;return *instance;}
SurfaceController::SurfaceController():QObject(qApp){
    deleteConnection=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(document==&doc)cancel();});
    activeConnection=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(active()&&document!=&doc)cancel();});
    auto* timer=new QTimer(this);timer->setInterval(150);connect(timer,&QTimer::timeout,this,[this]{if(active()&&!valid())cancel();});timer->start();
}
bool SurfaceController::handles(std::size_t i){return om9_surface_kind(om9_command_id(i))!=0;}
bool SurfaceController::matches(std::size_t i,const QString& name){return handles(i)&&om9_surface_kind(name.toUtf8().constData())==om9_surface_kind(om9_command_id(i));}
void SurfaceController::activate(){enabled=true;qApp->installEventFilter(this);}
void SurfaceController::deactivate(){enabled=false;cancel();}
bool SurfaceController::active()const{return document&&om9_surface_phase()!=0;}
bool SurfaceController::available(std::size_t i)const{
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    return enabled&&handles(i)&&doc&&gui&&!gui->isAboutToClose()&&!gui->getInEdit()&&!dialog&&dynamic_cast<Gui::View3DInventor*>(gui->getActiveView())&&Gui::Control().isAllowedAlterDocument(doc);
}
bool SurfaceController::valid()const{
    auto* gui=Gui::Application::Instance->activeDocument();return enabled&&document==App::GetApplication().getActiveDocument()&&gui&&!gui->isAboutToClose()&&!gui->getInEdit()&&Gui::Control().isAllowedAlterDocument(document);
}
void SurfaceController::prompt(const QString& text){CurveController::instance().setPrompt(text);CurveController::instance().logMessage(text);}
void SurfaceController::refresh(){
    const QString name=options.kind==1?"Sweep1":options.kind==2?"Sweep2":"Loft";
    const auto phase=om9_surface_phase();
    if(phase==1)prompt(name+": Select first rail (Esc cancels)");
    else if(phase==2)prompt(name+": Select second rail (Undo / Esc)");
    else if(phase==3)prompt(name+": Select profiles in order; Enter for options (Undo / Cancel)");
}
bool SurfaceController::start(std::size_t i){
    if(!available(i))return false;
    CoreDistance::instance().cancel();CorePictureFrame::instance().cancel();CoreViewControls::instance().cancel();CurveController::instance().cancel();cancel();
    command=i;document=App::GetApplication().getActiveDocument();options={om9_surface_kind(om9_command_id(i)),0,false,false};
    if(!om9_surface_start(om9_command_id(i))){document=nullptr;return false;}
    qApp->installEventFilter(this);refresh();
    // FreeCAD preserves ordered selection entries; subedges remain individual inputs.
    const auto selected=Gui::Selection().getSelection(document->getName());
    for(const auto& sel:selected)add(sel.FeatName,sel.SubName?sel.SubName:"");
    return false; // History is recorded only after a successful geometry commit.
}
void SurfaceController::clearPreview(){for(const auto& [root,node]:previews){if(root->findChild(node)>=0)root->removeChild(node);root->unref();}previews.clear();}
void SurfaceController::cancel(){
    om9_surface_cancel();document=nullptr;inputs.clear();clearPreview();
    if(dialog){auto* old=dialog.data();dialog=nullptr;old->disconnect(this);old->hide();old->deleteLater();}status=nullptr;buttons=nullptr;
}
void SurfaceController::error(const std::exception& e){const auto text=QString::fromUtf8(e.what());prompt(text);if(status)status->setText(text);if(buttons)buttons->button(QDialogButtonBox::Ok)->setEnabled(false);Base::Console().warning("OpenMatrix9 surface: %s\n",e.what());}
void SurfaceController::add(const std::string& object,const std::string& sub){
    if(!active()||!valid()||om9_surface_phase()==4)return;
    try{
        Base::PyGILStateLocker lock;SurfaceInput input{object,sub};ShapeRef wire(surfaceWire(*document,input));ShapeRef closed(PyObject_CallMethod(wire.p,"isClosed",nullptr));
        if(!closed.p){PyErr_Clear();throw std::runtime_error("Cannot inspect curve closure");}input.closed=PyObject_IsTrue(closed.p)==1;
        const auto key=object+"."+sub;if(!om9_surface_add(key.c_str(),input.closed)){prompt(message());return;}
        inputs.push_back(input);CurveController::instance().logMessage("Selected: "+QString::fromStdString(key));refresh();
    }catch(const std::exception& e){error(e);}
}
void SurfaceController::submit(const QString& text){
    if(!active())return;if(!valid()){cancel();return;}const auto input=text.trimmed();
    if(input.compare("Cancel",Qt::CaseInsensitive)==0||input.compare("Esc",Qt::CaseInsensitive)==0){cancel();prompt("Surface command cancelled");return;}
    if(dialog){if(input.isEmpty()||input.compare("OK",Qt::CaseInsensitive)==0)commit();else if(input.compare("Preview",Qt::CaseInsensitive)==0)preview();else prompt("Use the surface options dialog, OK or Cancel");return;}
    if(input.compare("Undo",Qt::CaseInsensitive)==0){if(om9_surface_undo()){inputs.pop_back();refresh();}else prompt(message());return;}
    if(input.isEmpty()){
        if(!om9_surface_finish()){prompt(message());return;}optionsDialog();return;
    }
    // Object names, optionally Object.EdgeN, are accepted by the same handler as picks.
    const auto dot=input.indexOf('.');const auto object=dot<0?input:input.left(dot),sub=dot<0?QString():input.mid(dot+1);
    add(object.toUtf8().constData(),sub.toUtf8().constData());
}
void SurfaceController::optionsDialog(){
    dialog=new QDialog(Gui::getMainWindow());dialog->setObjectName("OM9SurfaceOptions");dialog->setWindowTitle(options.kind==1?"Sweep 1 Options":options.kind==2?"Sweep 2 Options":"Loft Options");dialog->setAttribute(Qt::WA_DeleteOnClose);
    auto* layout=new QVBoxLayout(dialog);auto* form=new QFormLayout;layout->addLayout(form);
    if(options.kind==3){
        auto* style=new QComboBox(dialog);style->setObjectName("OM9LoftStyle");style->addItems({"Normal","Straight Sections"});form->addRow("Style",style);
        connect(style,&QComboBox::currentIndexChanged,this,[this](int i){options.style=i;preview();});
        auto* closed=new QCheckBox("Connect last section to first",dialog);closed->setObjectName("OM9ClosedLoft");closed->setEnabled(inputs.size()>=3);form->addRow("Closed loft",closed);connect(closed,&QCheckBox::toggled,this,[this](bool b){options.closed=b;preview();});
    }else if(options.kind==1){
        auto* frenet=new QCheckBox("Frenet frame",dialog);frenet->setObjectName("OM9SweepFrenet");form->addRow("Orientation",frenet);connect(frenet,&QCheckBox::toggled,this,[this](bool b){options.frenet=b;preview();});
    }
    auto* table=new QTableWidget(int(inputs.size()),3,dialog);table->setObjectName("OM9SurfaceInputs");table->setHorizontalHeaderLabels({"Input order","Reverse","Seam (0–1)"});layout->addWidget(table);
    const unsigned rails=options.kind==1?1:options.kind==2?2:0;
    for(std::size_t i=0;i<inputs.size();++i){
        const auto& item=inputs[i];auto* label=new QTableWidgetItem(QString("%1: %2%3").arg(i<rails?"Rail":"Profile").arg(QString::fromStdString(item.object)).arg(item.sub.empty()?QString():"."+QString::fromStdString(item.sub)));label->setFlags(label->flags()&~Qt::ItemIsEditable);table->setItem(int(i),0,label);
        auto* reverse=new QCheckBox(table);table->setCellWidget(int(i),1,reverse);connect(reverse,&QCheckBox::toggled,this,[this,i](bool b){inputs[i].reverse=b;preview();});
        auto* seam=new QDoubleSpinBox(table);seam->setDecimals(4);seam->setRange(0,.9999);seam->setSingleStep(.05);seam->setEnabled(item.closed&&i>=rails);table->setCellWidget(int(i),2,seam);connect(seam,&QDoubleSpinBox::valueChanged,this,[this,i](double v){inputs[i].seam=v;preview();});
    }
    table->resizeColumnsToContents();
    status=new QLabel(dialog);status->setObjectName("OM9SurfaceStatus");status->setWordWrap(true);layout->addWidget(status);
    buttons=new QDialogButtonBox(QDialogButtonBox::Ok|QDialogButtonBox::Cancel,dialog);layout->addWidget(buttons);
    connect(buttons,&QDialogButtonBox::accepted,this,[this]{commit();});connect(buttons,&QDialogButtonBox::rejected,dialog,&QDialog::reject);
    connect(dialog,&QDialog::rejected,this,[this]{cancel();prompt("Surface command cancelled");});
    dialog->resize(560,380);dialog->show();preview();
}
void SurfaceController::preview(){
    clearPreview();if(!active()||!valid())return;
    try{
        Base::PyGILStateLocker lock;ShapeRef shape(buildSurface(*document,inputs,options));ShapeRef inventor(PyObject_CallMethod(shape.p,"writeInventor",nullptr));
        if(!inventor.p){PyErr_Clear();throw std::runtime_error("Cannot render surface preview");}const char* text=PyUnicode_AsUTF8(inventor.p);if(!text)throw std::runtime_error("Invalid preview data");
        auto* gui=Gui::Application::Instance->activeDocument();for(auto* mdi:gui->getMDIViews()){
            auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);if(!view)continue;auto* root=dynamic_cast<SoSeparator*>(view->getViewer()->getSceneGraph());if(!root)continue;
            SoInput input;input.setBuffer(text,std::strlen(text));auto* mesh=SoDB::readAll(&input);if(!mesh)throw std::runtime_error("Cannot load preview mesh");
            auto* wrapper=new SoSeparator;wrapper->setName("OM9SurfacePreview");auto* skip=new SoPickStyle;skip->style=SoPickStyle::UNPICKABLE;wrapper->addChild(skip);wrapper->addChild(mesh);root->ref();root->addChild(wrapper);previews.emplace_back(root,wrapper);
        }
        if(status)status->setText("Valid surface preview. Adjust directions/seams, then click OK.");if(buttons)buttons->button(QDialogButtonBox::Ok)->setEnabled(true);
    }catch(const std::exception& e){clearPreview();error(e);}
}
void SurfaceController::commit(){
    if(!active()||!valid()){cancel();return;}
    try{
        Base::PyGILStateLocker lock;ShapeRef shape(buildSurface(*document,inputs,options));commitSurface(*document,shape.p,inputs,options);
        om9_sidebar_record_execution(command,true);cancel();prompt("Surface created. Command:");
    }catch(const std::exception& e){error(e);}
}
bool SurfaceController::eventFilter(QObject* watched,QEvent* event){
    if(event->type()==QEvent::MouseButtonRelease&&releaseTarget==watched&&static_cast<QMouseEvent*>(event)->button()==Qt::LeftButton){releaseTarget=nullptr;return true;}
    if(!active()||Gui::Application::Instance->isClosing())return false;if(!valid()){cancel();return false;}
    auto* view=containing(watched);
    if(event->type()==QEvent::KeyPress&&CoreKeyboard::inputContext(watched)){
        auto* key=static_cast<QKeyEvent*>(event);if(key->key()==Qt::Key_Escape){cancel();prompt("Surface command cancelled");return true;}
        if(view&&(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter||key->key()==Qt::Key_Space)){submit({});return true;}
    }
    if(dialog||!view||event->type()!=QEvent::MouseButtonPress)return false;
    auto* mouse=static_cast<QMouseEvent*>(event);if(mouse->button()!=Qt::LeftButton||mouse->modifiers()&(Qt::ControlModifier|Qt::AltModifier|Qt::MetaModifier))return false;
    auto* viewer=view->getViewer();auto* widget=qobject_cast<QWidget*>(watched);const auto logical=viewer->viewport()->mapFrom(widget,mouse->position().toPoint());
    std::unique_ptr<SoPickedPoint> picked(viewer->pickPoint(viewer->fromQPoint(logical)));auto* provider=picked?dynamic_cast<Gui::ViewProviderDocumentObject*>(viewer->getViewProviderByPath(picked->getPath())):nullptr;
    releaseTarget=watched;
    if(!provider||!provider->getObject()||!provider->isSelectable())return true;
    std::string sub;provider->getElementPicked(picked.get(),sub);
    // A whole curve is preferable to one segment; surfaces require an edge pick.
    auto* object=provider->getObject();
    try{Base::PyGILStateLocker lock;ShapeRef wire(surfaceWire(*document,{object->getNameInDocument(),""}));sub.clear();}catch(const std::exception&){}
    add(object->getNameInDocument(),sub);return true;
}
}
