#pragma once
#include "ThreeDmGeometry.h"
#include <filesystem>
#include <variant>
#include <string>
namespace OpenMatrix9Gui::ThreeDm {
struct ExchangeItem {std::string name,layer,sourceUuid,sourceClass;std::array<int,3> color{180,180,180};bool visible=true,locked=false,retained=false;std::variant<TopoDS_Shape,MeshData> geometry;};
struct ExchangeModel {std::vector<ExchangeItem> items;double scaleMm=1.0,tolerance=1e-6;};
ExchangeModel readArchive(const std::filesystem::path&,double customUnitMm=0.0,bool preserve=false);
void writeArchive5(const ExchangeModel&,const std::filesystem::path&);
}
