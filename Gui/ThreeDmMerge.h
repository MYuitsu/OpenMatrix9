#pragma once
#include "ThreeDmInventory.h"
namespace OpenMatrix9Gui::ThreeDm {
void writePreservedArchive(const QJsonObject&,const std::filesystem::path&);
// Internal current-graph staging for subsequent flattening; always SDK80,
// never a Rhino5 preservation export or a bypass of its orientation guard.
void writeGeometryStagingArchive(const QJsonObject&,const std::filesystem::path&);
}
