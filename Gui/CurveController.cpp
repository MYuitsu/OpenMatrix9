#include "SurfaceController.h"
#include "EditController.h"
#include "Phase3Inputs.h"
#include "CurveController.h"
#include "CommandConsole.h"
#include "RustBridge.h"
#include "NativeCommands.h"
#include "CoreWorkspace.h"
#include "CoreViewControls.h"
#include "CoreNotes.h"
#include "CorePictureFrame.h"
#include "CoreDistance.h"
#include "CoreSnaps.h"
#include "LayerController.h"
#include "LayerDocumentAdapter.h"
#include "CoreRebuild.h"
#include "CurveGeometry.h"
#include "ModelingCurveEditor.h"
#include "CoreKeyboard.h"
#include "CoreMouse.h"
#include <Base/Interpreter.h>
#include <Base/Placement.h>
#include <Base/Console.h>
#include <Base/Exception.h>
#include <App/Application.h>
#include <App/Document.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/MainWindow.h>
#include <Gui/Control.h>
#include <Gui/Inventor/SoFCBoundingBox.h>
#include <Gui/Selection/SoFCUnifiedSelection.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <Inventor/nodes/SoBaseColor.h>
#include <Inventor/nodes/SoCoordinate3.h>
#include <Inventor/nodes/SoLineSet.h>
#include <Inventor/nodes/SoPointSet.h>
#include <Inventor/nodes/SoDrawStyle.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <QApplication>
#include <QDockWidget>
#include <QLineEdit>
#include <QLabel>
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QPlainTextEdit>
#include <QFontDatabase>
#include <QTextBlock>
#include <QTimer>
#include <QMouseEvent>
#include <QKeyEvent>
#include <cmath>
#include <algorithm>
#include <stdexcept>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <TopoDS_Wire.hxx>

