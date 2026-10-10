#pragma once
#include <Python.h>
#include <QJsonObject>
#include <string>
namespace App {class Document;}
namespace OpenMatrix9Gui {
struct CurveEditRequest {QJsonObject basis;};
struct CurveEditResult {bool changed=false;};
CurveEditResult editModelingCurve(App::Document&,const std::string&,const CurveEditRequest&);
QJsonObject readModelingCurveBasis(App::Document&,const std::string&);
bool modelingCurveEditorAvailable();
bool startModelingCurveEditor();
bool modelingCurveJoinAvailable();
bool startModelingCurveJoin();
}
void AddModelingCurveMethods(PyObject*);
