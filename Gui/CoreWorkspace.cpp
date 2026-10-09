#include "CoreWorkspace.h"
#include "CadPresentation.h"
#include "RustBridge.h"
#include "CoreCPlanes.h"
#include "CameraState.h"
#include "CameraTrace.h"
#include <App/Application.h>
#include <App/Document.h>
#include <Base/Exception.h>
#include <Base/Console.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/MainWindow.h>
#include <Gui/Control.h>
#include <Gui/Command.h>
#include <Gui/View3DInventor.h>
#include <Gui/ViewProvider.h>
#include <Gui/View3DInventorViewer.h>
#include <Gui/Inventor/SoFCBoundingBox.h>
#include <Gui/Selection/SoFCUnifiedSelection.h>
#include <Inventor/SoRenderManager.h>
#include <Inventor/nodes/SoSeparator.h>
#include <Inventor/nodes/SoSwitch.h>
#include <Inventor/nodes/SoTransform.h>
#include <Inventor/nodes/SoPickStyle.h>
#include <Inventor/nodes/SoBaseColor.h>
#include <Inventor/nodes/SoCoordinate3.h>
#include <Inventor/nodes/SoLineSet.h>
#include <Inventor/nodes/SoDrawStyle.h>
#include <Inventor/nodes/SoOrthographicCamera.h>
#include <Inventor/nodes/SoPerspectiveCamera.h>
#include <Inventor/actions/SoSearchAction.h>
#include <QApplication>
#include <QMdiArea>
#include <QMdiSubWindow>
#include <QTimer>
#include <QKeyEvent>
#include <QScopedValueRollback>
#include <QSignalBlocker>
#include <QDockWidget>
#include <QTabBar>
#include <QLabel>
#include <QToolButton>
#include <QHBoxLayout>
#include <QMenu>
#include <QActionGroup>
#include <QMouseEvent>
#include <QVariant>
#include <functional>
#include <array>
#include <algorithm>
#include <cmath>
#include <string>