namespace {
void updateInputFrame() {
    auto* doc=Gui::Application::Instance->activeDocument();
    auto* view=doc?dynamic_cast<Gui::View3DInventor*>(doc->getActiveView()):nullptr;
    const auto plane=OpenMatrix9Gui::CoreWorkspace::instance().plane(view);const auto origin=plane.getPosition();
    const auto x=plane.getRotation().multVec(Base::Vector3d(1,0,0)),y=plane.getRotation().multVec(Base::Vector3d(0,1,0)),z=plane.getRotation().multVec(Base::Vector3d(0,0,1));
    om9_curve_frame(origin.x,origin.y,origin.z,x.x,x.y,x.z,y.x,y.y,y.z,z.x,z.y,z.z);
}
void commit(App::Document& doc,const char* label) {
    TopoDS_Shape shape;
    if(std::string(label)=="InterpCrv") {if(!om9_curve_spline_publish())throw std::runtime_error("No committed interpolated spline");shape=OpenMatrix9Gui::publishedSplineShape();}
    else {const auto count=om9_curve_count();if(count<2||count>4097)throw std::runtime_error("Invalid Curve output");BRepBuilderAPI_MakePolygon builder;for(std::size_t i=0;i<count;++i)builder.Add(gp_Pnt(om9_curve_coordinate(i,0),om9_curve_coordinate(i,1),om9_curve_coordinate(i,2)));if(!builder.IsDone())throw std::runtime_error("Native curve polygon construction failed");shape=builder.Wire();}
    if(!BRepCheck_Analyzer(shape).IsValid())throw std::runtime_error("Native kernel produced invalid geometry");
    OpenMatrix9Gui::requireLayerGeometryEditable(doc);
    const int transaction=om9OpenTransaction(doc,std::string(label));
    try {
        OpenMatrix9Gui::LayerGeometryTransaction layers(doc,transaction);
        OpenMatrix9Gui::createCurveFeature(doc,shape,label);
        doc.recompute();layers.finish();om9CommitTransaction(doc);
    }catch(...){if(OpenMatrix9Gui::ownsLayerGeometryTransaction(doc,transaction))om9AbortTransaction(doc);throw;}
}
}
namespace OpenMatrix9Gui {
bool isCurveCommand(std::size_t index) {const char* id=om9_command_id(index);return id && (std::string(id)=="Line" || std::string(id)=="Polyline" || std::string(id)=="InterpCrv" || std::string(id)=="Rebuild" || std::string(id)=="PointsOn" || std::string(id)=="Join");}
CurveController& CurveController::instance(){static auto* controller=new CurveController;return *controller;}
CurveController::CurveController():QObject(qApp) {
    deleteConnection=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(document==doc.getName())cancel();});
    activeConnection=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(om9_curve_active()&&documentIdentity!=&doc)cancel();});
    auto* timer=new QTimer(this);timer->setInterval(150);connect(timer,&QTimer::timeout,this,[this]{if(om9_curve_active()&&!validDocument())cancel();});timer->start();
    qApp->installEventFilter(this);
}
void CurveController::activate() {
    enabled=true;
    if(!dock) {
        auto* window=Gui::getMainWindow();dock=new QDockWidget("Command",window);dock->setObjectName("OM9CommandFrame");
        auto* title=new QWidget(dock);title->setFixedHeight(0);dock->setTitleBarWidget(title);
        auto* body=new QWidget(dock);auto* layout=new QVBoxLayout(body);layout->setContentsMargins(4,2,4,2);layout->setSpacing(0);
        console=new CommandConsole(body);layout->addWidget(console,1);
        console->completionEnabled=[this]{return enabled&&!SurfaceController::instance().active()&&!EditController::instance().active()&&!om9_curve_active()&&!CoreRebuild::instance().active()&&!CoreDistance::instance().active()&&!CorePictureFrame::instance().active()&&!CoreViewControls::instance().toolActive();};
        console->completionAvailable=[](const QString& name){for(std::size_t i=0;i<om9_command_count();++i)if(name.compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0)return om9NativeCommandAvailable(i);return false;};
        console->accepted=[this]{acceptInput();};
        console->cancelled=[this]{SurfaceController::instance().cancel();EditController::instance().cancel();console->setInputText({});historyPosition=-1;CoreDistance::instance().cancel();CorePictureFrame::instance().cancel();cancel();};
        console->recalled=[this](int direction){
            if(inputHistory.isEmpty())return;
            if(historyPosition<0){if(direction>0)return;historyDraft=console->inputText();historyPosition=inputHistory.size();}
            historyPosition=std::clamp(historyPosition+direction,0,int(inputHistory.size()));console->setInputText(historyPosition==inputHistory.size()?historyDraft:inputHistory[historyPosition]);
        };
        const auto font=QFontDatabase::systemFont(QFontDatabase::FixedFont);body->setFont(font);
        QPalette palette=dock->palette();palette.setColor(QPalette::Window,QColor(130,180,140));palette.setColor(QPalette::Base,QColor(130,180,140));palette.setColor(QPalette::Text,Qt::black);palette.setColor(QPalette::WindowText,Qt::black);dock->setPalette(palette);body->setAutoFillBackground(true);
        dock->setStyleSheet("QPlainTextEdit#OM9CommandTranscript { background:#82b48c; color:black; border:none; }");
        dock->setWidget(body);window->addDockWidget(Qt::TopDockWidgetArea,dock);window->resizeDocks({dock},{console->sizeHint().height()+4},Qt::Vertical);
    }
    dock->show();
}
void CurveController::deactivate(){enabled=false;cancel();if(dock)dock->hide();}
bool CurveController::available(std::size_t index)const {
    if(QString::fromUtf8(om9_command_id(index))=="PointsOn")return enabled&&modelingCurveEditorAvailable();
    if(QString::fromUtf8(om9_command_id(index))=="Join")return enabled&&modelingCurveJoinAvailable();
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    return enabled&&isCurveCommand(index)&&doc&&gui&&!gui->getInEdit()&&dynamic_cast<Gui::View3DInventor*>(gui->getActiveView())&&om9AlterDocument(doc);
}
bool CurveController::validDocument()const {auto* doc=App::GetApplication().getActiveDocument();return !document.empty()&&doc&&documentIdentity==doc&&document==doc->getName()&&available(command);}
bool CurveController::start(std::size_t index) {
    if(available(index)){SurfaceController::instance().cancel();EditController::instance().cancel();}
    if(!available(index))return false;CoreDistance::instance().cancel();CorePictureFrame::instance().cancel();CoreViewControls::instance().cancel();cancel();command=index;documentIdentity=App::GetApplication().getActiveDocument();document=App::GetApplication().getActiveDocument()->getName();
    if(CoreRebuild::handles(index))return CoreRebuild::instance().start(index);
    if(QString::fromUtf8(om9_command_id(index))=="PointsOn")return startModelingCurveEditor();
    if(QString::fromUtf8(om9_command_id(index))=="Join"){const bool ok=startModelingCurveJoin();om9_sidebar_record_execution(index,ok);refresh();return ok;}
    const bool ok=om9_curve_start(om9_command_id(index));if(ok)updateInputFrame();refresh();qApp->installEventFilter(this);CoreMouse::instance().prioritize();return ok;
}
void CurveController::cancel(){CoreRebuild::instance().cancel();clearPreview();om9_curve_cancel();document.clear();documentIdentity=nullptr;refresh();}
void CurveController::acceptInput(){submit(console?console->takeInput():QString());}
bool CurveController::pendingInput()const{return console&&!console->inputText().trimmed().isEmpty();}
void CurveController::logMessage(const QString& message){if(console)console->logMessage(message);}
void CurveController::setPrompt(const QString& message){if(console)console->setPrompt(message);}
void CurveController::clearPreview(){
    for(const auto& [owner,node]:previewOwners){if(owner->findChild(node)>=0)owner->removeChild(node);owner->unref();}
    previewOwners.clear();
}
void CurveController::updatePreview(const double* hover){
    clearPreview();const auto count=om9_curve_preview_count();
    const bool interpolated=QString::fromUtf8(om9_command_id(command))=="InterpCrv";
    if(!count||!om9_curve_active()||(!interpolated&&QString::fromUtf8(om9_command_id(command))!="Polyline"))return;
    auto* gui=Gui::Application::Instance->activeDocument();if(!gui)return;
    std::vector<SbVec3f> picked;for(std::size_t i=0;i<count;++i)picked.emplace_back(float(om9_curve_preview_coordinate(i,0)),float(om9_curve_preview_coordinate(i,1)),float(om9_curve_preview_coordinate(i,2)));
    std::vector<SbVec3f> line=picked;
    if(hover)line.emplace_back(float(hover[0]),float(hover[1]),float(hover[2]));
    if(interpolated){
        line.clear();
        if(om9_curve_preview_spline_closed(hover,false))for(int i=0;i<=128;++i)line.emplace_back(float(om9_spline_value(i/128.,0)),float(om9_spline_value(i/128.,1)),float(om9_spline_value(i/128.,2)));
    }
    for(auto* base:gui->getMDIViews()){
        auto* view=dynamic_cast<Gui::View3DInventor*>(base);if(!view)continue;
        auto* root=dynamic_cast<SoSeparator*>(view->getViewer()->getSceneGraph());if(!root)continue;
        SoSeparator* owner=root;for(int i=0;i<root->getNumChildren();++i)if(auto* selection=dynamic_cast<Gui::SoFCUnifiedSelection*>(root->getChild(i))){owner=selection;break;}
        owner->ref();auto* wrapper=new SoSeparator;owner->addChild(wrapper);previewOwners.emplace_back(owner,wrapper);
        auto* skip=new Gui::SoSkipBoundingGroup;skip->mode=Gui::SoSkipBoundingGroup::INCLUDE_BBOX;wrapper->addChild(skip);
        auto* content=new SoSeparator;content->setName("OM9CurvePreview");skip->addChild(content);
        auto* pick=new SoPickStyle;pick->style=SoPickStyle::UNPICKABLE;content->addChild(pick);
        auto* color=new SoBaseColor;color->rgb.setValue(0,130.f/255.f,85.f/255.f);content->addChild(color);
        auto* coords=new SoCoordinate3;coords->setName("OM9CurvePreviewLine");if(!line.empty())coords->point.setValues(0,int(line.size()),line.data());content->addChild(coords);
        auto* style=new SoDrawStyle;style->lineWidth=1;content->addChild(style);
        auto* lines=new SoLineSet;lines->numVertices.set1Value(0,int(line.size()));content->addChild(lines);
        auto* markers=new SoSeparator;content->addChild(markers);
        auto* points=new SoCoordinate3;points->setName("OM9CurvePreviewPoints");points->point.setValues(0,int(count),picked.data());markers->addChild(points);
        auto* outer=new SoDrawStyle;outer->pointSize=7;markers->addChild(outer);markers->addChild(new SoPointSet);
        auto* white=new SoBaseColor;white->rgb.setValue(0.8f,1,0.9f);markers->addChild(white);
        auto* inner=new SoDrawStyle;inner->pointSize=3;markers->addChild(inner);markers->addChild(new SoPointSet);
        view->getViewer()->redraw();
    }
}
void CurveController::refresh(){if(CoreRebuild::instance().active()||CoreDistance::instance().active()||CorePictureFrame::instance().active())return;char message[2048]={};om9_curve_message(message,sizeof(message));setPrompt(QString::fromUtf8(message));if(om9_curve_active())logMessage(QString::fromUtf8(message));}
void CurveController::submit(const QString& text) {
    if(!enabled)return;
    const auto input=text.trimmed();
    if(!input.isEmpty()){if(inputHistory.isEmpty()||inputHistory.back()!=input)inputHistory.append(input);if(inputHistory.size()>1000)inputHistory.removeFirst();}
    historyPosition=-1;historyDraft.clear();
    CoreKeyboard::instance().record(input);
    if(LayerController::submit(input))return;
    if(CoreKeyboard::instance().submit(input))return;
    if(CoreSnaps::submit(input))return;
    if(SurfaceController::instance().active()){SurfaceController::instance().submit(input);return;}
    if(EditController::instance().active()){EditController::instance().submit(input);return;}
    for(std::size_t i=0;i<om9_command_count();++i){if(SurfaceController::matches(i,input)){SurfaceController::instance().start(i,input);return;}if(EditController::matches(i,input)&&(om9_edit_kind(input.toUtf8().constData())!=1||phase3SurfaceJoinSelection())){EditController::instance().start(i,true);return;}}
    if(CoreRebuild::instance().active()){CoreRebuild::instance().submit(input);return;}
    for(std::size_t i=0;i<om9_command_count();++i)if(om9_3dm_operation(i)&&input.compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0){cancel();om9_sidebar_record_execution(i,om9ExecuteNativeCommand(i));refresh();return;}
    if(input.compare("Distance",Qt::CaseInsensitive)==0||input.compare("Angle",Qt::CaseInsensitive)==0){for(std::size_t i=0;i<om9_command_count();++i)if(CoreDistance::handles(i)&&input.compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0){CoreDistance::instance().start(i);return;}}
    if(CoreDistance::instance().active()){
        for(std::size_t i=0;i<om9_command_count();++i)if(isCurveCommand(i)&&input.compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0){start(i);return;}
        CoreDistance::instance().submit(input);return;
    }
    if(input.compare("PictureFrame",Qt::CaseInsensitive)==0){for(std::size_t i=0;i<om9_command_count();++i)if(CorePictureFrame::handles(i)){CorePictureFrame::instance().start(i);return;}}
    if(input.left(12).compare("ViewportTabs",Qt::CaseInsensitive)==0&&(input.size()==12||input[12].isSpace())) {
        for(std::size_t i=0;i<om9_command_count();++i)if(QString::fromUtf8(om9_command_id(i))=="ViewportTabs"){om9_sidebar_record_execution(i,CoreWorkspace::instance().executeTabs(input.mid(12).trimmed()));refresh();return;}
    }
    for(std::size_t i=0;i<om9_command_count();++i)if(CoreNotes::handles(i)&&(text.trimmed().compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0||text.trimmed().compare(QString::fromUtf8(CoreNotes::alias(i)),Qt::CaseInsensitive)==0)){om9_sidebar_record_execution(i,CoreNotes::execute(i));refresh();return;}
    if(input.compare("ViewCaptureToFile",Qt::CaseInsensitive)==0) {
        for(std::size_t i=0;i<om9_command_count();++i)if(QString::fromUtf8(om9_command_id(i))=="ViewCaptureToFile"){om9_sidebar_record_execution(i,CoreViewControls::instance().execute(i));refresh();return;}
    }
    if(CorePictureFrame::instance().active()) {
        for(std::size_t i=0;i<om9_command_count();++i)if((isCurveCommand(i)||CoreViewControls::handles(i))&&input.compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0) {
            if(isCurveCommand(i))start(i);else om9_sidebar_record_execution(i,CoreViewControls::instance().execute(i));return;
        }
        CorePictureFrame::instance().submit(input);return;
    }
    if(om9_curve_active()) {
        if(!validDocument()){cancel();return;}updateInputFrame();result(om9_curve_input(text.toUtf8().constData()));return;
    }
    for(std::size_t i=0;i<om9_command_count();++i)if((isCurveCommand(i)||CoreWorkspace::handles(i)||CoreViewControls::handles(i))&&text.trimmed().compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0){if(CoreViewControls::handles(i)){om9_sidebar_record_execution(i,CoreViewControls::instance().execute(i));}else if(CoreWorkspace::handles(i)){om9_sidebar_record_execution(i,CoreWorkspace::instance().execute(i));refresh();}else if(!start(i))setPrompt("Open an editable document with a 3D view first");return;}
    if(input.isEmpty()){
        if(om9_sidebar_history_count()){const auto previous=om9_sidebar_history_command(0);if(om9NativeCommandAvailable(previous)){logMessage("Command: "+QString::fromUtf8(om9_command_id(previous)));om9_sidebar_record_execution(previous,om9ExecuteNativeCommand(previous));return;}}
        refresh();return;
    }
    setPrompt("Unknown or unsupported command. Available: Line, Polyline, InterpCrv, Rebuild");logMessage(console->property("om9Prompt").toString());
}
void CurveController::result(unsigned int effect) {
    refresh();
    updatePreview();
    if(effect==2) {
        auto* doc=App::GetApplication().getActiveDocument();
        if(!doc || document!=doc->getName() || !available(command)){cancel();return;}
        try {commit(*doc,om9_command_id(command));om9_sidebar_record_execution(command,true);logMessage(QString::fromUtf8(om9_command_id(command))+" created");document.clear();}
        catch(const Base::Exception& error){cancel();setPrompt(QString::fromUtf8(error.what()));logMessage(console->property("om9Prompt").toString());Base::Console().error("OpenMatrix9 Curve: %s\n",error.what());}
        catch(const std::exception& error){cancel();setPrompt(QString::fromUtf8(error.what()));logMessage(console->property("om9Prompt").toString());Base::Console().error("OpenMatrix9 Curve: %s\n",error.what());}
    }else if(effect==3)document.clear();
}
bool CurveController::eventFilter(QObject* watched,QEvent* event) {
    if(event->type()==QEvent::MouseButtonRelease&&releaseTarget==watched&&static_cast<QMouseEvent*>(event)->button()==Qt::LeftButton){releaseTarget.clear();return true;}
    if(!enabled||Gui::Application::Instance->isClosing())return false;
    if(event->type()==QEvent::KeyPress&&CoreKeyboard::inputContext(watched)){
        auto* key=static_cast<QKeyEvent*>(event);
        auto* gui=Gui::Application::Instance->activeDocument();auto* view=gui?dynamic_cast<Gui::View3DInventor*>(gui->getActiveView()):nullptr;auto* widget=qobject_cast<QWidget*>(watched);
        const bool viewport=view&&widget&&!gui->getInEdit()&&!view->getViewer()->isEditing()&&!Gui::Control().activeDialog()&&(widget==view->getViewer()||view->getViewer()->isAncestorOf(widget));
        if(viewport&&!(key->modifiers()&(Qt::ControlModifier|Qt::AltModifier|Qt::MetaModifier))&&!key->text().isEmpty()&&key->text().front().isPrint()){console->setFocus();console->insertInput(key->text());return true;}
        if(viewport&&!om9_curve_active()&&!CoreDistance::instance().active()&&!CorePictureFrame::instance().active()&&(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter)){acceptInput();return true;}
    }
    if(!om9_curve_active())return false;
    if(event->type()==QEvent::KeyPress) {
        auto* key=static_cast<QKeyEvent*>(event);
        if(key->key()==Qt::Key_Escape&&watched!=console){cancel();return true;}
        if(key->key()==Qt::Key_F4&&CoreKeyboard::pointKeyAllowed(watched,key)) {
            if(!validDocument()){cancel();return true;}
            auto* view=dynamic_cast<Gui::View3DInventor*>(Gui::Application::Instance->activeDocument()->getActiveView());
            const auto origin=CoreWorkspace::instance().plane(view).getPosition();
            result(om9_curve_point(origin.x,origin.y,origin.z));return true;
        }
        if(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter) {
            auto* gui=Gui::Application::Instance->activeDocument();
            auto* view=gui?dynamic_cast<Gui::View3DInventor*>(gui->getActiveView()):nullptr;
            auto* widget=qobject_cast<QWidget*>(watched);
            if(view&&widget&&view->getViewer()->isAncestorOf(widget)){acceptInput();return true;}
        }
    }
    const bool hovering=event->type()==QEvent::MouseMove;
    if(!hovering&&event->type()!=QEvent::MouseButtonPress)return false;
    auto* mouse=static_cast<QMouseEvent*>(event);if(!hovering&&mouse->button()!=Qt::LeftButton)return false;
    if(!validDocument()){cancel();return false;}
    auto* view=dynamic_cast<Gui::View3DInventor*>(Gui::Application::Instance->activeDocument()->getActiveView());auto* viewer=view->getViewer();auto* widget=qobject_cast<QWidget*>(watched);
    if(!widget||!viewer->isAncestorOf(widget))return false;
    if(!hovering)releaseTarget=watched;
    const auto pos=viewer->viewport()->mapFrom(widget,mouse->position().toPoint());
    // C-plane belongs to the native viewport, independently of its camera orientation.
    try {
        updateInputFrame();
        Base::Vector3d snapped;
        const auto deliver=[&](double x,double y,double z){
            const bool shift=mouse->modifiers().testFlag(Qt::ShiftModifier);
            if(hovering){double p[3];if(om9_curve_preview_point(x,y,z,shift,p))updatePreview(p);else updatePreview();}
            else result(om9_curve_mouse_point(x,y,z,shift));
        };
        if(CoreSnaps::pick(view,pos,snapped)){deliver(snapped.x,snapped.y,snapped.z);return !hovering;}
        const auto pixel=viewer->fromQPoint(pos);
        SbVec3f nearPoint,farPoint;viewer->projectPointToLine(pixel,nearPoint,farPoint);
        const auto direction=farPoint-nearPoint;
        const auto plane=CoreWorkspace::instance().plane(view);
        const auto normal=plane.getRotation().multVec(Base::Vector3d(0,0,1));
        const double dot=direction[0]*normal.x+direction[1]*normal.y+direction[2]*normal.z;
        if(std::abs(dot)<=1e-6*direction.length())throw Base::RuntimeError("Ray parallel to construction plane");
        const auto p=viewer->getPointOnXYPlaneOfPlacement(pixel,plane);
        deliver(p[0],p[1],p[2]);
    }catch(const Base::Exception&) {
        if(hovering){updatePreview();return false;}
        setPrompt("This view cannot intersect its construction plane. Change view or enter coordinates; the command remains active.");
        logMessage(console->property("om9Prompt").toString());
    }
    return !hovering;
}
}
