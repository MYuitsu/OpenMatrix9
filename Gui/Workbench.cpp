#include "Workbench.h"
#include "RustBridge.h"
#include "MatrixSidebar.h"
#include "NativeCommands.h"
#include "CurveController.h"
#include "CoreWorkspace.h"
#include "CoreViewControls.h"
#include "CoreNotes.h"
#include "CorePictureFrame.h"
#include "CoreDistance.h"
#include "CoreSnaps.h"
#include "CoreKeyboard.h"
#include "CoreMouse.h"
#include <App/Application.h>
#include <Gui/MainWindow.h>
#include <QDir>

#include <Gui/MenuManager.h>
#include <Gui/ToolBarManager.h>


using namespace OpenMatrix9Gui;


TYPESYSTEM_SOURCE(
    OpenMatrix9Gui::Workbench,
    Gui::StdWorkbench
)


Workbench::Workbench() = default;

Workbench::~Workbench() = default;


void Workbench::activated()
{
    Gui::StdWorkbench::activated();

    om9_workbench_activated();
    if(!sidebar) {
        const QString resources=QDir(QString::fromStdString(App::Application::getHomePath())).filePath("Mod/OpenMatrix9/Resources");
        sidebar=new MatrixSidebar(Gui::getMainWindow(),resources,{om9NativeCommandAvailable,om9ExecuteNativeCommand});
    }
    static_cast<MatrixSidebar*>(sidebar.data())->activate();
    CurveController::instance().activate();
    CoreWorkspace::instance().activate();
    CoreViewControls::instance().activate();
    CoreNotes::activate();
    CorePictureFrame::instance().activate();
    CoreDistance::instance().activate();
    CoreSnaps::activate();
    CoreKeyboard::instance().activate();
    CoreMouse::instance().activate();
}


void Workbench::deactivated()
{
    CoreMouse::instance().deactivate();
    CoreKeyboard::instance().deactivate();
    CoreSnaps::deactivate();
    CoreDistance::instance().deactivate();
    CorePictureFrame::instance().deactivate();
    om9_workbench_deactivated();
    CoreNotes::deactivate();
    CoreViewControls::instance().deactivate();
    CoreWorkspace::instance().deactivate();
    CurveController::instance().deactivate();
    if(sidebar)static_cast<MatrixSidebar*>(sidebar.data())->deactivate();

    Gui::StdWorkbench::deactivated();
}


Gui::MenuItem* Workbench::setupMenuBar() const
{
    Gui::MenuItem* root =
        Gui::StdWorkbench::setupMenuBar();

    // Main OpenMatrix9 menu
    auto* openMatrixMenu =
        new Gui::MenuItem(root);

    openMatrixMenu->setCommand(
        "&OpenMatrix9"
    );
    auto* workspaceMenu=new Gui::MenuItem();workspaceMenu->setCommand("Workspace");
    for(std::size_t i=0;i<om9_workspace_command_count();++i)
        *workspaceMenu << om9_command_id(om9_workspace_command(i));
    *openMatrixMenu << workspaceMenu;
    auto* infoMenu=new Gui::MenuItem();infoMenu->setCommand("Info & Settings");
    *infoMenu << "OM9_ProjectNotes" << "ViewportTabs" << "CommandHistory" << "Properties" << "F6";*openMatrixMenu << infoMenu;
    auto* snapMenu=new Gui::MenuItem();snapMenu->setCommand("Snaps");*snapMenu << "Osnap E" << "Osnap M" << "Osnap P" << "Ortho";*openMatrixMenu << snapMenu;

    const std::size_t groupCount =
        om9_menu_group_count();

    for (
        std::size_t groupIndex = 0;
        groupIndex < groupCount;
        ++groupIndex
    )
    {
        const char* title =
            om9_menu_group_title(groupIndex);

        if (!title) {
            continue;
        }

        auto* submenu =
            new Gui::MenuItem();

        submenu->setCommand(title);
        if(std::string(title)=="File")*submenu << "Import3dm" << "Export3dm";

        const std::size_t commandCount =
            om9_menu_group_command_count(
                groupIndex
            );

        for (
            std::size_t commandIndex = 0;
            commandIndex < commandCount;
            ++commandIndex
        )
        {
            const char* command =
                om9_menu_group_command_id(
                    groupIndex,
                    commandIndex
                );

            if (command) {
                *submenu << command;
            }
        }

        *openMatrixMenu << submenu;
    }

    return root;
}


Gui::ToolBarItem* Workbench::setupToolBars() const
{
    // The eleven quick positions belong to the sidebar.
    return Gui::StdWorkbench::setupToolBars();
}


Gui::ToolBarItem*
Workbench::setupCommandBars() const
{
    return
        Gui::StdWorkbench::setupCommandBars();
}
