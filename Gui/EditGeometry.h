// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "CurveGeometry.h"
#include <memory>
#include <string>
#include <set>
namespace OpenMatrix9Gui {
using EditShape=std::shared_ptr<CurvePyRef>;
struct EditInput {std::string name,signature;EditShape shape;};
using EditShapes=std::vector<EditShape>;
EditInput editInput(App::Document&,const std::string&,unsigned kind);
void verifyEditInputs(App::Document&,const std::vector<EditInput>&);
EditShapes buildEdit(const std::vector<EditInput>&,unsigned kind,std::size_t first,unsigned mode);
std::vector<EditShapes> splitEditCurves(const std::vector<EditInput>&);
std::size_t pickedEditSegment(const EditShapes&,const std::set<std::size_t>&,const std::array<double,3>&);
void verifyEditPickBoundary(const EditShapes&,const std::set<std::size_t>&,std::size_t,const std::array<double,3>&,double pixelUncertainty);
void commitEdit(App::Document&,const std::vector<EditInput>&,const EditShapes&,unsigned kind,bool remove,unsigned mode,std::size_t first);
}