namespace {
class ViewportLabel final:public QLabel {
public:
    using QLabel::QLabel;
    std::function<void()> activate,toggle;
protected:
    void mousePressEvent(QMouseEvent* e)override{if(e->button()==Qt::LeftButton&&activate)activate();e->accept();}
    void mouseDoubleClickEvent(QMouseEvent* e)override{if(e->button()==Qt::LeftButton&&toggle)toggle();e->accept();}
};
QMdiSubWindow* subWindow(Gui::View3DInventor* view){return qobject_cast<QMdiSubWindow*>(view?view->parentWidget():nullptr);}
constexpr const char* titles[]={"Looking Down","Perspective","Side View","Through Finger"};
OpenMatrix9Gui::CadPresentation* presentation(Gui::View3DInventor* view,bool create=false){
    auto* root=dynamic_cast<SoSeparator*>(view->getViewer()->getSceneGraph());if(!root)return nullptr;
    SoSearchAction search;search.setName("OM9ViewportPresentation");search.setInterest(SoSearchAction::FIRST);search.apply(root);
    if(auto* path=search.getPath())return dynamic_cast<OpenMatrix9Gui::CadPresentation*>(path->getTail());
    if(!create)return nullptr;
    // Each view owns its selection root; shared document providers remain untouched.
    for(int i=0;i<root->getNumChildren();++i)if(auto* owner=dynamic_cast<Gui::SoFCUnifiedSelection*>(root->getChild(i))){
        OpenMatrix9Gui::CadPresentation::initClass();auto* style=new OpenMatrix9Gui::CadPresentation;style->setName("OM9ViewportPresentation");owner->insertChild(style,0);return style;
    }
    return nullptr;
}
Gui::View3DInventor* activeView() {
    auto* doc=Gui::Application::Instance->activeDocument();
    return doc?dynamic_cast<Gui::View3DInventor*>(doc->getActiveView()):nullptr;
}
std::array<Gui::View3DInventor*,4> slots(Gui::Document* document) {
    std::array<Gui::View3DInventor*,4> result{};
    if(auto* doc=document;doc&&!doc->isAboutToClose())
        for(auto* mdi:doc->getMDIViews())
            if(auto* view=dynamic_cast<Gui::View3DInventor*>(mdi);view&&view->property("om9ViewSlot").isValid()) {
                int slot=view->property("om9ViewSlot").toInt();if(slot>=0&&slot<4)result[slot]=view;
            }
    return result;
}
std::array<Gui::View3DInventor*,4> slots(){return slots(Gui::Application::Instance->activeDocument());}
SbRotation rotation(int slot,bool plane) {
    return SbRotation(float(om9_core_view_rotation(slot,0,plane)),float(om9_core_view_rotation(slot,1,plane)),float(om9_core_view_rotation(slot,2,plane)),float(om9_core_view_rotation(slot,3,plane)));
}
SoSwitch* grid(Gui::View3DInventor* view,bool create=false) {
    auto* root=dynamic_cast<SoSeparator*>(view->getViewer()->getSceneGraph());if(!root)return nullptr;
    SoSearchAction search;search.setName("OM9ConstructionGrid");search.setInterest(SoSearchAction::FIRST);search.apply(root);
    if(auto* path=search.getPath())return dynamic_cast<SoSwitch*>(path->getTail());
    if(!create)return nullptr;
    // Rendering is a Qt callback boundary. An unavailable/malformed plane must
    // hide the grid, while command input continues to receive the explicit
    // error from plane(); do not silently draw or accept a default CPlane.
    Base::Placement plane;
    try {plane=OpenMatrix9Gui::CoreWorkspace::instance().plane(view);view->setProperty("om9CPlaneError",QVariant());}
    catch(const Base::Exception& error){const auto message=QString::fromUtf8(error.what());if(view->property("om9CPlaneError").toString()!=message)Base::Console().error("OpenMatrix9 construction grid unavailable: %s\n",error.what());view->setProperty("om9CPlaneError",message);return nullptr;}
    catch(const std::exception& error){const auto message=QString::fromUtf8(error.what());if(view->property("om9CPlaneError").toString()!=message)Base::Console().error("OpenMatrix9 construction grid unavailable: %s\n",error.what());view->setProperty("om9CPlaneError",message);return nullptr;}
    catch(...){Base::Console().error("OpenMatrix9 construction grid unavailable: invalid construction-plane state\n");return nullptr;}
    auto* skip=new Gui::SoSkipBoundingGroup;
    // FreeCAD's fit-all action excludes SoSkipBoundingGroup through its action
    // element. Automatic clipping must still see the grid, including empty files.
    skip->mode=Gui::SoSkipBoundingGroup::INCLUDE_BBOX;
    auto* toggle=new SoSwitch;toggle->setName("OM9ConstructionGrid");toggle->whichChild=SO_SWITCH_ALL;
    auto* separator=new SoSeparator;
    auto* pick=new SoPickStyle;pick->style=SoPickStyle::UNPICKABLE;separator->addChild(pick);
    auto* transform=new SoTransform;transform->setName("OM9ConstructionGridTransform");
    const auto origin=plane.getPosition();const auto& quaternion=plane.getRotation();
    transform->translation.setValue(float(origin.x),float(origin.y),float(origin.z));transform->rotation.setValue(float(quaternion[0]),float(quaternion[1]),float(quaternion[2]),float(quaternion[3]));separator->addChild(transform);
    // World-space construction lines: five 1 mm subdivisions in each major cell.
    // Separate separators prevent color state leaking into subsequent geometry.
    for(unsigned int kind=0;kind<4;++kind){
        auto* layer=new SoSeparator;layer->setName(kind==0?"OM9GridMinor":kind==1?"OM9GridMajor":kind==2?"OM9GridAxisX":"OM9GridAxisY");
        auto* color=new SoBaseColor;const float gray=kind==0?0.19f:0.38f;color->rgb.setValue(kind<2?SbColor(gray,gray,gray):kind==2?SbColor(0.65f,0,0):SbColor(0,0.55f,0.55f));layer->addChild(color);
        auto* coords=new SoCoordinate3;auto* lines=new SoLineSet;int vertex=0,line=0;
        auto segment=[&](float x0,float y0,float x1,float y1){coords->point.set1Value(vertex++,x0,y0,0);coords->point.set1Value(vertex++,x1,y1,0);lines->numVertices.set1Value(line++,2);};
        if(kind<2)for(int i=-20;i<=20;++i){if(om9_core_grid_line_kind(i)!=kind)continue;segment(float(i),-20,float(i),20);segment(-20,float(i),20,float(i));}
        else if(kind==2)segment(-20,0,20,0);else segment(0,-20,0,20);
        layer->addChild(coords);auto* style=new SoDrawStyle;style->lineWidth=1;layer->addChild(style);layer->addChild(lines);
        separator->addChild(layer);
    }
    toggle->addChild(separator);skip->addChild(toggle);
    // Native image capture traverses the view-provider root, not all viewer overlays.
    SoSeparator* owner=root;
    for(int i=0;i<root->getNumChildren();++i)if(auto* selection=dynamic_cast<Gui::SoFCUnifiedSelection*>(root->getChild(i))){owner=selection;break;}
    owner->addChild(skip);return toggle;
}
std::string id(std::size_t command){const char* name=om9_command_id(command);return name?name:"";}
}
namespace OpenMatrix9Gui {
CoreWorkspace& CoreWorkspace::instance(){static auto* value=new CoreWorkspace;return *value;}
CoreWorkspace::CoreWorkspace():QObject(qApp) {
    auto defer=[this]{QTimer::singleShot(0,this,[this]{if(enabled)ensure();});};
    activeConnection=App::GetApplication().signalActiveDocument.connect([defer](const App::Document&){defer();});
    newConnection=App::GetApplication().signalNewDocument.connect([this,defer](const App::Document& doc,bool){previousDisplay.erase(doc.getName());defer();});
    saveConnection=App::GetApplication().signalStartSaveDocument.connect([this](const App::Document& doc,const std::string&) {
        auto values=doc.Meta.getValues();auto views=slots(Gui::Application::Instance->getDocument(&doc));
        for(int slot=0;slot<4;++slot)if(auto* view=views[slot]) {
            auto suffix=std::to_string(slot);values["OpenMatrix9.ViewCamera."+suffix]=view->getCamera();
            if(auto* node=grid(view))values["OpenMatrix9.ViewGrid."+suffix]=(enabled?node->whichChild.getValue()!=SO_SWITCH_NONE:!view->property("om9GridHidden").toBool())?"1":"0";
        }
        if(values!=doc.Meta.getValues())const_cast<App::Document&>(doc).Meta.setValues(std::move(values));
    });
    auto* timer=new QTimer(this);timer->setInterval(200);connect(timer,&QTimer::timeout,this,[this]{if(enabled){if(pendingEnsure)ensure();else updateTabs();}});timer->start();
    qApp->installEventFilter(this);
}
bool CoreWorkspace::handles(std::size_t command) {auto name=id(command);return name=="RestoreViewports"||name=="SynchronizeViews"||name=="CenterViewport"||name=="ShowGrid"||name=="ViewportTabs";}
bool CoreWorkspace::available(std::size_t command)const {return enabled&&handles(command)&&activeView()&&Gui::Control().isAllowedAlterView(App::GetApplication().getActiveDocument());}
void CoreWorkspace::activate(){
    auto* area=Gui::getMainWindow()->findChild<QMdiArea*>();if(!enabled&&area){previousMdiMode=int(area->viewMode());previousMdiMaximizeOption=area->testOption(QMdiArea::DontMaximizeSubWindowOnActivation);area->setOption(QMdiArea::DontMaximizeSubWindowOnActivation,true);}
    auto pref=App::GetApplication().GetParameterGroupByPath("User parameter:BaseApp/Preferences/Mod/OpenMatrix9/ViewportTabs");
    if(!om9_core_tabs_load(static_cast<unsigned int>(pref->GetInt("State",3))))om9_core_tabs_load(3);
    enabled=true;layoutDocument.clear();updateTabs();ensure();
    for(auto* doc:App::GetApplication().getDocuments())for(auto* view:slots(Gui::Application::Instance->getDocument(doc)))if(view)if(auto* node=grid(view))node->whichChild=view->property("om9GridHidden").toBool()?SO_SWITCH_NONE:SO_SWITCH_ALL;
}
void CoreWorkspace::deactivate(){
    enabled=false;pendingEnsure=false;layoutDocument.clear();
    if(tabsDock)tabsDock->hide();
    for(auto* doc:App::GetApplication().getDocuments())for(auto* view:slots(Gui::Application::Instance->getDocument(doc)))if(view){
        if(auto* title=view->findChild<QWidget*>("OM9ViewportTitle"))title->hide();
        if(auto* menu=view->findChild<QMenu*>("OM9ViewportMenu"))menu->close();
        if(auto* style=presentation(view)){SoSearchAction search;search.setNode(style);search.apply(view->getViewer()->getSceneGraph());if(auto* path=search.getPath())if(auto* owner=dynamic_cast<SoGroup*>(path->getNodeFromTail(1)))owner->removeChild(style);}
        if(auto* sub=subWindow(view);sub&&sub->property("om9PreviousWindowFlags").isValid()){sub->setWindowFlags(Qt::WindowFlags(sub->property("om9PreviousWindowFlags").toUInt()));sub->setProperty("om9PreviousWindowFlags",QVariant());}
        auto* display=view->getViewer();if(auto mode=previousDisplay.find(doc->getName());mode!=previousDisplay.end()){display->updateOverrideMode("OM9Restore");display->setOverrideMode(mode->second);}
        if(display->property("om9PreviousOverride").isValid()){display->updateOverrideMode(display->property("om9PreviousOverride").toString().toStdString());display->getSoRenderManager()->setRenderMode(SoRenderManager::RenderMode(display->property("om9PreviousRenderMode").toInt()));display->setProperty("om9PreviousOverride",QVariant());display->setProperty("om9PreviousRenderMode",QVariant());}
        if(auto* node=grid(view)){view->setProperty("om9GridHidden",node->whichChild.getValue()==SO_SWITCH_NONE);node->whichChild=SO_SWITCH_NONE;}
        auto* viewer=view->getViewer();if(viewer->property("om9PreviousBackground").isValid()){viewer->setBackgroundColor(viewer->property("om9PreviousBackground").value<QColor>());viewer->setGradientBackground(Gui::View3DInventorViewer::Background(viewer->property("om9PreviousGradient").toInt()));viewer->setProperty("om9PreviousBackground",QVariant());}
    }
    restoreCadProviders();
    previousDisplay.clear();
    if(auto* area=Gui::getMainWindow()->findChild<QMdiArea*>()) {
        QSignalBlocker block(area);
        auto* current=area->activeSubWindow();
        for(auto* sub:area->subWindowList())if(sub->widget()->property("om9ViewSlot").isValid())sub->show();
        if(previousMdiMode>=0)area->setViewMode(QMdiArea::ViewMode(previousMdiMode));area->setOption(QMdiArea::DontMaximizeSubWindowOnActivation,previousMdiMaximizeOption);
        if(current)area->setActiveSubWindow(current);
    }
}
void CoreWorkspace::ensure(bool restore) {
    if(!enabled||arranging||Gui::Application::Instance->isClosing())return;auto* doc=Gui::Application::Instance->activeDocument();if(!doc||doc->isAboutToClose()||!activeView()){pendingEnsure=false;updateTabs();return;}
    if(doc->getInEdit()||!Gui::Control().isAllowedAlterView(doc->getDocument())){pendingEnsure=true;updateTabs();return;}
    QScopedValueRollback<bool> guard(arranging,true);
    auto* previousView=activeView();
    auto views=slots();
    bool created=false;
    auto existing=doc->getMDIViewsOfType(Gui::View3DInventor::getClassTypeId());
    // A closing native view can remain in the document list until deferred
    // deletion completes. createView copies the first view's camera, so retry
    // after teardown instead of creating against a camera already removed.
    for(auto* mdi:existing) {
        auto* viewer=static_cast<Gui::View3DInventor*>(mdi)->getViewer();
        if(!viewer||!viewer->getSoRenderManager()->getCamera()){pendingEnsure=true;return;}
    }
    if(!previousDisplay.contains(doc->getDocument()->getName()))previousDisplay.emplace(doc->getDocument()->getName(),previousView->getViewer()->getOverrideMode());
    // Creating another native view can forward its override through external links.
    adaptCadProviders(doc,previousDisplay);
    for(int i=0;i<4;++i) {
        bool fresh=!views[i];
        if(fresh) {
            created=true;
            for(auto* mdi:existing)if(!mdi->property("om9ViewSlot").isValid()){views[i]=static_cast<Gui::View3DInventor*>(mdi);break;}
            if(!views[i])views[i]=dynamic_cast<Gui::View3DInventor*>(doc->createView(Gui::View3DInventor::getClassTypeId()));
            if(!views[i])return;views[i]->setProperty("om9ViewSlot",i);
        }
        auto* view=views[i];auto* viewer=view->getViewer();view->setWindowTitle(titles[i]);view->setProperty("om9DocumentName",QString::fromUtf8(doc->getDocument()->getName()));
        if(!viewer->property("om9PreviousOverride").isValid()){viewer->setProperty("om9PreviousOverride",QString::fromStdString(viewer->getOverrideMode()));viewer->setProperty("om9PreviousRenderMode",int(viewer->getSoRenderManager()->getRenderMode()));}
        ensureTitle(view);auto* node=grid(view,true);if(node&&(fresh||restore))node->whichChild=SO_SWITCH_ALL;
        if(!viewer->property("om9PreviousBackground").isValid()){viewer->setProperty("om9PreviousBackground",viewer->backgroundColor());viewer->setProperty("om9PreviousGradient",int(viewer->getGradientBackground()));}
        viewer->setBackgroundColor(QColor(0,0,0));viewer->setGradientBackground(Gui::View3DInventorViewer::Background::NoGradient);
        if(!qgetenv("OM9_TRACE_CAMERA").isEmpty()&&!viewer->property("om9CameraTrace").toBool()){viewer->setProperty("om9CameraTrace",true);QObject::connect(viewer,&Gui::View3DInventorViewer::cameraChanged,this,[view,i]{if(auto* camera=dynamic_cast<SoPerspectiveCamera*>(view->getViewer()->getSoRenderManager()->getCamera()))traceCameraChange(i,camera->heightAngle.getValue());});}
        if(fresh||restore) {
            viewer->setAnimationEnabled(false);
            viewer->setCameraType(om9_core_view_perspective(i)?SoPerspectiveCamera::getClassTypeId():SoOrthographicCamera::getClassTypeId());
            auto* camera=viewer->getSoRenderManager()->getCamera();camera->orientation=rotation(i,false);camera->focalDistance=100;
            if(auto* ortho=dynamic_cast<SoOrthographicCamera*>(camera))ortho->height=50;
            center(view);view->setProperty("om9ViewportMode","Wireframe");
            if(restore)if(auto* root=dynamic_cast<SoSeparator*>(viewer->getSceneGraph()))for(int child=root->getNumChildren()-1;child>=0;--child)if(root->getChild(child)->getName()==SbName("OM9WorkspaceBackground"))root->removeChild(child);
            if(fresh&&!restore) {
                const auto& metadata=doc->getDocument()->Meta.getValues();auto suffix=std::to_string(i);
                if(auto entry=metadata.find("OpenMatrix9.ViewCamera."+suffix);entry!=metadata.end()&&entry->second.size()<16384) {
                    try {restoreCameraState(viewer,entry->second);}catch(const Base::Exception&){}
                }
                if(auto entry=metadata.find("OpenMatrix9.ViewGrid."+suffix);node&&entry!=metadata.end()) {node->whichChild=entry->second=="0"?SO_SWITCH_NONE:SO_SWITCH_ALL;view->setProperty("om9GridHidden",entry->second=="0");}
            }
        }
    }
    // Keep faces and native edges available for rendering and Wireframe edge picking.
    adaptCadProviders(doc,previousDisplay);
    views[0]->getViewer()->setOverrideMode("Flat Lines");
    for(auto* view:views)if(view){auto* viewer=view->getViewer();const bool shaded=view->property("om9ViewportMode").toString()=="Shaded";viewer->updateOverrideMode(shaded?"Shaded":"Wireframe");viewer->getSoRenderManager()->setRenderMode(SoRenderManager::AS_IS);if(auto* style=presentation(view,true))style->mode=shaded?2:1;}
    if(created||restore||layoutDocument!=doc->getDocument()->getName()) {
        layout(restore);Gui::getMainWindow()->setActiveWindow(previousView);
    }
    pendingEnsure=false;
    updateTabs();
}
void CoreWorkspace::layout(bool restore) {
    if(Gui::Application::Instance->isClosing())return;
    if(auto* doc=Gui::Application::Instance->activeDocument();doc&&doc->isAboutToClose())return;
    auto* area=Gui::getMainWindow()->findChild<QMdiArea*>();if(!area)return;
    QSignalBlocker block(area);
    area->setViewMode(QMdiArea::SubWindowView);auto views=slots();auto size=area->viewport()->size();
    if(auto* doc=Gui::Application::Instance->activeDocument())layoutDocument=doc->getDocument()->getName();
    for(auto* sub:area->subWindowList())if(sub->widget()->property("om9ViewSlot").isValid()) {
        bool belongs=false;for(auto* view:views)if(view==sub->widget())belongs=true;
        if(!belongs)sub->hide();
    }
    if(!restore)if(auto* current=area->activeSubWindow();current&&current->isMaximized())for(auto* view:views)if(view==current->widget()){for(auto* view:views)if(auto* sub=subWindow(view)){if(sub==current)sub->showMaximized();else sub->hide();}updateTabs();return;}
    if(auto* current=area->activeSubWindow();current&&current->isMaximized())current->showNormal();
    for(int i=0;i<4;++i)if(views[i])for(auto* sub:area->subWindowList())if(sub->widget()==views[i]) {
        sub->showNormal();sub->setGeometry((i%2)*size.width()/2,(i/2)*size.height()/2,size.width()/2,size.height()/2);
    }
    updateTabs();
}
void CoreWorkspace::updateTabs() {
    if(!enabled||Gui::Application::Instance->isClosing())return;
    reconcileDisplay();
    for(auto* view:slots())if(view)updateTitle(view);
    auto* window=Gui::getMainWindow();auto* area=window->findChild<QMdiArea*>();if(!area)return;
    if(!tabsDock) {
        tabsDock=new QDockWidget("Viewport Tabs",window);tabsDock->setObjectName("OM9ViewportTabsDock");tabsDock->setFeatures(QDockWidget::NoDockWidgetFeatures);
        auto* title=new QWidget(tabsDock);title->setFixedHeight(0);tabsDock->setTitleBarWidget(title);
        tabs=new QTabBar(tabsDock);tabs->setObjectName("OM9ViewportTabs");tabs->setExpanding(false);tabs->setDrawBase(false);tabs->setElideMode(Qt::ElideNone);tabsDock->setWidget(tabs);
        for(auto* title:titles)tabs->addTab(QString::fromUtf8(title));
        connect(tabs,&QTabBar::currentChanged,this,[this,area](int slot){
            if(!enabled||!Gui::Control().isAllowedAlterView(App::GetApplication().getActiveDocument())){updateTabs();return;}
            auto views=slots();if(slot<0||slot>=4||!views[slot]){updateTabs();return;}
            bool maximized=area->activeSubWindow()&&area->activeSubWindow()->isMaximized();
            Gui::getMainWindow()->setActiveWindow(views[slot]);
            if(maximized)if(auto* current=area->activeSubWindow()){current->showMaximized();layout();}
            updateTabs();
        });
        connect(area,&QMdiArea::subWindowActivated,this,[this](QMdiSubWindow*){if(enabled&&!arranging)updateTabs();});
        tabsAlignment=-1;
    }
    const auto state=om9_core_tabs_state();int alignment=int(state>>1);
    if(alignment!=tabsAlignment) {
        constexpr Qt::DockWidgetArea locations[]={Qt::TopDockWidgetArea,Qt::BottomDockWidgetArea,Qt::LeftDockWidgetArea,Qt::RightDockWidgetArea};
        constexpr QTabBar::Shape shapes[]={QTabBar::RoundedNorth,QTabBar::RoundedSouth,QTabBar::RoundedWest,QTabBar::RoundedEast};
        tabs->setShape(shapes[alignment]);window->addDockWidget(locations[alignment],tabsDock,alignment<2?Qt::Vertical:Qt::Horizontal);tabsAlignment=alignment;
    }
    QSignalBlocker block(tabs);auto views=slots();const bool allowed=Gui::Control().isAllowedAlterView(App::GetApplication().getActiveDocument());
    for(int i=0;i<4;++i)tabs->setTabEnabled(i,allowed&&views[i]);
    if(auto* view=activeView();view&&view->property("om9ViewSlot").isValid())tabs->setCurrentIndex(view->property("om9ViewSlot").toInt());
    bool visible=(state&1)&&activeView();if(tabsDock->isVisible()!=visible)tabsDock->setVisible(visible);
    // App.closeDocument need not request a main-window action refresh.
    if(auto* command=Gui::Application::Instance->commandManager().getCommandByName("ViewportTabs"))command->testActive();
}
bool CoreWorkspace::executeTabs(const QString& options) {
    std::size_t command=om9_command_count();for(std::size_t i=0;i<command;++i)if(id(i)=="ViewportTabs"){command=i;break;}
    if(!available(command))return false;
    auto bytes=options.toUtf8();if(!om9_core_tabs_apply(reinterpret_cast<const unsigned char*>(bytes.constData()),std::size_t(bytes.size())))return false;
    App::GetApplication().GetParameterGroupByPath("User parameter:BaseApp/Preferences/Mod/OpenMatrix9/ViewportTabs")->SetInt("State",om9_core_tabs_state());
    updateTabs();return true;
}
void CoreWorkspace::reconcileDisplay(){
    auto* doc=Gui::Application::Instance->activeDocument();
    // Re-entry must snapshot native display state before adapting it.
    if(!doc||!previousDisplay.contains(doc->getDocument()->getName()))return;
    auto views=slots();bool basic=true;
    for(auto* view:views)if(view){auto* viewer=view->getViewer();const auto native=viewer->getOverrideMode();
        if(native=="Wireframe")view->setProperty("om9ViewportMode","Wireframe");
        else if(native=="Shaded")view->setProperty("om9ViewportMode","Shaded");
        else {view->setProperty("om9ViewportMode",QString::fromStdString(native));basic=false;}
        if(auto* style=presentation(view))style->mode=native=="Wireframe"?1:(native=="Shaded"?2:0);
    }
    if(basic){
        // Native attachment applies each viewer's override to the shared provider.
        // Repair newly added geometry too, retaining native pickable edge topology.
        adaptCadProviders(doc,previousDisplay);
        for(auto* provider:doc->getViewProvidersOfType(Gui::ViewProvider::getClassTypeId()))if(provider->getOverrideMode()!="Flat Lines")provider->setOverrideMode("Flat Lines");
        for(auto* view:views)if(view){auto* manager=view->getViewer()->getSoRenderManager();if(manager->getRenderMode()!=SoRenderManager::AS_IS)manager->setRenderMode(SoRenderManager::AS_IS);}
    }
}
bool CoreWorkspace::selectTitle(Gui::View3DInventor* view){
    auto* doc=Gui::Application::Instance->activeDocument();if(!enabled||!doc||doc->isAboutToClose()||!view||view->getGuiDocument()!=doc||!Gui::Control().isAllowedAlterView(doc->getDocument()))return false;
    Gui::getMainWindow()->setActiveWindow(view);updateTabs();return true;
}
bool CoreWorkspace::toggleTitle(Gui::View3DInventor* view){
    if(!selectTitle(view))return false;auto* sub=subWindow(view);if(!sub)return false;
    const int slot=view->property("om9ViewSlot").toInt();const int next=om9_viewport_next_single(sub->isMaximized()?slot:-1,std::size_t(slot));
    if(next==-2)return false;
    if(next<0)layout(true);else{sub->showMaximized();layout();}updateTabs();return true;
}
bool CoreWorkspace::displayTitle(Gui::View3DInventor* view,std::size_t mode){
    const int native=om9_viewport_native_mode(mode);if(native<0||!selectTitle(view))return false;
    auto* viewer=view->getViewer();adaptCadProviders(view->getGuiDocument(),previousDisplay);viewer->setOverrideMode("Flat Lines");viewer->updateOverrideMode(native==1?"Wireframe":"Shaded");viewer->getSoRenderManager()->setRenderMode(SoRenderManager::AS_IS);if(auto* style=presentation(view,true))style->mode=native==1?1:2;
    view->setProperty("om9ViewportMode",QString::fromUtf8(om9_viewport_mode_name(mode)));viewer->getSoRenderManager()->scheduleRedraw();updateTitle(view);return true;
}
void CoreWorkspace::ensureTitle(Gui::View3DInventor* view){
    if(auto* sub=subWindow(view);sub&&!sub->property("om9PreviousWindowFlags").isValid()){
        sub->setProperty("om9PreviousWindowFlags",uint(sub->windowFlags()));sub->setWindowFlags(sub->windowFlags()|Qt::FramelessWindowHint);
    }
    if(!view->findChild<QWidget*>("OM9ViewportTitle")){
        QPointer<Gui::View3DInventor> target(view);
        auto* header=new QWidget(view);header->setObjectName("OM9ViewportTitle");header->setProperty("om9ViewportChrome",true);
        auto* row=new QHBoxLayout(header);row->setContentsMargins(3,0,0,0);row->setSpacing(2);
        auto* label=new ViewportLabel(header);label->setObjectName("OM9ViewportLabel");label->setText(titles[view->property("om9ViewSlot").toInt()]);label->activate=[this,target]{if(target)selectTitle(target);};label->toggle=[this,target]{if(target)toggleTitle(target);};row->addWidget(label);
        auto* arrow=new QToolButton(header);arrow->setObjectName("OM9ViewportDropdown");arrow->setArrowType(Qt::DownArrow);arrow->setFocusPolicy(Qt::NoFocus);arrow->setAutoRaise(true);arrow->setFixedWidth(12);row->addWidget(arrow);
        auto* menu=new QMenu(header);menu->setObjectName("OM9ViewportMenu");auto* max=menu->addAction("Maximize");max->setObjectName("OM9ViewportMaximize");connect(max,&QAction::triggered,this,[this,target]{if(target)toggleTitle(target);});menu->addSeparator();
        auto* group=new QActionGroup(menu);group->setExclusive(true);
        for(std::size_t i=0;i<om9_viewport_mode_count();++i){auto* action=menu->addAction(QString::fromUtf8(om9_viewport_mode_name(i)));action->setData(int(i));action->setCheckable(true);group->addAction(action);connect(action,&QAction::triggered,this,[this,target,i]{if(target)displayTitle(target,i);});}
        menu->addSeparator();for(auto* name:{"Print Preview","Flat Shade","Shade Selected Objects Only"}){auto* action=menu->addAction(name);action->setEnabled(false);}menu->addSeparator();
        for(auto* name:{"Pan, Zoom, and Rotate","Set View"}){auto* child=menu->addMenu(name);child->menuAction()->setEnabled(false);}
        addCPlaneMenu(menu,view);
        menu->addMenu("Set Camera")->menuAction()->setEnabled(false);
        connect(menu,&QMenu::aboutToShow,this,[this,target]{if(target)updateTitle(target);});
        connect(arrow,&QToolButton::clicked,this,[this,target,header,menu]{if(target&&selectTitle(target))menu->popup(header->mapToGlobal(QPoint(0,header->height())));});
    }
    updateTitle(view);
}
void CoreWorkspace::updateTitle(Gui::View3DInventor* view){
    auto* header=view->findChild<QWidget*>("OM9ViewportTitle");if(!header)return;
    auto* doc=Gui::Application::Instance->activeDocument();const bool current=enabled&&doc&&view->getGuiDocument()==doc;
    const bool allowed=current&&Gui::Control().isAllowedAlterView(doc->getDocument());
    const bool active=view==activeView();header->setStyleSheet(QString("QWidget#OM9ViewportTitle { background:%1; color:black; } QLabel { background:transparent; color:black; } QToolButton { border:none; background:transparent; color:black; }").arg(active?"#82b48c":"#cdd7dc"));
    header->setFixedSize(header->sizeHint().width(),header->fontMetrics().lineSpacing()+2);header->move(view->getViewer()->viewport()->mapTo(view,QPoint(0,0)));header->setVisible(current);header->raise();
    auto* menu=header->findChild<QMenu*>("OM9ViewportMenu");if(!menu)return;
    auto* max=menu->findChild<QAction*>("OM9ViewportMaximize");max->setText(subWindow(view)&&subWindow(view)->isMaximized()?"Restore 4V":"Maximize");max->setEnabled(allowed);
    if(auto* cplane=menu->findChild<QMenu*>("OM9CPlaneMenu"))cplane->menuAction()->setEnabled(allowed);
    const auto mode=view->property("om9ViewportMode").toString();for(auto* action:menu->actions())if(action->data().isValid()) {action->setEnabled(allowed&&om9_viewport_native_mode(std::size_t(action->data().toInt()))>=0);QSignalBlocker guard(action);action->setChecked(action->text()==mode);}
    header->findChild<QToolButton*>("OM9ViewportDropdown")->setEnabled(allowed);
}
void CoreWorkspace::center(Gui::View3DInventor* view) {
    auto* camera=view->getViewer()->getSoRenderManager()->getCamera();SbVec3f direction;camera->orientation.getValue().multVec(SbVec3f(0,0,-1),direction);
    camera->position=-direction*camera->focalDistance.getValue();
    // Moving the camera does not update Coin's default 1..10 clipping range.
    const float distance=camera->focalDistance.getValue();
    camera->nearDistance=std::max(0.001f,distance*0.001f);
    camera->farDistance=std::max(camera->nearDistance.getValue()+1.0f,distance*10.0f);
}
Base::Placement CoreWorkspace::plane(Gui::View3DInventor* view)const {
    if(!view||!view->property("om9ViewSlot").isValid())return {};
    int slot=view->property("om9ViewSlot").toInt();return constructionPlane(view,Base::Placement(Base::Vector3d(),Base::Rotation(om9_core_view_rotation(slot,0,true),om9_core_view_rotation(slot,1,true),om9_core_view_rotation(slot,2,true),om9_core_view_rotation(slot,3,true))));
}
bool CoreWorkspace::execute(std::size_t command) {
    if(!available(command))return false;auto name=id(command);
    if(name=="ViewportTabs")return executeTabs(QString());
    if(name=="RestoreViewports"){ensure(true);return true;}
    if(name=="CenterViewport"){center(activeView());return true;}
    if(name=="ShowGrid") {
        bool visible=false;for(auto* view:slots())if(view)if(auto* node=grid(view)){visible=node->whichChild.getValue()!=SO_SWITCH_NONE;break;}
        for(auto* view:slots())if(view)if(auto* node=grid(view))node->whichChild=visible?SO_SWITCH_NONE:SO_SWITCH_ALL;
        return true;
    }
    auto* source=activeView()->getViewer()->getSoRenderManager()->getCamera();auto* ortho=dynamic_cast<SoOrthographicCamera*>(source);
    auto* perspective=dynamic_cast<SoPerspectiveCamera*>(source);
    const double height=ortho?ortho->height.getValue():(perspective?om9_core_perspective_height(source->focalDistance.getValue(),perspective->heightAngle.getValue()):0.0);
    if(!std::isfinite(height)||height<=0.0)return false;
    SbVec3f direction;source->orientation.getValue().multVec(SbVec3f(0,0,-1),direction);auto focus=source->position.getValue()+direction*source->focalDistance.getValue();
    for(auto* view:slots())if(view) {
        auto* camera=view->getViewer()->getSoRenderManager()->getCamera();auto* target=dynamic_cast<SoOrthographicCamera*>(camera);if(!target)continue;
        target->height=float(height);SbVec3f axis;camera->orientation.getValue().multVec(SbVec3f(0,0,-1),axis);camera->position=focus-axis*camera->focalDistance.getValue();
    }
    return true;
}
bool CoreWorkspace::toggleActiveGrid(){
    auto* view=activeView();if(!enabled||!view||!Gui::Control().isAllowedAlterView(App::GetApplication().getActiveDocument()))return false;
    if(auto* node=grid(view)){node->whichChild=node->whichChild.getValue()==SO_SWITCH_NONE?SO_SWITCH_ALL:SO_SWITCH_NONE;return true;}
    return false;
}
bool CoreWorkspace::eventFilter(QObject* object,QEvent* event) {
    if(!enabled||arranging||Gui::Application::Instance->isClosing())return false;
    if(event->type()!=QEvent::Close&&event->type()!=QEvent::Resize&&event->type()!=QEvent::KeyPress)return false;
    auto* area=Gui::getMainWindow()->findChild<QMdiArea*>();
    if(event->type()==QEvent::Close)if(auto* widget=qobject_cast<QWidget*>(object);widget&&widget->property("om9ViewSlot").isValid()&&!widget->property("om9CloseObserved").toBool()) {
        widget->setProperty("om9CloseObserved",true);
        connect(widget,&QObject::destroyed,this,[this]{pendingEnsure=true;QTimer::singleShot(0,this,[this]{if(enabled)ensure();});});
    }
    if(event->type()==QEvent::Resize&&area&&(object==area||object==area->viewport())){QTimer::singleShot(0,this,[this]{if(enabled)layout();});return false;}
    if(event->type()==QEvent::Resize)for(auto* view:slots())if(view&&(object==view||object==view->getViewer()->viewport()))updateTitle(view);
    if(event->type()!=QEvent::KeyPress)return false;
    auto* view=activeView();auto* widget=qobject_cast<QWidget*>(object);if(!view||!widget||!view->getViewer()->isAncestorOf(widget))return false;
    const auto key=static_cast<QKeyEvent*>(event)->key();
    if((key==Qt::Key_F5||key==Qt::Key_F7)&&!Gui::Control().isAllowedAlterView(App::GetApplication().getActiveDocument()))return true;
    if(static_cast<QKeyEvent*>(event)->key()==Qt::Key_F7){toggleActiveGrid();return true;}
    if(static_cast<QKeyEvent*>(event)->key()==Qt::Key_F5){center(view);return true;}
    return false;
}
}
