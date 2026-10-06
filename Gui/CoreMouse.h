#pragma once
#include <QObject>
#include <QPointer>
#include <QPoint>
#include <QString>
#include <string>
class QMenu;class QRubberBand;class QTimer;
namespace Gui {class View3DInventor;}
namespace OpenMatrix9Gui {
class CoreMouse final:public QObject {
public:
    static CoreMouse& instance();
    void activate();void deactivate();void prioritize();
    void cancel(bool rollback=true);
protected:bool eventFilter(QObject*,QEvent*)override;
private:
    CoreMouse();void motion(const QPoint&);void select(const QPoint&,bool rectangle);
    void confirm();void recent();
    bool enabled=false,dragging=false,longClick=false;
    unsigned int action=0,button=0,modifiers=0;
    QPoint anchor,last;
    QPointer<Gui::View3DInventor> target;
    QPointer<Gui::View3DInventor> releaseView;
    unsigned int releaseButton=0;
    QPointer<QRubberBand> rubber;
    QPointer<QMenu> popup;
    QTimer* hold;
    std::string originalCamera;
};
}
