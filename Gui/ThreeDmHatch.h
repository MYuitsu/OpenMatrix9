#pragma once
#include "ThreeDmGeometry.h"
#include <QJsonObject>
namespace OpenMatrix9Gui::ThreeDm {
void validateHatch(const ON_Hatch&);
void validateHatchRhino5Data(const ON_Hatch&);
QJsonArray hatchLoopInventory(const ON_Hatch&);
QJsonObject hatchLoopFields(const ON_Hatch&);
void applyHatchLoopFields(ON_Hatch&,const QJsonValue&);
void transformHatchNative(ON_Hatch&,const ON_Xform&,ONX_Model*);
QJsonObject hatchFacts(const ON_Hatch&);
QJsonObject hatchPatternFacts(const ON_HatchPattern&);
QJsonObject hatchPatternRecord(const ON_HatchPattern&);
QJsonObject hatchCurrentFields(const ON_Hatch&,const ONX_Model&);
QJsonArray hatchPatternChoices(const ONX_Model&);
void applyHatchFields(ON_Geometry&,const QJsonValue&,ONX_Model&);
}
