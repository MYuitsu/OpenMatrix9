#include "CurveController.h"
#include "CoreUnits.h"
#include "CommandConsole.h"
#include "RustBridge.h"
#include "NativeCommands.h"
#include "CoreWorkspace.h"
#include "CoreViewControls.h"
#include "CoreNotes.h"
#include "CorePictureFrame.h"
#include "CoreDistance.h"
#include "CoreSnaps.h"
#include "CoreKeyboard.h"
#include "CoreMouse.h"
#include "CurveGeometry.h"
#include "CurveCircle.h"
#include "CurveEllipse.h"
#include "CoreLayers.h"
#include "CoreRebuild.h"
#include "SurfaceController.h"
#include "EditController.h"
#include "HistoryController.h"
#include "CageController.h"
#include "SolidController.h"
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
#include <Inventor/nodes/SoCamera.h>
#include <Inventor/SoRenderManager.h>
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
#include <QToolButton>
#include <cmath>
#include <algorithm>
#include <stdexcept>

namespace {
void updateInputFrame() {
    auto* doc=Gui::Application::Instance->activeDocument();
    auto* view=doc?dynamic_cast<Gui::View3DInventor*>(doc->getActiveView()):nullptr;
    const auto plane=OpenMatrix9Gui::CoreWorkspace::instance().plane(view);const auto origin=plane.getPosition();
    const auto x=plane.getRotation().multVec(Base::Vector3d(1,0,0)),y=plane.getRotation().multVec(Base::Vector3d(0,1,0)),z=plane.getRotation().multVec(Base::Vector3d(0,0,1));
    om9_curve_frame(origin.x,origin.y,origin.z,x.x,x.y,x.z,y.x,y.y,y.z,z.x,z.y,z.z);
}
struct PyRef {
    PyObject* value;
    explicit PyRef(PyObject* value):value(value){if(!value){PyErr_Print();throw std::runtime_error("Native Part adapter failed");}}
    ~PyRef(){Py_DECREF(value);}
    PyRef(const PyRef&)=delete;PyRef& operator=(const PyRef&)=delete;
};
// Typed native Python API adapter to the host's Part/OpenCascade bindings.
// No command text is evaluated as Python code.
void commit(App::Document& doc,const char* label) {
    if(std::string(label)=="Ellipse"){OpenMatrix9Gui::commitEllipse(doc);return;}
    Base::PyGILStateLocker lock;
    PyRef app(PyImport_ImportModule("FreeCAD")),part(PyImport_ImportModule("Part"));
    PyRef vector(PyObject_GetAttrString(app.value,"Vector"));
    PyRef points(PyList_New(0));
    const auto count=om9_curve_count();if(count<2 || count>4097)throw std::runtime_error("Invalid Curve output");
    for(std::size_t i=0;i<count;++i) {
        double x=om9_curve_coordinate(i,0),y=om9_curve_coordinate(i,1),z=om9_curve_coordinate(i,2);
        if(!std::isfinite(x)||!std::isfinite(y)||!std::isfinite(z))throw std::runtime_error("Nonfinite Curve output");
        PyRef point(PyObject_CallFunction(vector.value,"ddd",x,y,z));
        if(PyList_Append(points.value,point.value)<0)throw std::runtime_error("Cannot append Curve point");
    }
    PyRef shape([&]()->PyObject* {
        if(std::string(label)=="Circle") {
            OpenMatrix9Gui::verifyCircleReferences(doc);
            if(om9_circle_deformable()) {
                if(!om9_curve_spline_publish())throw std::runtime_error("No committed deformable Circle");
                auto spline=OpenMatrix9Gui::publishedSplineShape();
                PyRef edges(PyList_New(0));if(PyList_Append(edges.value,spline.value)<0)throw std::runtime_error("Cannot append deformable Circle edge");
                return PyObject_CallMethod(part.value,"Wire","O",edges.value);
            }
            double circle[7];if(!om9_curve_circle_plan(circle))throw std::runtime_error("No committed analytic Circle");
            PyRef center(PyObject_CallFunction(vector.value,"ddd",circle[0],circle[1],circle[2]));
            PyRef normal(PyObject_CallFunction(vector.value,"ddd",circle[3],circle[4],circle[5]));
            PyRef edge(PyObject_CallMethod(part.value,"makeCircle","dOO",circle[6],center.value,normal.value));
            PyRef edges(PyList_New(0));if(PyList_Append(edges.value,edge.value)<0)throw std::runtime_error("Cannot append Circle edge");
            return PyObject_CallMethod(part.value,"Wire","O",edges.value);
        }
        if(std::string(label)=="InterpCrv") {
            if(!om9_curve_spline_publish())throw std::runtime_error("No committed interpolated spline");
            auto spline=OpenMatrix9Gui::publishedSplineShape();return Py_NewRef(spline.value);
        }
        return PyObject_CallMethod(part.value,"makePolygon","O",points.value);
    }());
    PyRef valid(PyObject_CallMethod(shape.value,"isValid",nullptr));
    if(PyObject_IsTrue(valid.value)!=1)throw std::runtime_error("Part produced invalid geometry");
    const auto circleLayer=std::string(label)=="Circle"?OpenMatrix9Gui::CoreLayers::captureCircle(doc):OpenMatrix9Gui::CircleLayerSnapshot{};
    doc.openTransaction(std::string(label));
    try {
        bool historyCreated=false;
        PyRef pyDoc(doc.getPyObject());PyRef object([&]()->PyObject*{
            if(std::string(label)=="Circle")if(auto* history=OpenMatrix9Gui::circleHistoryObject(doc,shape.value)){historyCreated=true;return history->getPyObject();}
            return PyObject_CallMethod(pyDoc.value,"addObject","ss","Part::Feature",label);
        }());
        if(!historyCreated&&PyObject_SetAttrString(object.value,"Shape",shape.value)<0)throw std::runtime_error("Cannot assign Curve shape");
        if(std::string(label)=="Rectangle"||std::string(label)=="Circle") {
            PyRef property(PyObject_CallMethod(object.value,"addProperty","sss","App::PropertyString","OM9FeatureId","OpenMatrix9"));
            PyRef id(PyUnicode_FromString(std::string(label)=="Circle"?"OM9-CURVE-005":"OM9-CURVE-004"));
            if(PyObject_SetAttrString(object.value,"OM9FeatureId",id.value)<0)throw std::runtime_error("Cannot assign Curve feature ID");
        }
        if(std::string(label)=="Circle") {
            for(const auto& parameter:std::vector<std::pair<const char*,double>>{{"CircleFitDeviation",om9_circle_deviation(false)},{"CircleApproxDeviation",om9_circle_deviation(true)}}){
                PyRef property(PyObject_CallMethod(object.value,"addProperty","sss","App::PropertyLength",parameter.first,"OpenMatrix9"));PyRef value(PyFloat_FromDouble(parameter.second));
                if(PyObject_SetAttrString(object.value,parameter.first,value.value)<0)throw std::runtime_error("Cannot persist Circle deviation");
            }
            PyRef property(PyObject_CallMethod(object.value,"addProperty","sss","App::PropertyBool","CircleDeformable","OpenMatrix9"));
            if(PyObject_SetAttrString(object.value,"CircleDeformable",om9_circle_deformable()?Py_True:Py_False)<0)throw std::runtime_error("Cannot persist Circle output kind");
        }
        // New Curve output must remain visible on the selected black Matrix canvas.
        // Existing objects and their layer/user colors are left intact.
        PyRef view(PyObject_GetAttrString(object.value,"ViewObject"));PyRef color(Py_BuildValue("(ddd)",0.0,130.0/255.0,85.0/255.0));
        if(PyObject_SetAttrString(view.value,"LineColor",color.value)<0)throw std::runtime_error("Cannot assign Curve line color");
        if(std::string(label)=="Circle"){
            PyRef name(PyObject_GetAttrString(object.value,"Name"));const char* nativeName=PyUnicode_AsUTF8(name.value);
            auto* native=nativeName?doc.getObject(nativeName):nullptr;if(!native)throw std::runtime_error("Circle output identity missing");
            OpenMatrix9Gui::CoreLayers::applyCircle(doc,*native,circleLayer);
        }
        PyRef outputName(PyObject_GetAttrString(object.value,"Name"));
        auto* nativeOutput=doc.getObject(PyUnicode_AsUTF8(outputName.value));
        if(!nativeOutput)throw std::runtime_error("Curve output identity missing");
        OpenMatrix9Gui::addUnitContextProvenance(*nativeOutput,OpenMatrix9Gui::unitContextSnapshot(doc));
        doc.recompute();doc.commitTransaction();
    }catch(...){doc.abortTransaction();throw;}
}
}
namespace OpenMatrix9Gui {
bool isCurveCommand(std::size_t index) {const char* id=om9_command_id(index);return id && (std::string(id)=="Line" || std::string(id)=="Polyline" || std::string(id)=="InterpCrv" || std::string(id)=="Rebuild" || std::string(id)=="Rectangle" || std::string(id)=="Circle" || std::string(id)=="Ellipse");}
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
        console->completionEnabled=[this]{return enabled&&!CageController::instance().active()&&!HistoryController::instance().active()&&!om9_curve_active()&&!CoreRebuild::instance().active()&&!EditController::instance().active()&&!SolidController::instance().active()&&!SurfaceController::instance().active()&&!CoreDistance::instance().active()&&!CorePictureFrame::instance().active()&&!CoreViewControls::instance().toolActive();};
        console->completionAvailable=[](const QString& name){for(std::size_t i=0;i<om9_command_count();++i)if(CageController::matches(i,name)||HistoryController::matches(i,name)||EditController::matches(i,name)||SolidController::matches(i,name)||SurfaceController::matches(i,name)||name.compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0)return om9NativeCommandAvailable(i);return false;};
        console->accepted=[this]{acceptInput();};
        console->cancelled=[this]{cancelInput();};
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
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    return enabled&&isCurveCommand(index)&&doc&&gui&&!gui->getInEdit()&&dynamic_cast<Gui::View3DInventor*>(gui->getActiveView())&&Gui::Control().isAllowedAlterDocument(doc);
}
bool CurveController::validDocument()const {
    auto* doc=App::GetApplication().getActiveDocument();
    if(document.empty()||!doc||documentIdentity!=doc||document!=doc->getName()||!available(command))return false;
    try {return unitContextSnapshot(*doc)==unitsSnapshot;}catch(...){return false;}
}
bool CurveController::start(std::size_t index) {
    if(!available(index))return false;CoreDistance::instance().cancel();CorePictureFrame::instance().cancel();CoreViewControls::instance().cancel();cancel();command=index;documentIdentity=App::GetApplication().getActiveDocument();document=App::GetApplication().getActiveDocument()->getName();
    if(CoreRebuild::handles(index))return CoreRebuild::instance().start(index);
    try {
        auto& doc=*App::GetApplication().getActiveDocument();unitsSnapshot=unitContextSnapshot(doc);
        const bool ok=om9_curve_start(om9_command_id(index))&&om9_curve_input_units(unitInputScale(doc));
        if(!ok){cancel();return false;}updateInputFrame();refresh();qApp->installEventFilter(this);CoreMouse::instance().prioritize();return true;
    }catch(const std::exception& error){cancel();setPrompt(QString::fromUtf8(error.what()));return false;}
}
void CurveController::cancel(){if(om9_curve_active()||!document.empty())CoreSnaps::clearTransient();unitsSnapshot.clear();CageController::instance().cancel();HistoryController::instance().cancel();CoreRebuild::instance().cancel();EditController::instance().cancel();SurfaceController::instance().cancel();SolidController::instance().cancel();clearPreview();clearCircleReferences();clearEllipseReferences();om9_curve_cancel();document.clear();documentIdentity=nullptr;refresh();}
void CurveController::cancelInput(){if(console)console->setInputText({});historyPosition=-1;CoreDistance::instance().cancel();CorePictureFrame::instance().cancel();cancel();}
void CurveController::acceptInput(){submit(console?console->takeInput():QString());}
bool CurveController::pendingInput()const{return console&&!console->inputText().trimmed().isEmpty();}
void CurveController::logMessage(const QString& message){if(console)console->logMessage(message);}
void CurveController::setPrompt(const QString& message){if(console)console->setPrompt(message);}
void CurveController::clearPreview(){
    for(const auto& [owner,node]:previewOwners){if(owner->findChild(node)>=0)owner->removeChild(node);owner->unref();}
    previewOwners.clear();
}
void CurveController::updatePreview(const double* hover,bool close,bool square){
    clearPreview();const auto count=om9_curve_preview_count();
    const bool interpolated=QString::fromUtf8(om9_command_id(command))=="InterpCrv";
    const bool rectangle=QString::fromUtf8(om9_command_id(command))=="Rectangle";
    const bool circle=QString::fromUtf8(om9_command_id(command))=="Circle";
    const bool ellipse=QString::fromUtf8(om9_command_id(command))=="Ellipse";
    const bool contact=hover&&((circle&&om9_circle_reference_mode()==3)||(ellipse&&om9_ellipse_reference_mode()==2));
    if((!count&&!contact)||!om9_curve_active()||(!circle&&!ellipse&&!rectangle&&!interpolated&&QString::fromUtf8(om9_command_id(command))!="Polyline"))return;
    auto* gui=Gui::Application::Instance->activeDocument();if(!gui)return;
    std::vector<SbVec3f> picked;for(std::size_t i=0;i<count;++i)picked.emplace_back(float(om9_curve_preview_coordinate(i,0)),float(om9_curve_preview_coordinate(i,1)),float(om9_curve_preview_coordinate(i,2)));
    if(contact)picked.emplace_back(float(hover[0]),float(hover[1]),float(hover[2]));
    std::vector<SbVec3f> line=picked;
    if(hover)line.emplace_back(float(hover[0]),float(hover[1]),float(hover[2]));
    if(rectangle||circle||ellipse){
        line.clear();const auto outline=om9_curve_outline(hover,square);
        for(std::size_t i=0;i<outline;++i)line.emplace_back(float(om9_curve_outline_coordinate(i,0)),float(om9_curve_outline_coordinate(i,1)),float(om9_curve_outline_coordinate(i,2)));
        if(circle&&hover){
            double measures[4];
            if(om9_circle_hover_measure(hover,measures))setPrompt(QString("Circle Radius=%1 mm; Diameter=%2 mm; Circumference=%3 mm; Area=%4 mm2 (click to commit)").arg(measures[0],0,'g',10).arg(measures[1],0,'g',10).arg(measures[2],0,'g',10).arg(measures[3],0,'g',10));
            else {char message[2048]={};om9_curve_message(message,sizeof(message));setPrompt(QString::fromUtf8(message));}
        }
    }
    if(interpolated){
        if(om9_curve_preview_spline_closed(hover,close)){
            line.clear();for(int i=0;i<=128;++i)line.emplace_back(float(om9_spline_value(i/128.,0)),float(om9_spline_value(i/128.,1)),float(om9_spline_value(i/128.,2)));
        }else line.clear();
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
        auto* points=new SoCoordinate3;points->setName("OM9CurvePreviewPoints");points->point.setValues(0,int(picked.size()),picked.data());markers->addChild(points);
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
    QString layerError;if(CoreLayers::submit(input,layerError)){if(!layerError.isEmpty()){setPrompt(layerError);logMessage(layerError);}else if(om9_curve_active())refresh();else setPrompt("Current layer updated");return;}
    if(SolidController::instance().active()){SolidController::instance().submit(input);return;}
    for(std::size_t i=0;i<om9_command_count();++i)if(SolidController::matches(i,input)){SolidController::instance().start(i);return;}
    if(SurfaceController::instance().active()){SurfaceController::instance().submit(input);return;}
    if(EditController::instance().active()){EditController::instance().submit(input);return;}
    if(HistoryController::instance().active()){HistoryController::instance().submit(input);return;}
    if(CageController::instance().active()){CageController::instance().submit(input);return;}
    for(std::size_t i=0;i<om9_command_count();++i)if(HistoryController::matches(i,input)){HistoryController::instance().start(i);return;}
    for(std::size_t i=0;i<om9_command_count();++i)if(CageController::matches(i,input)){CageController::instance().start(i);return;}
    for(std::size_t i=0;i<om9_command_count();++i)if(EditController::matches(i,input)){EditController::instance().start(i,true);return;}
    for(std::size_t i=0;i<om9_command_count();++i)if(SurfaceController::matches(i,input)){SurfaceController::instance().start(i,input);return;}
    CoreKeyboard::instance().record(input);
    if(CoreKeyboard::instance().submit(input))return;
    if(CoreSnaps::submit(input))return;
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
        if(!validDocument()){cancel();return;}
        try {
            updateInputFrame();
            const auto beforeNativeInput=om9_curve_preview_count();
            if(QString::fromUtf8(om9_command_id(command))=="Circle")if(auto native=circleNativeInput(*App::GetApplication().getActiveDocument(),text)){CoreSnaps::acceptedPoint(*native!=0&&*native!=3&&om9_curve_preview_count()>beforeNativeInput);result(*native);return;}
            if(QString::fromUtf8(om9_command_id(command))=="Ellipse")if(auto native=ellipseNativeInput(*App::GetApplication().getActiveDocument(),text)){CoreSnaps::acceptedPoint(*native!=0&&*native!=3&&om9_curve_preview_count()>beforeNativeInput);result(*native);return;}
            const auto before=om9_curve_preview_count();const auto effect=om9_curve_input(text.toUtf8().constData());CoreSnaps::acceptedPoint(effect!=0&&effect!=3&&om9_curve_preview_count()>before);result(effect);
        }catch(const std::exception& error){clearPreview();setPrompt(QString::fromUtf8(error.what()));logMessage(console->property("om9Prompt").toString());}
        return;
    }
    for(std::size_t i=0;i<om9_command_count();++i)if((isCurveCommand(i)||CoreWorkspace::handles(i)||CoreViewControls::handles(i))&&text.trimmed().compare(QString::fromUtf8(om9_command_id(i)),Qt::CaseInsensitive)==0){if(CoreViewControls::handles(i)){om9_sidebar_record_execution(i,CoreViewControls::instance().execute(i));}else if(CoreWorkspace::handles(i)){om9_sidebar_record_execution(i,CoreWorkspace::instance().execute(i));refresh();}else if(!available(i))setPrompt("Open an editable document with a 3D view first");else start(i);return;}
    if(input.isEmpty()){
        const auto previous=om9_command_repeat_candidate();if(previous<om9_command_count()&&om9NativeCommandAvailable(previous)){logMessage("Command: "+QString::fromUtf8(om9_command_id(previous)));om9ExecuteNativeCommand(previous);return;}
        refresh();return;
    }
    setPrompt("Unknown or unsupported command. Available: Line, Polyline, InterpCrv, Rebuild, Rectangle, Circle, Ellipse");logMessage(console->property("om9Prompt").toString());
}
void CurveController::result(unsigned int effect) {
    if(QString::fromUtf8(om9_command_id(command))=="Ellipse")ellipseNativeResult(effect);
    if(QString::fromUtf8(om9_command_id(command))=="Circle"&&App::GetApplication().getActiveDocument()) {
        try {effect=circleNativeResult(*App::GetApplication().getActiveDocument(),effect);}
        catch(const std::exception& error){clearPreview();setPrompt(QString::fromUtf8(error.what()));logMessage(console->property("om9Prompt").toString());return;}
    }
    refresh();
    updatePreview();
    if(effect==2) {
        auto* doc=App::GetApplication().getActiveDocument();
        if(!validDocument()){cancel();setPrompt("Document or unit context changed; restart the command");return;}
        try {commit(*doc,om9_command_id(command));CoreSnaps::clearTransient();unitsSnapshot.clear();clearCircleReferences();clearEllipseReferences();om9_sidebar_record_execution(command,true);logMessage(QString::fromUtf8(om9_command_id(command))+" created");document.clear();documentIdentity=nullptr;}
        catch(const Base::Exception& error){cancel();setPrompt(QString::fromUtf8(error.what()));logMessage(console->property("om9Prompt").toString());Base::Console().error("OpenMatrix9 Curve: %s\n",error.what());}
        catch(const std::exception& error){cancel();setPrompt(QString::fromUtf8(error.what()));logMessage(console->property("om9Prompt").toString());Base::Console().error("OpenMatrix9 Curve: %s\n",error.what());}
    }else if(effect==3){CoreSnaps::clearTransient();unitsSnapshot.clear();clearEllipseReferences();document.clear();}
}
bool CurveController::eventFilter(QObject* watched,QEvent* event) {
    if(enabled)if(auto* button=qobject_cast<QToolButton*>(watched);button&&button->property("om9Command").isValid()) {
        const auto index=std::size_t(button->property("om9Command").toULongLong());
        const auto id=QString::fromUtf8(om9_command_id(index));
        if(id=="Circle"||id=="Ellipse") {
            if(event->type()==QEvent::ContextMenu)return true;
            if(event->type()==QEvent::MouseButtonPress&&static_cast<QMouseEvent*>(event)->button()==Qt::RightButton&&button->isEnabled()&&available(index)){start(index);result(om9_curve_input(id=="Ellipse"?"Diameter":"2Point"));return true;}
            if(event->type()==QEvent::MouseButtonRelease&&static_cast<QMouseEvent*>(event)->button()==Qt::RightButton)return true;
        }
    }
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
        if(key->key()==Qt::Key_Escape&&watched!=console&&!QApplication::activePopupWidget()&&!QApplication::activeModalWidget()){cancel();return true;}
        if(key->key()==Qt::Key_F4&&CoreKeyboard::pointKeyAllowed(watched,key)) {
            if(!validDocument()){cancel();return true;}
            auto* view=dynamic_cast<Gui::View3DInventor*>(Gui::Application::Instance->activeDocument()->getActiveView());
            try {
                updateInputFrame();
                const auto origin=CoreWorkspace::instance().plane(view).getPosition();
                const auto effect=om9_curve_point(origin.x,origin.y,origin.z);CoreSnaps::acceptedPoint(effect==1||effect==2);result(effect);
            }catch(const std::exception& error){clearPreview();setPrompt(QString::fromUtf8(error.what()));}
            return true;
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
        const auto beforeNativePick=om9_curve_preview_count();
        if(QString::fromUtf8(om9_command_id(command))=="Circle") {
            if(!hovering){if(auto native=circleNativePick(*App::GetApplication().getActiveDocument(),view,pos)){CoreSnaps::acceptedPoint(*native!=0&&*native!=3&&om9_curve_preview_count()>beforeNativePick);result(*native);return true;}}
            else if(om9_circle_reference_mode()==3) {if(auto p=circleNativeHover(*App::GetApplication().getActiveDocument(),view,pos))updatePreview(p->data());else clearPreview();return false;}
            else if(om9_circle_reference_mode()==1||om9_circle_reference_mode()==2)return false;
            else {Base::PyGILStateLocker lock;verifyCircleReferences(*App::GetApplication().getActiveDocument());}
        }
        if(QString::fromUtf8(om9_command_id(command))=="Ellipse") {
            if(!hovering){if(auto native=ellipseNativePick(*App::GetApplication().getActiveDocument(),view,pos)){CoreSnaps::acceptedPoint(*native!=0&&*native!=3&&om9_curve_preview_count()>beforeNativePick);result(*native);return true;}}
            else if(om9_ellipse_reference_mode()==2){if(auto p=ellipseNativeHover(*App::GetApplication().getActiveDocument(),view,pos))updatePreview(p->data());else clearPreview();return false;}
            else if(om9_ellipse_reference_mode()==1)return false;
            {Base::PyGILStateLocker lock;verifyEllipseReferences(*App::GetApplication().getActiveDocument());}
        }
        // OM9-CURVE-003: close within 10 logical pixels, independently of Osnap.
        if(QString::fromUtf8(om9_command_id(command))=="InterpCrv"&&om9_curve_preview_count()>=3&&!mouse->modifiers().testFlag(Qt::AltModifier)){
            const double first[3]={om9_curve_preview_coordinate(0,0),om9_curve_preview_coordinate(0,1),om9_curve_preview_coordinate(0,2)};
            SbVec3f projected;viewer->getSoRenderManager()->getCamera()->getViewVolume(float(viewer->viewport()->width())/viewer->viewport()->height()).projectToScreen(SbVec3f(float(first[0]),float(first[1]),float(first[2])),projected);
            const double x=projected[0]*(viewer->viewport()->width()-1),y=(1-projected[1])*(viewer->viewport()->height()-1);
            if(std::hypot(x-pos.x(),y-pos.y())<=10&&projected[2]>=0&&projected[2]<=1){if(hovering)updatePreview(first,true);else result(om9_curve_input("Close"));return !hovering;}
        }
        Base::Vector3d snapped;
        const auto deliver=[&](double x,double y,double z){
            const bool shift=mouse->modifiers().testFlag(Qt::ShiftModifier);
            if(hovering&&(QString::fromUtf8(om9_command_id(command))=="Rectangle"||QString::fromUtf8(om9_command_id(command))=="Circle"||QString::fromUtf8(om9_command_id(command))=="Ellipse")){const double p[3]={x,y,z};updatePreview(p,false,shift);}
            else if(hovering){double p[3];if(om9_curve_preview_point(x,y,z,shift,p))updatePreview(p);else updatePreview();}
            else {const auto effect=om9_curve_mouse_point(x,y,z,shift);CoreSnaps::acceptedPoint(effect==1||effect==2);result(effect);}
        };
        if(CoreSnaps::pick(view,pos,snapped)){deliver(snapped.x,snapped.y,snapped.z);return !hovering;}
        const auto pixel=viewer->fromQPoint(pos);
        SbVec3f nearPoint,farPoint;viewer->projectPointToLine(pixel,nearPoint,farPoint);
        const auto direction=farPoint-nearPoint;
        const auto plane=CoreWorkspace::instance().plane(view);
        auto normal=plane.getRotation().multVec(Base::Vector3d(0,0,1));
        auto origin=plane.getPosition();double rectanglePlane[6];
        const bool rectangle=om9_curve_pick_plane(rectanglePlane);
        if(rectangle){origin=Base::Vector3d(rectanglePlane[0],rectanglePlane[1],rectanglePlane[2]);normal=Base::Vector3d(rectanglePlane[3],rectanglePlane[4],rectanglePlane[5]);}
        const double dot=direction[0]*normal.x+direction[1]*normal.y+direction[2]*normal.z;
        if(std::abs(dot)<=1e-6*direction.length())throw Base::RuntimeError("Ray parallel to construction plane");
        if(rectangle){
            const double t=((origin.x-nearPoint[0])*normal.x+(origin.y-nearPoint[1])*normal.y+(origin.z-nearPoint[2])*normal.z)/dot;
            deliver(nearPoint[0]+t*direction[0],nearPoint[1]+t*direction[1],nearPoint[2]+t*direction[2]);
        }else{const auto p=viewer->getPointOnXYPlaneOfPlacement(pixel,plane);deliver(p[0],p[1],p[2]);}
    }catch(const Base::Exception&) {
        if(hovering){updatePreview();return false;}
        setPrompt("This view cannot intersect its construction plane. Change view or enter coordinates; the command remains active.");
        logMessage(console->property("om9Prompt").toString());
    }catch(const std::exception& error) {
        clearPreview();if(!hovering){setPrompt(QString::fromUtf8(error.what()));logMessage(console->property("om9Prompt").toString());}
    }
    return !hovering;
}
}
