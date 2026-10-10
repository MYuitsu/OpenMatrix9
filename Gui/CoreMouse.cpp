#include "CoreMouse.h"
#include "CoreKeyboard.h"
#include "CoreViewControls.h"
#include "CoreDistance.h"
#include "CorePictureFrame.h"
#include "CurveController.h"
#include "NativeCommands.h"
#include "RustBridge.h"
#include "CameraState.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/MainWindow.h>
#include <Gui/Control.h>
#include <Gui/Selection/Selection.h>
#include <Gui/Selection/BoxSelection.h>
#include <Gui/Navigation/NavigationStyle.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <Inventor/SoPickedPoint.h>
#include <Inventor/SoRenderManager.h>
#include <Inventor/nodes/SoCamera.h>
#include <Inventor/nodes/SoPerspectiveCamera.h>
#include <QApplication>
#include <QMouseEvent>
#include <QWheelEvent>
#include <QKeyEvent>
#include <QTimer>
#include <QRubberBand>
#include <QMenu>
#include <QCursor>
#include <cmath>
#include <algorithm>
#include <memory>
#include <limits>
namespace {
Gui::View3DInventor* containing(QObject* object){
    auto* widget=qobject_cast<QWidget*>(object);auto* doc=Gui::Application::Instance->activeDocument();if(!widget||!doc)return nullptr;
    for(auto* mdi:doc->getMDIViews())if(auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);view&&(widget==view->getViewer()||view->getViewer()->isAncestorOf(widget)))return view;
    return nullptr;
}
bool pointTool(){return om9_curve_active()||OpenMatrix9Gui::CoreDistance::instance().active()||OpenMatrix9Gui::CorePictureFrame::instance().active();}
unsigned int mods(Qt::KeyboardModifiers value){return (value.testFlag(Qt::ShiftModifier)?1U:0U)|(value.testFlag(Qt::ControlModifier)?2U:0U)|(value.testFlag(Qt::AltModifier)?4U:0U)|(value.testFlag(Qt::MetaModifier)?8U:0U);}
SbVec3f direction(SoCamera* camera){SbVec3f result;camera->orientation.getValue().multVec(SbVec3f(0,0,-1),result);return result;}
SbVec3f focal(SoCamera* camera){return camera->position.getValue()+direction(camera)*camera->focalDistance.getValue();}
QPoint position(Gui::View3DInventor* view,QWidget* widget,const QPoint& point){return view->getViewer()->viewport()->mapFrom(widget,point);}
}
namespace OpenMatrix9Gui {
CoreMouse& CoreMouse::instance(){static auto* instance=new CoreMouse;return *instance;}
CoreMouse::CoreMouse():QObject(qApp){
    hold=new QTimer(this);hold->setSingleShot(true);connect(hold,&QTimer::timeout,this,[this]{
        if(!target||dragging||button!=2)return;
        const auto command=[](){for(std::size_t i=0;i<om9_command_count();++i)if(QString::fromUtf8(om9_command_id(i))=="F6")return i;return std::numeric_limits<std::size_t>::max();}();
        cancel(false);
        CoreKeyboard::instance().execute(command);
    });
}
void CoreMouse::prioritize(){if(enabled)qApp->installEventFilter(this);}
void CoreMouse::activate(){enabled=true;prioritize();}
void CoreMouse::deactivate(){cancel();enabled=false;if(popup)popup->close();}
void CoreMouse::cancel(bool rollback){
    if(target&&button){releaseView=target;releaseButton=button;}
    const auto view=target;const auto operation=action;const auto camera=originalCamera;
    // releaseMouse can synchronously emit UngrabMouse. Clear the gesture first.
    hold->stop();target=nullptr;button=action=modifiers=0;dragging=longClick=false;originalCamera.clear();
    if(view){auto* widget=view->getViewer()->viewport();if(QWidget::mouseGrabber()==widget)widget->releaseMouse();
        if(rollback&&operation>=2&&operation<=7&&!camera.empty())try{restoreCameraState(view->getViewer(),camera);}catch(...){}}
    if(rubber)delete rubber.data();rubber=nullptr;
}
void CoreMouse::motion(const QPoint& point){
    auto* viewer=target->getViewer();auto* camera=viewer->getSoRenderManager()->getCamera();if(!camera)return;
    const auto delta=point-last;const float height=float(std::max(1,viewer->viewport()->height()));
    if(action==1){if(!rubber){rubber=new QRubberBand(QRubberBand::Rectangle,viewer->viewport());rubber->setObjectName("OM9MouseSelection");}rubber->setGeometry(QRect(anchor,point).normalized());rubber->show();}
    else if(action==2){
        const auto volume=camera->getViewVolume(float(viewer->viewport()->width())/height);const auto normal=direction(camera);const auto focus=focal(camera);
        auto world=[&](QPoint p){SbVec3f a,b;volume.projectPointToLine(SbVec2f(float(p.x())/std::max(1,viewer->viewport()->width()-1),1.f-float(p.y())/std::max(1,viewer->viewport()->height()-1)),a,b);const auto ray=b-a;const float den=normal.dot(ray);return std::abs(den)>1e-9f?a+ray*(normal.dot(focus-a)/den):focus;};
        camera->position=camera->position.getValue()+world(last)-world(point);
    }else if(action==4){CoreViewControls::scaleCamera(target,std::exp2(double(delta.y())/height));}
    else if(action==5){camera->position=camera->position.getValue()+direction(camera)*(camera->focalDistance.getValue()*float(delta.y())/height);}
    else {
        SbVec3f x;camera->orientation.getValue().multVec(SbVec3f(1,0,0),x);
        const auto rotation=action==6?SbRotation(direction(camera),float(delta.y())*0.01f):SbRotation(SbVec3f(0,0,1),-float(delta.x())*0.01f)*SbRotation(x,-float(delta.y())*0.01f);
        viewer->navigationStyle()->reorientCamera(camera,rotation,action==7?camera->position.getValue():focal(camera));
    }
    last=point;
}
void CoreMouse::select(const QPoint& point,bool rectangle){
    auto* doc=App::GetApplication().getActiveDocument();if(!doc||!Gui::Control().isAllowedAlterSelection(doc))return;
    auto* viewer=target->getViewer();const char* name=doc->getName();
    if(rectangle){
        const auto previous=Gui::Selection().getSelectionT(name,Gui::ResolveMode::NoResolve);
        // Native crossing tests geometry. Its window mode tests centers; remove
        // partial objects to obtain Rhino's full-window containment behavior.
        Gui::applyBoxSelection(viewer,{viewer->fromQPoint(anchor),viewer->fromQPoint(point)},(modifiers&3)==3,(modifiers&1)!=0);
        if(point.x()>=anchor.x()){
            const QRect rect=QRect(anchor,point).normalized();auto selected=Gui::Selection().getSelectionT(name,Gui::ResolveMode::NoResolve);
            for(const auto& selection:selected){if((modifiers&1)&&std::find(previous.begin(),previous.end(),selection)!=previous.end())continue;auto* object=selection.getObject();if(!object)continue;auto* provider=dynamic_cast<Gui::ViewProviderDocumentObject*>(target->getGuiDocument()->getViewProvider(object));if(!provider)continue;
                const auto bounds=provider->getBoundingBox(selection.getSubName().c_str(),nullptr,true,viewer);if(!bounds.IsValid())continue;
                const auto volume=viewer->getSoRenderManager()->getCamera()->getViewVolume(float(viewer->viewport()->width())/std::max(1,viewer->viewport()->height()));
                double x0=1e20,y0=1e20,x1=-1e20,y1=-1e20;
                for(int c=0;c<8;++c){SbVec3f p;volume.projectToScreen(SbVec3f(float(c&1?bounds.MaxX:bounds.MinX),float(c&2?bounds.MaxY:bounds.MinY),float(c&4?bounds.MaxZ:bounds.MinZ)),p);double x=p[0]*(viewer->viewport()->width()-1),y=(1-p[1])*(viewer->viewport()->height()-1);x0=std::min(x0,x);x1=std::max(x1,x);y0=std::min(y0,y);y1=std::max(y1,y);}
                if(!om9_mouse_window_contains(rect.left(),rect.top(),rect.right(),rect.bottom(),x0,y0,x1,y1))Gui::Selection().rmvSelection(name,object->getNameInDocument(),selection.getSubName().c_str());
            }
        }
        return;
    }
    std::unique_ptr<SoPickedPoint> picked(viewer->pickPoint(viewer->fromQPoint(point)));
    auto* provider=picked?dynamic_cast<Gui::ViewProviderDocumentObject*>(viewer->getViewProviderByPath(picked->getPath())):nullptr;
    if(!provider||!provider->isSelectable()||!provider->getObject()){if(modifiers==0)Gui::Selection().clearSelection(name);return;}
    auto* object=provider->getObject();std::string sub;
    if((modifiers&3)==3)provider->getElementPicked(picked.get(),sub);
    if(modifiers==2)Gui::Selection().rmvSelection(name,object->getNameInDocument());
    else {const auto p=picked->getPoint();Gui::Selection().addSelection(name,object->getNameInDocument(),sub.c_str(),p[0],p[1],p[2]);}
}
void CoreMouse::confirm(){
    if(pointTool()||CurveController::instance().pendingInput()){CurveController::instance().acceptInput();return;}
    if(om9_sidebar_history_count()){const auto command=om9_sidebar_history_command(0);if(om9NativeCommandAvailable(command))om9_sidebar_record_execution(command,om9ExecuteNativeCommand(command));}
}
void CoreMouse::recent(){
    if(popup)delete popup.data();popup=new QMenu(Gui::getMainWindow());popup->setObjectName("OM9MouseHistory");
    for(std::size_t i=0;i<om9_sidebar_history_count();++i){const auto command=om9_sidebar_history_command(i);auto* item=popup->addAction(QString::fromUtf8(om9_command_menu_text(command)));item->setEnabled(om9NativeCommandAvailable(command));connect(item,&QAction::triggered,this,[command]{om9_sidebar_record_execution(command,om9ExecuteNativeCommand(command));});}
    if(popup->isEmpty())popup->addAction("Chưa có lệnh gần đây")->setEnabled(false);popup->popup(QCursor::pos());
}
bool CoreMouse::eventFilter(QObject* object,QEvent* event){
    if(!Gui::Application::Instance)return false;
    if(event->type()==QEvent::MouseButtonRelease&&releaseView&&unsigned(static_cast<QMouseEvent*>(event)->button())==releaseButton&&containing(object)==releaseView){releaseView=nullptr;releaseButton=0;return true;}
    if(!enabled||Gui::Application::Instance->isClosing())return false;
    if(target&&(event->type()==QEvent::ApplicationDeactivate||(event->type()==QEvent::UngrabMouse&&containing(object)==target))){cancel(false);return false;}
    if(event->type()==QEvent::FocusIn){if(containing(object))prioritize();return false;}
    if(target&&(target->getGuiDocument()!=Gui::Application::Instance->activeDocument()))cancel();
    if(event->type()==QEvent::KeyPress&&target&&static_cast<QKeyEvent*>(event)->key()==Qt::Key_Escape&&CoreKeyboard::inputContext(object)){cancel();return true;}
    auto* view=containing(object);if(!view)return false;
    if(event->type()==QEvent::ContextMenu)return true;
    if(!CoreKeyboard::inputContext(object)){return false;}
    auto* doc=App::GetApplication().getActiveDocument();if(!doc)return false;
    if(event->type()==QEvent::Wheel){
        auto* wheel=static_cast<QWheelEvent*>(event);if(!Gui::Control().isAllowedAlterView(doc))return true;
        Gui::getMainWindow()->setActiveWindow(view);auto* camera=view->getViewer()->getSoRenderManager()->getCamera();if(!camera)return true;
        const double steps=wheel->angleDelta().y()?double(wheel->angleDelta().y())/120.:double(wheel->pixelDelta().y())/40.;
        if(wheel->modifiers()==Qt::AltModifier)camera->position=camera->position.getValue()+direction(camera)*float(steps*camera->focalDistance.getValue()*0.1);
        else CoreViewControls::scaleCamera(view,std::pow(0.9,std::clamp(steps,-20.,20.)));
        return true;
    }
    if(event->type()!=QEvent::MouseButtonPress&&event->type()!=QEvent::MouseMove&&event->type()!=QEvent::MouseButtonRelease)return false;
    auto* mouse=static_cast<QMouseEvent*>(event);const auto point=position(view,qobject_cast<QWidget*>(object),mouse->position().toPoint());
    if(event->type()==QEvent::MouseButtonPress){
        releaseView=nullptr;releaseButton=0;
        if(mouse->button()!=Qt::LeftButton&&mouse->button()!=Qt::RightButton&&mouse->button()!=Qt::MiddleButton)return false;
        Gui::getMainWindow()->setActiveWindow(view);
        if(CoreViewControls::instance().toolActive())return false;
        const auto buttons=unsigned(mouse->button()),keys=mods(mouse->modifiers());
        if(buttons==1&&view->getGuiDocument()->getInEdit())return false;
        auto selected=om9_mouse_action(buttons,keys,dynamic_cast<SoPerspectiveCamera*>(view->getViewer()->getSoRenderManager()->getCamera())!=nullptr);
        if(buttons==4){const auto prefs=App::GetApplication().GetParameterGroupByPath("User parameter:BaseApp/Preferences/Mod/OpenMatrix9/Mouse");const auto mode=prefs->GetInt("MiddleButtonAction",0);if(mode==1)selected=keys==2?4:keys==4?3:2;else if(mode==2)selected=keys==2?4:keys==1?2:3;else if(mode==3)return false;}
        if(buttons==1&&pointTool()&&selected!=4)return false;
        if(selected==0)return false;
        cancel(false);target=view;action=selected;button=buttons;modifiers=keys;anchor=last=point;originalCamera=view->getCamera();
        view->getViewer()->viewport()->grabMouse();
        if(button==2){const auto prefs=App::GetApplication().GetParameterGroupByPath("User parameter:BaseApp/Preferences/Mod/OpenMatrix9/Mouse");hold->start(std::clamp(int(prefs->GetInt("ContextMenuDelay",500)),100,5000));}
        return true;
    }
    if(!target||target!=view)return false;
    if(event->type()==QEvent::MouseMove){
        if(!(unsigned(mouse->buttons())&button)){cancel(false);return false;}
        if(longClick)return true;
        if(!dragging&&!om9_mouse_moved(anchor.x(),anchor.y(),point.x(),point.y(),4.))return true;
        dragging=true;hold->stop();
        if((action==1&&!Gui::Control().isAllowedAlterSelection(doc))||(action!=1&&!Gui::Control().isAllowedAlterView(doc))){cancel();return true;}
        motion(point);return true;
    }
    if(unsigned(mouse->button())!=button)return true;
    hold->stop();const auto released=button;const auto operation=action;const bool clicked=!dragging&&!longClick;
    if(dragging&&action==1)select(point,true);else if(clicked&&released==1&&!pointTool())select(point,false);
    cancel(false);
    releaseView=nullptr;releaseButton=0;
    if(clicked&&released==2&&Gui::Control().isAllowedAlterDocument(doc))confirm();
    if(clicked&&released==4&&operation==8)recent();return true;
}
}
