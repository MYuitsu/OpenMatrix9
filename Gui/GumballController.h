// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "GumballGeometry.h"
#include <QObject>
#include <QPointer>
#include <QJsonObject>
#include <QJsonArray>
#include <QPoint>
#include <optional>
#include <fastsignals/signal.h>
class QMenu;class QLineEdit;class QWidget;class QEvent;
namespace Gui {class View3DInventor;}
namespace OpenMatrix9Gui {
class GumballOverlay;
class GumballController final:public QObject {
public:
    static GumballController& instance();
    static bool handles(std::size_t);static bool matches(std::size_t,const QString&);
    static bool checked(std::size_t);
    bool available(std::size_t)const;bool execute(std::size_t);
    void activate();void deactivate();bool active()const;void cancel();
    bool submit(const QString&);bool processEvent(QObject*,QEvent*);
    void refresh();QJsonObject state();QJsonArray handlePositions()const;
    void addMethods(_object*);
private:
    GumballController();
    bool applyCommand(unsigned,unsigned);bool event(const Om9GumballEvent&);
    void alignmentMenu();void message(const QString&);void persist();
    void widgetMenu(const QPoint&);
    void syncViews();void clearOverlays();void clearPreviews();
    void loadSettings();bool applySettings(const Om9GumballSettings&);void settingsDialog();void orbitAroundPivot();
    GumballOverlay* overlayFor(Gui::View3DInventor*)const;
    void updateOverlay();void updatePreview();void finishGesture();void releaseMouse();
    Om9GumballSnapshot snapshot()const;
    std::uint64_t session=0;bool workbench=false,relocated=false,publishing=false;
    std::optional<GumballInputs> inputs;
    QString error,selectionKey,documentName;
    QPointer<Gui::View3DInventor> view;
    QPointer<QMenu> popup;
    struct ViewOverlay {QPointer<Gui::View3DInventor> view;std::unique_ptr<GumballOverlay> overlay;};
    std::vector<ViewOverlay> views;
    GumballOverlay* overlay=nullptr;
    std::vector<std::array<double,3>> extentPoints;
    std::array<double,12> extentFrame{};
    QJsonArray gestureCamera;
    QPointer<QWidget> mouseOwner;
    QPoint pressPosition;
    bool mouseDown=false,moved=false,widgetRelocate=false,suppressRelease=false;
    std::vector<CoordinateInput> relocateInputs;
    unsigned relocatePointCount=0;
    fastsignals::scoped_connection deleteConnection,activeConnection;
};
}
