// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <QJsonObject>
#include <array>
namespace App {class Document;class DocumentObject;}
namespace OpenMatrix9Gui {
struct ObjectSnapshot;
struct DefinitionReaders {
    QJsonObject (*leaf)(App::Document&,const char*,const char*);
    void (*reframe)(QJsonObject&,App::DocumentObject&,const std::array<double,16>&);
};
// Synchronous GUI-thread native adapter. Rust owns/validates the occurrence
// graph; existing Rust-backed leaf readers supply independent geometry/attrs.
QJsonObject definitionSnapshot(App::DocumentObject&,const ObjectSnapshot&,DefinitionReaders);
}
