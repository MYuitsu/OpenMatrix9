#pragma once
#include "ThreeDmGeometry.h"
#include <filesystem>
#include <QJsonObject>
#include <QJsonArray>
namespace OpenMatrix9Gui::ThreeDm {
struct NativeReferenceResolution;
struct ArchiveInventory { QJsonObject document; std::shared_ptr<ONX_Model> nativeModel; std::shared_ptr<NativeReferenceResolution> nativeReferences; };
ArchiveInventory inspectArchive(const std::filesystem::path&, double customScaleMm=0);
std::string inventoryJson(const ArchiveInventory&);
QJsonObject nativeGeometryFacts(const ON_Geometry&);
}
