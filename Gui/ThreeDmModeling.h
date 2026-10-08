#pragma once
#include <filesystem>
#include <QJsonObject>
namespace OpenMatrix9Gui::ThreeDm {
QJsonObject prepareModelingArchive(const std::filesystem::path&,const std::filesystem::path&,double customUnitMm=0);
QJsonObject modelingCapabilities(unsigned kind);
}
