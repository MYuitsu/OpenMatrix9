// SPDX-License-Identifier: LGPL-2.1-or-later
#include "EditController.h"
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
#include <App/DocumentObject.h>
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
#include <Inventor/SoRenderManager.h>
#include <Inventor/actions/SoRayPickAction.h>
#include <Inventor/nodes/SoCamera.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <Inventor/nodes/SoSwitch.h>
#include <QApplication>
#include <QDialog>
#include <QDialogButtonBox>
#include <QPushButton>
#include <QLabel>
#include <QCheckBox>
#include <QVBoxLayout>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QTimer>
#include <cstring>
#include <cmath>
namespace {
Gui::View3DInventor* containing(QObject* object){auto* w=qobject_cast<QWidget*>(object);auto* g=Gui::Application::Instance->activeDocument();if(!w||!g)return nullptr;for(auto* mdi:g->getMDIViews())if(auto* v=dynamic_cast<Gui::View3DInventor*>(mdi);v&&(w==v->getViewer()||v->getViewer()->isAncestorOf(w)))return v;return nullptr;}
const char* caption(unsigned k){const char* names[]={"","Join","Explode","Trim","BooleanDifference","BooleanIntersection","BooleanUnion","Boolean2Objects"};return names[k];}
}
namespace OpenMatrix9Gui {
EditController& EditController::instance(){static auto* c=new EditController;return *c;}
EditController::EditController():QObject(qApp){
    deleteConnection=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& d){if(document==&d)cancel();});
    activeConnection=App::GetApplication().signalActiveDocument.connect([this](const App::Document& d){if(active()&&document!=&d)cancel();});
    auto* timer=new QTimer(this);timer->setInterval(150);connect(timer,&QTimer::timeout,this,[this]{if(active()&&!valid())cancel();});timer->start();
}
bool EditController::handles(std::size_t i){return om9_edit_kind(om9_command_id(i))!=0;}
bool EditController::matches(std::size_t i,const QString& text){return handles(i)&&om9_edit_kind(text.toUtf8().constData())==om9_edit_kind(om9_command_id(i));}
void EditController::activate(){enabled=true;qApp->installEventFilter(this);}
void EditController::deactivate(){enabled=false;cancel();}
bool EditController::active()const{return document&&om9_edit_phase()!=0;}
bool EditController::available(std::size_t i)const{auto* d=App::GetApplication().getActiveDocument();auto* g=Gui::Application::Instance->activeDocument();return enabled&&handles(i)&&d&&g&&!g->isAboutToClose()&&!g->getInEdit()&&!dialog&&dynamic_cast<Gui::View3DInventor*>(g->getActiveView())&&Gui::Control().isAllowedAlterDocument(d);}
bool EditController::valid()const{auto* g=Gui::Application::Instance->activeDocument();return enabled&&document==App::GetApplication().getActiveDocument()&&g&&!g->isAboutToClose()&&!g->getInEdit()&&Gui::Control().isAllowedAlterDocument(document);}
void EditController::prompt(const QString& s){CurveController::instance().setPrompt(s);CurveController::instance().logMessage(s);}
void EditController::refresh(){QString text=QString::fromUtf8(caption(kind))+": ";switch(om9_edit_phase()){
    case 1:text+=(kind==4?"Select objects to subtract from":kind==5?"Select first set":kind==7?"Select exactly two solid objects":"Select objects");text+="; Enter / Undo / Cancel";break;
    case 2:text+=(kind==4?"Select cutters":"Select second set");text+="; Enter / Undo / Cancel";break;
    case 3:text+="Preview; OK / Cancel";if(kind==7)text+="; click viewport or Next to cycle";break;
    case 4:text+="Click segments or surface regions to remove; Enter accepts / Undo / Cancel";break;
}prompt(text);}
bool EditController::start(std::size_t i){
    if(!available(i))return false;CoreDistance::instance().cancel();CorePictureFrame::instance().cancel();CoreViewControls::instance().cancel();CurveController::instance().cancel();cancel();
    kind=om9_edit_kind(om9_command_id(i));command=i;document=App::GetApplication().getActiveDocument();om9_edit_start(om9_command_id(i));qApp->installEventFilter(this);refresh();
    for(const auto& sel:Gui::Selection().getSelection(document->getName())){if(sel.SubName&&*sel.SubName){prompt("Edit operates on whole native objects; clear subelement selection and reselect");continue;}add(sel.FeatName);}return false;
}
void EditController::clearPreview(){for(auto [root,node]:previews){if(root->findChild(node)>=0)root->removeChild(node);root->unref();}previews.clear();trimPreviewSources.clear();for(auto [node,mode]:hidden){if(node->whichChild.getValue()==SO_SWITCH_NONE)node->whichChild=mode;node->unref();}hidden.clear();}
void EditController::cancel(){om9_edit_cancel();document=nullptr;clearPreview();{Base::PyGILStateLocker lock;output.clear();inputs.clear();fragments.clear();}removed.clear();trimUndo.clear();if(dialog){auto* old=dialog.data();dialog=nullptr;old->disconnect(this);old->hide();old->deleteLater();}status=nullptr;buttons=nullptr;}
void EditController::error(const std::exception& e){prompt(QString::fromUtf8(e.what()));if(status)status->setText(QString::fromUtf8(e.what()));if(buttons)buttons->button(QDialogButtonBox::Ok)->setEnabled(false);Base::Console().warning("OpenMatrix9 Edit: {}\n",e.what());}
void EditController::add(const std::string& name){if(!active()||!valid()||om9_edit_phase()>2)return;try{Base::PyGILStateLocker lock;auto input=editInput(*document,name,kind);if(!om9_edit_add(name.c_str()))throw std::runtime_error("Input already selected or this step is full");inputs.push_back(std::move(input));refresh();}catch(const std::exception& e){error(e);}}
void EditController::submit(const QString& text){
    if(!active())return;if(!valid()){cancel();return;}const auto s=text.trimmed();
    if(s.compare("Cancel",Qt::CaseInsensitive)==0||s.compare("Esc",Qt::CaseInsensitive)==0){cancel();prompt("Edit cancelled. Command:");return;}
    try{Base::PyGILStateLocker lock;
        if(s.compare("Undo",Qt::CaseInsensitive)==0){if(om9_edit_phase()==4){if(trimUndo.empty())throw std::runtime_error("No trimmed segment to undo");auto [i,j]=trimUndo.back();trimUndo.pop_back();removed[i].erase(j);preview();}else{const auto before=om9_edit_count(1)+om9_edit_count(2);if(!om9_edit_undo())throw std::runtime_error("No selected input to undo");if(om9_edit_count(1)+om9_edit_count(2)<before)inputs.pop_back();refresh();}return;}
        if(s.compare("Next",Qt::CaseInsensitive)==0){if(!om9_edit_cycle())throw std::runtime_error("Next is available in Boolean2Objects preview");preview();return;}
        if(s.isEmpty()||s.compare("OK",Qt::CaseInsensitive)==0){if(om9_edit_phase()==3||om9_edit_phase()==4){commit();return;}if(!om9_edit_finish())throw std::runtime_error("Select enough valid inputs before pressing Enter");if(om9_edit_phase()>=3)prepare();else refresh();return;}
        if(om9_edit_phase()==4){auto at=s.indexOf('@');auto coords=s.mid(at+1).split(',');if(at<=0||coords.size()!=3)throw std::runtime_error("Pick a segment or enter Object@x,y,z in world mm");std::array<double,3> p;for(unsigned i=0;i<3;++i){bool ok;p[i]=coords[i].toDouble(&ok);if(!ok||!std::isfinite(p[i]))throw std::runtime_error("Invalid trim point");}trim(s.left(at).toStdString(),p);return;}
        if(om9_edit_phase()==3)throw std::runtime_error("Use preview options, OK or Cancel");add(s.toStdString());
    }catch(const std::exception& e){if(!dialog&&fragments.empty()&&om9_edit_phase()>=3)om9_edit_back();error(e);}
}
void EditController::prepare(){
    verifyEditInputs(*document,inputs);
    if(kind==3){fragments=splitEditCurves(inputs);removed.resize(inputs.size());refresh();preview();return;}
    dialog=new QDialog(Gui::getMainWindow());dialog->setObjectName("OM9EditOptions");dialog->setWindowTitle(QString::fromUtf8(caption(kind)));dialog->setAttribute(Qt::WA_DeleteOnClose);auto* layout=new QVBoxLayout(dialog);
    if(kind>=4){auto* remove=new QCheckBox(kind==4?"DeleteInput (cutters)":"DeleteInput",dialog);remove->setObjectName("OM9EditDeleteInput");remove->setChecked(om9_edit_delete_input(-1));layout->addWidget(remove);connect(remove,&QCheckBox::toggled,this,[this](bool b){om9_edit_delete_input(b?1:0);preview();});}
    if(kind==7){auto* next=new QPushButton("Next Boolean result",dialog);next->setObjectName("OM9EditNext");layout->addWidget(next);connect(next,&QPushButton::clicked,this,[this]{om9_edit_cycle();preview();});}
    status=new QLabel(dialog);status->setObjectName("OM9EditStatus");status->setWordWrap(true);layout->addWidget(status);buttons=new QDialogButtonBox(QDialogButtonBox::Ok|QDialogButtonBox::Cancel,dialog);layout->addWidget(buttons);
    connect(buttons,&QDialogButtonBox::accepted,this,[this]{commit();});connect(buttons,&QDialogButtonBox::rejected,this,[this]{cancel();prompt("Edit cancelled. Command:");});connect(dialog,&QDialog::rejected,this,[this]{cancel();prompt("Edit cancelled. Command:");});refresh();preview();dialog->show();
}
void EditController::preview(){
    clearPreview();try{Base::PyGILStateLocker lock;verifyEditInputs(*document,inputs);output.clear();
        if(kind==3){for(std::size_t i=0;i<fragments.size();++i)for(std::size_t j=0;j<fragments[i].size();++j)if(!removed[i].contains(j))output.push_back(fragments[i][j]);}
        else output=buildEdit(inputs,kind,om9_edit_count(1),om9_edit_mode());
        CurvePyRef part(PyImport_ImportModule("Part"));auto* g=Gui::Application::Instance->activeDocument();
        // Transient Coin switches hide inputs without writing Visibility or adding objects.
        for(const auto& i:inputs)if(auto* provider=g->getViewProvider(document->getObject(i.name.c_str()))){auto* node=provider->getModeSwitch();const int mode=node->whichChild.getValue();node->ref();hidden.emplace_back(node,mode);node->whichChild=SO_SWITCH_NONE;}
        std::vector<std::tuple<EditShapes,std::size_t,std::size_t>> batches;
        if(kind==3){for(std::size_t i=0;i<fragments.size();++i)for(std::size_t j=0;j<fragments[i].size();++j)if(!removed[i].contains(j))batches.emplace_back(EditShapes{fragments[i][j]},i,j);}
        else batches.emplace_back(output,0,0);
        for(const auto& [shapes,source,fragment]:batches){CurvePyRef list(PyList_New(0));
            for(const auto& s:shapes)if(PyList_Append(list.value,s->value)<0)throw std::runtime_error("Cannot collect preview");
            if(PyList_Size(list.value)==0)continue;
            CurvePyRef compound(PyObject_CallMethod(part.value,"makeCompound","O",list.value)),inventor(PyObject_CallMethod(compound.value,"writeInventor",nullptr));const char* text=PyUnicode_AsUTF8(inventor.value);if(!text)throw std::runtime_error("Cannot render edit preview");
            for(auto* mdi:g->getMDIViews()){auto* v=dynamic_cast<Gui::View3DInventor*>(mdi);if(!v)continue;auto* root=dynamic_cast<SoSeparator*>(v->getViewer()->getSceneGraph());if(!root)continue;SoInput input;input.setBuffer(text,std::strlen(text));auto* mesh=SoDB::readAll(&input);if(!mesh)throw std::runtime_error("Cannot load edit preview");auto* wrapper=new SoSeparator;wrapper->setName("OM9EditPreview");auto* skip=new SoPickStyle;skip->style=kind==3?SoPickStyle::SHAPE:SoPickStyle::UNPICKABLE;wrapper->addChild(skip);wrapper->addChild(mesh);root->ref();root->addChild(wrapper);previews.emplace_back(root,wrapper);if(kind==3)trimPreviewSources.emplace_back(wrapper,source,fragment);}
        }
        if(status){QString text="Valid native geometry. OK accepts; Cancel discards.";if(kind==7){const char* modes[]={"Union","A minus B","B minus A","Intersection","Inversion Intersection"};text=QString::fromUtf8(modes[om9_edit_mode()])+". "+text;}status->setText(text);}if(buttons)buttons->button(QDialogButtonBox::Ok)->setEnabled(true);refresh();
    }catch(const std::exception& e){Base::PyGILStateLocker lock;output.clear();clearPreview();error(e);}
}
void EditController::trim(const std::string& name,const std::array<double,3>& p){Base::PyGILStateLocker lock;verifyEditInputs(*document,inputs);for(std::size_t i=0;i<inputs.size();++i)if(inputs[i].name==name){auto j=pickedEditSegment(fragments[i],removed[i],p);removed[i].insert(j);trimUndo.emplace_back(i,j);preview();return;}throw std::runtime_error("Pick a selected input curve");}
void EditController::commit(){
    if(!active()||!valid()){cancel();return;}try{Base::PyGILStateLocker lock;if(kind==3&&trimUndo.empty())throw std::runtime_error("Trim a segment before accepting");
        if(kind!=3)output=buildEdit(inputs,kind,om9_edit_count(1),om9_edit_mode());
        else{output.clear();for(std::size_t i=0;i<fragments.size();++i)for(std::size_t j=0;j<fragments[i].size();++j)if(!removed[i].contains(j))output.push_back(fragments[i][j]);}
        commitEdit(*document,inputs,output,kind,om9_edit_delete_input(-1),om9_edit_mode(),om9_edit_count(1));om9_sidebar_record_execution(command,true);cancel();prompt("Edit completed. Command:");
    }catch(const std::exception& e){error(e);}
}
bool EditController::eventFilter(QObject* watched,QEvent* event){
    if(event->type()==QEvent::MouseButtonRelease&&releaseTarget==watched&&static_cast<QMouseEvent*>(event)->button()==Qt::LeftButton){releaseTarget=nullptr;return true;}
    if(!active()||Gui::Application::Instance->isClosing())return false;if(!valid()){cancel();return false;}auto* view=containing(watched);
    if(event->type()==QEvent::KeyPress&&CoreKeyboard::inputContext(watched)){auto* key=static_cast<QKeyEvent*>(event);if(key->key()==Qt::Key_Escape){cancel();prompt("Edit cancelled. Command:");return true;}if(view&&(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter||key->key()==Qt::Key_Space)){submit({});return true;}}
    if(!view||event->type()!=QEvent::MouseButtonPress)return false;auto* mouse=static_cast<QMouseEvent*>(event);if(mouse->button()!=Qt::LeftButton||mouse->modifiers()&(Qt::ControlModifier|Qt::AltModifier|Qt::MetaModifier))return false;
    releaseTarget=watched;
    if(kind==7&&om9_edit_phase()==3){om9_edit_cycle();preview();return true;}if(om9_edit_phase()==3)return true;
    auto* viewer=view->getViewer();auto* w=qobject_cast<QWidget*>(watched);const auto logical=viewer->viewport()->mapFrom(w,mouse->position().toPoint());
    auto* render=viewer->getSoRenderManager();const auto viewport=render->getViewportRegion();
    SoRayPickAction ray(viewport);ray.setPoint(viewer->fromQPoint(logical));ray.setRadius(viewer->getPickRadius());ray.apply(render->getSceneGraph());const auto* picked=ray.getPickedPoint();
    auto* provider=picked?dynamic_cast<Gui::ViewProviderDocumentObject*>(viewer->getViewProviderByPath(picked->getPath())):nullptr;
    if(om9_edit_phase()==4){if(picked)for(const auto& [node,source,fragment]:trimPreviewSources)if(picked->getPath()->containsNode(node)){try{Base::PyGILStateLocker lock;verifyEditInputs(*document,inputs);const auto i=source,j=fragment;const auto p=picked->getPoint();
        const auto volume=render->getCamera()->getViewVolume(viewport.getViewportAspectRatio());
        // Coin's scale uses the horizontal viewport dimension. Evaluate at pick
        // depth so perspective zoom and physical-pixel picking agree.
        const double worldPerPixel=volume.getWorldToScreenScale(p,1.f)/std::max(1,int(viewport.getViewportSizePixels()[0]));
        verifyEditPickBoundary(fragments[i],removed[i],j,{p[0],p[1],p[2]},worldPerPixel*(viewer->getPickRadius()+1.f));removed[i].insert(j);trimUndo.emplace_back(i,j);preview();}catch(const std::exception& e){error(e);}return true;}return true;}
    if(!provider||!provider->getObject()||!provider->isSelectable())return true;
    try{const std::string name=provider->getObject()->getNameInDocument();if(om9_edit_phase()==4){const auto p=picked->getPoint();trim(name,{p[0],p[1],p[2]});}else add(name);}catch(const std::exception& e){error(e);}return true;
}
}
