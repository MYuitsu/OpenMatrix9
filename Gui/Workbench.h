#pragma once

#include <Gui/Workbench.h>
#include <QPointer>
class QDockWidget;

namespace OpenMatrix9Gui
{
class MatrixSidebar;

class Workbench final : public Gui::StdWorkbench
{
    TYPESYSTEM_HEADER_WITH_OVERRIDE();

public:
    Workbench();
    ~Workbench() override;

    void activated() override;
    void deactivated() override;

protected:
    Gui::MenuItem* setupMenuBar() const override;

    Gui::ToolBarItem* setupToolBars() const override;

    Gui::ToolBarItem* setupCommandBars() const override;
private:
    QPointer<QDockWidget> sidebar;
};

} // namespace OpenMatrix9Gui
