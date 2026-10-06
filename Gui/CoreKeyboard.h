#pragma once
#include <QObject>
#include <QPointer>
#include <QString>
#include <QStringList>
#include <cstddef>
class QDockWidget;class QMenu;
class QKeyEvent;
namespace OpenMatrix9Gui {
class CoreKeyboard final:public QObject {
public:
    static CoreKeyboard& instance();
    void activate();void deactivate();
    static bool handles(std::size_t command);
    bool available(std::size_t command)const;
    bool execute(std::size_t command);
    bool submit(const QString& text);
    void record(const QString& text);
    static bool checked(std::size_t command);
    static bool inputContext(QObject* receiver);
    static bool pointKeyAllowed(QObject* receiver,const QKeyEvent* key);
protected:bool eventFilter(QObject*,QEvent*)override;
private:
    CoreKeyboard();void history();void contextMenu();
    bool enabled=false;
    QPointer<QDockWidget> historyDock;
    QPointer<QMenu> menu;
    QStringList inputs;
};
}
