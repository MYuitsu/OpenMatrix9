#include "CoreDistance.h"
#include "CoreKeyboard.h"
#include "CoreMouse.h"
#include "CoreSnaps.h"
#include "CorePictureFrame.h"
#include "CoreViewControls.h"
#include "CurveController.h"
#include "RustBridge.h"
#include <App/Application.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/MainWindow.h>
#include <Gui/Control.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Base/Console.h>
#include <QApplication>
#include <QLabel>
#include <QMouseEvent>
#include <QKeyEvent>
#include <QTimer>
#include <QVariant>
#include <cmath>
#include <exception>
#include <stdexcept>
namespace {
Gui::View3DInventor* view(){auto* doc=Gui::Application::Instance->activeDocument();return doc?dynamic_cast<Gui::View3DInventor*>(doc->getActiveView()):nullptr;}
const char* units[]={"mm","cm","m","in","ft"};
}
namespace OpenMatrix9Gui {
CoreDistance& CoreDistance::instance(){static auto* controller=new CoreDistance;return *controller;}
CoreDistance::CoreDistance():QObject(qApp){
    deletedConnection=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(document==&doc)cancel();});
    activeConnection=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(running&&document!=&doc)cancel();});
    qApp->installEventFilter(this);auto* timer=new QTimer(this);timer->setInterval(100);
    connect(timer,&QTimer::timeout,this,[this]{if(running&&!valid())cancel();});timer->start();
}
bool CoreDistance::handles(std::size_t index){auto* name=om9_command_id(index);return name&&(QString::fromUtf8(name)=="Distance"||QString::fromUtf8(name)=="Angle");}
bool CoreDistance::available()const{
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    return enabled&&doc&&gui&&!gui->isAboutToClose()&&!gui->getInEdit()&&view()&&om9AlterView(doc);
}
bool CoreDistance::active()const{return running;}
bool CoreDistance::valid()const{return document==App::GetApplication().getActiveDocument()&&available();}
void CoreDistance::activate(){enabled=true;}
void CoreDistance::deactivate(){enabled=false;cancel();}
void CoreDistance::cancel(){if(running)CoreSnaps::clearTransient();running=false;hasFirst=false;angleCount=0;document=nullptr;accepted.clear();coordinateContext={};}
void CoreDistance::prompt(const QString& message){CurveController::instance().setPrompt(message);CurveController::instance().logMessage(message);}
bool CoreDistance::checkContext(){
    if(!running||!valid()){cancel();return false;}
    try{verifyCoordinateContext(*document,coordinateContext);return true;}
    catch(const std::exception& error){cancel();prompt(QString::fromUtf8(error.what()));}
    catch(...){cancel();prompt("Measurement input context is unavailable; restart the command");}
    return false;
}
std::optional<Base::Vector3d> CoreDistance::previousPoint()const{
    if(accepted.empty())return std::nullopt;
    return accepted.back().point;
}
bool CoreDistance::start(std::size_t index){
    if(!handles(index)||!available())return false;
    cancel();CorePictureFrame::instance().cancel();CoreViewControls::instance().cancel();CurveController::instance().cancel();
    try{
        document=App::GetApplication().getActiveDocument();
        coordinateContext=captureCoordinateContext(*document,view());
        command=index;unit=0;running=true;
        measuringAngle=QString::fromUtf8(om9_command_id(index))=="Angle";
        qApp->installEventFilter(this);CoreMouse::instance().prioritize();prompt(measuringAngle?"Angle: Start of first line":"Distance: Pick first point or enter x,y,z; Unit=mm/cm/m/in/ft; Esc cancels");
    }catch(const std::exception& error){cancel();CoreSnaps::clearTransient();prompt(QString::fromUtf8(error.what()));}
    catch(...){cancel();CoreSnaps::clearTransient();prompt("Cannot start measurement input");}
    return false;
}
void CoreDistance::finish(const CoordinateInput& input,const QString& message){
    // Serialize a candidate copy first: a rejected endpoint or report failure must
    // not advance the accepted-point sequence or publish partial provenance.
    auto completed=accepted;completed.push_back(input);
    const auto report=coordinateInputReport(completed);
    std::vector<QString> transcript;transcript.reserve(completed.size());
    for(std::size_t index=0;index<completed.size();++index){
        const auto& value=completed[index].point;
        transcript.push_back(QString("Point %1 (world, mm): %2, %3, %4")
            .arg(static_cast<unsigned>(index+1)).arg(value.x,0,'g',15).arg(value.y,0,'g',15).arg(value.z,0,'g',15));
    }
    if(!checkContext())return;
    Gui::getMainWindow()->setProperty("om9MeasurementReport",QString::fromUtf8(report.data(),static_cast<qsizetype>(report.size())));
    for(const auto& line:transcript){Base::Console().message("{}\n",line.toUtf8().constData());CurveController::instance().logMessage(line);}
    Base::Console().message("{}\n",message.toUtf8().constData());
    om9_sidebar_record_execution(command,true);cancel();prompt(message);
}
void CoreDistance::point(const CoordinateInput& input){
    if(!checkContext())return;
    if(input.kind!=CoordinateValueKind::Point)throw std::runtime_error("Expected a point for measurement");
    const auto& value=input.point;
    if(!std::isfinite(om9_measure_distance(value.x,value.y,value.z,value.x,value.y,value.z,0))){prompt(measuringAngle?"Angle: Invalid point coordinates":"Distance: Invalid point coordinates");return;}
    if(measuringAngle){
        if(angleCount==1||angleCount==3){const auto& start=anglePoints[angleCount-1];if(om9_measure_distance(start.x,start.y,start.z,value.x,value.y,value.z,0)<=1e-12){prompt("Angle: Line must have a nonzero length; pick its end again");return;}}
        const char* messages[]={"Angle: Start of first line","Angle: End of first line","Angle: Start of second line","Angle: End of second line"};
        if(angleCount<3){accepted.push_back(input);anglePoints[angleCount++]=value;CoreSnaps::acceptedPoint(true);prompt(messages[angleCount]);return;}
        double points[12];for(unsigned int i=0;i<4;++i)for(unsigned int j=0;j<3;++j)points[i*3+j]=(i==3?value:anglePoints[i])[j];
        const auto angle=om9_measure_angle(points);
        if(!std::isfinite(angle)){prompt("Angle: Invalid line; pick its end again");return;}
        const auto message=QString("Angle: %1 degrees").arg(angle,0,'g',15);
        finish(input,message);return;
    }
    if(!hasFirst){accepted.push_back(input);first=value;hasFirst=true;CoreSnaps::acceptedPoint(true);prompt("Distance: Pick second point or enter x,y,z; Unit=mm/cm/m/in/ft");return;}
    const auto distance=om9_measure_distance(first.x,first.y,first.z,value.x,value.y,value.z,unit);
    if(!std::isfinite(distance)){prompt("Distance: Invalid measurement");return;}
    const auto message=QString("Distance: %1 %2").arg(distance,0,'g',15).arg(units[unit]);
    finish(input,message);
}
void CoreDistance::submit(const QString& text){
    if(!checkContext())return;const auto input=text.trimmed();
    if(input.compare("Cancel",Qt::CaseInsensitive)==0){const auto message=measuringAngle?"Angle cancelled":"Distance cancelled";cancel();prompt(message);return;}
    if(measuringAngle&&(input.startsWith("TwoObjects",Qt::CaseInsensitive)||input.startsWith("Unit=",Qt::CaseInsensitive))){prompt("Angle: This option is not implemented yet");return;}
    if(input.startsWith("Unit=",Qt::CaseInsensitive)){
        const auto requested=input.mid(5).trimmed();for(unsigned int i=0;i<5;++i)if(requested.compare(units[i],Qt::CaseInsensitive)==0){unit=i;prompt(QString("Distance: %1 point; Unit=%2").arg(hasFirst?"Pick second":"Pick first").arg(units[i]));return;}
        prompt("Distance: Unit must be mm, cm, m, in or ft");return;
    }
    try{point(parseCoordinateInput(coordinateContext,text,CoordinateExpected::Point,previousPoint()));}
    catch(const std::exception& error){prompt(QString::fromUtf8(error.what()));}
    catch(...){cancel();prompt("Cannot resolve measurement input; restart the command");}
}
bool CoreDistance::eventFilter(QObject* watched,QEvent* event){
    try{
    if(event->type()==QEvent::MouseButtonRelease&&releaseTarget==watched&&static_cast<QMouseEvent*>(event)->button()==Qt::LeftButton){releaseTarget.clear();return true;}
    if(!running)return false;
    if(Gui::Application::Instance->isClosing()){cancel();return false;}
    if(!valid()){cancel();return false;}
    if(event->type()==QEvent::KeyPress&&static_cast<QKeyEvent*>(event)->key()==Qt::Key_F4&&CoreKeyboard::pointKeyAllowed(watched,static_cast<QKeyEvent*>(event))){if(checkContext())point(worldCoordinateInput(coordinateContext,coordinateContext.plane.getPosition(),CoordinateSource::OriginShortcut,previousPoint()));return true;}
    if(event->type()==QEvent::KeyPress&&static_cast<QKeyEvent*>(event)->key()==Qt::Key_Escape){const auto message=measuringAngle?"Angle cancelled":"Distance cancelled";cancel();prompt(message);return true;}
    if(event->type()!=QEvent::MouseButtonPress)return false;
    const auto* mouse=static_cast<QMouseEvent*>(event);if(mouse->button()!=Qt::LeftButton)return false;
    auto* viewer=view()->getViewer();auto* widget=qobject_cast<QWidget*>(watched);if(!widget||!viewer->isAncestorOf(widget))return false;
    releaseTarget=watched;
    if(!checkContext())return true;
    const auto logical=viewer->viewport()->mapFrom(widget,mouse->position().toPoint());
    Base::Vector3d snapped;if(CoreSnaps::pick(view(),logical,snapped)){point(worldCoordinateInput(coordinateContext,snapped,CoordinateSource::WorldSnap,previousPoint()));return true;}
    const auto pixel=viewer->fromQPoint(logical);
    SbVec3f nearPoint,farPoint;viewer->projectPointToLine(pixel,nearPoint,farPoint);const auto direction=farPoint-nearPoint;
    const auto& plane=coordinateContext.plane;
    const auto normal=plane.getRotation().multVec(Base::Vector3d(0,0,1));
    if(std::abs(direction[0]*normal.x+direction[1]*normal.y+direction[2]*normal.z)<=1e-6*direction.length()){prompt(measuringAngle?"Angle: View ray is parallel to construction plane":"Distance: View ray is parallel to construction plane");return true;}
    const auto value=viewer->getPointOnXYPlaneOfPlacement(pixel,plane);point(worldCoordinateInput(coordinateContext,Base::Vector3d(value[0],value[1],value[2]),CoordinateSource::WorldPick,previousPoint()));return true;
    }catch(const std::exception& error){prompt(QString::fromUtf8(error.what()));return true;}
    catch(...){cancel();prompt("Cannot resolve measurement input; restart the command");return true;}
}
}
