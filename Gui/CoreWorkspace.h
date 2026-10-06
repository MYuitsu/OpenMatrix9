#pragma once
#include <QObject>
#include <QPointer>
#include <QString>
#include <Base/Placement.h>
#include <fastsignals/signal.h>
#include <cstddef>
#include <string>
#include <map>
namespace Gui { class View3DInventor; }
class QDockWidget;
class QTabBar;
namespace OpenMatrix9Gui {
class CoreWorkspace final : public QObject {
public:
    static CoreWorkspace& instance();
    void activate();
    void deactivate();
    bool available(std::size_t command) const;
    bool execute(std::size_t command);
    bool executeTabs(const QString& options);
    bool toggleActiveGrid();
    Base::Placement plane(Gui::View3DInventor* view) const;
    static bool handles(std::size_t command);
protected:
    bool eventFilter(QObject*, QEvent*) override;
private:
    CoreWorkspace();
    void ensure(bool restore=false);
    void layout(bool restore=false);
    void updateTabs();
    void center(Gui::View3DInventor* view);
    void reconcileDisplay();
    void ensureTitle(Gui::View3DInventor*);
    void updateTitle(Gui::View3DInventor*);
    bool selectTitle(Gui::View3DInventor*);
    bool toggleTitle(Gui::View3DInventor*);
    bool displayTitle(Gui::View3DInventor*,std::size_t);
    std::map<std::string,std::string> previousDisplay;

    bool enabled=false;
    bool arranging=false;
    bool pendingEnsure=false;
    int previousMdiMode=-1;
    bool previousMdiMaximizeOption=false;
    int tabsAlignment=-1;
    QPointer<QDockWidget> tabsDock;
    QPointer<QTabBar> tabs;
    std::string layoutDocument;
    fastsignals::scoped_connection activeConnection,newConnection,saveConnection;
};
}
