// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <QJsonObject>
namespace App {class Document;class DocumentObject;}
namespace OpenMatrix9Gui {
// Proves source bytes and stored/native manifest agreement only. Caller must
// separately prove model/helper roles, bindings, dependencies and current edits.
QJsonObject verifyLayerSourceArchive(App::Document&,App::DocumentObject&);
}
