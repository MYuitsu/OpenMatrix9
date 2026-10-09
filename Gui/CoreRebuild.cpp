#include "CoreRebuild.h"
#include "CurveController.h"
#include "CurveGeometry.h"
#include "RustBridge.h"
#include <Base/Interpreter.h>
#include <App/Application.h>
#include <App/Document.h>
#include <App/GeoFeature.h>
#include <App/PropertyGeo.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/MainWindow.h>
#include <Gui/Control.h>
#include <Gui/Selection/Selection.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <Inventor/nodes/SoBaseColor.h>
#include <Inventor/nodes/SoCoordinate3.h>
#include <Inventor/nodes/SoLineSet.h>
#include <QApplication>
#include <QDialog>
#include <QFormLayout>
#include <QSpinBox>
#include <QCheckBox>
#include <QLabel>
#include <QPushButton>
#include <QDialogButtonBox>
#include <QPointer>
#include <QTimer>
#include <QKeyEvent>
#include <set>
#include <cmath>
#include <algorithm>
namespace OpenMatrix9Gui {
struct CoreRebuild::State {
    struct Input {
        std::string name;
        CurvePyRef object,shape,edge;
        bool closed;
        Base::Placement globalPlacement;
        int count=0,degree=0;
        Input(std::string n,PyObject* o,PyObject* s,PyObject* e,bool c):name(std::move(n)),object(o),shape(s),edge(e),closed(c){}
    };
    App::Document* document=nullptr;
    std::size_t command=0;
    bool selecting=false;
    std::vector<std::unique_ptr<Input>> inputs;
    QPointer<QDialog> dialog;
    QSpinBox *count=nullptr,*degree=nullptr;
    QCheckBox* remove=nullptr;
    QLabel *deviation=nullptr,*error=nullptr;
    std::vector<std::pair<SoSeparator*,SoSeparator*>> previews;
    fastsignals::scoped_connection documentChanged,documentDeleted;
};
CoreRebuild& CoreRebuild::instance(){static auto* tool=new CoreRebuild;return *tool;}
CoreRebuild::CoreRebuild():QObject(qApp),state(std::make_unique<State>()) {
    qApp->installEventFilter(this);
    state->documentChanged=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(active()&&state->document!=&doc)cancel();});
    state->documentDeleted=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(state->document==&doc)cancel();});
    auto* timer=new QTimer(this);timer->setInterval(100);connect(timer,&QTimer::timeout,this,[this]{if(active()&&!valid())cancel();});timer->start();
}
bool CoreRebuild::handles(std::size_t index){auto* id=om9_command_id(index);return id&&std::string(id)=="Rebuild";}
bool CoreRebuild::active()const{return state->selecting||state->dialog;}
bool CoreRebuild::valid()const {
    auto* gui=Gui::Application::Instance->activeDocument();
    return state->document&&state->document==App::GetApplication().getActiveDocument()&&gui&&!gui->getInEdit()&&!gui->isAboutToClose()&&Gui::Control().isAllowedAlterDocument(state->document);
}
void CoreRebuild::clearPreview(){for(auto [root,node]:state->previews){if(root->findChild(node)>=0)root->removeChild(node);root->unref();}state->previews.clear();}
void CoreRebuild::cancel(){
    clearPreview();state->selecting=false;state->document=nullptr;
    if(state->dialog){auto* d=state->dialog.data();state->dialog=nullptr;d->setObjectName({});d->hide();d->deleteLater();}
    Base::PyGILStateLocker lock;state->inputs.clear();
}
bool CoreRebuild::start(std::size_t command){
    cancel();state->document=App::GetApplication().getActiveDocument();state->command=command;
    if(!valid()){cancel();return false;}
    state->selecting=true;
    if(Gui::Selection().hasSelection(state->document->getName()))options();
    else {CurveController::instance().setPrompt("Rebuild: Select curves, then press Enter. Esc cancels.");CurveController::instance().logMessage("Rebuild: Select curves, then press Enter. Esc cancels.");}
    return active();
}
void CoreRebuild::submit(const QString& input){
    if(input.compare("Cancel",Qt::CaseInsensitive)==0||input.compare("Esc",Qt::CaseInsensitive)==0){cancel();CurveController::instance().setPrompt("Command: ");}
    else if(state->selecting&&input.trimmed().isEmpty())options();
    else if(state->selecting)CurveController::instance().setPrompt("Rebuild: Select curves and press Enter, or Esc to cancel.");
}
bool CoreRebuild::eventFilter(QObject* watched,QEvent* event){
    if(!active()||event->type()!=QEvent::KeyPress)return false;
    auto* key=static_cast<QKeyEvent*>(event);
    if(key->key()==Qt::Key_Escape){cancel();CurveController::instance().setPrompt("Command: ");return true;}
    if(!state->selecting||!(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter))return false;
    auto* gui=Gui::Application::Instance->activeDocument();auto* view=gui?dynamic_cast<Gui::View3DInventor*>(gui->getActiveView()):nullptr;auto* widget=qobject_cast<QWidget*>(watched);
    if(view&&widget&&(widget==view->getViewer()||view->getViewer()->isAncestorOf(widget))){options();return true;}
    return false;
}
void CoreRebuild::options(){
    if(!valid()){cancel();return;}
    Base::PyGILStateLocker lock;
    try {
        state->inputs.clear();CurvePyRef doc(state->document->getPyObject());std::set<std::string> seen;
        for(const auto& selection:Gui::Selection().getSelection(state->document->getName())){
            if(selection.SubName&&*selection.SubName)throw std::runtime_error("Select whole curve objects; surface edges cannot be rebuilt here");
            if(!seen.insert(selection.FeatName).second)continue;
            if(seen.size()>16)throw std::runtime_error("Rebuild supports up to 16 curves per operation");
            CurvePyRef object(PyObject_CallMethod(doc.value,"getObject","s",selection.FeatName));
            if(!PyObject_HasAttrString(object.value,"Shape"))throw std::runtime_error("Selection contains an object without curve geometry");
            CurvePyRef shape(PyObject_GetAttrString(object.value,"Shape")),world(PyObject_CallMethod(shape.value,"copy",nullptr));
            auto* native=state->document->getObject(selection.FeatName);
            const auto globalPlacement=App::GeoFeature::getGlobalPlacement(native);
            if(native->getPropertyByName<App::PropertyPlacement>("Placement")){
                // Shape includes the child's local placement; add the parent only.
                CurvePyRef global(PyObject_CallMethod(object.value,"getGlobalPlacement",nullptr)),local(PyObject_GetAttrString(object.value,"Placement"));
                CurvePyRef inverse(PyObject_CallMethod(local.value,"inverse",nullptr)),parent(PyNumber_Multiply(global.value,inverse.value)),matrix(PyObject_CallMethod(parent.value,"toMatrix",nullptr));
                CurvePyRef transformed(PyObject_CallMethod(world.value,"transformShape","OO",matrix.value,Py_False));
            }
            CurvePyRef faces(PyObject_GetAttrString(world.value,"Faces")),edges(PyObject_GetAttrString(world.value,"Edges"));
            if(PySequence_Size(faces.value)||PySequence_Size(edges.value)<1)throw std::runtime_error("Rebuild requires curves; surfaces and solids are unsupported");
            CurvePyRef edge([&]()->PyObject*{
                if(PySequence_Size(edges.value)==1)return PySequence_GetItem(edges.value,0);
                CurvePyRef wires(PyObject_GetAttrString(world.value,"Wires"));
                if(PySequence_Size(wires.value)!=1)throw std::runtime_error("Select one connected curve per object");
                CurvePyRef wire(PySequence_GetItem(wires.value,0)),wireEdges(PyObject_GetAttrString(wire.value,"Edges"));
                if(PySequence_Size(wireEdges.value)!=PySequence_Size(edges.value))throw std::runtime_error("Selection contains disconnected curve segments");
                return Py_NewRef(wire.value);
            }()),closed(PyObject_CallMethod(edge.value,"isClosed",nullptr));
            if(PyObject_IsTrue(closed.value)<0)throw std::runtime_error("Cannot determine curve closure");
            auto input=std::make_unique<State::Input>(selection.FeatName,Py_NewRef(object.value),Py_NewRef(shape.value),Py_NewRef(edge.value),PyObject_IsTrue(closed.value)==1);
            input->globalPlacement=globalPlacement;
            CurvePyRef firstEdge(PySequence_GetItem(edges.value,0)),curve(PyObject_GetAttrString(firstEdge.value,"Curve"));
            if(PyObject_HasAttrString(curve.value,"NbPoles")){CurvePyRef n(PyObject_GetAttrString(curve.value,"NbPoles"));input->count=int(PyLong_AsLong(n.value));}
            if(PyObject_HasAttrString(curve.value,"Degree")){CurvePyRef n(PyObject_GetAttrString(curve.value,"Degree"));input->degree=int(PyLong_AsLong(n.value));}
            state->inputs.push_back(std::move(input));
        }
        if(state->inputs.empty()){CurveController::instance().setPrompt("Rebuild: Select at least one curve, then press Enter.");return;}
        state->selecting=false;
        auto* dialog=new QDialog(Gui::getMainWindow());state->dialog=dialog;dialog->setObjectName("OM9RebuildDialog");dialog->setWindowTitle("Rebuild Curve");dialog->setAttribute(Qt::WA_DeleteOnClose);
        auto* layout=new QFormLayout(dialog);
        QStringList current;for(const auto& i:state->inputs)current<<QString("%1: PointCount (%2), Degree (%3)").arg(QString::fromStdString(i->name)).arg(i->count?QString::number(i->count):"analytic").arg(i->degree?QString::number(i->degree):"analytic");
        layout->addRow(new QLabel(current.join('\n'),dialog));
        state->count=new QSpinBox(dialog);state->count->setObjectName("OM9RebuildPointCount");state->count->setRange(2,256);state->count->setValue(std::clamp(state->inputs[0]->count,4,256));layout->addRow("PointCount",state->count);
        state->degree=new QSpinBox(dialog);state->degree->setObjectName("OM9RebuildDegree");state->degree->setRange(1,11);state->degree->setValue(std::clamp(state->inputs[0]->degree,1,3));layout->addRow("Degree",state->degree);
        state->remove=new QCheckBox("DeleteInput",dialog);state->remove->setObjectName("OM9RebuildDeleteInput");state->remove->setChecked(true);layout->addRow(state->remove);
        auto* layer=new QCheckBox("Create new object on current layer",dialog);layer->setEnabled(false);layer->setToolTip("Active layers are not available in this workbench. Output is created at the document root.");layout->addRow(layer);
        state->deviation=new QLabel("Maximum deviation: click Preview",dialog);state->deviation->setObjectName("OM9RebuildDeviation");layout->addRow(state->deviation);
        state->error=new QLabel(dialog);state->error->setWordWrap(true);state->error->setObjectName("OM9RebuildError");layout->addRow(state->error);
        auto* buttons=new QDialogButtonBox(dialog);auto* ok=buttons->addButton(QDialogButtonBox::Ok);ok->setObjectName("OM9RebuildOK");auto* no=buttons->addButton(QDialogButtonBox::Cancel);no->setObjectName("OM9RebuildCancel");auto* preview=buttons->addButton("Preview",QDialogButtonBox::ActionRole);preview->setObjectName("OM9RebuildPreview");layout->addRow(buttons);
        connect(preview,&QPushButton::clicked,this,[this]{calculate(false);});connect(ok,&QPushButton::clicked,this,[this]{calculate(true);});connect(no,&QPushButton::clicked,this,[this]{cancel();CurveController::instance().setPrompt("Command: ");});
        connect(dialog,&QDialog::rejected,this,[this]{cancel();CurveController::instance().setPrompt("Command: ");});
        auto stale=[this]{clearPreview();if(state->dialog){state->deviation->setText("Maximum deviation: click Preview");state->error->clear();}};
        connect(state->count,&QSpinBox::valueChanged,this,stale);connect(state->degree,&QSpinBox::valueChanged,this,stale);
        CurveController::instance().setPrompt("Rebuild: Adjust options, Preview, then OK; Cancel discards the preview.");dialog->show();
    }catch(const std::exception& e){cancel();CurveController::instance().setPrompt(QString::fromUtf8(e.what()));CurveController::instance().logMessage(QString::fromUtf8(e.what()));}
}
void CoreRebuild::calculate(bool commit){
    if(!valid()||!state->dialog){cancel();return;}
    Base::PyGILStateLocker lock;clearPreview();
    try {
        const auto poles=std::size_t(state->count->value()),degree=std::size_t(state->degree->value());
        if(!om9_spline_options(poles,degree))throw std::runtime_error("PointCount must exceed Degree (1 to 11)");
        CurvePyRef doc(state->document->getPyObject()),part(PyImport_ImportModule("Part")),app(PyImport_ImportModule("FreeCAD"));
        CurvePyRef vertex(PyObject_GetAttrString(part.value,"Vertex")),vector(PyObject_GetAttrString(app.value,"Vector"));
        std::vector<CurvePyRef> outputs;double maximum=0;
        for(const auto& input:state->inputs){
            CurvePyRef current(PyObject_CallMethod(doc.value,"getObject","s",input->name.c_str()));
            if(current.value!=input->object.value)throw std::runtime_error("A selected curve was removed or replaced; start Rebuild again");
            if(!input->globalPlacement.isSame(App::GeoFeature::getGlobalPlacement(state->document->getObject(input->name.c_str())),1e-12))throw std::runtime_error("A selected curve's placement changed; start Rebuild again");
            CurvePyRef shape(PyObject_GetAttrString(current.value,"Shape")),same(PyObject_CallMethod(shape.value,"isEqual","O",input->shape.value));
            if(PyObject_IsTrue(same.value)!=1)throw std::runtime_error("A selected curve changed; start Rebuild again");
            if(state->remove->isChecked()){CurvePyRef uses(PyObject_GetAttrString(current.value,"InList"));if(PySequence_Size(uses.value)>0)throw std::runtime_error("A selected curve has dependents. Clear DeleteInput to keep those links.");}
            const auto n=std::max<std::size_t>(513,poles*4+1);auto samples=sampleCurve(input->edge.value,n);
            if(input->closed)samples.back()=samples.front();
            std::vector<double> xyz;xyz.reserve(n*3);for(const auto& p:samples)xyz.insert(xyz.end(),p.begin(),p.end());
            if(!om9_spline_rebuild(xyz.data(),n,poles,degree,input->closed)){char message[1024]={};om9_spline_message(message,sizeof(message));throw std::runtime_error(message);}
            outputs.push_back(publishedSplineShape());
            if(!commit){
                // Bidirectional sampled geometric distance, explicitly an estimate.
                for(auto pair:{std::pair{input->edge.value,outputs.back().value},std::pair{outputs.back().value,input->edge.value}}){
                    for(const auto& p:sampleCurve(pair.first,129)){
                        CurvePyRef v(PyObject_CallFunction(vector.value,"ddd",p[0],p[1],p[2])),point(PyObject_CallOneArg(vertex.value,v.value));
                        CurvePyRef distance(PyObject_CallMethod(point.value,"distToShape","O",pair.second)),first(PySequence_GetItem(distance.value,0));
                        const double d=PyFloat_AsDouble(first.value);if(PyErr_Occurred()||!std::isfinite(d))throw std::runtime_error("Cannot measure rebuild deviation");maximum=std::max(maximum,d);
                    }
                }
            }
        }
        if(commit){
            state->document->openTransaction("Rebuild Curve");
            try{
                for(std::size_t i=0;i<outputs.size();++i){
                    auto object=createCurveFeature(*state->document,outputs[i].value,"Rebuild");
                    CurvePyRef oldView(PyObject_GetAttrString(state->inputs[i]->object.value,"ViewObject")),newView(PyObject_GetAttrString(object.value,"ViewObject"));
                    for(const char* attr:{"LineColor","PointColor","LineWidth"}){CurvePyRef property(PyObject_GetAttrString(oldView.value,attr));if(PyObject_SetAttrString(newView.value,attr,property.value)<0)throw std::runtime_error("Cannot copy curve appearance");}
                }
                if(state->remove->isChecked())for(const auto& input:state->inputs)state->document->removeObject(input->name.c_str());
                state->document->recompute();state->document->commitTransaction();
            }catch(...){state->document->abortTransaction();throw;}
            const auto command=state->command;cancel();om9_sidebar_record_execution(command,true);CurveController::instance().setPrompt("Command: ");CurveController::instance().logMessage("Rebuild completed");return;
        }
        for(const auto& output:outputs){
            const auto sampled=sampleCurve(output.value,257);std::vector<SbVec3f> points;for(auto p:sampled)points.emplace_back(float(p[0]),float(p[1]),float(p[2]));
            for(auto* base:Gui::Application::Instance->activeDocument()->getMDIViews())if(auto* view=dynamic_cast<Gui::View3DInventor*>(base)){
                auto* root=dynamic_cast<SoSeparator*>(view->getViewer()->getSceneGraph());if(!root)continue;root->ref();auto* node=new SoSeparator;root->addChild(node);state->previews.emplace_back(root,node);
                node->setName("OM9RebuildPreview");auto* pick=new SoPickStyle;pick->style=SoPickStyle::UNPICKABLE;node->addChild(pick);auto* color=new SoBaseColor;color->rgb.setValue(0.2f,1,0.65f);node->addChild(color);
                auto* coords=new SoCoordinate3;coords->point.setValues(0,int(points.size()),points.data());node->addChild(coords);auto* line=new SoLineSet;line->numVertices.set1Value(0,int(points.size()));node->addChild(line);view->getViewer()->redraw();
            }
        }
        state->error->clear();state->deviation->setText(QString("Maximum deviation (sampled): %1 mm").arg(maximum,0,'g',8));
    }catch(const std::exception& e){clearPreview();if(state->dialog){state->error->setText(QString::fromUtf8(e.what()));state->deviation->setText("Maximum deviation: unavailable");}}
}
}
