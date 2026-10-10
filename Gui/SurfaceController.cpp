// SPDX-License-Identifier: LGPL-2.1-or-later
#include "SurfaceController.h"
#include "EditController.h"
#include "SurfaceSeams.h"
#include "SurfaceRefit.h"
#include "SurfaceConstraints.h"
#include "RustBridge.h"
#include "CurveController.h"
#include "CoreDistance.h"
#include "CorePictureFrame.h"
#include "CoreViewControls.h"
#include "CoreKeyboard.h"
#include <Base/Interpreter.h>
#include <Base/Console.h>
#include <App/Application.h>
#include <App/Document.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <Gui/MainWindow.h>
#include <Gui/Selection/Selection.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/ViewProviderDocumentObject.h>
#include <Inventor/SoDB.h>
#include <Inventor/SoInput.h>
#include <Inventor/SoPickedPoint.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <QApplication>
#include <QDialog>
#include <QDialogButtonBox>
#include <QPushButton>
#include <QLabel>
#include <QCheckBox>
#include <QComboBox>
#include <QDoubleSpinBox>
#include <QSpinBox>
#include <QFormLayout>
#include <QVBoxLayout>
#include <QTableWidget>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QTimer>
#include <QSignalBlocker>
#include <memory>
#include <algorithm>
#include <cmath>
#include <cstring>
#include <stdexcept>
namespace {
QString message(){char text[2048]={};om9_surface_message(text,sizeof(text));return QString::fromUtf8(text);}
Gui::View3DInventor* containing(QObject* object){
    auto* widget=qobject_cast<QWidget*>(object);auto* gui=Gui::Application::Instance->activeDocument();if(!widget||!gui)return nullptr;
    for(auto* mdi:gui->getMDIViews())if(auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);view&&(widget==view->getViewer()||view->getViewer()->isAncestorOf(widget)))return view;
    return nullptr;
}
struct ShapeRef { PyObject* p; explicit ShapeRef(PyObject* p):p(p){}~ShapeRef(){Py_XDECREF(p);} };
}
namespace OpenMatrix9Gui {
SurfaceController& SurfaceController::instance(){static auto* instance=new SurfaceController;return *instance;}
SurfaceController::SurfaceController():QObject(qApp){
    deleteConnection=App::GetApplication().signalDeleteDocument.connect([this](const App::Document& doc){if(document==&doc)cancel();});
    activeConnection=App::GetApplication().signalActiveDocument.connect([this](const App::Document& doc){if(active()&&document!=&doc)cancel();});
    auto* timer=new QTimer(this);timer->setInterval(150);connect(timer,&QTimer::timeout,this,[this]{if(active()&&!valid())cancel();});timer->start();
}
bool SurfaceController::handles(std::size_t i){return om9_surface_kind(om9_command_id(i))!=0;}
bool SurfaceController::matches(std::size_t i,const QString& name){return handles(i)&&om9_surface_kind(name.toUtf8().constData())==om9_surface_kind(om9_command_id(i));}
void SurfaceController::activate(){enabled=true;qApp->installEventFilter(this);}
void SurfaceController::deactivate(){enabled=false;cancel();}
bool SurfaceController::active()const{return document&&om9_surface_phase()!=0;}
bool SurfaceController::available(std::size_t i)const{
    auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();
    return enabled&&handles(i)&&doc&&!om9ReadOnlyFile(*doc)&&!doc->testStatus(App::Document::Restoring)&&gui&&!gui->isAboutToClose()&&!gui->getInEdit()&&!dialog&&dynamic_cast<Gui::View3DInventor*>(gui->getActiveView())&&om9AlterDocument(doc);
}
bool SurfaceController::valid()const{
    auto* gui=Gui::Application::Instance->activeDocument();return enabled&&document&&document==App::GetApplication().getActiveDocument()&&!om9ReadOnlyFile(*document)&&!document->testStatus(App::Document::Restoring)&&gui&&!gui->isAboutToClose()&&!gui->getInEdit()&&om9AlterDocument(document);
}
void SurfaceController::prompt(const QString& text){CurveController::instance().setPrompt(text);CurveController::instance().logMessage(text);}
void SurfaceController::refresh(){
    const QString name=options.kind==1?"Sweep1":options.kind==2?"Sweep2":"Loft";
    const auto phase=om9_surface_phase();
    if(phase==1)prompt(name+": Select first rail (Esc cancels)");
    else if(phase==2)prompt(name+": Select second rail (Undo / Esc)");
    else if(phase==3)prompt(name+": Select profiles in order; Enter for options (Undo / Cancel)");
    else if(phase==5)prompt(name+": Chain Edges — select touching edges; Enter ends this rail (Undo / Cancel)");
}
bool SurfaceController::start(std::size_t i,const QString& invoked){
    if(!available(i))return false;
    if(invoked.contains("History",Qt::CaseInsensitive)){prompt("History commands are unavailable in this modeling workflow");return false;}
    EditController::instance().cancel();
    CoreDistance::instance().cancel();CorePictureFrame::instance().cancel();CoreViewControls::instance().cancel();CurveController::instance().cancel();cancel();
    command=i;document=App::GetApplication().getActiveDocument();options={om9_surface_kind(om9_command_id(i)),0,false,false};
    options.history=false;options.historyCommand=false;
    if(!om9_surface_start(om9_command_id(i))){document=nullptr;return false;}
    qApp->installEventFilter(this);refresh();
    // FreeCAD preserves ordered selection entries; subedges remain individual inputs.
    const auto selected=Gui::Selection().getSelection(document->getName());
    for(const auto& sel:selected)if(!add(sel.FeatName,sel.SubName?sel.SubName:"")){cancel();prompt("Preselection rejected as a whole; correct the selection and restart");break;}
    return false; // History is recorded only after a successful geometry commit.
}
void SurfaceController::clearPreview(){for(const auto& [root,node]:previews){if(root->findChild(node)>=0)root->removeChild(node);root->unref();}previews.clear();}
void SurfaceController::cancel(){
    om9_surface_cancel();document=nullptr;inputs.clear();chainInputs.clear();slashPicking=0;slashTable=nullptr;clearPreview();
    if(dialog){auto* old=dialog.data();dialog=nullptr;old->disconnect(this);old->hide();old->deleteLater();}status=nullptr;buttons=nullptr;
}
void SurfaceController::error(const std::exception& e){const auto text=QString::fromUtf8(e.what());prompt(text);if(status)status->setText(text);if(buttons)buttons->button(QDialogButtonBox::Ok)->setEnabled(false);Base::Console().warning("OpenMatrix9 surface: %s\n",e.what());}
bool SurfaceController::add(const std::string& object,const std::string& sub){
    if(!active()||!valid()||om9_surface_phase()==4)return false;
    try{
        Base::PyGILStateLocker lock;SurfaceInput input{object,sub};input.guards.push_back(capturePhase3Object(*document,object,sub));ShapeRef wire(surfaceWire(*document,input));ShapeRef closed(PyObject_CallMethod(wire.p,"isClosed",nullptr));
        if(!closed.p){PyErr_Clear();throw std::runtime_error("Cannot inspect curve closure");}input.closed=PyObject_IsTrue(closed.p)==1;
        const auto key=object+"."+sub;
        if(om9_surface_phase()==5){
            ShapeRef edges(PyObject_GetAttrString(wire.p,"Edges"));
            if(!edges.p||PySequence_Size(edges.p)!=1)throw std::runtime_error("Chain Edges requires one edge per pick");
            if(!om9_surface_chain_add(key.c_str())){prompt(message());return false;}
            chainInputs.push_back(input);refresh();return true;
        }
        if(!om9_surface_add(key.c_str(),input.closed)){prompt(message());return false;}
        inputs.push_back(input);CurveController::instance().logMessage("Selected: "+QString::fromStdString(key));refresh();return true;
    }catch(const std::exception& e){error(e);return false;}
}
void SurfaceController::submit(const QString& text){
    if(!active())return;if(!valid()){cancel();return;}const auto input=text.trimmed();
    if(input.compare("Cancel",Qt::CaseInsensitive)==0||input.compare("Esc",Qt::CaseInsensitive)==0){cancel();prompt("Surface command cancelled");return;}
    if(dialog){
        if(input.isEmpty()&&slashPicking){slashPicking=0;preview();prompt("Slash picking finished; adjust options or click OK");return;}
        if(input.startsWith("AddSlash=",Qt::CaseInsensitive)){
            const auto values=input.mid(9).split(',');bool first=false,second=false;
            const double a=values.size()==2?values[0].toDouble(&first):0,b=values.size()==2?values[1].toDouble(&second):0;
            if(first&&second)addSlash(a,b);else prompt("Use AddSlash=railA_fraction,railB_fraction");return;
        }
        if(input.compare("AddSlash",Qt::CaseInsensitive)==0){
            auto* button=dialog->findChild<QPushButton*>("OM9AddSlash");if(button&&button->isEnabled())button->click();else prompt("Add Slash needs one profile and two open rails");return;}
        if(input.compare("ClearSlashes",Qt::CaseInsensitive)==0){options.slashes.clear();updateSlashes();preview();return;}
        if(input.isEmpty()||input.compare("OK",Qt::CaseInsensitive)==0)commit();
        else if(input.compare("Automatic",Qt::CaseInsensitive)==0)alignInputs(true);
        else if(input.compare("Natural",Qt::CaseInsensitive)==0)alignInputs(false);
        else if(input.compare("Preview",Qt::CaseInsensitive)==0){const bool enabled=options.preview;options.preview=true;preview();options.preview=enabled;}
        else prompt("Use the surface options dialog, Automatic, Natural, Preview, OK or Cancel");return;
    }
    if(input.compare("ChainEdges",Qt::CaseInsensitive)==0||input.compare("Chain Edges",Qt::CaseInsensitive)==0){if(om9_surface_chain_start()){chainInputs.clear();refresh();}else prompt(message());return;}
    if(input.compare("Undo",Qt::CaseInsensitive)==0){const bool chain=om9_surface_phase()==5;if(om9_surface_undo()){if(chain)chainInputs.pop_back();else inputs.pop_back();refresh();}else prompt(message());return;}
    if(input.isEmpty()){
        if(om9_surface_phase()==5){
            if(chainInputs.empty()){prompt("Select touching rail edges first");return;}
            try {Base::PyGILStateLocker lock;SurfaceInput rail=chainInputs.front();
                rail.guards.clear();
                for(const auto& edge:chainInputs){rail.chain.emplace_back(edge.object,edge.sub);rail.guards.insert(rail.guards.end(),edge.guards.begin(),edge.guards.end());}
                ShapeRef wire(surfaceWire(*document,rail)),closed(PyObject_CallMethod(wire.p,"isClosed",nullptr));
                if(!closed.p)throw std::runtime_error("Cannot inspect rail chain");rail.closed=PyObject_IsTrue(closed.p)==1;
                if(!om9_surface_chain_finish(rail.closed)){prompt(message());return;}
                inputs.push_back(rail);chainInputs.clear();refresh();
            }catch(const std::exception& e){error(e);}return;
        }
        if(!om9_surface_finish()){prompt(message());return;}optionsDialog();return;
    }
    // Object names, optionally Object.EdgeN, are accepted by the same handler as picks.
    const auto dot=input.indexOf('.');const auto object=dot<0?input:input.left(dot),sub=dot<0?QString():input.mid(dot+1);
    add(object.toUtf8().constData(),sub.toUtf8().constData());
}
void SurfaceController::optionsDialog(){
    if(options.historyCommand&&options.kind!=3){const unsigned rails=options.kind==1?1:2;bool closed=inputs.size()>=rails+2;for(unsigned i=0;i<rails;++i)closed=closed&&inputs[i].closed;options.closed=closed;}
    if(options.historyCommand&&options.kind==2&&inputs.size()==3)options.maintainHeight=true;
    dialog=new QDialog(Gui::getMainWindow());dialog->setObjectName("OM9SurfaceOptions");dialog->setWindowTitle(options.kind==1?"Sweep 1 Options":options.kind==2?"Sweep 2 Options":"Loft Options");dialog->setAttribute(Qt::WA_DeleteOnClose);
    auto* layout=new QVBoxLayout(dialog);auto* form=new QFormLayout;layout->addLayout(form);
    if(options.kind==3){
        auto* style=new QComboBox(dialog);style->setObjectName("OM9LoftStyle");style->addItems({"Normal","Straight Sections","Loose","Tight","Uniform","Developable"});form->addRow("Style",style);
        connect(style,&QComboBox::currentIndexChanged,this,[this](int i){options.style=i;preview();});
        auto* closed=new QCheckBox("Connect last section to first",dialog);closed->setObjectName("OM9ClosedLoft");closed->setEnabled(inputs.size()>=3);form->addRow("Closed loft",closed);connect(closed,&QCheckBox::toggled,this,[this](bool b){options.closed=b;preview();});
    }else if(options.kind==1){
        auto* frenet=new QCheckBox("Frenet frame",dialog);frenet->setObjectName("OM9SweepFrenet");form->addRow("Orientation",frenet);connect(frenet,&QCheckBox::toggled,this,[this](bool b){options.frenet=b;preview();});
    }
    if(options.kind==2){
        auto* height=new QCheckBox("Keep section height as rail width changes",dialog);height->setObjectName("OM9MaintainHeight");height->setEnabled(inputs.size()==3);
        height->setChecked(options.maintainHeight);
        height->setToolTip("Supported with one profile. Multiple-profile height constraints require a different solver.");form->addRow("Maintain Height",height);
        connect(height,&QCheckBox::toggled,this,[this](bool b){options.maintainHeight=b;preview();});
    }
    if(options.kind!=3){
        auto* closed=new QCheckBox("Connect last profile to first on closed rails",dialog);closed->setObjectName("OM9ClosedSweep");
        const auto rails=options.kind==1?1u:2u;bool enabled=inputs.size()>=rails+2;
        for(unsigned i=0;i<rails;++i)enabled=enabled&&inputs[i].closed;closed->setEnabled(enabled);form->addRow("Closed Sweep",closed);
        closed->setChecked(options.closed);
        connect(closed,&QCheckBox::toggled,this,[this](bool b){options.closed=b;preview();});
    }
    auto* sectionMode=new QComboBox(dialog);sectionMode->setObjectName("OM9SectionMode");sectionMode->addItems({"Do Not Simplify","Rebuild","Refit"});form->addRow("Cross-section curves",sectionMode);
    auto* pointCount=new QSpinBox(dialog);pointCount->setObjectName("OM9SectionPointCount");pointCount->setRange(2,256);pointCount->setValue(16);pointCount->setEnabled(false);form->addRow("Control points",pointCount);
    connect(sectionMode,&QComboBox::currentIndexChanged,this,[this,pointCount](int i){options.sectionMode=i;pointCount->setEnabled(i==1);preview();});
    connect(pointCount,&QSpinBox::valueChanged,this,[this](int i){options.pointCount=i;preview();});
    auto* tolerance=new QDoubleSpinBox(dialog);tolerance->setObjectName("OM9RefitTolerance");tolerance->setDecimals(7);tolerance->setRange(1e-7,1e6);tolerance->setValue(options.tolerance);tolerance->setSuffix(" mm");tolerance->setEnabled(false);form->addRow("Refit tolerance",tolerance);
    connect(sectionMode,&QComboBox::currentIndexChanged,this,[this,tolerance](int i){tolerance->setEnabled(i==2);preview();});
    connect(tolerance,&QDoubleSpinBox::valueChanged,this,[this](double value){options.tolerance=value;preview();});
    {Base::PyGILStateLocker lock;
        if(options.kind==2){
            const bool eligible=surfaceConstraintProfilesEligible(*document,inputs,2);
            for(unsigned rail=0;rail<2;++rail){auto* continuity=new QComboBox(dialog);continuity->setObjectName(rail==0?"OM9ContinuityA":"OM9ContinuityB");continuity->addItems({"Position (G0)","Tangency (G1)","Curvature (G2)"});
                continuity->setEnabled(eligible&&surfaceConstraintHasSupport(*document,inputs[rail]));
                continuity->setToolTip("Requires an unambiguous surface edge, two or more matching open nonrational profiles and original section mode.");form->addRow(rail==0?"Rail A continuity":"Rail B continuity",continuity);
                connect(continuity,&QComboBox::currentIndexChanged,this,[this,rail](int value){(rail==0?options.continuityA:options.continuityB)=value;preview();});}
        }else if(options.kind==3){
            const bool eligible=surfaceConstraintProfilesEligible(*document,inputs,0);
            for(unsigned end=0;end<2;++end){auto* match=new QCheckBox("Maintain support-surface tangency",dialog);match->setObjectName(end==0?"OM9MatchStartTangents":"OM9MatchEndTangents");
                match->setEnabled(eligible&&surfaceConstraintHasSupport(*document,end==0?inputs.front():inputs.back()));form->addRow(end==0?"Match Start Tangent":"Match End Tangent",match);
                connect(match,&QCheckBox::toggled,this,[this,end](bool value){(end==0?options.matchStart:options.matchEnd)=value;preview();});}
        }
    }
    if(options.kind==2){
        auto* add=new QPushButton("Add Slash — pick a point on each rail",dialog);add->setObjectName("OM9AddSlash");add->setEnabled(inputs.size()==3&&!inputs[0].closed&&!inputs[1].closed);form->addRow(add);
        connect(add,&QPushButton::clicked,this,[this]{slashPicking=1;prompt("Add Slash: pick a point on rail A, then rail B; Enter finishes");});
        slashTable=new QTableWidget(0,2,dialog);slashTable->setObjectName("OM9SlashInputs");slashTable->setHorizontalHeaderLabels({"Rail A arc fraction","Rail B arc fraction"});form->addRow(slashTable);
        auto* remove=new QPushButton("Remove selected slash",dialog);remove->setObjectName("OM9RemoveSlash");form->addRow(remove);
        connect(remove,&QPushButton::clicked,this,[this]{const int row=slashTable?slashTable->currentRow():-1;if(row>=0&&std::size_t(row)<options.slashes.size()){options.slashes.erase(options.slashes.begin()+row);updateSlashes();preview();}});
    }
    auto* dynamic=new QCheckBox("Update preview after option changes",dialog);dynamic->setObjectName("OM9SurfaceDynamicPreview");dynamic->setChecked(true);form->addRow("Preview",dynamic);
    connect(dynamic,&QCheckBox::toggled,this,[this](bool b){options.preview=b;preview();});
    auto* previewButton=new QPushButton("Preview",dialog);previewButton->setObjectName("OM9SurfacePreviewButton");form->addRow(previewButton);connect(previewButton,&QPushButton::clicked,this,[this]{const bool enabled=options.preview;options.preview=true;preview();options.preview=enabled;});
    auto* table=new QTableWidget(int(inputs.size()),3,dialog);table->setObjectName("OM9SurfaceInputs");table->setHorizontalHeaderLabels({"Input order","Reverse","Seam (0–1)"});layout->addWidget(table);
    const unsigned rails=options.kind==1?1:options.kind==2?2:0;
    for(std::size_t i=0;i<inputs.size();++i){
        const auto& item=inputs[i];auto* label=new QTableWidgetItem(QString("%1: %2%3").arg(i<rails?"Rail":"Profile").arg(QString::fromStdString(item.object)).arg(item.sub.empty()?QString():"."+QString::fromStdString(item.sub)));label->setFlags(label->flags()&~Qt::ItemIsEditable);table->setItem(int(i),0,label);
        auto* reverse=new QCheckBox(table);table->setCellWidget(int(i),1,reverse);connect(reverse,&QCheckBox::toggled,this,[this,i](bool b){inputs[i].reverse=b;preview();});
        auto* seam=new QDoubleSpinBox(table);seam->setDecimals(4);seam->setRange(0,.9999);seam->setSingleStep(.05);seam->setEnabled(item.closed&&i>=rails);table->setCellWidget(int(i),2,seam);connect(seam,&QDoubleSpinBox::valueChanged,this,[this,i](double v){inputs[i].seam=v;preview();});
    }
    table->resizeColumnsToContents();
    auto* automatic=new QPushButton("Automatic",dialog);automatic->setObjectName("OM9SurfaceAutomaticSeams");form->addRow("Align seams / directions",automatic);connect(automatic,&QPushButton::clicked,this,[this]{alignInputs(true);});
    auto* natural=new QPushButton("Natural",dialog);natural->setObjectName("OM9SurfaceNaturalSeams");form->addRow("Restore original seams",natural);connect(natural,&QPushButton::clicked,this,[this]{alignInputs(false);});
    status=new QLabel(dialog);status->setObjectName("OM9SurfaceStatus");status->setWordWrap(true);layout->addWidget(status);
    buttons=new QDialogButtonBox(QDialogButtonBox::Ok|QDialogButtonBox::Cancel,dialog);layout->addWidget(buttons);
    connect(buttons,&QDialogButtonBox::accepted,this,[this]{commit();});connect(buttons,&QDialogButtonBox::rejected,dialog,&QDialog::reject);
    connect(dialog,&QDialog::rejected,this,[this]{cancel();prompt("Surface command cancelled");});
    dialog->resize(560,380);dialog->show();preview();
}
void SurfaceController::updateSlashes(){
    if(!slashTable)return;slashTable->setRowCount(int(options.slashes.size()));
    for(std::size_t i=0;i<options.slashes.size();++i)for(int column=0;column<2;++column){
        auto* item=new QTableWidgetItem(QString::number(column==0?options.slashes[i].first:options.slashes[i].second,'g',12));
        item->setFlags(item->flags()&~Qt::ItemIsEditable);slashTable->setItem(int(i),column,item);}
}
void SurfaceController::addSlash(double a,double b){
    if(options.kind!=2||inputs.size()!=3||inputs[0].closed||inputs[1].closed||options.continuityA||options.continuityB){prompt("Add Slash needs one profile and two open rails without G1/G2 constraints");return;}
    auto candidate=options.slashes;candidate.emplace_back(a,b);std::sort(candidate.begin(),candidate.end());
    std::vector<double> pairs;for(const auto& [x,y]:candidate)pairs.insert(pairs.end(),{x,y});
    if(!std::isfinite(om9_surface_slash_parameter(pairs.data(),candidate.size(),.5))){prompt("Slash pairs must be strictly increasing interior positions on both rails (maximum 64)");return;}
    options.slashes=std::move(candidate);updateSlashes();preview();
}
void SurfaceController::alignInputs(bool automatic){
    if(!active()||!valid()||!dialog)return;
    try {Base::PyGILStateLocker lock;auto proposed=inputs;const unsigned rails=options.kind==1?1:options.kind==2?2:0;
        if(automatic){
            ShapeRef reference(surfaceWire(*document,inputs[rails]));
            for(std::size_t i=rails+1;i<inputs.size();++i){auto natural=inputs[i];natural.seam=0;natural.reverse=false;ShapeRef wire(surfaceWire(*document,natural));
                automaticSurfaceAlignment(reference.p,wire.p,proposed[i].seam,proposed[i].reverse);}
        }else for(std::size_t i=rails;i<inputs.size();++i)proposed[i].seam=0;
        inputs=std::move(proposed);auto* table=dialog->findChild<QTableWidget*>("OM9SurfaceInputs");
        for(std::size_t i=rails;i<inputs.size();++i){auto* reverse=qobject_cast<QCheckBox*>(table->cellWidget(int(i),1));auto* seam=qobject_cast<QDoubleSpinBox*>(table->cellWidget(int(i),2));
            QSignalBlocker blockReverse(reverse),blockSeam(seam);reverse->setChecked(inputs[i].reverse);seam->setValue(inputs[i].seam);inputs[i].seam=seam->value();}
        preview();
    }catch(const std::exception& e){error(e);}
}
void SurfaceController::preview(){
    clearPreview();if(!active()||!valid())return;
    try{
        Base::PyGILStateLocker lock;ShapeRef shape(buildSurface(*document,inputs,options));
        if(!options.preview){if(status)status->setText("Preview hidden. Click Preview to display it; OK validates the current settings.");if(buttons)buttons->button(QDialogButtonBox::Ok)->setEnabled(true);return;}
        ShapeRef inventor(PyObject_CallMethod(shape.p,"writeInventor",nullptr));
        if(!inventor.p){PyErr_Clear();throw std::runtime_error("Cannot render surface preview");}const char* text=PyUnicode_AsUTF8(inventor.p);if(!text)throw std::runtime_error("Invalid preview data");
        auto* gui=Gui::Application::Instance->activeDocument();for(auto* mdi:gui->getMDIViews()){
            auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);if(!view)continue;auto* root=dynamic_cast<SoSeparator*>(view->getViewer()->getSceneGraph());if(!root)continue;
            SoInput input;input.setBuffer(text,std::strlen(text));auto* mesh=SoDB::readAll(&input);if(!mesh)throw std::runtime_error("Cannot load preview mesh");
            auto* wrapper=new SoSeparator;wrapper->setName("OM9SurfacePreview");auto* skip=new SoPickStyle;skip->style=SoPickStyle::UNPICKABLE;wrapper->addChild(skip);wrapper->addChild(mesh);root->ref();root->addChild(wrapper);previews.emplace_back(root,wrapper);
        }
        if(status)status->setText("Valid surface preview. Adjust directions/seams, then click OK.");if(buttons)buttons->button(QDialogButtonBox::Ok)->setEnabled(true);
    }catch(const std::exception& e){clearPreview();error(e);}
}
void SurfaceController::commit(){
    if(!active()||!valid()){cancel();return;}
    try{
        Base::PyGILStateLocker lock;ShapeRef shape(buildSurface(*document,inputs,options));commitSurface(*document,shape.p,inputs,options);
        om9_sidebar_record_execution(command,true);cancel();prompt("Surface created. Command:");
    }catch(const std::exception& e){error(e);}
}
bool SurfaceController::eventFilter(QObject* watched,QEvent* event){
    if(event->type()==QEvent::MouseButtonRelease&&releaseTarget==watched&&static_cast<QMouseEvent*>(event)->button()==Qt::LeftButton){releaseTarget=nullptr;return true;}
    if(!active()||Gui::Application::Instance->isClosing())return false;if(!valid()){cancel();return false;}
    auto* view=containing(watched);
    if(event->type()==QEvent::KeyPress&&CoreKeyboard::inputContext(watched)){
        auto* key=static_cast<QKeyEvent*>(event);if(key->key()==Qt::Key_Escape){cancel();prompt("Surface command cancelled");return true;}
        if(view&&(key->key()==Qt::Key_Return||key->key()==Qt::Key_Enter||key->key()==Qt::Key_Space)){submit({});return true;}
    }
    if((dialog&&!slashPicking)||!view||event->type()!=QEvent::MouseButtonPress)return false;
    auto* mouse=static_cast<QMouseEvent*>(event);if(mouse->button()!=Qt::LeftButton||mouse->modifiers()&(Qt::ControlModifier|Qt::AltModifier|Qt::MetaModifier))return false;
    auto* viewer=view->getViewer();auto* widget=qobject_cast<QWidget*>(watched);const auto logical=viewer->viewport()->mapFrom(widget,mouse->position().toPoint());
    std::unique_ptr<SoPickedPoint> picked(viewer->pickPoint(viewer->fromQPoint(logical)));auto* provider=picked?dynamic_cast<Gui::ViewProviderDocumentObject*>(viewer->getViewProviderByPath(picked->getPath())):nullptr;
    releaseTarget=watched;
    if(!provider||!provider->getObject()||!provider->isSelectable())return true;
    std::string sub;provider->getElementPicked(picked.get(),sub);
    // A whole curve is preferable to one segment; surfaces require an edge pick.
    auto* object=provider->getObject();
    if(dialog&&slashPicking){
        const auto& rail=inputs[slashPicking-1];bool matches=rail.object==object->getNameInDocument()&&(rail.sub.empty()||rail.sub==sub);
        for(const auto& ref:rail.chain)matches=matches||(ref.first==object->getNameInDocument()&&(ref.second.empty()||ref.second==sub));
        if(!matches){prompt(slashPicking==1?"Pick the first slash point on rail A":"Pick the second slash point on rail B");return true;}
        try{Base::PyGILStateLocker lock;ShapeRef wire(surfaceWire(*document,rail));const auto& p=picked->getPoint();
            const double fraction=surfaceRailFraction(wire.p,{p[0],p[1],p[2]});
            if(slashPicking==1){slashFirst=fraction;slashPicking=2;prompt("Add Slash: pick the matching point on rail B");}
            else {addSlash(slashFirst,fraction);slashPicking=1;prompt("Add Slash: pick another point on rail A, or Enter to finish");}
        }catch(const std::exception& e){error(e);}return true;
    }
    if(om9_surface_phase()!=5){try{Base::PyGILStateLocker lock;SurfaceInput whole{object->getNameInDocument(),""};whole.guards.push_back(capturePhase3Object(*document,whole.object));ShapeRef wire(surfaceWire(*document,whole));sub.clear();}catch(const std::exception&){} }
    add(object->getNameInDocument(),sub);return true;
}
}
