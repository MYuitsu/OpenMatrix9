#pragma once
#include "ThreeDmGeometry.h"
#include <filesystem>
#include <variant>
#include <string>
#include <set>
namespace OpenMatrix9Gui::ThreeDm {
struct ExchangeItem {std::string name,layer,sourceUuid,sourceClass,representationIssue;std::array<int,3> color{180,180,180};int wireDensity=1;double conversionSeconds=0;bool visible=true,locked=false,retained=false;std::variant<TopoDS_Shape,MeshData,ON_PointCloud> geometry;};
struct ExchangeModel {std::vector<ExchangeItem> items;double scaleMm=1.0,tolerance=1e-6;};
ExchangeModel readArchive(const std::filesystem::path&,double customUnitMm=0.0,bool preserve=false,bool definitionMembersOnly=false,const std::string& sourceRoot="");
ExchangeModel readArchiveSubset(const std::filesystem::path&,double,bool,const std::set<std::string>&);
void writeArchive5(const ExchangeModel&,const std::filesystem::path&);
bool writeModelRhino5(const ONX_Model&,const std::filesystem::path&,ON_TextLog* log=nullptr);
void transformNativeGeometry(ON_Geometry&,const ON_Xform&,ONX_Model* model=nullptr);
}
