// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <QObject>
#include <QPointer>
#include <fastsignals/signal.h>
#include <cstddef>
#include <vector>
#include "SolidReferences.h"
#include "CoreSolidResult.h"
#include "CommandOptions.h"
class SoSeparator;
namespace App {class Document;}
namespace Gui {class View3DInventor;}
namespace OpenMatrix9Gui {
class SolidController final:public QObject {
public:
    static SolidController& instance();
    static bool handles(std::size_t);
    static bool matches(std::size_t,const QString&);
    void activate();void deactivate();
    bool available(std::size_t)const;bool active()const;
    bool start(std::size_t,bool recordInvocation=true);void cancel();void submit(const QString&);
    bool optionEvent(unsigned,const QString& text={},double number=0.);
protected:
    bool eventFilter(QObject*,QEvent*)override;
private:
    SolidController();
    bool valid()const;void frame(Gui::View3DInventor*);void prompt();void result(unsigned);
    void recordAcceptedPointPhase(unsigned);
    void preview(const double*,bool);void clearPreview();void commit();
    void reference(const QString&,const std::array<double,3>&);
    void solveTangent(int solution=-1);
    std::map<std::size_t,SolidCurveReference> tangentReferences;
    std::optional<SolidCurveReference> pathReference;
    std::vector<SolidSourceEvidence> fitSources;
    bool enabled=false;
    App::Document* document=nullptr;
    std::size_t command=0;
    unsigned kind=0;
    unsigned inputEffect=0;
    CommandOptionsBridge optionBridge;
    std::string unitContext;
    QPointer<QObject> releaseTarget;
    Qt::MouseButton releaseButton=Qt::LeftButton;
    std::vector<std::pair<SoSeparator*,SoSeparator*>> previews;
    fastsignals::scoped_connection deleteConnection,activeConnection;
};
}
