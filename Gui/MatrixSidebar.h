#pragma once
#include <QDockWidget>
#include <QPointer>
#include <QHash>
#include <QImage>
#include <functional>
#include <cstddef>
#include <vector>
class QMainWindow;
class QVBoxLayout;
class QToolButton;
class QTimer;
namespace OpenMatrix9Gui {
struct HostCallbacks {
    std::function<bool(std::size_t)> available;
    std::function<bool(std::size_t)> execute;
};
class MatrixSidebar : public QDockWidget {
public:
    MatrixSidebar(QMainWindow* window, const QString& resourceRoot, HostCallbacks host);
    void activate();
    void deactivate();
    void refreshState();
    void refreshAvailability();
    QIcon iconForCommand(std::size_t command);
private:
    QIcon iconForKey(const QString& key,const QSize& size=QSize(24,24));
    QToolButton* commandButton(std::size_t command, QWidget* parent, const QString& name);
    QWidget* section(int index, const QString& title, QWidget* body);
    void populateGrid();
    void populateHistory();
    void refreshLayers();
    void invoke(std::size_t command);
    std::size_t findCommand(const char* icon) const;
    QString tooltip(std::size_t command) const;
    QMainWindow* window;
    QString resourceRoot;
    HostCallbacks host;
    std::vector<QWidget*> sections;
    std::vector<QWidget*> bodies;
    QWidget* grid;
    QWidget* history;
    QTimer* timer;
    std::vector<std::pair<QPointer<QDockWidget>,bool>> alteredDocks;
    QHash<QString,QImage> atlases;
    std::vector<std::size_t> renderedHistory;
    bool active=false;
};
}
