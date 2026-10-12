// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "SolidReferences.h"
#include <QJsonObject>
#include <QJsonArray>
#include <QString>
#include <memory>
#include <cstdint>
namespace OpenMatrix9Gui {
// GUI-thread capture only; immutable values contain no native/Python owner.
struct CurveNativeSource {
    ObjectSnapshot guard;
    QJsonObject snapshot;
    QByteArray contentDigest;
    std::string frame;
};
struct CurveNativeAdmission {
    std::uint64_t generation,revision,context;
    std::string command,units;
};
struct CurveNativeBinding {
    std::shared_ptr<const CurveNativeSource> source;
    std::string role;
    std::size_t index=0,begin=0,end=0;
};
CurveNativeAdmission curveNativeAdmission(App::Document&,const char* command);
std::shared_ptr<const CurveNativeSource> captureCurveNativeSource(App::Document&,const std::string& name,
    const std::string& sub,const std::vector<SolidPoint>& selected,const std::string& command);
void verifyCurveNativeSource(App::Document&,const CurveNativeSource&,const std::string& command);
QJsonObject curveNativeSourceReference(const CurveNativeBinding&);
unsigned acceptCurveNativeInput(App::Document&,const CurveNativeAdmission&,const char* action,unsigned kind,
    const SolidPoint&,const SolidPoint&,const std::vector<SolidPoint>&,
    const QString& text,const std::vector<CurveNativeBinding>&,std::optional<double> sourceCurveFraction={});
unsigned acceptCircleNativeSolution(App::Document&,const CurveNativeAdmission&,const std::array<double,7>&,const QJsonArray&);
}
