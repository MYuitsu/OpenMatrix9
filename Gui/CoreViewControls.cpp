#include "CoreViewControls.h"
#include "CurveController.h"
#include "RustBridge.h"
#include "CameraState.h"
#include "CorePictureFrame.h"
#include "CoreDistance.h"
#include <App/Application.h>
#include <App/Document.h>
#include <Base/Console.h>
#include <Base/Exception.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/MainWindow.h>
#include <Gui/Control.h>
#include <Gui/Selection/Selection.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Inventor/SoRenderManager.h>
#include <Inventor/SbBox2s.h>
#include <Inventor/nodes/SoOrthographicCamera.h>
#include <Inventor/nodes/SoPerspectiveCamera.h>
#include <QApplication>
#include <QRubberBand>
#include <QMouseEvent>
#include <QKeyEvent>
#include <QPainter>
#include <QLabel>
#include <QFileDialog>
#include <QFileInfo>
#include <QImage>
#include <QSaveFile>
#include <QMessageBox>
#include <algorithm>
#include <cmath>
#include <limits>

namespace {
class CrosshairOverlay final:public QWidget {
public:
    explicit CrosshairOverlay(QWidget* viewport):QWidget(viewport) {
        setObjectName("OM9Crosshairs");setAttribute(Qt::WA_TransparentForMouseEvents);
        setAttribute(Qt::WA_NoSystemBackground);setAttribute(Qt::WA_TranslucentBackground);
        setGeometry(viewport->rect());hide();
    }
    void track(const QPoint& point) {position=point;setProperty("om9Cursor",point);show();raise();update();}
protected:
    void paintEvent(QPaintEvent*)override {
        QPainter painter(this);painter.setPen(QPen(QColor(0,0,0,100),3));draw(painter);
        painter.setPen(QPen(Qt::white,1));draw(painter);
    }
private:
    void draw(QPainter& painter) {painter.drawLine(0,position.y(),width()-1,position.y());painter.drawLine(position.x(),0,position.x(),height()-1);}
    QPoint position;
};
Gui::View3DInventor* active() {
    auto* doc=Gui::Application::Instance->activeDocument();return doc?dynamic_cast<Gui::View3DInventor*>(doc->getActiveView()):nullptr;
}
std::string id(std::size_t command){const char* value=om9_command_id(command);return value?value:"";}
std::size_t find(const char* name){for(std::size_t i=0;i<om9_command_count();++i)if(id(i)==name)return i;return std::numeric_limits<std::size_t>::max();}
CrosshairOverlay* overlay(Gui::View3DInventor* view,bool create) {
    auto* viewport=view->getViewer()->viewport();auto* result=dynamic_cast<CrosshairOverlay*>(viewport->findChild<QWidget*>("OM9Crosshairs"));
    if(!result&&create){result=new CrosshairOverlay(viewport);viewport->setMouseTracking(true);}
    return result;
}
Gui::View3DInventor* containing(QWidget* widget) {
    if(!widget)return nullptr;auto* doc=Gui::Application::Instance->activeDocument();if(!doc)return nullptr;
    for(auto* mdi:doc->getMDIViews())if(auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);view&&view->getViewer()->isAncestorOf(widget))return view;
    return nullptr;
}
}
namespace OpenMatrix9Gui {
CoreViewControls& CoreViewControls::instance(){static auto* value=new CoreViewControls;return *value;}
CoreViewControls::CoreViewControls():QObject(qApp) {
    activeConnection=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(toolView&&toolView->getGuiDocument()->getDocument()!=&doc)stopTool(true);});
    qApp->installEventFilter(this);
}
bool CoreViewControls::handles(std::size_t command){auto name=id(command);return name=="Zoom_Window"||name=="Zoom_Dynamic"||name=="Zoom_Extents"||name=="Zoom_Selected"||name=="ViewCaptureToFile"||name=="Crosshairs";}
bool CoreViewControls::available(std::size_t command)const {
    if(!enabled||!handles(command)||!active())return false;
    if(id(command)=="ViewCaptureToFile")return true;
    if(!Gui::Control().isAllowedAlterView(App::GetApplication().getActiveDocument()))return false;
    return id(command)!="Zoom_Selected"||!Gui::Selection().getSelectionT(App::GetApplication().getActiveDocument()->getName(),Gui::ResolveMode::NoResolve).empty();
}
void CoreViewControls::activate(){
    qApp->installEventFilter(this);
    enabled=true;refreshCrosshairs();
}
void CoreViewControls::deactivate(){stopTool(true);enabled=false;refreshCrosshairs();}
void CoreViewControls::cancel(){if(kind)stopTool(true);}
void CoreViewControls::scaleCamera(Gui::View3DInventor* view,double factor){
    if(!view)return;auto* camera=view->getViewer()->getSoRenderManager()->getCamera();
    if(auto* ortho=dynamic_cast<SoOrthographicCamera*>(camera)){double value=om9_view_scaled_value(false,ortho->height.getValue(),factor);if(std::isfinite(value))ortho->height=float(value);}
    else if(auto* perspective=dynamic_cast<SoPerspectiveCamera*>(camera)){double value=om9_view_scaled_value(true,perspective->heightAngle.getValue(),factor);if(std::isfinite(value))perspective->heightAngle=float(value);}
}
void CoreViewControls::refreshCrosshairs() {
    for(auto* doc:App::GetApplication().getDocuments())if(auto* gui=Gui::Application::Instance->getDocument(doc))
        for(auto* mdi:gui->getMDIViews())if(auto* view=dynamic_cast<Gui::View3DInventor*>(mdi))
            if(auto* lines=overlay(view,enabled&&om9_core_crosshairs())){lines->setGeometry(view->getViewer()->viewport()->rect());if(!enabled||!om9_core_crosshairs())lines->hide();}
}
void CoreViewControls::prompt() {
    if(kind)CurveController::instance().setPrompt(kind==1?"Zoom_Window: Drag a rectangle; Esc to cancel":"Zoom_Dynamic: Drag vertically; Esc to cancel");
    else {char message[2048]={};om9_curve_message(message,sizeof(message));CurveController::instance().setPrompt(QString::fromUtf8(message));}
}
void CoreViewControls::startTool(unsigned int value,std::size_t command,Gui::View3DInventor* view) {
    CorePictureFrame::instance().cancel();
    CoreDistance::instance().cancel();
    stopTool(true);if(!view||!om9_view_tool_start(value))return;
    kind=value;toolCommand=command;toolView=view;originalCamera=view->getCamera();originalCursor=view->getViewer()->viewport()->cursor();
    // FreeCAD may install its dock-overlay filter after this controller was created.
    // Give the explicitly started tool first refusal for its own viewport events.
    qApp->installEventFilter(this);
    view->getViewer()->viewport()->setCursor(value==1?Qt::CrossCursor:Qt::SizeVerCursor);prompt();
}
void CoreViewControls::stopTool(bool rollback) {
    if(toolView) {
        auto* viewport=toolView->getViewer()->viewport();if(QWidget::mouseGrabber()==viewport)viewport->releaseMouse();viewport->setCursor(originalCursor);
        if(rollback&&!originalCamera.empty())try{restoreCameraState(toolView->getViewer(),originalCamera);}catch(const Base::Exception& error){Base::Console().warning("OpenMatrix9 view cancel: %s\n",error.what());}
    }
    if(rubber){delete rubber.data();rubber=nullptr;}toolView=nullptr;kind=0;dragging=false;originalCamera.clear();om9_view_tool_cancel();prompt();
}
bool CoreViewControls::execute(std::size_t command) {
    if(!available(command))return false;
    if(id(command)=="ViewCaptureToFile")return capture();
    if(id(command)=="Zoom_Extents"||id(command)=="Zoom_Selected") {
        cancel();auto* view=active();auto* viewer=view->getViewer();const auto before=view->getCamera();
        if(id(command)=="Zoom_Extents") {
            SbBox3f bounds;if(!viewer->getSceneBoundBox(bounds))return false;
            viewer->viewBoundBox(bounds);
        } else {
            // Use all selected targets, without the host's maximum-selection
            // truncation applied by viewSelection().
            const auto targets=Gui::Selection().getSelectionT(App::GetApplication().getActiveDocument()->getName(),Gui::ResolveMode::NoResolve);
            auto* gui=view->getGuiDocument();bool valid=false;
            for(const auto& target:targets)if(auto* provider=dynamic_cast<Gui::ViewProviderDocumentObject*>(gui->getViewProvider(target.getObject())))if(provider->getBoundingBox(target.getSubName().c_str()).IsValid()){valid=true;break;}
            if(!valid)return false;
            viewer->viewObjects(targets,false);
        }
        if(auto* camera=dynamic_cast<SoPerspectiveCamera*>(viewer->getSoRenderManager()->getCamera())) {
            // Coin fits a bounding sphere using radius/tan(halfAngle). The
            // tangent to that sphere needs radius/sin(halfAngle), otherwise
            // near corners of a small 3D object can lie outside the viewport.
            const float aspect=viewer->getSoRenderManager()->getViewportRegion().getViewportAspectRatio();
            const float half=std::atan(std::tan(camera->heightAngle.getValue()/2)*std::min(1.0f,aspect));
            const float distance=camera->focalDistance.getValue();
            const float delta=distance*(1/std::cos(half)-1);
            SbVec3f direction;camera->orientation.getValue().multVec(SbVec3f(0,0,-1),direction);
            camera->position=camera->position.getValue()-direction*delta;
            camera->focalDistance=distance+delta;
            camera->nearDistance=camera->nearDistance.getValue()+delta;
            camera->farDistance=camera->farDistance.getValue()+delta;
        }
        return before!=view->getCamera();
    }
    if(id(command)=="Crosshairs"){om9_core_crosshairs_toggle();refreshCrosshairs();return true;}
    startTool(id(command)=="Zoom_Window"?1:2,command,active());return false; // history waits for completed drag
}
bool CoreViewControls::capture() {
    QPointer<Gui::View3DInventor> target=active();if(!target)return false;
    QFileDialog dialog(Gui::getMainWindow(),"Save viewport image");dialog.setObjectName("OM9CaptureDialog");
    dialog.setAcceptMode(QFileDialog::AcceptSave);dialog.setFileMode(QFileDialog::AnyFile);
    dialog.setNameFilters({"PNG image (*.png)","Bitmap image (*.bmp)","JPEG image (*.jpg *.jpeg)"});dialog.setDefaultSuffix("png");
    connect(&dialog,&QFileDialog::filterSelected,&dialog,[&dialog](const QString& filter){dialog.setDefaultSuffix(filter.startsWith("Bitmap")?"bmp":filter.startsWith("JPEG")?"jpg":"png");});
    if(dialog.exec()!=QDialog::Accepted||dialog.selectedFiles().isEmpty())return false;
    if(!target||!target->getGuiDocument()||target->getGuiDocument()->isAboutToClose())return false;
    const QString path=dialog.selectedFiles().front();const auto suffix=QFileInfo(path).suffix().toLower();
    if(suffix!="png"&&suffix!="bmp"&&suffix!="jpg"&&suffix!="jpeg") {QMessageBox::warning(Gui::getMainWindow(),"Viewport image","Choose a PNG, BMP or JPEG filename.");return false;}
    try {
        auto* viewer=target->getViewer();const auto size=viewer->getSoRenderManager()->getViewportRegion().getViewportSizePixels();
        QImage image;viewer->savePicture(size[0],size[1],0,QColor(),image);
        QSaveFile file(path);
        if(image.isNull()||!file.open(QIODevice::WriteOnly)||!image.save(&file,suffix.toLatin1().constData())||!file.commit()) {
            QMessageBox::warning(Gui::getMainWindow(),"Viewport image","Could not save the viewport image. "+file.errorString());return false;
        }
        return true;
    } catch(const Base::Exception& error) {QMessageBox::warning(Gui::getMainWindow(),"Viewport image",QString::fromUtf8(error.what()));return false;}
}
bool CoreViewControls::eventFilter(QObject* object,QEvent* event) {
    if(!enabled)return false;
    if(kind&&!toolView){stopTool(false);}
    if(kind&&event->type()==QEvent::KeyPress&&static_cast<QKeyEvent*>(event)->key()==Qt::Key_Escape){stopTool(true);return true;}
    auto* widget=qobject_cast<QWidget*>(object);auto* view=containing(widget);
    if(event->type()==QEvent::Resize&&view)if(auto* lines=overlay(view,false))lines->setGeometry(view->getViewer()->viewport()->rect());
    if(!view||!(event->type()==QEvent::MouseMove||event->type()==QEvent::MouseButtonPress||event->type()==QEvent::MouseButtonRelease))return false;
    auto* mouse=static_cast<QMouseEvent*>(event);auto* viewport=view->getViewer()->viewport();
    const auto position=viewport->mapFrom(widget,mouse->position().toPoint());
    if(event->type()==QEvent::MouseMove&&om9_core_crosshairs())if(auto* lines=overlay(view,true))lines->track(position);
    const bool allowed=Gui::Control().isAllowedAlterView(App::GetApplication().getActiveDocument());
    if(!kind)return false;
    if(!allowed||view!=toolView){stopTool(true);return false;}
    const QPoint point(std::clamp(position.x(),0,std::max(0,viewport->width()-1)),std::clamp(position.y(),0,std::max(0,viewport->height()-1)));
    if(event->type()==QEvent::MouseButtonPress&&mouse->button()==Qt::LeftButton) {
        dragging=om9_view_tool_press(point.x(),point.y());if(dragging){viewport->grabMouse();if(kind==1){rubber=new QRubberBand(QRubberBand::Rectangle,viewport);rubber->setGeometry(QRect(point,QSize()));rubber->show();}}return true;
    }
    if(!dragging)return false;
    if(event->type()==QEvent::MouseMove) {
        auto effect=om9_view_tool_motion(point.x(),point.y(),viewport->height());
        if(effect==1&&rubber)rubber->setGeometry(QRect(QPoint(qRound(om9_view_tool_value(0)),qRound(om9_view_tool_value(1))),QPoint(qRound(om9_view_tool_value(2)),qRound(om9_view_tool_value(3)))).normalized());
        if(effect==2) {
            scaleCamera(view,om9_view_tool_value(0));
        }
        return true;
    }
    if(event->type()==QEvent::MouseButtonRelease&&mouse->button()==Qt::LeftButton) {
        auto effect=om9_view_tool_release(point.x(),point.y());dragging=false;if(QWidget::mouseGrabber()==viewport)viewport->releaseMouse();if(rubber)rubber->hide();
        if(effect==1) {
            // Native NavigationStyle::boxZoom consumes top-down physical pixels.
            const auto ratio=view->getViewer()->devicePixelRatioF();
            const auto pixels=view->getViewer()->getSoRenderManager()->getViewportRegion().getViewportSizePixels();
            viewport->setProperty("om9ZoomViewportPixels",QSize(pixels[0],pixels[1]));
            view->getViewer()->boxZoom(SbBox2s(short(qRound(om9_view_tool_value(0)*ratio)),short(qRound(om9_view_tool_value(1)*ratio)),short(qRound(om9_view_tool_value(2)*ratio)),short(qRound(om9_view_tool_value(3)*ratio))));
        }
        if(effect==1||effect==3){const bool changed=view->getCamera()!=originalCamera;om9_sidebar_record_execution(toolCommand,changed);stopTool(false);}
        return true;
    }
    return false;
}
}
