#pragma once
#include "ThreeDmGeometry.h"
#include <filesystem>
#include <QJsonObject>
#include <QJsonArray>
namespace OpenMatrix9Gui::ThreeDm {
struct ArchiveInventory { QJsonObject document; };
ArchiveInventory inspectArchive(const std::filesystem::path&, double customScaleMm=0);
std::string inventoryJson(const ArchiveInventory&);
}
