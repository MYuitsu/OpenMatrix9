#include "CoreDistance.h"
#include "CoreKeyboard.h"
#include "CoreMouse.h"
#include "CoreSnaps.h"
#include "CoreWorkspace.h"
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
#include <QRegularExpression>
#include <QTimer>
#include <cmath>
#include <exception>
namespace {
Gui::View3DInventor* view(){auto* doc=Gui::Application::Instance->activeDocument();return doc?dynamic_cast<Gui::View3DInventor*>(doc->getActiveView()):nullptr;}
const char* units[]={"mm","cm","m","in","ft"};
}
namespace OpenMatrix9Gui {
CoreDistance& CoreDistance::instance(){static auto* controller=new CoreDistance;return *controller;}
CoreDistance::CoreDistance():QObject(qApp){
    qApp->installEventFilter(this);auto* timer=new QTimer(this);timer->setInterval(100);
    connect(timer,&QTimer::timeout,this,[this]{if(running&&!valid())cancel();});timer->start();
}
bool CoreDistance::handles(std::size_t index){auto* name=om9_command_id(index);return name&&(QString::fromUtf8(name)=="Distance"||QString::fromUtf8(name)=="Angle");}
bool CoreDistance::available()const{
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    return enabled&&doc&&gui&&!gui->isAboutToClose()&&!gui->getInEdit()&&view()&&Gui::Control().isAllowedAlterView(doc);
}
bool CoreDistance::active()const{return running;}
bool CoreDistance::valid()const{return document==App::GetApplication().getActiveDocument()&&available();}
void CoreDistance::activate(){enabled=true;}
void CoreDistance::deactivate(){enabled=false;cancel();}
void CoreDistance::cancel(){if(running)CoreSnaps::clearTransient();running=false;hasFirst=false;angleCount=0;document=nullptr;}
void CoreDistance::prompt(const QString& message){CurveController::instance().setPrompt(message);CurveController::instance().logMessage(message);}
bool CoreDistance::start(std::size_t index){
    if(!handles(index)||!available())return false;
    cancel();CorePictureFrame::instance().cancel();CoreViewControls::instance().cancel();CurveController::instance().cancel();
    try {plane=CoreWorkspace::instance().plane(view());}
    catch(const std::exception& error){cancel();CoreSnaps::clearTransient();prompt(QString::fromUtf8(error.what()));return false;}
    document=App::GetApplication().getActiveDocument();command=index;unit=0;running=true;
    measuringAngle=QString::fromUtf8(om9_command_id(index))=="Angle";
    qApp->installEventFilter(this);CoreMouse::instance().prioritize();prompt(measuringAngle?"Angle: Start of first line":"Distance: Pick first point or enter x,y,z; Unit=mm/cm/m/in/ft; Esc cancels");return false;
}
void CoreDistance::point(const Base::Vector3d& value){
    if(!valid()){cancel();return;}
    if(!std::isfinite(om9_measure_distance(value.x,value.y,value.z,value.x,value.y,value.z,0))){prompt(measuringAngle?"Angle: Invalid point coordinates":"Distance: Invalid point coordinates");return;}
    if(measuringAngle){
        if(angleCount==1||angleCount==3){const auto& start=anglePoints[angleCount-1];if(om9_measure_distance(start.x,start.y,start.z,value.x,value.y,value.z,0)<=1e-12){prompt("Angle: Line must have a nonzero length; pick its end again");return;}}
        anglePoints[angleCount++]=value;
        const char* messages[]={"Angle: Start of first line","Angle: End of first line","Angle: Start of second line","Angle: End of second line"};
        if(angleCount<4){CoreSnaps::acceptedPoint(true);prompt(messages[angleCount]);return;}
        double points[12];for(unsigned int i=0;i<4;++i)for(unsigned int j=0;j<3;++j)points[i*3+j]=anglePoints[i][j];
        const auto angle=om9_measure_angle(points);
        if(!std::isfinite(angle)){--angleCount;prompt("Angle: Invalid line; pick its end again");return;}
        const auto message=QString("Angle: %1 degrees").arg(angle,0,'g',15);
        Base::Console().message("{}\n",message.toUtf8().constData());om9_sidebar_record_execution(command,true);cancel();prompt(message);return;
    }
    if(!hasFirst){first=value;hasFirst=true;CoreSnaps::acceptedPoint(true);prompt("Distance: Pick second point or enter x,y,z; Unit=mm/cm/m/in/ft");return;}
    const auto distance=om9_measure_distance(first.x,first.y,first.z,value.x,value.y,value.z,unit);
    if(!std::isfinite(distance)){prompt("Distance: Invalid measurement");return;}
    const auto message=QString("Distance: %1 %2").arg(distance,0,'g',15).arg(units[unit]);
    Base::Console().message("{}\n",message.toUtf8().constData());om9_sidebar_record_execution(command,true);cancel();prompt(message);
}
void CoreDistance::submit(const QString& text){
    if(!running||!valid()){cancel();return;}const auto input=text.trimmed();
    if(input.compare("Cancel",Qt::CaseInsensitive)==0){const auto message=measuringAngle?"Angle cancelled":"Distance cancelled";cancel();prompt(message);return;}
    if(measuringAngle&&(input.startsWith("TwoObjects",Qt::CaseInsensitive)||input.startsWith("Unit=",Qt::CaseInsensitive))){prompt("Angle: This option is not implemented yet");return;}
    if(input.startsWith("Unit=",Qt::CaseInsensitive)){
        const auto requested=input.mid(5).trimmed();for(unsigned int i=0;i<5;++i)if(requested.compare(units[i],Qt::CaseInsensitive)==0){unit=i;prompt(QString("Distance: %1 point; Unit=%2").arg(hasFirst?"Pick second":"Pick first").arg(units[i]));return;}
        prompt("Distance: Unit must be mm, cm, m, in or ft");return;
    }
    const auto fields=input.split(QRegularExpression("[,\\s]+"),Qt::SkipEmptyParts);
    if(fields.size()==3){bool ok[3];Base::Vector3d value;for(int i=0;i<3;++i)value[i]=fields[i].toDouble(&ok[i]);if(ok[0]&&ok[1]&&ok[2]){point(value);return;}}
    prompt(measuringAngle?"Angle: Enter a point x,y,z":"Distance: Enter a point x,y,z or Unit=mm/cm/m/in/ft");
}
bool CoreDistance::eventFilter(QObject* watched,QEvent* event){
    if(event->type()==QEvent::MouseButtonRelease&&releaseTarget==watched&&static_cast<QMouseEvent*>(event)->button()==Qt::LeftButton){releaseTarget.clear();return true;}
    if(!running||Gui::Application::Instance->isClosing())return false;
    if(!valid()){cancel();return false;}
    if(event->type()==QEvent::KeyPress&&static_cast<QKeyEvent*>(event)->key()==Qt::Key_F4&&CoreKeyboard::pointKeyAllowed(watched,static_cast<QKeyEvent*>(event))){point(plane.getPosition());return true;}
    if(event->type()==QEvent::KeyPress&&static_cast<QKeyEvent*>(event)->key()==Qt::Key_Escape){const auto message=measuringAngle?"Angle cancelled":"Distance cancelled";cancel();prompt(message);return true;}
    if(event->type()!=QEvent::MouseButtonPress)return false;
    const auto* mouse=static_cast<QMouseEvent*>(event);if(mouse->button()!=Qt::LeftButton)return false;
    auto* viewer=view()->getViewer();auto* widget=qobject_cast<QWidget*>(watched);if(!widget||!viewer->isAncestorOf(widget))return false;
    releaseTarget=watched;
    const auto logical=viewer->viewport()->mapFrom(widget,mouse->position().toPoint());
    Base::Vector3d snapped;if(CoreSnaps::pick(view(),logical,snapped)){point(snapped);return true;}
    const auto pixel=viewer->fromQPoint(logical);
    SbVec3f nearPoint,farPoint;viewer->projectPointToLine(pixel,nearPoint,farPoint);const auto direction=farPoint-nearPoint;
    const auto normal=plane.getRotation().multVec(Base::Vector3d(0,0,1));
    if(std::abs(direction[0]*normal.x+direction[1]*normal.y+direction[2]*normal.z)<=1e-6*direction.length()){prompt(measuringAngle?"Angle: View ray is parallel to construction plane":"Distance: View ray is parallel to construction plane");return true;}
    const auto value=viewer->getPointOnXYPlaneOfPlacement(pixel,plane);point(Base::Vector3d(value[0],value[1],value[2]));return true;
}
}
