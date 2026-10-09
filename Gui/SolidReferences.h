// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "EditGeometry.h"
#include <array>
#include <map>
#include <optional>
namespace OpenMatrix9Gui {
using SolidPoint=std::array<double,3>;
struct SolidCurveReference {EditInput input; EditShape edge; std::string sub;};
SolidCurveReference solidCurveReference(App::Document&,const std::string&);
std::pair<SolidPoint,SolidPoint> solidOnCurve(const SolidCurveReference&,const SolidPoint&,std::optional<double> fraction={});
std::vector<SolidPoint> solidCurvePlanePoints(const SolidCurveReference&);
double solidCurveFraction(const SolidCurveReference&,const SolidPoint&);
std::vector<SolidPoint> solidSelectedFitPoints(App::Document&,bool surfacePoles=false);
std::pair<SolidPoint,double> solidTangentSphere(const std::map<std::size_t,SolidCurveReference>&,const double* constraints,std::size_t count,const double* frame,double radius,int solution=-1);
std::pair<SolidPoint,double> nativeTangentCircle(const std::map<std::size_t,SolidCurveReference>&,const double* constraints,std::size_t count,const double* frame,double radius,int solution=-1,bool fromFirst=false,double* contacts=nullptr);
void verifySolidReferences(App::Document&,const std::map<std::size_t,SolidCurveReference>&,const std::optional<SolidCurveReference>&);
}
