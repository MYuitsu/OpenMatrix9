// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "SurfaceGeometry.h"
#include <QObject>
#include <QPointer>
#include <fastsignals/signal.h>
#include <cstddef>
#include <stdexcept>
class QDialog;class QLabel;class QDialogButtonBox;class QTableWidget;class SoSeparator;
namespace App { class Document; }
namespace OpenMatrix9Gui {
class SurfaceController final:public QObject {
public:
    static SurfaceController& instance();
    static bool handles(std::size_t);
    static bool matches(std::size_t,const QString&);
    void activate();void deactivate();
    bool available(std::size_t)const;bool active()const;
    bool start(std::size_t,const QString& invoked={});void cancel();void submit(const QString&);
protected:
    bool eventFilter(QObject*,QEvent*)override;
private:
    SurfaceController();
    bool valid()const;void prompt(const QString&);void refresh();
    void add(const std::string&,const std::string&);void optionsDialog();void preview();void commit();void clearPreview();
    void error(const std::exception&);
    void alignInputs(bool automatic);
    void addSlash(double,double);void updateSlashes();
    int slashPicking=0;double slashFirst=0;
    QPointer<QTableWidget> slashTable;
    bool enabled=false;
    App::Document* document=nullptr;std::size_t command=0;
    SurfaceOptions options;std::vector<SurfaceInput> inputs;
    std::vector<SurfaceInput> chainInputs;
    QPointer<QDialog> dialog;QPointer<QLabel> status;QPointer<QDialogButtonBox> buttons;
    QPointer<QObject> releaseTarget;
    std::vector<std::pair<SoSeparator*,SoSeparator*>> previews;
    fastsignals::scoped_connection deleteConnection,activeConnection;
};
}
