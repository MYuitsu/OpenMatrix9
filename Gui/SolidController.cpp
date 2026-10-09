// SPDX-License-Identifier: LGPL-2.1-or-later
// Spec: OM9-SOLID-012 (Box), OM9-SOLID-014 (Sphere).
#include "SolidController.h"
#include "RustBridge.h"
#include "CurveController.h"
#include "CoreWorkspace.h"
#include "CoreDistance.h"
#include "CorePictureFrame.h"
#include "CoreViewControls.h"
#include "CoreSnaps.h"
#include "CoreKeyboard.h"
#include "CoreMouse.h"
#include <Base/Interpreter.h>
#include <Base/Console.h>
#include <App/Application.h>
#include <App/Document.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Inventor/SoDB.h>
#include <Inventor/SoInput.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <QApplication>
#include <QMouseEvent>
#include <QKeyEvent>
#include <QTimer>
#include <QToolButton>
#include <cmath>
#include <cstring>
#include <stdexcept>
#include <sstream>
#include <iomanip>
namespace {
Gui::View3DInventor* activeView(){auto* gui=Gui::Application::Instance->activeDocument();return gui?dynamic_cast<Gui::View3DInventor*>(gui->getActiveView()):nullptr;}
Gui::View3DInventor* containing(QObject* object){
    auto* widget=qobject_cast<QWidget*>(object);auto* gui=Gui::Application::Instance->activeDocument();if(!widget||!gui)return nullptr;
    for(auto* mdi:gui->getMDIViews())if(auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);view&&(widget==view->getViewer()||view->getViewer()->isAncestorOf(widget)))return view;
    return nullptr;
}
std::string pythonError(){
    PyObject *type=nullptr,*value=nullptr,*trace=nullptr;PyErr_Fetch(&type,&value,&trace);PyErr_NormalizeException(&type,&value,&trace);
    PyObject* text=value?PyObject_Str(value):nullptr;const char* bytes=text?PyUnicode_AsUTF8(text):nullptr;
    std::string message=bytes?bytes:"Native solid adapter failed";Py_XDECREF(text);Py_XDECREF(type);Py_XDECREF(value);Py_XDECREF(trace);PyErr_Clear();return message;
}
struct Ref {
    PyObject* p;explicit Ref(PyObject* p):p(p){if(!p)throw std::runtime_error(pythonError());}
    ~Ref(){Py_DECREF(p);}Ref(const Ref&)=delete;
};
void set(PyObject* object,const char* name,PyObject* value){if(PyObject_SetAttrString(object,name,value)<0)throw std::runtime_error(pythonError());}
void addStringProperty(PyObject* object,const char* name,const char* text){
    Ref added(PyObject_CallMethod(object,"addProperty","sss","App::PropertyString",name,"OpenMatrix9"));Ref value(PyUnicode_FromString(text));set(object,name,value.p);
}
bool flag(PyObject* object,const char* method){Ref v(PyObject_CallMethod(object,method,nullptr));const int result=PyObject_IsTrue(v.p);if(result<0)throw std::runtime_error(pythonError());return result==1;}
// Typed Python API calls invoke the host Part kernel without executing CMD text.
PyObject* buildSolid(unsigned kind,const double* b){
    Ref part(PyImport_ImportModule("Part")),app(PyImport_ImportModule("FreeCAD"));
    PyObject* shape=nullptr;
    if(kind==1){
        Ref box(PyObject_CallMethod(part.p,"makeBox","ddd",b[12],b[13],b[14]));Ref matrix(PyObject_CallMethod(app.p,"Matrix",nullptr));
        for(unsigned row=0;row<3;++row)for(unsigned col=0;col<4;++col){const auto key="A"+std::to_string(row+1)+std::to_string(col+1);Ref value(PyFloat_FromDouble(col<3?b[3+col*3+row]:b[row]));set(matrix.p,key.c_str(),value.p);}
        Ref transform(PyObject_CallMethod(box.p,"transformShape","O",matrix.p));shape=box.p;Py_INCREF(shape);
    }else if(kind==2){Ref center(PyObject_CallMethod(app.p,"Vector","ddd",b[0],b[1],b[2]));shape=PyObject_CallMethod(part.p,"makeSphere","dO",b[12],center.p);}
    else throw std::runtime_error("Unsupported solid command");
    Ref result(shape);Ref solids(PyObject_GetAttrString(result.p,"Solids")),volume(PyObject_GetAttrString(result.p,"Volume"));
    const double v=PyFloat_AsDouble(volume.p);if(PyErr_Occurred())throw std::runtime_error(pythonError());
    if(flag(result.p,"isNull")||!flag(result.p,"isValid")||!flag(result.p,"isClosed")||PySequence_Size(solids.p)!=1||!std::isfinite(v)||v<=0)throw std::runtime_error("Kernel did not produce one valid closed solid");
    Py_INCREF(result.p);return result.p;
}
}
namespace OpenMatrix9Gui {
SolidController& SolidController::instance(){static auto* controller=new SolidController;return *controller;}
SolidController::SolidController():QObject(qApp){
    deleteConnection=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(document==&doc)cancel();});
    activeConnection=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(active()&&document!=&doc)cancel();});
    auto* timer=new QTimer(this);timer->setInterval(100);connect(timer,&QTimer::timeout,this,[this]{if(active()&&!valid())cancel();});timer->start();
}
bool SolidController::handles(std::size_t i){return om9_solid_kind(om9_command_id(i))!=0;}
bool SolidController::matches(std::size_t i,const QString& name){return handles(i)&&om9_solid_kind(name.toUtf8().constData())==om9_solid_kind(om9_command_id(i));}
void SolidController::activate(){enabled=true;qApp->installEventFilter(this);}
void SolidController::deactivate(){enabled=false;cancel();}
bool SolidController::active()const{return document&&om9_solid_phase()!=0;}
bool SolidController::available(std::size_t i)const{
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    return enabled&&handles(i)&&doc&&gui&&!gui->isAboutToClose()&&!gui->getInEdit()&&activeView()&&Gui::Control().isAllowedAlterDocument(doc);
}
bool SolidController::valid()const{return document==App::GetApplication().getActiveDocument()&&available(command);}
void SolidController::frame(Gui::View3DInventor* view){
    const auto p=CoreWorkspace::instance().plane(view);const auto origin=p.getPosition();double b[12]={origin.x,origin.y,origin.z};
    for(unsigned i=0;i<3;++i){Base::Vector3d axis;axis[i]=1.;axis=p.getRotation().multVec(axis);for(unsigned j=0;j<3;++j)b[3+i*3+j]=axis[j];}
    if(!om9_solid_frame(b))throw std::runtime_error("Invalid construction plane");
}
void SolidController::prompt(){char b[2048]={};om9_solid_message(b,sizeof(b));CurveController::instance().setPrompt(QString::fromUtf8(b));CurveController::instance().logMessage(QString::fromUtf8(b));}
bool SolidController::start(std::size_t i){
    if(!available(i))return false;
    CoreDistance::instance().cancel();CorePictureFrame::instance().cancel();CoreViewControls::instance().cancel();CurveController::instance().cancel();cancel();
    try {
    // Reject corrupt saved frame metadata before creating a Rust session.
    CoreWorkspace::instance().plane(activeView());
    document=App::GetApplication().getActiveDocument();command=i;kind=om9_solid_kind(om9_command_id(i));
    if(!om9_solid_start(om9_command_id(i))){document=nullptr;return false;}
    frame(activeView());qApp->installEventFilter(this);CoreMouse::instance().prioritize();prompt();return false; // history on commit only
    }catch(const std::exception& error){cancel();CoreSnaps::clearTransient();CurveController::instance().setPrompt(QString::fromUtf8(error.what()));return false;}
}
void SolidController::clearPreview(){for(const auto& [root,node]:previews){if(root->findChild(node)>=0)root->removeChild(node);root->unref();}previews.clear();}
void SolidController::cancel(){if(document)CoreSnaps::clearTransient();clearPreview();om9_solid_cancel();document=nullptr;Base::PyGILStateLocker lock;tangentReferences.clear();pathReference.reset();}
void SolidController::reference(const QString& name,const SolidPoint& pick){
    frame(activeView());Base::PyGILStateLocker lock;auto native=solidCurveReference(*document,name.toStdString());const unsigned mode=om9_solid_reference_mode();
    if(mode==1){pathReference=std::move(native);result(om9_solid_reference(pick[0],pick[1],pick[2],0,0,1));}
    else if(mode==3){const auto located=solidOnCurve(native,pick);const auto i=om9_solid_constraints(nullptr);tangentReferences[i]=std::move(native);result(om9_solid_reference(located.first[0],located.first[1],located.first[2],0,0,1));}
    else throw std::runtime_error("No native curve selection is expected");
}
void SolidController::solveTangent(int solution){
    Base::PyGILStateLocker lock;verifySolidReferences(*document,tangentReferences,pathReference);double b[12],frameData[12];const auto count=om9_solid_constraints(b);if(!om9_solid_reference_frame(frameData))throw std::runtime_error("Missing tangent construction plane");
    const auto [center,radius]=solidTangentSphere(tangentReferences,b,count,frameData,om9_solid_tangent_radius(),solution);result(om9_solid_resolve(center[0],center[1],center[2],radius));
}
void SolidController::submit(const QString& text){
    if(!active())return;if(!valid()){cancel();return;}
    // Keep shared Osnap/Ortho controls usable during point entry.
    if(CoreSnaps::submit(text)||CoreKeyboard::instance().submit(text))return;
    try {frame(activeView());}
    catch(const std::exception& error){cancel();CurveController::instance().setPrompt(QString::fromUtf8(error.what()));return;}
    try{
        const auto t=text.trimmed();
        if((om9_solid_reference_mode()==1&&t.startsWith("Path=",Qt::CaseInsensitive))||(om9_solid_reference_mode()==3&&t.startsWith("Curve=",Qt::CaseInsensitive))){
            const auto value=t.section('=',1);const auto split=value.indexOf('@');const auto name=split<0?value:value.left(split);SolidPoint pick{0,0,0};
            if(split>=0){const auto coords=value.mid(split+1).split(',');if(coords.size()!=3)throw std::runtime_error("Curve pick syntax: Curve=Object.EdgeN@worldX,worldY,worldZ");for(int i=0;i<3;++i){bool ok=false;pick[i]=coords[i].toDouble(&ok);if(!ok||!std::isfinite(pick[i]))throw std::runtime_error("Curve pick coordinates must be finite");}}
            reference(name,pick);return;
        }
        if(om9_solid_reference_mode()==2&&t.startsWith("OnCurve=",Qt::CaseInsensitive)){if(!pathReference)throw std::runtime_error("Reselect the curve");bool ok=false;const auto fraction=t.section('=',1).toDouble(&ok);if(!ok)throw std::runtime_error("OnCurve requires a fraction within 0..1");Base::PyGILStateLocker lock;verifySolidReferences(*document,tangentReferences,pathReference);const auto [p,n]=solidOnCurve(*pathReference,{0,0,0},fraction);result(om9_solid_reference(p[0],p[1],p[2],n[0],n[1],n[2]));return;}
        if(om9_solid_phase()==21&&t.startsWith("Solution=",Qt::CaseInsensitive)){bool ok=false;const auto value=t.section('=',1).toInt(&ok);if(!ok||value<1)throw std::runtime_error("Solution index starts at 1");solveTangent(value-1);return;}
        if(om9_solid_reference_mode()==4 && ((t.isEmpty()&&om9_solid_point_count()==0)||t.compare("Selection",Qt::CaseInsensitive)==0)){
            Base::PyGILStateLocker lock;const auto points=solidSelectedFitPoints(*document);std::vector<double> flat;for(const auto& p:points)flat.insert(flat.end(),p.begin(),p.end());if(!points.empty()&&om9_solid_fit_points(flat.data(),points.size())!=1)throw std::runtime_error("Invalid selected fit point batch");if(!t.isEmpty()){prompt();return;}
        }
        const auto previousPoints=om9_solid_point_count();
        const auto effect=om9_solid_input(t.toUtf8().constData());
        CoreSnaps::acceptedPoint(effect!=0&&effect!=3&&om9_solid_point_count()>previousPoints);
        result(effect);
        {Base::PyGILStateLocker lock;const auto count=om9_solid_constraints(nullptr);for(auto it=tangentReferences.begin();it!=tangentReferences.end();)if(it->first>=count)it=tangentReferences.erase(it);else ++it;if(om9_solid_phase()==16)pathReference.reset();}
    }catch(const std::exception& error){clearPreview();CurveController::instance().setPrompt(QString::fromUtf8(error.what()));}
}
void SolidController::result(unsigned effect){
    clearPreview();
    if(effect==1&&om9_solid_phase()==21){try{solveTangent();}catch(const std::exception& error){CurveController::instance().setPrompt(QString::fromUtf8(error.what()));}return;}
    if(effect==2){
        try{commit();om9_sidebar_record_execution(command,true);cancel();CurveController::instance().setPrompt("Solid created. Command:");}
        catch(const std::exception& error){cancel();CurveController::instance().setPrompt(QString::fromUtf8(error.what()));Base::Console().warning("OpenMatrix9 solid: %s\n",error.what());}
    }else if(effect==3){cancel();CurveController::instance().setPrompt("Solid command cancelled. Command:");}
    else prompt();
}
void SolidController::preview(const double* point,bool ortho){
    clearPreview();double b[15];if(!om9_solid_geometry(b,true,point[0],point[1],point[2],ortho))return;
    try{
        Base::PyGILStateLocker lock;Ref shape(buildSolid(kind,b)),inventor(PyObject_CallMethod(shape.p,"writeInventor",nullptr));const char* text=PyUnicode_AsUTF8(inventor.p);if(!text)throw std::runtime_error(pythonError());
        auto* gui=Gui::Application::Instance->activeDocument();for(auto* mdi:gui->getMDIViews()){
            auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);if(!view)continue;auto* root=dynamic_cast<SoSeparator*>(view->getViewer()->getSceneGraph());if(!root)continue;
            SoInput input;input.setBuffer(text,std::strlen(text));auto* mesh=SoDB::readAll(&input);if(!mesh)throw std::runtime_error("Cannot render solid preview");
            auto* node=new SoSeparator;node->setName("OM9SolidPreview");auto* skip=new SoPickStyle;skip->style=SoPickStyle::UNPICKABLE;node->addChild(skip);node->addChild(mesh);root->ref();root->addChild(node);previews.emplace_back(root,node);view->getViewer()->redraw();
        }
    }catch(const std::exception&){clearPreview();}
}
void SolidController::commit(){
    if(!valid())throw std::runtime_error("Document is no longer editable");double b[15];if(!om9_solid_geometry(b,false,0,0,0,false))throw std::runtime_error("No solid output");
    Base::PyGILStateLocker lock;verifySolidReferences(*document,tangentReferences,pathReference);Ref shape(buildSolid(kind,b));const char* name=kind==1?"Box":"Sphere";
    document->openTransaction(name);
    try{
        Ref doc(document->getPyObject()),object(PyObject_CallMethod(doc.p,"addObject","ss","Part::Feature",name));set(object.p,"Shape",shape.p);
        addStringProperty(object.p,"OM9FeatureId",kind==1?"OM9-SOLID-012":"OM9-SOLID-014");addStringProperty(object.p,"OM9Command",name);
        std::ostringstream settings;settings<<std::setprecision(17)<<"mm;mode="<<om9_solid_mode_name()<<";origin="<<b[0]<<","<<b[1]<<","<<b[2]<<";dimensions="<<b[12]<<","<<b[13]<<","<<b[14]<<";axes=";for(unsigned i=3;i<12;++i)settings<<b[i]<<",";if(pathReference)settings<<";path="<<pathReference->input.name<<"."<<pathReference->sub;for(const auto& [i,ref]:tangentReferences)settings<<";tangent"<<i<<"="<<ref.input.name<<"."<<ref.sub;addStringProperty(object.p,"SolidParameters",settings.str().c_str());
        Ref view(PyObject_GetAttrString(object.p,"ViewObject")),color(Py_BuildValue("(ddd)",166./255.,104./255.,209./255.));set(view.p,"ShapeColor",color.p);set(view.p,"LineColor",color.p);
        document->recompute();document->commitTransaction();
    }catch(...){document->abortTransaction();throw;}
}
bool SolidController::eventFilter(QObject* watched,QEvent* event){
    if(event->type()==QEvent::MouseButtonRelease&&releaseTarget==watched&&static_cast<QMouseEvent*>(event)->button()==releaseButton){releaseTarget=nullptr;return true;}
    if(auto* button=qobject_cast<QToolButton*>(watched);button&&button->property("om9Command").isValid()){
        const auto i=std::size_t(button->property("om9Command").toULongLong());
        if(handles(i)&&event->type()==QEvent::ContextMenu)return true;
        if(handles(i)&&event->type()==QEvent::MouseButtonPress&&static_cast<QMouseEvent*>(event)->button()==Qt::RightButton&&button->isEnabled()&&available(i)){
            start(i);submit(kind==1?"3Point":"2Point");releaseTarget=watched;releaseButton=Qt::RightButton;return true;
        }
    }
    if(!active()||Gui::Application::Instance->isClosing())return false;if(!valid()){cancel();return false;}
    auto* view=containing(watched);
    if(event->type()==QEvent::KeyPress&&CoreKeyboard::inputContext(watched)){
        auto* key=static_cast<QKeyEvent*>(event);
        if(key->key()==Qt::Key_Escape){CurveController::instance().cancelInput();CurveController::instance().setPrompt("Solid command cancelled. Command:");return true;}
        if(key->key()==Qt::Key_F4&&CoreKeyboard::pointKeyAllowed(watched,key)){
            try {frame(activeView());const auto p=CoreWorkspace::instance().plane(activeView()).getPosition();const auto effect=om9_solid_point(p.x,p.y,p.z,false);CoreSnaps::acceptedPoint(effect==1||effect==2);result(effect);}
            catch(const std::exception& error){cancel();CurveController::instance().setPrompt(QString::fromUtf8(error.what()));}
            return true;
        }
        if(view&&(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter||key->key()==Qt::Key_Space)){CurveController::instance().acceptInput();return true;}
    }
    const bool hover=event->type()==QEvent::MouseMove;
    if(!view||(!hover&&event->type()!=QEvent::MouseButtonPress))return false;
    auto* mouse=static_cast<QMouseEvent*>(event);if(!hover&&mouse->button()!=Qt::LeftButton)return false;
    if(mouse->modifiers()&(Qt::ControlModifier|Qt::AltModifier|Qt::MetaModifier))return false;
    if(!hover){releaseTarget=watched;releaseButton=Qt::LeftButton;}
    auto* viewer=view->getViewer();auto* widget=qobject_cast<QWidget*>(watched);const auto pos=viewer->viewport()->mapFrom(widget,mouse->position().toPoint());
    try {frame(view);}
    catch(const std::exception& error){cancel();CurveController::instance().setPrompt(QString::fromUtf8(error.what()));return !hover;}
    try{
        if(!hover&&(om9_solid_reference_mode()==1||om9_solid_reference_mode()==3)){
            Base::PyGILStateLocker lock;Ref viewObject(view->getPyObject());const auto pixel=viewer->fromQPoint(pos);Ref info(PyObject_CallMethod(viewObject.p,"getObjectInfo","((ii))",int(pixel[0]),int(pixel[1])));
            if(!PyDict_Check(info.p))throw std::runtime_error("Pick a native curve edge");auto* object=PyDict_GetItemString(info.p,"Object");auto* component=PyDict_GetItemString(info.p,"Component");if(!object||!component)throw std::runtime_error("Pick a native curve edge");
            const char* name=PyUnicode_AsUTF8(object);const char* edge=PyUnicode_AsUTF8(component);if(!name||!edge)throw std::runtime_error("Invalid native curve pick");SolidPoint pick;const char* axes[]={"x","y","z"};for(unsigned i=0;i<3;++i){auto* value=PyDict_GetItemString(info.p,axes[i]);if(!value)throw std::runtime_error("No world curve pick point");pick[i]=PyFloat_AsDouble(value);}reference(QString::fromUtf8(name)+"."+QString::fromUtf8(edge),pick);return true;
        }
        if(hover&&(om9_solid_reference_mode()==1||om9_solid_reference_mode()==3))return false;
        if(om9_solid_reference_mode()==2){
            if(hover)return false;
            if(!pathReference)throw std::runtime_error("Reselect native curve");
            Base::PyGILStateLocker lock;verifySolidReferences(*document,tangentReferences,pathReference);
            Ref viewObject(view->getPyObject());const auto pixel=viewer->fromQPoint(pos);Ref picks(PyObject_CallMethod(viewObject.p,"getObjectsInfo","((ii))",int(pixel[0]),int(pixel[1])));
            if(PyList_Check(picks.p))for(Py_ssize_t i=0;i<PyList_Size(picks.p);++i){auto* info=PyList_GetItem(picks.p,i);auto* object=PyDict_GetItemString(info,"Object");auto* component=PyDict_GetItemString(info,"Component");
                if(!object||!component||pathReference->input.name!=PyUnicode_AsUTF8(object)||pathReference->sub!=PyUnicode_AsUTF8(component))continue;
                SolidPoint seed;const char* keys[]={"x","y","z"};for(unsigned j=0;j<3;++j){auto* value=PyDict_GetItemString(info,keys[j]);if(!value)throw std::runtime_error("No world curve pick point");seed[j]=PyFloat_AsDouble(value);}
                const auto [c,n]=solidOnCurve(*pathReference,seed);result(om9_solid_reference(c[0],c[1],c[2],n[0],n[1],n[2]));return true;
            }
            throw std::runtime_error("Pick a center on the selected path, or enter OnCurve=fraction");
        }
        Base::Vector3d p;const auto pixel=viewer->fromQPoint(pos);
        if(!CoreSnaps::pick(view,pos,p)){
            SbVec3f nearPoint,farPoint;viewer->projectPointToLine(pixel,nearPoint,farPoint);Base::Vector3d a(nearPoint[0],nearPoint[1],nearPoint[2]),d(farPoint[0]-nearPoint[0],farPoint[1]-nearPoint[1],farPoint[2]-nearPoint[2]);
            double axis[6];
            if(om9_solid_height_axis(axis)){
                // Closest point on the height axis to the mouse ray.
                const Base::Vector3d origin(axis[0],axis[1],axis[2]),n(axis[3],axis[4],axis[5]);const auto delta=a-origin;const double dn=d.Dot(n),dd=d.Dot(d),den=dd-dn*dn;
                if(std::abs(den)<1e-8*dd)throw std::runtime_error("Enter Height numerically or switch to a side/perspective view");
                const double height=(dd*delta.Dot(n)-dn*delta.Dot(d))/den;p=origin+n*height;
            }else{
                double special[6];
                if(om9_solid_pick_plane(special)){const Base::Vector3d origin(special[0],special[1],special[2]),normal(special[3],special[4],special[5]);if(std::abs(d.Dot(normal))<=1e-6*d.Length())throw std::runtime_error("View is parallel to construction plane; switch view or enter coordinates");p=a+d*((origin-a).Dot(normal)/d.Dot(normal));}
                else{
                const auto plane=CoreWorkspace::instance().plane(view);const auto n=plane.getRotation().multVec(Base::Vector3d(0,0,1));
                if(std::abs(d.Dot(n))<=1e-6*d.Length())throw std::runtime_error("View is parallel to CPlane; enter coordinates or switch view");
                const auto hit=viewer->getPointOnXYPlaneOfPlacement(pixel,plane);p=Base::Vector3d(hit[0],hit[1],hit[2]);
                }
            }
        }
        const bool ortho=om9_ortho_active(mouse->modifiers().testFlag(Qt::ShiftModifier));const double point[3]={p.x,p.y,p.z};
        if(hover)preview(point,ortho);else {const auto effect=om9_solid_point(p.x,p.y,p.z,ortho);CoreSnaps::acceptedPoint(effect==1||effect==2);result(effect);}
    }catch(const std::exception& error){clearPreview();if(!hover)CurveController::instance().setPrompt(QString::fromUtf8(error.what()));}
    return !hover;
}
}
