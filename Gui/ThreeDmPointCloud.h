#pragma once
#include "ThreeDmGeometry.h"
#include <QJsonObject>
#include <filesystem>
namespace OpenMatrix9Gui::ThreeDm {
QJsonObject pointCloudFields(const ON_PointCloud&);
void validatePointCloud(const ON_PointCloud&);
void validateRhino5PointCloud(const ON_PointCloud&);
QJsonObject pointCloudSummary(const ON_PointCloud&);
QString writePointCloudFields(const ON_PointCloud&,const std::filesystem::path&);
void applyPointCloudFields(ON_Geometry&,const std::filesystem::path&,const QString& sha256);
}
