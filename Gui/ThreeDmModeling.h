#pragma once
#include <filesystem>
#include <QJsonObject>
namespace OpenMatrix9Gui::ThreeDm {
QJsonObject prepareModelingArchive(const std::filesystem::path&,const std::filesystem::path&,double customUnitMm=0,const std::filesystem::path& layerMetadata={});
QJsonObject modelingCapabilities(unsigned kind);
}
