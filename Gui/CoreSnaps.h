#pragma once
#include <Base/Vector3D.h>
#include <QPoint>
#include <QString>
#include <cstddef>
namespace Gui {class View3DInventor;}
namespace OpenMatrix9Gui {
class CoreSnaps {
public:
    static void activate();
    static void deactivate();
    static bool handles(std::size_t command);
    static bool available();
    static bool execute(std::size_t command);
    static bool checked(std::size_t command);
    static bool submit(const QString& text);
    static bool pick(Gui::View3DInventor* view,const QPoint& pixel,Base::Vector3d& output);
};
}
