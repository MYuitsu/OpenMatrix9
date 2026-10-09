#include "RustBridge.h"
#include "CoreThreeDm.h"
#include "NativeCommands.h"
#include "CurveController.h"
#include "SurfaceController.h"
#include "EditController.h"
#include "HistoryController.h"
#include "CageController.h"
#include "HistoryFeature.h"
#include "BuilderHistory.h"
#include "CageFeature.h"
#include "SolidController.h"
#include "CoreWorkspace.h"
#include "CoreViewControls.h"
#include "CoreNotes.h"
#include "CorePictureFrame.h"
#include "CoreDistance.h"
#include "CoreSnaps.h"
#include "CoreKeyboard.h"
#include <FCGlobal.h>
#include <Base/Console.h>
#include <Base/Interpreter.h>
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <Gui/Document.h>
#include <Gui/Selection/Selection.h>
#include <Gui/Application.h>
#include <Gui/Command.h>
#include <Gui/Action.h>
#include <Gui/Control.h>
#include <QApplication>
#include <QThread>
#include <QScopedValueRollback>
#include <set>
#include <string>
#include <tuple>
namespace {
bool dispatching=false;
using SelectionKeys=std::set<std::tuple<std::string,std::string,std::string>>;
SelectionKeys selectionKeys(const char* document) {
    SelectionKeys keys;
    for(const auto& sel:Gui::Selection().getSelection(document))
        keys.emplace(sel.DocName,sel.FeatName,sel.SubName?sel.SubName:"");
    return keys;
}
std::set<std::string> documents() {
    std::set<std::string> names;
    for(auto* doc:App::GetApplication().getDocuments())names.insert(doc->getName());
    return names;
}
class RustCommand final:public Gui::Command {
public:
    RustCommand(std::size_t i,const char* id):Gui::Command(id),index(i) {
        sAppModule="OpenMatrix9";sGroup="OpenMatrix9";sMenuText=om9_command_menu_text(i);sToolTipText=om9_command_tooltip(i);sStatusTip=sToolTipText;sWhatsThis=sToolTipText;sPixmap="";
        for(std::size_t key=0;key<om9_keyboard_count();++key)if(std::string(id)==om9_keyboard_target(key)){accelerator=om9_keyboard_key(key);sAccel=accelerator.c_str();break;}
        const auto permissions=om9_command_permissions(i);
        // The dispatched host command owns its transaction, including Undo/Redo.
        eType=NoTransaction;
        if(permissions&1U)eType|=AlterDoc;
        if(permissions&2U)eType|=Alter3DView;
        if(permissions&4U)eType|=AlterSelection;
    }
    const char* className() const override{return "OpenMatrix9Gui::RustCommand";}
    void activated(int) override{om9_sidebar_record_execution(index,om9ExecuteNativeCommand(index));}
    bool isActive() override{if(_pcAction){if(OpenMatrix9Gui::CoreSnaps::handles(index))_pcAction->setBlockedChecked(OpenMatrix9Gui::CoreSnaps::checked(index));if(OpenMatrix9Gui::CoreKeyboard::handles(index))_pcAction->setBlockedChecked(OpenMatrix9Gui::CoreKeyboard::checked(index));}return om9NativeCommandAvailable(index);}
    Gui::Action* createAction() override{auto* action=Gui::Command::createAction();if(OpenMatrix9Gui::CoreSnaps::handles(index)||std::string(om9_command_id(index))=="Ortho")action->setCheckable(true);return action;}
private:std::size_t index;std::string accelerator;
};
}
bool om9NativeCommandAvailable(std::size_t index) {
    if(!Gui::Application::Instance || !qApp || QThread::currentThread()!=qApp->thread())return false;
    if(OpenMatrix9Gui::HistoryController::handles(index))return OpenMatrix9Gui::HistoryController::instance().available(index);
    if(OpenMatrix9Gui::CageController::handles(index))return OpenMatrix9Gui::CageController::instance().available(index);
    if(auto operation=om9_3dm_operation(index)) {auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();return doc&&gui&&!gui->getInEdit()&&Gui::Control().isAllowedAlterDocument(doc)&&(operation==1||Gui::Selection().hasSelection(doc->getName()));}
    if(OpenMatrix9Gui::CoreKeyboard::handles(index))return OpenMatrix9Gui::CoreKeyboard::instance().available(index);
    if(OpenMatrix9Gui::CoreSnaps::handles(index))return OpenMatrix9Gui::CoreSnaps::available();
    if(OpenMatrix9Gui::CoreDistance::handles(index))return OpenMatrix9Gui::CoreDistance::instance().available();
    if(OpenMatrix9Gui::CorePictureFrame::handles(index))return OpenMatrix9Gui::CorePictureFrame::instance().available();
    if(OpenMatrix9Gui::CoreNotes::handles(index))return OpenMatrix9Gui::CoreNotes::available(index);
    if(OpenMatrix9Gui::SurfaceController::handles(index))return OpenMatrix9Gui::SurfaceController::instance().available(index);
    if(OpenMatrix9Gui::EditController::handles(index))return OpenMatrix9Gui::EditController::instance().available(index);
    if(OpenMatrix9Gui::SolidController::handles(index))return OpenMatrix9Gui::SolidController::instance().available(index);
    if(OpenMatrix9Gui::isCurveCommand(index))return OpenMatrix9Gui::CurveController::instance().available(index);
    if(OpenMatrix9Gui::CoreWorkspace::handles(index))return OpenMatrix9Gui::CoreWorkspace::instance().available(index);
    if(OpenMatrix9Gui::CoreViewControls::handles(index))return OpenMatrix9Gui::CoreViewControls::instance().available(index);
    const char* native=om9_command_native_id(index);if(!native)return false;
    auto* doc=App::GetApplication().getActiveDocument();
    const std::string id=native;
    const auto permissions=om9_command_permissions(index);
    if((permissions&1U) && !Gui::Control().isAllowedAlterDocument(doc))return false;
    if((permissions&2U) && !Gui::Control().isAllowedAlterView(doc))return false;
    if((permissions&4U) && !Gui::Control().isAllowedAlterSelection(doc))return false;
    if(id=="OM9_SelectAllObjects" || id=="OM9_ClearSelection" || id=="Std_Delete" || id=="Std_ViewFitSelection") {
        if(!doc)return false;
        auto* guiDoc=Gui::Application::Instance->activeDocument();
        if(!guiDoc || guiDoc->getInEdit())return false;
        if(id=="OM9_SelectAllObjects" && doc->countObjects()==0)return false;
        if(id!="OM9_SelectAllObjects" && !Gui::Selection().hasSelection(doc->getName()))return false;
        if(id=="Std_Delete")for(const auto& sel:Gui::Selection().getCompleteSelection())if(sel.pDoc!=doc)return false;
        if(id=="OM9_ClearSelection" || id=="OM9_SelectAllObjects")return true;
    }
    auto* command=Gui::Application::Instance->commandManager().getCommandByName(native);
    return command && command->isActive();
}
bool om9ExecuteNativeCommand(std::size_t index) {
    if(OpenMatrix9Gui::HistoryController::handles(index))return OpenMatrix9Gui::HistoryController::instance().start(index);
    if(OpenMatrix9Gui::CageController::handles(index))return OpenMatrix9Gui::CageController::instance().start(index);
    if(OpenMatrix9Gui::EditController::handles(index))return OpenMatrix9Gui::EditController::instance().start(index);
    if(OpenMatrix9Gui::SolidController::handles(index))return OpenMatrix9Gui::SolidController::instance().start(index);
    if(OpenMatrix9Gui::SurfaceController::handles(index))return OpenMatrix9Gui::SurfaceController::instance().start(index);
    if(auto operation=om9_3dm_operation(index)) {
        if(!om9NativeCommandAvailable(index))return false;
        return executeThreeDm(operation);
    }
    if(OpenMatrix9Gui::CoreKeyboard::handles(index))return OpenMatrix9Gui::CoreKeyboard::instance().execute(index);
    if(OpenMatrix9Gui::CoreSnaps::handles(index))return OpenMatrix9Gui::CoreSnaps::execute(index);
    if(OpenMatrix9Gui::CoreDistance::handles(index))return OpenMatrix9Gui::CoreDistance::instance().start(index);
    if(OpenMatrix9Gui::CorePictureFrame::handles(index))return OpenMatrix9Gui::CorePictureFrame::instance().start(index);
    if(OpenMatrix9Gui::CoreNotes::handles(index))return OpenMatrix9Gui::CoreNotes::execute(index);
    if(OpenMatrix9Gui::CoreViewControls::handles(index))return OpenMatrix9Gui::CoreViewControls::instance().execute(index);
    if(OpenMatrix9Gui::CoreWorkspace::handles(index))return OpenMatrix9Gui::CoreWorkspace::instance().execute(index);
    if(OpenMatrix9Gui::isCurveCommand(index)){OpenMatrix9Gui::CurveController::instance().start(index);return false;} // Record successful history only on geometry commit.
    if(dispatching || !om9NativeCommandAvailable(index))return false;
    const std::string native=om9_command_native_id(index);
    auto before=documents();auto* doc=App::GetApplication().getActiveDocument();
    const std::string name=doc?doc->getName():"";
    bool saved=false;
    fastsignals::scoped_connection saveConnection(App::GetApplication().signalFinishSaveDocument.connect(
        [&saved,&name](const App::Document& document,const std::string&) {if(name==document.getName())saved=true;}));
    const int undos=doc?doc->getAvailableUndos():0,redos=doc?doc->getAvailableRedos():0;
    const int objects=doc?doc->countObjects():0;
    const auto selected=name.empty()?SelectionKeys{}:selectionKeys(name.c_str());
    QScopedValueRollback<bool> guard(dispatching,true);
    try {
        if(native=="OM9_SelectAllObjects") {
            Gui::Selection().clearSelection(name.c_str());
            if(auto* current=App::GetApplication().getDocument(name.c_str())) {
                std::vector<App::DocumentObject*> objects;
                for(auto* object:current->getObjects())if(!OpenMatrix9Gui::CoreNotes::isStorageObject(object)&&!dynamic_cast<OpenMatrix9Gui::HistorySettings*>(object)&&!OpenMatrix9Gui::isBuilderStorageObject(object)&&!OpenMatrix9Gui::isCageStorageObject(object))objects.push_back(object);
                Gui::Selection().setSelection(name.c_str(),objects);
            }
        }
        else if(native=="OM9_ClearSelection")Gui::Selection().clearSelection(name.c_str());
        else Gui::Application::Instance->commandManager().runCommandByName(native.c_str());
        // Reacquire by name after modal operations: a document may have closed.
        auto* after=name.empty()?nullptr:App::GetApplication().getDocument(name.c_str());
        unsigned int effects=documents()!=before?1U:0U;
        if(saved)effects|=2U;
        if(after) {
            if(after->getAvailableUndos()!=undos || after->getAvailableRedos()!=redos)effects|=4U;
            if(selectionKeys(name.c_str())!=selected)effects|=8U;
            if(after->countObjects()<objects)effects|=16U;
        }
        if(native.rfind("Std_View",0)==0)effects|=32U;
        return om9_command_success(index,effects);
    } catch(const std::exception& error) {Base::Console().error("OpenMatrix9: %s\n",error.what());return false;}
      catch(...) {Base::Console().error("OpenMatrix9: native command failed\n");return false;}
}
void CreateOpenMatrix9Commands() {
    auto& manager=Gui::Application::Instance->commandManager();
    for(std::size_t i=0;i<om9_command_count();++i)if(const char* id=om9_command_id(i);id && !manager.getCommandByName(id))manager.addCommand(new RustCommand(i,id));
}
