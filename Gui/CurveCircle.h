// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <array>
#include <optional>
#include <QString>
#include <QPoint>
struct _object;
namespace App {class Document;class DocumentObject;}
namespace Gui {class View3DInventor;}
namespace OpenMatrix9Gui {
void clearCircleReferences();
void verifyCircleReferences(App::Document&);
std::optional<unsigned> circleNativeInput(App::Document&,const QString&);
std::optional<unsigned> circleNativePick(App::Document&,Gui::View3DInventor*,const QPoint&);
std::optional<std::array<double,3>> circleNativeHover(App::Document&,Gui::View3DInventor*,const QPoint&);
unsigned circleNativeResult(App::Document&,unsigned);
App::DocumentObject* circleHistoryObject(App::Document&,_object* shape);
}
