#include "CoreSnaps.h"
#include "CoreSnapGeometry.h"
#include "RustBridge.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/PropertyGeo.h>
#include <App/ComplexGeoData.h>
#include <App/GeoFeatureGroupExtension.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <Gui/Command.h>
#include <Gui/Action.h>
#include <Gui/MainWindow.h>
#include <QToolButton>
#include <QSignalBlocker>
#include <Gui/ViewProvider.h>
#include <Gui/View3DInventor.h>
#include <Gui/View3DInventorViewer.h>
#include <Inventor/SoRenderManager.h>
#include <Inventor/nodes/SoCamera.h>
#include <Base/Exception.h>
#include <limits>
#include <vector>
#include <unordered_set>
#include <utility>
namespace {
bool enabled=false;
unsigned int modeBit(const QString& id){
    if(id.compare("Osnap E",Qt::CaseInsensitive)==0)return 2;
    if(id.compare("Osnap M",Qt::CaseInsensitive)==0)return 4;
    if(id.compare("Osnap P",Qt::CaseInsensitive)==0)return 8;
    return 0;
}
auto preferences(){return App::GetApplication().GetParameterGroupByPath("User parameter:BaseApp/Preferences/Mod/OpenMatrix9/Osnap");}
void syncAction(){
    for(const auto& entry:{std::pair{"Osnap E",2U},std::pair{"Osnap M",4U},std::pair{"Osnap P",8U}}){
        auto* command=Gui::Application::Instance->commandManager().getCommandByName(entry.first);
        if(command&&command->getAction())command->getAction()->setBlockedChecked((om9_snap_state()&entry.second)!=0);
    }
    if(auto* button=Gui::getMainWindow()->findChild<QToolButton*>("OM9OsnapMaster")){
        const QSignalBlocker blocker(button);button->setChecked((om9_snap_state()&1U)!=0);button->setEnabled(enabled);
    }
}
void save(){preferences()->SetInt("State",static_cast<long>(om9_snap_state()));syncAction();}
}
namespace OpenMatrix9Gui {
void CoreSnaps::activate(){enabled=true;if(!om9_snap_load(static_cast<unsigned int>(preferences()->GetInt("State",0))))om9_snap_load(0);syncAction();}
void CoreSnaps::deactivate(){enabled=false;syncAction();}
bool CoreSnaps::handles(std::size_t index){auto* id=om9_command_id(index);return id&&modeBit(QString::fromUtf8(id));}
bool CoreSnaps::available(){return enabled&&!Gui::Application::Instance->isClosing()&&Gui::Control().isAllowedAlterView(App::GetApplication().getActiveDocument());}
bool CoreSnaps::checked(std::size_t command){return handles(command)&&(om9_snap_state()&modeBit(QString::fromUtf8(om9_command_id(command))))!=0;}
bool CoreSnaps::execute(std::size_t command){if(!handles(command)||!available())return false;om9_snap_toggle(modeBit(QString::fromUtf8(om9_command_id(command))));save();return true;}
bool CoreSnaps::submit(const QString& text){
    const auto input=text.trimmed();
    if(auto mode=modeBit(input)){if(available()){om9_snap_toggle(mode);save();}return true;}
    if(input.compare("Osnap On",Qt::CaseInsensitive)==0||input.compare("Osnap Off",Qt::CaseInsensitive)==0||input.compare("Osnap Toggle",Qt::CaseInsensitive)==0){
        if(available()){auto state=om9_snap_state();if(input.endsWith("Toggle",Qt::CaseInsensitive)||bool(state&1U)!=input.endsWith("On",Qt::CaseInsensitive))om9_snap_toggle(1);save();}return true;
    }
    return false;
}
bool CoreSnaps::pick(Gui::View3DInventor* view,const QPoint& pixel,Base::Vector3d& output){
    const auto state=om9_snap_state();
    if(!enabled||!(state&1U)||!(state&14U)||!view)return false;
    auto* viewer=view->getViewer();auto* camera=viewer->getSoRenderManager()->getCamera();if(!camera)return false;
    auto* gui=Gui::Application::Instance->activeDocument();auto* document=App::GetApplication().getActiveDocument();if(!gui||!document||gui->isAboutToClose())return false;
    const int width=viewer->viewport()->width(),height=viewer->viewport()->height();if(width<=0||height<=0)return false;
    const auto volume=camera->getViewVolume(float(width)/height);
    std::vector<double> packed;std::vector<Base::Vector3d> points;
    for(auto* object:document->getObjects()){
        auto* provider=gui->getViewProvider(object);if(!provider||!provider->isVisible())continue;
        bool visible=true;std::unordered_set<const App::DocumentObject*> parents;
        for(auto* parent=App::GeoFeatureGroupExtension::getGroupOfObject(object);parent;parent=App::GeoFeatureGroupExtension::getGroupOfObject(parent)){
            if(!parents.insert(parent).second){visible=false;break;}
            auto* parentProvider=gui->getViewProvider(parent);
            if(!parentProvider||!parentProvider->isVisible()){visible=false;break;}
        }
        if(!visible)continue;
        try {std::vector<Base::Vector3d> candidates;
            if(state&2U)candidates=endCandidates(object);
            if(state&4U){auto mid=midCandidates(object);candidates.insert(candidates.end(),mid.begin(),mid.end());}
            if(state&8U){auto points=pointCandidates(object);candidates.insert(candidates.end(),points.begin(),points.end());}
            for(const auto& point:candidates){
            SbVec3f projected;volume.projectToScreen(SbVec3f(float(point.x),float(point.y),float(point.z)),projected);
            if(projected[0]<0||projected[0]>1||projected[1]<0||projected[1]>1||projected[2]<0||projected[2]>1)continue;
            points.push_back(point);packed.insert(packed.end(),{point.x,point.y,point.z,double(projected[0])*width,(1.-double(projected[1]))*height});
        }}catch(const Base::Exception&){continue;}
    }
    const auto index=om9_snap_mode_pick(packed.data(),points.size(),pixel.x(),pixel.y(),8.,state&2U?2U:state&4U?4U:8U);
    if(index==std::numeric_limits<std::size_t>::max()||index>=points.size())return false;
    output=points[index];return true;
}
}
