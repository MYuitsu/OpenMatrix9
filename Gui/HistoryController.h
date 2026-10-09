// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "EditGeometry.h"
#include <App/Application.h>
#include <QObject>
#include <QPointer>
#include <QString>
#include <cstddef>
class QDialog;
class QLabel;
namespace OpenMatrix9Gui {
class HistoryController final:public QObject {
public:
    static HistoryController& instance();
    static bool handles(std::size_t);
    static bool matches(std::size_t,const QString&);
    bool available(std::size_t)const;
    bool active()const;
    bool start(std::size_t);
    void cancel();
    void submit(const QString&);
protected:bool eventFilter(QObject*,QEvent*)override;
private:
    HistoryController();
    App::Document* document=nullptr;
    std::size_t command=0;
    unsigned kind=0;
    double tolerance=1e-7;
    std::vector<EditInput> inputs;
    QPointer<QDialog> dialog;
    QPointer<QLabel> status;
    bool valid()const;
    void prompt(const QString&);
    void add(const std::string&);
    void joinOptions();
    void settingsOptions();
    void commit();
    void setOption(unsigned,bool);
    fastsignals::scoped_connection deleted,changed;
};
}
