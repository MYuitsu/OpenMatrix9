#pragma once
#include <QObject>
#include <QString>
#include <QPointer>
#include "CoreCoordinates.h"
#include <fastsignals/signal.h>
#include <cstddef>
namespace App {class Document;}
namespace OpenMatrix9Gui {
class CoreDistance final:public QObject {
public:
    static CoreDistance& instance();
    static bool handles(std::size_t command);
    bool available()const;
    bool active()const;
    void activate();
    void deactivate();
    bool start(std::size_t command);
    void cancel();
    void submit(const QString& text);
protected:
    bool eventFilter(QObject*,QEvent*)override;
private:
    CoreDistance();
    void point(const CoordinateInput&);
    void finish(const CoordinateInput&,const QString&);
    bool checkContext();
    std::optional<Base::Vector3d> previousPoint()const;
    void prompt(const QString&);
    bool valid()const;
    bool enabled=false,running=false,hasFirst=false;
    unsigned int unit=0;
    std::size_t command=0;
    App::Document* document=nullptr;
    CoordinateContext coordinateContext;
    std::vector<CoordinateInput> accepted;
    Base::Vector3d first;
    QPointer<QObject> releaseTarget;
    fastsignals::scoped_connection deletedConnection,activeConnection;
    bool measuringAngle=false;
    unsigned int angleCount=0;
    Base::Vector3d anglePoints[4];
};
}
