// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <array>
#include <optional>
#include <QString>
#include <QPoint>
namespace App {class Document;}
namespace Gui {class View3DInventor;}
namespace OpenMatrix9Gui {
void clearEllipseReferences();
void verifyEllipseReferences(App::Document&);
std::optional<unsigned> ellipseNativeInput(App::Document&,const QString&);
std::optional<unsigned> ellipseNativePick(App::Document&,Gui::View3DInventor*,const QPoint&);
std::optional<std::array<double,3>> ellipseNativeHover(App::Document&,Gui::View3DInventor*,const QPoint&);
void ellipseNativeResult(unsigned);
void commitEllipse(App::Document&);
}
