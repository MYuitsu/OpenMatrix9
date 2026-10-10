#pragma once
#include "ThreeDmGeometry.h"
#include "LayerExchangeAdapter.h"
#include <filesystem>
#include <variant>
#include <string>
#include <set>
namespace OpenMatrix9Gui::ThreeDm {
// transferObjectId is an untrusted native fact. Only an exact extended payload
// binding may consume it; regular/native-only imports retain physical UUID IDs.
struct ExchangeItem {std::string name,layer,sourceUuid,sourceClass,sourceRootUuid,representationIssue,transferObjectId;std::array<int,3> color{180,180,180};int wireDensity=1;double conversionSeconds=0;bool visible=true,locked=false,retained=false;std::variant<TopoDS_Shape,MeshData,ON_PointCloud> geometry;std::optional<NativeObjectLayerRow> ownState;};
struct PendingBrep {size_t item;BrepAssembly assembly;};
struct ExchangeModel {std::vector<ExchangeItem> items;double scaleMm=1.0,tolerance=1e-6;std::vector<PendingBrep> pendingBreps;NativeLayerTable layerTable;std::optional<NativeLayerContext> layerContext;};
// Caller has verified the extended metadata against these exact archive bytes.
// Only detached metadata changes; native-only imports must never call this.
void applyExtendedLayerOverlay(ExchangeModel&,std::uint64_t sourceSnapshot);
ExchangeModel readWorkingArchiveDeferred(const std::filesystem::path&,double customUnitMm=0.0);
void finishDeferredBrep(ExchangeModel&,size_t pendingIndex);
ExchangeModel readArchive(const std::filesystem::path&,double customUnitMm=0.0,bool preserve=false,bool definitionMembersOnly=false,const std::string& sourceRoot="",bool modelSpaceOnly=false);
ExchangeModel readArchiveSubset(const std::filesystem::path&,double,bool,const std::set<std::string>&,bool preserve=true);
void writeArchive5(const ExchangeModel&,const std::filesystem::path&);
bool writeModelRhino5(const ONX_Model&,const std::filesystem::path&,ON_TextLog* log=nullptr);
void transformNativeGeometry(ON_Geometry&,const ON_Xform&,ONX_Model* model=nullptr);
}
