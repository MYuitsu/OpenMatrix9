// SPDX-License-Identifier: LGPL-2.1-or-later
#include "EditSpecialTypes.h"
namespace OpenMatrix9Gui {
void initializeEditSpecialTypes(){}
bool specialExplodeSource(PyObject*){return false;}
EditShapes specialExplodeComponents(PyObject*,bool){phase3Require(1);return {};}
std::string specialExplodeSignature(PyObject*){phase3Require(1);return {};}
EditShape specialExplodePreview(const EditShapes&){phase3Require(1);return {};}
bool isExplodedText(PyObject*){return false;}
void transformExplodedText(PyObject*,PyObject*){phase3Require(1);}
CurvePyRef createExplodedText(PyObject*,PyObject*){phase3Require(1);return CurvePyRef(Py_NewRef(Py_None));}
void applyExplodeMetadata(PyObject*,const EditShape&){}
}
