// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <string>
#include <vector>
#include <cstdint>
namespace App { class Document; class DocumentObject; }
namespace OpenMatrix9Gui {
void registerLayerDocumentTypes();
// Geometry-free canonical controller foundation. All plans/guards are Rust;
// FreeCAD object/property lifetimes and command-owned transactions stay native.
void initializeLayerDocument(App::Document&, const std::string& snapshotJson);
std::string layerDocumentSnapshot(App::Document&);
std::string layerDocumentCommand(App::Document&, const std::string& commandJson);
std::string layerDocumentText(App::Document&,const std::string&,const std::vector<std::string>& selected);
std::string layerDocumentPanel(App::Document&);
struct LayerExportSelection {std::string snapshotJson;std::vector<std::string> objectIds;};
// Rust subset retains the entire palette. These functions extract metadata only;
// current geometry remains owned by the existing native/kernel staging adapter.
LayerExportSelection layerDocumentExportSelection(App::Document&,const std::vector<std::string>& nativeNames);
void validateLayerDocumentExportSelection(App::Document&,const std::vector<std::string>& nativeNames,const std::string& expectedJson);
bool isLayerStorageObject(const App::DocumentObject*);
void requireLayerGeometryEditable(App::Document&);
// Exchange bindings carry observed source state; the native creation observer
// must not apply drawing defaults while the receiving adapter binds objects.
class LayerNativeReceiveScope final {
public:
    explicit LayerNativeReceiveScope(App::Document&);
    ~LayerNativeReceiveScope();
    LayerNativeReceiveScope(const LayerNativeReceiveScope&)=delete;
    LayerNativeReceiveScope& operator=(const LayerNativeReceiveScope&)=delete;
private:
    std::string identity;
};
// Native names map to owned logical IDs; Rust checks the complete batch.
// Returns the checked snapshot generation for native geometry witnesses.
std::uint64_t layerMutationGeneration(App::Document&,const std::vector<std::string>& nativeNames,std::uint32_t operation);
bool ownsLayerGeometryTransaction(App::Document&,int transactionId);
// Source palette/own-state merge is computed by Rust before geometry mutation.
// Host names are supplied only after the native geometry binding has succeeded.
class LayerDocumentReceivePlan final {
public:
    LayerDocumentReceivePlan(App::Document&,const std::string& sourceJson,const std::vector<std::string>& sourceIds);
    ~LayerDocumentReceivePlan();
    LayerDocumentReceivePlan(const LayerDocumentReceivePlan&)=delete;
    LayerDocumentReceivePlan& operator=(const LayerDocumentReceivePlan&)=delete;
    void finish(int transactionId,const std::vector<std::string>& nativeNames);
private:
    App::Document& doc;
    std::string identity,canonicalBefore;
    std::vector<std::string> mappedIds;
    std::uint64_t before=0,after=0,plan=0;
    bool hadState=false,finished=false;
};
// Captures owned Rust state inside the caller's explicitly owned transaction.
// No observer may commit/abort a transaction belonging to another command.
class LayerGeometryTransaction final {
public:
    LayerGeometryTransaction(App::Document&,int transactionId);
    ~LayerGeometryTransaction();
    LayerGeometryTransaction(const LayerGeometryTransaction&)=delete;
    LayerGeometryTransaction& operator=(const LayerGeometryTransaction&)=delete;
    void finish();
private:
    App::Document& doc;
    std::string identity,canonicalBefore;
    std::uint64_t before=0;
    int transactionId=0;
    bool hadState=false,finished=false;
};
}
