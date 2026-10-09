#pragma once
#include "CoreCoordinates.h"
#include <QObject>
#include <QString>
#include <QImage>
#include <Base/Placement.h>
#include <fastsignals/signal.h>
#include <cstddef>
namespace App {class Document;}
class SoSeparator;
namespace OpenMatrix9Gui {
class CorePictureFrame final:public QObject {
public:
    static CorePictureFrame& instance();
    static bool handles(std::size_t command);
    bool available()const;
    void activate();
    void deactivate();
    bool start(std::size_t command);
    bool active()const{return running;}
    void cancel();
    void submit(const QString& text);
protected:
    bool eventFilter(QObject*,QEvent*)override;
private:
    CorePictureFrame();
    bool valid()const;
    bool checkContext();
    void prompt(const QString& error={});
    void point(const CoordinateInput&,bool ortho=false);
    void finish(const double* placement,const CoordinateInput&);
    bool plan(const Base::Vector3d* reference,double width,bool ortho,double* output)const;
    void clearPreview();
    void preview(const double* placement);
    bool enabled=false,running=false,hasCorner=false,vertical=false,selfIllumination=true,autoname=true;
    QString source;
    QImage image;
    CoordinateContext coordinateContext;
    std::vector<CoordinateInput> acceptedInputs;
    Base::Vector3d corner;
    App::Document* document=nullptr;
    std::size_t command=0;
    fastsignals::scoped_connection deletedConnection,activeConnection;
    SoSeparator* previewOwner=nullptr;
    SoSeparator* previewNode=nullptr;
};
}
