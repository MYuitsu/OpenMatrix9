// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <QJsonObject>
namespace App {class DocumentObject;}
namespace OpenMatrix9Gui {
struct ObjectSnapshot;
// GUI-thread only. Rust validates/owns bounded native identity and appearance.
QJsonObject sourceAttributes(App::DocumentObject&,const ObjectSnapshot&);
}
