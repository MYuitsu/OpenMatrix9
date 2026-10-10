#pragma once
#include "ThreeDmArchive.h"
#include <QJsonObject>
#include <QJsonArray>
namespace OpenMatrix9Gui::ThreeDm {
QJsonObject encodeExchangeItem(const ExchangeItem&,const std::filesystem::path&,int);
// Versioned strict JSON is owned/validated by Rust; Qt carries it unchanged.
QJsonObject encodeExchangeLayers(const ExchangeModel&,const std::string& document,std::uint64_t generation=0);
void decodeExchangeLayers(ExchangeModel&,const QJsonObject&);
QJsonArray stagePreservedArchive(const std::filesystem::path&,const std::filesystem::path&,double,bool,const ONX_Model*,bool working=false);
}
