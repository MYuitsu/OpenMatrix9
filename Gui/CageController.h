// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <App/Application.h>
#include <QObject>
#include <QPointer>
#include <QString>
#include <cstddef>
class QDialog;
class QLabel;
namespace OpenMatrix9Gui {
// Permanent native exception: Qt/FreeCAD GUI and document inspection require
// native APIs. Rust owns independent snapshots, phases, validation and options.
class CageController final:public QObject {
public:
    static CageController& instance();
    static bool handles(std::size_t);
    static bool matches(std::size_t,const QString&);
    bool available(std::size_t)const;
    bool active()const;
    bool start(std::size_t);
    void submit(const QString&);
    void cancel();
protected:bool eventFilter(QObject*,QEvent*)override;
private:
    CageController();
    App::Document* document=nullptr;
    void* session=nullptr;
    std::size_t command=0;
    unsigned kind=0;
    QPointer<QDialog> dialog;
    QPointer<QLabel> status;
    fastsignals::scoped_connection deleted,changed;
    bool valid()const;
    void prompt(const QString&);
    void check(int)const;
    void add(const QString&,unsigned role=0);
    void selection();
    void chooseControl();
    void options();
    void closeDialog();
    void syncOptions();
    void commit();
};
}
