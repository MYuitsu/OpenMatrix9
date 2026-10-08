// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "EditGeometry.h"
#include <QObject>
#include <QPointer>
#include <fastsignals/signal.h>
#include <tuple>
class QDialog;class QLabel;class QDialogButtonBox;class SoSeparator;class SoSwitch;
namespace OpenMatrix9Gui {
class EditController final:public QObject {
public:
    static EditController& instance();static bool handles(std::size_t);static bool matches(std::size_t,const QString&);
    void activate();void deactivate();bool available(std::size_t)const;bool active()const;
    bool start(std::size_t);void cancel();void submit(const QString&);
protected:bool eventFilter(QObject*,QEvent*)override;
private:
    EditController();bool valid()const;void prompt(const QString&);void refresh();void add(const std::string&);
    void prepare();void preview();void clearPreview();void commit();void trim(const std::string&,const std::array<double,3>&);void error(const std::exception&);
    bool enabled=false;App::Document* document=nullptr;std::size_t command=0;unsigned kind=0;
    std::vector<EditInput> inputs;EditShapes output;std::vector<EditShapes> fragments;
    std::vector<std::set<std::size_t>> removed;std::vector<std::pair<std::size_t,std::size_t>> trimUndo;
    QPointer<QDialog> dialog;QPointer<QLabel> status;QPointer<QDialogButtonBox> buttons;QPointer<QObject> releaseTarget;
    std::vector<std::pair<SoSeparator*,SoSeparator*>> previews;
    std::vector<std::pair<SoSwitch*,int>> hidden;
    std::vector<std::tuple<SoSeparator*,std::size_t,std::size_t>> trimPreviewSources;
    fastsignals::scoped_connection deleteConnection,activeConnection;
};
}
