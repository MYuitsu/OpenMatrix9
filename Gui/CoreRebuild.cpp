#include <App/DocumentObject.h>
#include <sstream>
#include "LayerDocumentAdapter.h"
#include "CoreRebuild.h"
#include "CoreObjects.h"
#include "CurveController.h"
#include "CurveGeometry.h"
#include "RustBridge.h"
#include "SnapObjectInfo.h"
#include <Mod/Part/App/TopoShape.h>
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BSplineCurve.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Wire.hxx>
#include <Base/Interpreter.h>
#include <App/Application.h>
#include <App/Document.h>
#include <App/GeoFeature.h>
#include <App/PropertyGeo.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/MainWindow.h>
#include <Gui/Control.h>
#include <Gui/ViewProvider.h>
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
#include <deque>
#include <cstdint>
#include <QThread>
namespace {
// Rust-owned value policy; the caller retains exclusive GUI-thread storage.
struct ResultGate {std::uint64_t next=1,current=0,slots[16]{};};
static_assert(sizeof(ResultGate)==144);
extern "C" {
std::uint64_t om9_result_issue(ResultGate*);
void om9_result_invalidate(ResultGate*);
bool om9_result_claim(ResultGate*,std::uint64_t,bool);
bool om9_result_retire(ResultGate*,std::uint64_t);
}
std::string shapeBytes(const TopoDS_Shape& shape) {
    std::ostringstream stream;
    Part::TopoShape(shape).exportBrep(stream);
    if(!stream)throw std::runtime_error("Cannot serialize native result diagnostics");
    return stream.str();
}
}
namespace OpenMatrix9Gui {
struct CoreRebuild::State {
    struct Input {
        std::string name;
        TopoDS_Shape shape,edge;
        bool closed=false;
        Base::Matrix4D globalTransform;
        ObjectSnapshot guard;
        std::string signature;
        int count=0,degree=0;
    };
    std::uint64_t session=0;
    std::vector<Om9RebuildInput> witnesses(App::Document& doc) {
        std::vector<std::string> names;for(const auto& input:inputs)names.push_back(input->name);
        const auto layerGeneration=hasCanonicalLayerDocument(doc)?layerMutationGeneration(doc,names,1):0;
        std::vector<Om9RebuildInput> result;result.reserve(inputs.size());
        for(auto& input:inputs){
            const auto current=captureObject(doc,input->name);auto* object=doc.getObject(input->name.c_str());
            std::ostringstream signature;signature<<current.documentUid<<';'<<current.unitContext<<';'<<current.stamp.revision<<';'<<current.stamp.epoch<<';'<<layerGeneration<<std::hexfloat;
            for(double value:current.world)signature<<';'<<value;
            input->signature=signature.str();
            result.push_back({{current.stamp.document,current.stamp.object,current.stamp.generation,reinterpret_cast<const std::uint8_t*>(input->signature.data()),input->signature.size()},object&&!object->getInList().empty()?1u:0u});
        }
        return result;
    }
    App::Document* document=nullptr;
    std::size_t command=0;
    bool selecting=false;
    std::vector<std::unique_ptr<Input>> inputs;
    QPointer<QDialog> dialog;
    QSpinBox *count=nullptr,*degree=nullptr;
    QCheckBox* remove=nullptr;
    QLabel *deviation=nullptr,*error=nullptr;
    std::vector<std::pair<SoSeparator*,SoSeparator*>> previews;
    struct Lease {
        ResultGate gate;
        std::uint64_t created=0,released=0,shapesCreated=0,shapesReleased=0,retireFailures=0;
    };
    std::shared_ptr<Lease> lease=std::make_shared<Lease>();
    QObject* deliveryContext=nullptr;
    bool holdDelivery=false;
    std::deque<std::shared_ptr<Result>> held;
    std::uint64_t lastTicket=0;
    std::string lastOutcome,lastReason;
    std::vector<std::string> lastShapeBytes;
    fastsignals::scoped_connection documentChanged,documentDeleted;
};
// Deferred envelopes retain neither source wrappers nor document/view pointers.
// All production, delivery and destruction occur on the GUI thread.
struct CoreRebuild::Result {
    // This value-only lease outlives State when Qt discards queued functors
    // after the derived QObject has started destruction. No State cycle.
    std::shared_ptr<State::Lease> owner;
    std::uint64_t ticket;
    std::string documentName;
    std::vector<ObjectSnapshot> guards;
    std::vector<TopoDS_Shape> outputs;
    std::size_t command=0;
    bool commit=false,remove=false;
    double maximum=0;
    Result(State& s,std::uint64_t t):owner(s.lease),ticket(t){++owner->created;}
    ~Result(){
        owner->shapesReleased+=outputs.size();outputs.clear();
        if(!om9_result_retire(&owner->gate,ticket))++owner->retireFailures;
        ++owner->released;
    }
};
CoreRebuild& CoreRebuild::instance(){static auto* tool=new CoreRebuild;return *tool;}
CoreRebuild::CoreRebuild():QObject(qApp),state(std::make_unique<State>()) {
    state->deliveryContext=new QObject(this);
    connect(qApp,&QCoreApplication::aboutToQuit,this,[this]{shutdownDelivery();});
    qApp->installEventFilter(this);
    state->documentChanged=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(active()&&state->document!=&doc)cancel();});
    state->documentDeleted=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(state->document==&doc)cancel();});
    auto* timer=new QTimer(this);timer->setInterval(100);connect(timer,&QTimer::timeout,this,[this]{if(active()&&!valid())cancel();});timer->start();
}
CoreRebuild::~CoreRebuild(){
    state->documentChanged.disconnect();state->documentDeleted.disconnect();
    shutdownDelivery();
}
void CoreRebuild::shutdownDelivery(){
    cancel();
    // Discard queued native owners while the application/interpreter is alive.
    // Also safe when invoked again by the controller destructor.
    auto* context=state->deliveryContext;state->deliveryContext=nullptr;delete context;
    Base::PyGILStateLocker lock;state->held.clear();
}
bool CoreRebuild::handles(std::size_t index){auto* id=om9_command_id(index);return id&&std::string(id)=="Rebuild";}
bool CoreRebuild::active()const{return state->selecting||state->dialog;}
bool CoreRebuild::valid()const {
    auto* gui=Gui::Application::Instance->activeDocument();
    return state->document&&state->document==App::GetApplication().getActiveDocument()&&gui&&!gui->getInEdit()&&!gui->isAboutToClose()&&om9AlterDocument(state->document);
}
void CoreRebuild::clearPreview(){for(auto [root,node]:state->previews){if(root->findChild(node)>=0)root->removeChild(node);root->unref();}state->previews.clear();}
void CoreRebuild::cancel(){
    om9_result_invalidate(&state->lease->gate);
    om9_phase2_rebuild_drop(state->session);state->session=0;
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
        state->inputs.clear();std::set<std::string> seen;
        requireLayerGeometryEditable(*state->document);
        for(const auto& selection:Gui::Selection().getSelection(state->document->getName())){
            if(selection.SubName&&*selection.SubName)throw std::runtime_error("Select whole curve objects; surface edges cannot be rebuilt here");
            if(!seen.insert(selection.FeatName).second)continue;
            if(seen.size()>16)throw std::runtime_error("Rebuild supports up to 16 curves per operation");
            auto guard=captureObject(*state->document,selection.FeatName);
            auto* native=state->document->getObject(selection.FeatName);
            const auto info=classifySnapObject(native);
            if(!native||!info.native_cad||info.preview)throw std::runtime_error("Selection contains an object without editable native curve geometry");
            gp_GTrsf transform;for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)transform.SetValue(i,j,info.global_transform[i-1][j-1]);
            const auto world=BRepBuilderAPI_GTransform(info.shape,transform,true).Shape();
            if(TopExp_Explorer(world,TopAbs_FACE).More())throw std::runtime_error("Rebuild requires curves; surfaces and solids are unsupported");
            std::size_t edges=0;TopoDS_Edge first;
            for(TopExp_Explorer explorer(world,TopAbs_EDGE);explorer.More();explorer.Next()){if(!edges)first=TopoDS::Edge(explorer.Current());++edges;}
            if(!edges)throw std::runtime_error("Selection has no curve edges");
            TopoDS_Shape sampled=first;
            if(edges>1){
                TopExp_Explorer wires(world,TopAbs_WIRE);
                if(!wires.More())throw std::runtime_error("Select one connected curve per object");
                const auto wire=TopoDS::Wire(wires.Current());wires.Next();
                if(wires.More())throw std::runtime_error("Select one connected curve per object");
                validateCurveWire(wire,edges);sampled=wire;
            }
            auto input=std::make_unique<State::Input>();input->name=selection.FeatName;input->shape=info.shape;input->edge=sampled;input->closed=BRep_Tool::IsClosed(sampled);
            input->globalTransform=info.global_transform;
            input->guard=std::move(guard);
            double begin,end;const auto curve=Handle(Geom_BSplineCurve)::DownCast(BRep_Tool::Curve(first,begin,end));
            if(!curve.IsNull()){input->count=curve->NbPoles();input->degree=curve->Degree();}
            state->inputs.push_back(std::move(input));
        }
        if(state->inputs.empty()){CurveController::instance().setPrompt("Rebuild: Select at least one curve, then press Enter.");return;}
        const auto witnesses=state->witnesses(*state->document);
        const Om9RebuildOptions defaults{std::size_t(std::clamp(state->inputs[0]->degree,1,3)),std::size_t(std::clamp(state->inputs[0]->count,4,256)),1};
        state->session=om9_phase2_rebuild_create(witnesses.data(),witnesses.size(),&defaults);phase2Require(state->session!=0);
        state->selecting=false;
        auto* dialog=new QDialog(Gui::getMainWindow());state->dialog=dialog;dialog->setObjectName("OM9RebuildDialog");dialog->setProperty("om9RustSession",qulonglong(state->session));dialog->setWindowTitle("Rebuild Curve");dialog->setAttribute(Qt::WA_DeleteOnClose);
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
        auto stale=[this]{om9_result_invalidate(&state->lease->gate);clearPreview();if(state->dialog){state->deviation->setText("Maximum deviation: click Preview");state->error->clear();}};
        connect(state->count,&QSpinBox::valueChanged,this,stale);connect(state->degree,&QSpinBox::valueChanged,this,stale);connect(state->remove,&QCheckBox::toggled,this,stale);
        CurveController::instance().setPrompt("Rebuild: Adjust options, Preview, then OK; Cancel discards the preview.");dialog->show();
    }catch(const Standard_Failure& e){cancel();CurveController::instance().setPrompt(QString::fromUtf8(e.GetMessageString()));}
    catch(const std::exception& e){cancel();CurveController::instance().setPrompt(QString::fromUtf8(e.what()));CurveController::instance().logMessage(QString::fromUtf8(e.what()));}
}
void CoreRebuild::calculate(bool commit){
    if(!valid()||!state->dialog||!state->deliveryContext){cancel();return;}
    Base::PyGILStateLocker lock;clearPreview();
    om9_result_invalidate(&state->lease->gate);
    try {
        requireLayerGeometryEditable(*state->document);
        const Om9RebuildOptions options{std::size_t(state->degree->value()),std::size_t(state->count->value()),state->remove->isChecked()?1u:0u};
        phase2Require(om9_phase2_rebuild_replace(state->session,&options),state->session);
        const auto witnesses=state->witnesses(*state->document);phase2Require(om9_phase2_rebuild_check(state->session,witnesses.data(),witnesses.size()),state->session);
        Om9RebuildOptions owned{};phase2Require(om9_phase2_rebuild_get(state->session,&owned),state->session);
        const auto poles=owned.point_count,degree=owned.degree;
        if(!om9_spline_options(poles,degree))throw std::runtime_error("PointCount must exceed Degree (1 to 11)");
        const auto ticket=om9_result_issue(&state->lease->gate);
        if(!ticket)throw std::runtime_error("Too many pending Rebuild results or request IDs exhausted");
        std::shared_ptr<Result> result;
        try {result=std::make_shared<Result>(*state,ticket);}catch(...){om9_result_retire(&state->lease->gate,ticket);throw;}
        result->documentName=state->document->getName();result->command=state->command;
        result->commit=commit;result->remove=owned.delete_input!=0;
        auto& outputs=result->outputs;double maximum=0;
        for(const auto& input:state->inputs){
            verifyObject(*state->document,input->guard);
            auto* current=state->document->getObject(input->name.c_str());const auto info=classifySnapObject(current);
            if(!current||info.shape.IsNull()||!info.shape.IsEqual(input->shape))throw std::runtime_error("A selected curve changed; start Rebuild again");
            if(info.global_transform!=input->globalTransform)throw std::runtime_error("A selected curve's placement changed; start Rebuild again");
            if(result->remove&&!current->getInList().empty())throw std::runtime_error("A selected curve has dependents. Clear DeleteInput to keep those links.");
            const auto n=std::max<std::size_t>(513,poles*4+1);auto samples=sampleCurve(input->edge,n);
            if(input->closed)samples.back()=samples.front();
            std::vector<double> xyz;xyz.reserve(n*3);for(const auto& p:samples)xyz.insert(xyz.end(),p.begin(),p.end());
            if(!om9_spline_rebuild(xyz.data(),n,poles,degree,input->closed)){char message[1024]={};om9_spline_message(message,sizeof(message));throw std::runtime_error(message);}
            outputs.push_back(publishedNativeSplineShape());
            ++state->lease->shapesCreated;result->guards.push_back(input->guard);
            if(!commit){
                // Bidirectional sampled geometric distance, explicitly an estimate.
                for(const auto& pair:{std::pair{input->edge,outputs.back()},std::pair{outputs.back(),input->edge}}){
                    for(const auto& p:sampleCurve(pair.first,129)){
                        const auto vertex=BRepBuilderAPI_MakeVertex(gp_Pnt(p[0],p[1],p[2])).Vertex();
                        BRepExtrema_DistShapeShape distance(vertex,pair.second);
                        if(!distance.IsDone()||!std::isfinite(distance.Value()))throw std::runtime_error("Cannot measure native rebuild deviation");
                        maximum=std::max(maximum,distance.Value());
                    }
                }
            }
        }
        result->maximum=maximum;
        if(commit){publish(result);return;}
        if(state->holdDelivery)state->held.push_back(std::move(result));
        else QTimer::singleShot(0,state->deliveryContext,[this,result]{publish(result);});
    }catch(const Standard_Failure& e){clearPreview();if(state->dialog){state->error->setText(QString::fromUtf8(e.GetMessageString()));state->deviation->setText("Maximum deviation: unavailable");}}
    catch(const std::exception& e){clearPreview();if(state->dialog){state->error->setText(QString::fromUtf8(e.what()));state->deviation->setText("Maximum deviation: unavailable");}}
}
void CoreRebuild::publish(const std::shared_ptr<Result>& result){
    Base::PyGILStateLocker lock;
    state->lastTicket=result->ticket;state->lastOutcome="rejected";state->lastReason.clear();
    bool attaching=false;
    try {
        // Fixture diagnostics retain only serialized values, never shape owners.
        if(!qgetenv("OM9_TEST_NATIVE_DIR").isEmpty()){
            state->lastShapeBytes.clear();
            for(const auto& output:result->outputs)state->lastShapeBytes.push_back(shapeBytes(output));
        }
        auto* document=App::GetApplication().getDocument(result->documentName.c_str());
        // Resolve by name only after delivery. Copied session/source/context
        // guards distinguish a recreated document even when names/bytes match.
        bool contextValid=document&&valid()&&document==state->document&&state->dialog;
        std::string staleReason="Rebuild result request is canceled or superseded";
        if(contextValid){
            try {for(const auto& guard:result->guards)verifyObject(*document,guard);}
            catch(const std::exception& e){contextValid=false;staleReason=e.what();}
        }
        if(!om9_result_claim(&state->lease->gate,result->ticket,contextValid))
            throw std::runtime_error(staleReason);
        const auto& outputs=result->outputs;
        if(result->commit){
            requireLayerGeometryEditable(*document);
            const auto finalWitnesses=state->witnesses(*document);phase2Require(om9_phase2_rebuild_check(state->session,finalWitnesses.data(),finalWitnesses.size()),state->session);
            const int transaction=om9OpenTransaction(*document,"Rebuild Curve");
            try{
                if(!valid()||App::GetApplication().getDocument(result->documentName.c_str())!=document||!ownsLayerGeometryTransaction(*document,transaction))throw std::runtime_error("Rebuild transaction ownership changed");
                for(const auto& guard:result->guards)verifyObject(*document,guard);
                const auto commitWitnesses=state->witnesses(*document);phase2Require(om9_phase2_rebuild_check(state->session,commitWitnesses.data(),commitWitnesses.size()),state->session);
                std::unique_ptr<LayerGeometryTransaction> layers;if(hasCanonicalLayerDocument(*document))layers=std::make_unique<LayerGeometryTransaction>(*document,transaction);
                for(std::size_t i=0;i<outputs.size();++i){
                    if(!valid()||App::GetApplication().getDocument(result->documentName.c_str())!=document||!ownsLayerGeometryTransaction(*document,transaction))throw std::runtime_error("Rebuild transaction ownership changed");
                    for(const auto& guard:result->guards)verifyObject(*document,guard);
                    auto* object=createCurveFeature(*document,outputs[i],"Rebuild");
                    if(!valid()||App::GetApplication().getDocument(result->documentName.c_str())!=document||!ownsLayerGeometryTransaction(*document,transaction))throw std::runtime_error("Rebuild transaction ownership changed");
                    for(const auto& guard:result->guards)verifyObject(*document,guard);
                    auto* gui=Gui::Application::Instance->getDocument(document);
                    auto* oldView=gui?gui->getViewProvider(document->getObject(result->guards[i].objectName.c_str())):nullptr;
                    auto* newView=gui?gui->getViewProvider(object):nullptr;
                    if(oldView&&newView)for(const char* attr:{"LineColor","PointColor","LineWidth"})if(auto* property=oldView->getPropertyByName(attr))if(auto* target=newView->getPropertyByName(attr))target->Paste(*property);
                }
                if(result->remove)for(const auto& guard:result->guards)document->removeObject(guard.objectName.c_str());
                document->recompute();if(layers)layers->finish();om9CommitTransaction(*document);
            }catch(...){if(App::GetApplication().getDocument(result->documentName.c_str())==document&&ownsLayerGeometryTransaction(*document,transaction))om9AbortTransaction(*document);throw;}
            state->lastOutcome="committed";const auto command=result->command;cancel();om9_sidebar_record_execution(command,true);CurveController::instance().setPrompt("Command: ");CurveController::instance().logMessage("Rebuild completed");return;
        }
        attaching=true;
        for(const auto& output:outputs){
            const auto sampled=sampleCurve(output,257);std::vector<SbVec3f> points;for(auto p:sampled)points.emplace_back(float(p[0]),float(p[1]),float(p[2]));
            for(auto* base:Gui::Application::Instance->activeDocument()->getMDIViews())if(auto* view=dynamic_cast<Gui::View3DInventor*>(base)){
                auto* root=dynamic_cast<SoSeparator*>(view->getViewer()->getSceneGraph());if(!root)continue;root->ref();auto* node=new SoSeparator;root->addChild(node);state->previews.emplace_back(root,node);
                node->setName("OM9RebuildPreview");auto* pick=new SoPickStyle;pick->style=SoPickStyle::UNPICKABLE;node->addChild(pick);auto* color=new SoBaseColor;color->rgb.setValue(0.2f,1,0.65f);node->addChild(color);
                auto* coords=new SoCoordinate3;coords->point.setValues(0,int(points.size()),points.data());node->addChild(coords);auto* line=new SoLineSet;line->numVertices.set1Value(0,int(points.size()));node->addChild(line);view->getViewer()->redraw();
            }
        }
        state->lastOutcome="previewed";
        state->error->clear();state->deviation->setText(QString("Maximum deviation (sampled): %1 mm").arg(result->maximum,0,'g',8));
    }catch(const Standard_Failure& e){
        state->lastReason=e.GetMessageString();
        if(attaching)clearPreview();
        CurveController::instance().logMessage(QString::fromUtf8(e.GetMessageString()));
        if(result->commit&&state->dialog&&valid())state->error->setText(QString::fromUtf8(e.GetMessageString()));
    }catch(const std::exception& e){
        state->lastReason=e.what();
        if(attaching)clearPreview();
        // A late old result must not remove a newer preview or overwrite its UI.
        CurveController::instance().logMessage(QString::fromUtf8(e.what()));
        if(result->commit&&state->dialog&&valid())state->error->setText(QString::fromUtf8(e.what()));
    }
}
PyObject* CoreRebuild::deliveryTest(const char* action){
    if(QThread::currentThread()!=qApp->thread())throw std::runtime_error("Rebuild delivery fixture requires GUI thread");
    const std::string operation(action);
    if(!state->deliveryContext&&(operation=="hold"||operation=="normal"))throw std::runtime_error("Rebuild delivery is shut down");
    if(operation=="hold")state->holdDelivery=true;
    else if(operation=="shutdown")shutdownDelivery();
    else if(operation=="normal"){
        state->holdDelivery=false;
        while(!state->held.empty()){auto result=std::move(state->held.front());state->held.pop_front();QTimer::singleShot(0,state->deliveryContext,[this,result]{publish(result);});}
    }else if(operation=="oldest"||operation=="newest"){
        if(state->held.empty())throw std::runtime_error("No held native Rebuild result");
        auto result=operation=="oldest"?std::move(state->held.front()):std::move(state->held.back());
        if(operation=="oldest")state->held.pop_front();else state->held.pop_back();
        publish(result);result.reset();
    }else if(operation!="stats")throw std::runtime_error("Unknown Rebuild delivery fixture action");
    CurvePyRef report(Py_BuildValue("{s:K,s:K,s:K,s:K,s:K,s:K,s:K,s:K,s:K,s:s,s:s}",
        "created",state->lease->created,"released",state->lease->released,"shapes_created",state->lease->shapesCreated,"shapes_released",state->lease->shapesReleased,
        "live_envelopes",state->lease->created-state->lease->released,"held",static_cast<std::uint64_t>(state->held.size()),
        "preview_nodes",static_cast<std::uint64_t>(state->previews.size()),"last_ticket",state->lastTicket,
        "retire_failures",state->lease->retireFailures,"last_outcome",state->lastOutcome.c_str(),"last_reason",state->lastReason.c_str()));
    CurvePyRef held(PyList_New(0)),last(PyList_New(0));
    for(const auto& result:state->held){
        CurvePyRef row(Py_BuildValue("{s:K,s:s,s:K}","ticket",result->ticket,"document_name",result->documentName.c_str(),"source_count",static_cast<std::uint64_t>(result->guards.size()))),breps(PyList_New(0));
        for(const auto& output:result->outputs){const auto serialized=shapeBytes(output);CurvePyRef bytes(PyUnicode_FromStringAndSize(serialized.data(),Py_ssize_t(serialized.size())));if(PyList_Append(breps.value,bytes.value)<0)throw std::runtime_error("Cannot append result diagnostics");}
        if(PyDict_SetItemString(row.value,"breps",breps.value)<0||PyList_Append(held.value,row.value)<0)throw std::runtime_error("Cannot append held diagnostics");
    }
    for(const auto& bytes:state->lastShapeBytes){CurvePyRef text(PyUnicode_FromStringAndSize(bytes.data(),Py_ssize_t(bytes.size())));if(PyList_Append(last.value,text.value)<0)throw std::runtime_error("Cannot append delivered diagnostics");}
    if(PyDict_SetItemString(report.value,"held_results",held.value)<0||PyDict_SetItemString(report.value,"last_breps",last.value)<0)throw std::runtime_error("Cannot publish result diagnostics");
    return Py_NewRef(report.value);
}
void CoreRebuild::addDeliveryTestMethods(PyObject* module){
    if(qgetenv("OM9_TEST_NATIVE_DIR").isEmpty())return;
    static PyMethodDef methods[]={{"rebuildDeliveryTest",[](PyObject*,PyObject* args)->PyObject*{
        const char* action=nullptr;if(!PyArg_ParseTuple(args,"s",&action))return nullptr;
        try{return instance().deliveryTest(action);}catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
    },METH_VARARGS,"Native fixture-only hold/release of real solved Rebuild envelopes."},{nullptr,nullptr,0,nullptr}};
    if(PyModule_AddFunctions(module,methods)<0)throw std::runtime_error("Cannot register Rebuild delivery fixture API");
}
}
