#pragma once
#include <QObject>
#include <QPointer>
#include <QCursor>
#include <fastsignals/signal.h>
#include <cstddef>
#include <string>
class QRubberBand;
namespace Gui {class View3DInventor;}
namespace OpenMatrix9Gui {
class CoreViewControls final:public QObject {
public:
    static CoreViewControls& instance();
    static bool handles(std::size_t command);
    bool available(std::size_t command)const;
    bool execute(std::size_t command);
    void activate();
    void deactivate();
    void cancel();
    bool toolActive()const{return kind!=0;}
    static void scaleCamera(Gui::View3DInventor* view,double factor);
protected:
    bool eventFilter(QObject*,QEvent*)override;
private:
    CoreViewControls();
    void refreshCrosshairs();
    void startTool(unsigned int kind,std::size_t command,Gui::View3DInventor* view);
    void stopTool(bool rollback);
    void prompt();
    bool capture();
    bool enabled=false,dragging=false;
    unsigned int kind=0;
    std::size_t toolCommand=0;
    QPointer<Gui::View3DInventor> toolView;
    QPointer<QRubberBand> rubber;
    std::string originalCamera;
    QCursor originalCursor;
    fastsignals::scoped_connection activeConnection;
};
}
