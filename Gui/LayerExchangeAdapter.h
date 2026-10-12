// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "LayerRustAbi.h"
#if defined(_WIN64) && defined(WIN32)
#undef WIN32
#endif
#include <opennurbs.h>
#include <array>
#include <map>
#include <optional>
#include <string>
#include <vector>

namespace OpenMatrix9Gui::ThreeDm {
struct NativeLayerRow {
    std::string id, parentId, name, path;
    std::array<unsigned char,3> rgb{180,180,180};
    bool locked=false, visible=true;
    std::optional<bool> persistentLocked, persistentVisible;
};
struct NativeLayerTable { std::vector<NativeLayerRow> rows; std::string activeId; };
struct NativeLayerContext { std::string document; std::uint64_t generation=0; };
struct NativeObjectLayerRow {
    std::string id, layerId;
    std::array<unsigned char,3> rgb{180,180,180};
    bool locked=false, visible=true;
    unsigned char colorSource=1; // Rust schema: 1 ByLayer, 2 ByObject.
};
struct NativeLayerObjectBinding { std::string physicalId,sourceId; };
struct NativeRetainedLayerResult { NativeLayerTable table; std::vector<NativeObjectLayerRow> objects; };
// Detached archive only. Canonical model-space objects are explicit bindings;
// untouched native definition members keep their own ByParent/instance fields.
NativeRetainedLayerResult applyRetainedLayerOverlay(ONX_Model&,std::uint64_t,
    const std::vector<NativeLayerObjectBinding>&);
// Rust fullpath merge; returned source IDs map to actual SDK UUIDs, including
// insertion remaps. Metadata only, no native geometry/member pointer retained.
std::map<std::string,std::string> mergeRetainedNativePalette(ONX_Model&,const NativeLayerTable&);
// Rust owns copied metadata. This adapter never retains ONX/Qt/FreeCAD pointers.
class NativeLayerSnapshot final {
public:
    explicit NativeLayerSnapshot(std::uint64_t handle=0) noexcept : handle_(handle) {}
    ~NativeLayerSnapshot();
    NativeLayerSnapshot(const NativeLayerSnapshot&)=delete;
    NativeLayerSnapshot& operator=(const NativeLayerSnapshot&)=delete;
    NativeLayerSnapshot(NativeLayerSnapshot&&) noexcept;
    NativeLayerSnapshot& operator=(NativeLayerSnapshot&&) noexcept;
    std::uint64_t get() const noexcept { return handle_; }
    std::uint64_t release() noexcept {const auto handle=handle_;handle_=0;return handle;}
private:
    std::uint64_t handle_=0;
};
NativeLayerTable readNativeLayerTable(const ONX_Model&);
std::vector<NativeObjectLayerRow> readNativeObjectLayers(const ONX_Model&,bool modelSpaceOnly);
NativeLayerSnapshot createNativeLayerSnapshot(const NativeLayerTable&,
    const std::vector<NativeObjectLayerRow>&,const std::string& document,std::uint64_t generation,bool legacyNativeNames=false);
NativeLayerTable nativeLayerTableFromSnapshot(std::uint64_t);
NativeLayerContext nativeLayerContextFromSnapshot(std::uint64_t);
std::vector<NativeObjectLayerRow> nativeObjectLayersFromSnapshot(std::uint64_t);
std::string nativeLayerSnapshotJson(std::uint64_t);
NativeLayerSnapshot nativeLayerSnapshotFromJson(const std::string&);
// Fresh archive writer; all policy/preflight comes from the Rust snapshot API.
std::map<std::string,int> writeNativeLayerTable(ONX_Model&,const NativeLayerTable&);
}
