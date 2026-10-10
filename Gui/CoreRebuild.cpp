#include "CoreRebuild.h"
#include "CurveController.h"
#include "CurveGeometry.h"
#include "LayerDocumentAdapter.h"
#include "RustBridge.h"
#include "SnapObjectInfo.h"
#include <App/PropertyStandard.h>
#include <Mod/Part/App/PropertyTopoShape.h>
#include <BRepBuilderAPI_GTransform.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepExtrema_DistShapeShape.hxx>
#include <BRep_Tool.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <Geom_BSplineCurve.hxx>
#include <TopExp_Explorer.hxx>
#include <QByteArray>
#include <App/Application.h>
#include <App/Document.h>
#include <App/GeoFeature.h>
#include <App/PropertyGeo.h>

#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Vertex.hxx>
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
namespace OpenMatrix9Gui {
struct CoreRebuild::State {
    struct Input {
        std::string name;long id=0;TopoDS_Shape original,edge;bool closed=false;
        Base::Matrix4D transform;std::uint64_t generation=0;QByteArray signature;
        int count=0,degree=0;
    };
    std::uint64_t session=0;
    std::vector<Om9RebuildInput> witnesses(App::Document& doc) {
        std::vector<Om9RebuildInput> result;
        std::vector<std::string> names;names.reserve(inputs.size());for(const auto& input:inputs)names.push_back(input->name);
        const auto layerGeneration=layerMutationGeneration(doc,names,1);
        for(auto& input:inputs){
            auto* object=doc.getObject(input->name.c_str());const auto info=classifySnapObject(object);
            const bool same=object&&object->getID()==input->id&&!info.shape.IsNull()&&info.shape.IsEqual(input->original)&&info.global_transform==input->transform;
            input->signature=QByteArray::fromStdString(doc.Uid.getValueStr())+(same?":unchanged":":changed")+":"+QByteArray::number(qulonglong(layerGeneration));
            result.push_back({{std::uint64_t(reinterpret_cast<std::uintptr_t>(&doc)),object?std::uint64_t(object->getID())+1:0,input->generation,reinterpret_cast<const std::uint8_t*>(input->signature.constData()),std::size_t(input->signature.size())},object&&!object->getInList().empty()?1u:0u});
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
    fastsignals::scoped_connection documentChanged,documentDeleted,objectChanged,undo,redo;
};
CoreRebuild& CoreRebuild::instance(){static auto* tool=new CoreRebuild;return *tool;}
CoreRebuild::CoreRebuild():QObject(qApp),state(std::make_unique<State>()) {
    qApp->installEventFilter(this);
    state->documentChanged=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(active()&&state->document!=&doc)cancel();});
    state->documentDeleted=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(state->document==&doc)cancel();});
    state->objectChanged=App::GetApplication().signalChangedObject.connect([this](const App::DocumentObject& object,const App::Property&){if(object.getDocument()==state->document)for(auto& input:state->inputs)if(input->id==object.getID())++input->generation;});
    state->undo=App::GetApplication().signalUndoDocument.connect([this](const App::Document& doc){if(state->document==&doc)cancel();});
    state->redo=App::GetApplication().signalRedoDocument.connect([this](const App::Document& doc){if(state->document==&doc)cancel();});
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
    om9_phase2_rebuild_drop(state->session);state->session=0;state->inputs.clear();
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
    try {
        state->inputs.clear();std::set<std::string> seen;
        const auto selected=Gui::Selection().getSelection(state->document->getName());if(selected.empty()){CurveController::instance().setPrompt("Rebuild: Select at least one curve, then press Enter.");return;}phase2Require(om9_phase2_rebuild_input_count(selected.size()));
        requireLayerGeometryEditable(*state->document);
        std::vector<std::string> selectedNames;selectedNames.reserve(selected.size());for(const auto& selection:selected)selectedNames.emplace_back(selection.FeatName);
        layerMutationGeneration(*state->document,selectedNames,1);
        for(const auto& selection:selected){
            if(selection.SubName&&*selection.SubName)throw std::runtime_error("Select whole curve objects; surface edges cannot be rebuilt here");
            if(!seen.insert(selection.FeatName).second)continue;
            auto* object=state->document->getObject(selection.FeatName);const auto info=classifySnapObject(object);
            if(!object||!info.native_cad||info.preview||info.kind!=2)throw std::runtime_error("Rebuild requires native curves");
            gp_GTrsf transform;for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)transform.SetValue(i,j,info.global_transform[i-1][j-1]);
            const auto world=BRepBuilderAPI_GTransform(info.shape,transform,true).Shape();
            if(TopExp_Explorer(world,TopAbs_FACE).More())throw std::runtime_error("Rebuild requires curves, not faces");
            std::size_t edges=0;TopoDS_Edge first;for(TopExp_Explorer e(world,TopAbs_EDGE);e.More();e.Next()){if(edges==0)first=TopoDS::Edge(e.Current());++edges;}
            if(!edges)throw std::runtime_error("Selection has no curve edges");TopoDS_Shape sampled=world;
            if(edges>1){TopExp_Explorer wires(world,TopAbs_WIRE);if(!wires.More())throw std::runtime_error("Select one connected curve per object");const auto wire=TopoDS::Wire(wires.Current());wires.Next();if(wires.More())throw std::runtime_error("Select one connected curve per object");validateCurveWire(wire,edges);sampled=wire;}
            auto input=std::make_unique<State::Input>();input->name=selection.FeatName;input->id=object->getID();input->original=info.shape;input->transform=info.global_transform;input->edge=sampled;input->closed=BRep_Tool::IsClosed(sampled);
            double begin,end;const auto curve=Handle(Geom_BSplineCurve)::DownCast(BRep_Tool::Curve(first,begin,end));if(!curve.IsNull()){input->count=curve->NbPoles();input->degree=curve->Degree();}
            state->inputs.push_back(std::move(input));
        }
        if(state->inputs.empty()){CurveController::instance().setPrompt("Rebuild: Select at least one curve, then press Enter.");return;}
        const auto inputs=state->witnesses(*state->document);const Om9RebuildOptions defaults{std::size_t(std::clamp(state->inputs[0]->degree,1,3)),std::size_t(std::clamp(state->inputs[0]->count,4,256)),1};
        state->session=om9_phase2_rebuild_create(inputs.data(),inputs.size(),&defaults);phase2Require(state->session!=0);
        state->selecting=false;
        auto* dialog=new QDialog(Gui::getMainWindow());state->dialog=dialog;dialog->setObjectName("OM9RebuildDialog");dialog->setProperty("om9RustSession",qulonglong(state->session));dialog->setWindowTitle("Rebuild Curve");dialog->setAttribute(Qt::WA_DeleteOnClose);
        auto* layout=new QFormLayout(dialog);
        QStringList current;for(const auto& i:state->inputs)current<<QString("%1: PointCount (%2), Degree (%3)").arg(QString::fromStdString(i->name)).arg(i->count?QString::number(i->count):"analytic").arg(i->degree?QString::number(i->degree):"analytic");
        layout->addRow(new QLabel(current.join('\n'),dialog));
        state->count=new QSpinBox(dialog);state->count->setObjectName("OM9RebuildPointCount");state->count->setRange(2,256);state->count->setValue(std::clamp(state->inputs[0]->count,4,256));layout->addRow("PointCount",state->count);
        state->degree=new QSpinBox(dialog);state->degree->setObjectName("OM9RebuildDegree");state->degree->setRange(1,11);state->degree->setValue(std::clamp(state->inputs[0]->degree,1,3));layout->addRow("Degree",state->degree);
        state->remove=new QCheckBox("DeleteInput",dialog);state->remove->setObjectName("OM9RebuildDeleteInput");state->remove->setChecked(true);layout->addRow(state->remove);
        auto* layer=new QCheckBox("New objects use current layer",dialog);layer->setChecked(true);layer->setEnabled(false);layer->setToolTip("Output inherits the current layer and its color; input own state remains independent.");layout->addRow(layer);
        state->deviation=new QLabel("Maximum deviation: click Preview",dialog);state->deviation->setObjectName("OM9RebuildDeviation");layout->addRow(state->deviation);
        state->error=new QLabel(dialog);state->error->setWordWrap(true);state->error->setObjectName("OM9RebuildError");layout->addRow(state->error);
        auto* buttons=new QDialogButtonBox(dialog);auto* ok=buttons->addButton(QDialogButtonBox::Ok);ok->setObjectName("OM9RebuildOK");auto* no=buttons->addButton(QDialogButtonBox::Cancel);no->setObjectName("OM9RebuildCancel");auto* preview=buttons->addButton("Preview",QDialogButtonBox::ActionRole);preview->setObjectName("OM9RebuildPreview");layout->addRow(buttons);
        connect(preview,&QPushButton::clicked,this,[this]{calculate(false);});connect(ok,&QPushButton::clicked,this,[this]{calculate(true);});connect(no,&QPushButton::clicked,this,[this]{cancel();CurveController::instance().setPrompt("Command: ");});
        connect(dialog,&QDialog::rejected,this,[this]{cancel();CurveController::instance().setPrompt("Command: ");});
        auto stale=[this]{clearPreview();if(state->dialog){state->deviation->setText("Maximum deviation: click Preview");state->error->clear();}};
        connect(state->count,&QSpinBox::valueChanged,this,stale);connect(state->degree,&QSpinBox::valueChanged,this,stale);
        CurveController::instance().setPrompt("Rebuild: Adjust options, Preview, then OK; Cancel discards the preview.");dialog->show();
    }catch(const Standard_Failure& e){cancel();CurveController::instance().setPrompt(QString::fromUtf8(e.GetMessageString()));}
    catch(const std::exception& e){cancel();CurveController::instance().setPrompt(QString::fromUtf8(e.what()));CurveController::instance().logMessage(QString::fromUtf8(e.what()));}
}
void CoreRebuild::calculate(bool commit){
    if(!valid()||!state->dialog){cancel();return;}
    clearPreview();
    try {
        requireLayerGeometryEditable(*state->document);
        const Om9RebuildOptions options{std::size_t(state->degree->value()),std::size_t(state->count->value()),state->remove->isChecked()?1u:0u};phase2Require(om9_phase2_rebuild_replace(state->session,&options),state->session);
        const auto current=state->witnesses(*state->document);phase2Require(om9_phase2_rebuild_check(state->session,current.data(),current.size()),state->session);
        Om9RebuildOptions owned{};phase2Require(om9_phase2_rebuild_get(state->session,&owned),state->session);const auto poles=owned.point_count,degree=owned.degree;
        std::vector<TopoDS_Shape> outputs;double maximum=0;
        for(const auto& input:state->inputs){
            const auto n=std::max<std::size_t>(513,poles*4+1);auto samples=sampleCurve(input->edge,n);if(input->closed)samples.back()=samples.front();
            std::vector<double> xyz;xyz.reserve(n*3);for(const auto& point:samples)xyz.insert(xyz.end(),point.begin(),point.end());
            if(!om9_spline_rebuild(xyz.data(),n,poles,degree,input->closed)){char message[1024]={};om9_spline_message(message,sizeof(message));throw std::runtime_error(message);}
            outputs.push_back(publishedSplineShape());
            if(!commit){for(const auto& pair:{std::pair{input->edge,outputs.back()},std::pair{outputs.back(),input->edge}})for(const auto& point:sampleCurve(pair.first,129)){
                const auto vertex=BRepBuilderAPI_MakeVertex(gp_Pnt(point[0],point[1],point[2])).Vertex();BRepExtrema_DistShapeShape distance(vertex,pair.second);if(!distance.IsDone()||!std::isfinite(distance.Value()))throw std::runtime_error("Cannot measure native rebuild deviation");maximum=std::max(maximum,distance.Value());
            }}
        }
        if(commit){
            requireLayerGeometryEditable(*state->document);
            const auto finalWitnesses=state->witnesses(*state->document);phase2Require(om9_phase2_rebuild_check(state->session,finalWitnesses.data(),finalWitnesses.size()),state->session);
            const int transaction=state->document->openTransaction("Rebuild Curve");
            try{
                LayerGeometryTransaction layers(*state->document,transaction);
                const auto commitWitnesses=state->witnesses(*state->document);phase2Require(om9_phase2_rebuild_check(state->session,commitWitnesses.data(),commitWitnesses.size()),state->session);
                for(std::size_t i=0;i<outputs.size();++i){
                    auto* object=createCurveFeature(*state->document,outputs[i],"Rebuild");
                    auto* gui=Gui::Application::Instance->activeDocument();auto* oldView=gui->getViewProvider(state->document->getObject(state->inputs[i]->name.c_str()));auto* newView=gui->getViewProvider(object);
                    if(oldView&&newView)for(const char* name:{"LineColor","PointColor","LineWidth"})if(auto* oldProperty=oldView->getPropertyByName(name))if(auto* newProperty=newView->getPropertyByName(name))newProperty->Paste(*oldProperty);
                }
                if(owned.delete_input)for(const auto& input:state->inputs)state->document->removeObject(input->name.c_str());
                state->document->recompute();layers.finish();state->document->commitTransaction();
            }catch(...){if(ownsLayerGeometryTransaction(*state->document,transaction))state->document->abortTransaction();throw;}
            const auto command=state->command;cancel();om9_sidebar_record_execution(command,true);CurveController::instance().setPrompt("Command: ");CurveController::instance().logMessage("Rebuild completed");return;
        }
        for(const auto& output:outputs){
            const auto sampled=sampleCurve(output,257);std::vector<SbVec3f> points;for(auto p:sampled)points.emplace_back(float(p[0]),float(p[1]),float(p[2]));
            for(auto* base:Gui::Application::Instance->activeDocument()->getMDIViews())if(auto* view=dynamic_cast<Gui::View3DInventor*>(base)){
                auto* root=dynamic_cast<SoSeparator*>(view->getViewer()->getSceneGraph());if(!root)continue;root->ref();auto* node=new SoSeparator;root->addChild(node);state->previews.emplace_back(root,node);
                node->setName("OM9RebuildPreview");auto* pick=new SoPickStyle;pick->style=SoPickStyle::UNPICKABLE;node->addChild(pick);auto* color=new SoBaseColor;color->rgb.setValue(0.2f,1,0.65f);node->addChild(color);
                auto* coords=new SoCoordinate3;coords->point.setValues(0,int(points.size()),points.data());node->addChild(coords);auto* line=new SoLineSet;line->numVertices.set1Value(0,int(points.size()));node->addChild(line);view->getViewer()->redraw();
            }
        }
        state->error->clear();state->deviation->setText(QString("Maximum deviation (sampled): %1 mm").arg(maximum,0,'g',8));
    }catch(const Standard_Failure& e){clearPreview();if(state->dialog)state->error->setText(QString::fromUtf8(e.GetMessageString()));}
    catch(const std::exception& e){clearPreview();if(state->dialog){state->error->setText(QString::fromUtf8(e.what()));state->deviation->setText("Maximum deviation: unavailable");}}
}
}
