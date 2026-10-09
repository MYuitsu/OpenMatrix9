// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "EditGeometry.h"
namespace OpenMatrix9Gui {
void initializeEditSpecialTypes();
bool specialExplodeSource(PyObject*);
EditShapes specialExplodeComponents(PyObject*,bool allowBlocks);
std::string specialExplodeSignature(PyObject*);
EditShape specialExplodePreview(const EditShapes&);
bool isExplodedText(PyObject*);
void transformExplodedText(PyObject*,PyObject* matrix);
CurvePyRef createExplodedText(PyObject* document,PyObject* payload);
void applyExplodeMetadata(PyObject* object,const EditShape& component);
}
