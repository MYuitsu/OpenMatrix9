// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "CurveGeometry.h"
#include <memory>
#include <string>
#include <set>
namespace OpenMatrix9Gui {
using EditShape=std::shared_ptr<CurvePyRef>;
using EditShapes=std::vector<EditShape>;
struct EditInput {std::string name,signature;EditShape shape;bool mesh=false;EditShapes components;};
struct EditProjection {
    std::array<double,3> eye{0,0,0},right{1,0,0},up{0,1,0},forward{0,0,-1};
    bool perspective=false;
};
EditInput editInput(App::Document&,const std::string&,unsigned kind,bool allowBlocks=false);
// Native BRep snapshot in world placement, without an Edit command's type policy.
EditInput nativeShapeInput(App::Document&,const std::string&);
void verifyEditInputs(App::Document&,const std::vector<EditInput>&);
EditShapes buildEdit(const std::vector<EditInput>&,unsigned kind,std::size_t first,unsigned mode,double tolerance=1e-7);
EditShape joinEditCurves(PyObject* edges,double tolerance);
std::vector<EditShapes> splitEditCurves(const std::vector<EditInput>&,bool extendLines=false);
std::vector<EditShapes> splitProjectedEditCurves(const std::vector<EditInput>&,const EditProjection&,bool extendLines);
EditShapes extendedEditLines(const std::vector<EditInput>&);
bool isEditMesh(PyObject*);
void validateEditMesh(PyObject*,bool closed);
std::string editMeshSignature(PyObject*);
EditShape meshEditSolid(PyObject*);
EditShape meshEditResult(PyObject*);
std::size_t pickedEditSegment(const EditShapes&,const std::set<std::size_t>&,const std::array<double,3>&);
void verifyEditPickBoundary(const EditShapes&,const std::set<std::size_t>&,std::size_t,const std::array<double,3>&,double pixelUncertainty);
void commitEdit(App::Document&,const std::vector<EditInput>&,const EditShapes&,unsigned kind,bool remove,unsigned mode,std::size_t first,const EditProjection* projection=nullptr);
}
