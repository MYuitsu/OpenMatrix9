#pragma once
#include "ThreeDmArchive.h"
#include <QJsonObject>
#include <QJsonArray>
namespace OpenMatrix9Gui::ThreeDm {
QJsonObject encodeExchangeItem(const ExchangeItem&,const std::filesystem::path&,int);
QJsonArray stagePreservedArchive(const std::filesystem::path&,const std::filesystem::path&,double,bool,const ONX_Model*);
}
