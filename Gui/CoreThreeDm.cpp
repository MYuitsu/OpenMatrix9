#include <Python.h>
#include "CoreThreeDm.h"
#include <App/Application.h>
#include <App/Document.h>
#include <Gui/Application.h>
#include <Gui/Document.h>
#include <Gui/Control.h>
#include <Gui/Selection/Selection.h>
#include <QFileDialog>
#include <QInputDialog>
#include <QCheckBox>
#include <QGridLayout>
#include <Base/Console.h>
#include <Base/Interpreter.h>
static bool ready(){auto* doc=App::GetApplication().getActiveDocument();auto* gui=Gui::Application::Instance->activeDocument();return doc&&gui&&!gui->getInEdit()&&Gui::Control().isAllowedAlterDocument(doc);}
bool executeThreeDm(std::size_t operation){
    if(!ready())return false;
    if(operation==3||operation==4||operation==5){
        Base::PyGILStateLocker lock;
        auto* module=PyImport_ImportModule("ThreeDmClipboard");
        const auto method=operation==3?"copy_selection":operation==5?"copy_session":"paste_selection";
        auto* result=module?PyObject_CallMethod(module,method,nullptr):nullptr;
        Py_XDECREF(module);if(!result){if(PyErr_Occurred())PyErr_Print();return false;}Py_DECREF(result);return true;
    }
    const auto name=std::string(App::GetApplication().getActiveDocument()->getName());
    const auto identity=App::GetApplication().getActiveDocument()->Uid.getValueStr();
    QString path;
    bool geometryOnly=false,modeling=false;
    if(operation==1)path=QFileDialog::getOpenFileName(nullptr,"Import Rhino 3DM",{},"Rhino (*.3dm)");
    else if(operation==6||operation==7){QFileDialog dialog(nullptr,operation==6?"Export Session Rhino 5 3DM":"Export Selected with Layers Rhino 5 3DM");dialog.setOption(QFileDialog::DontUseNativeDialog);dialog.setAcceptMode(QFileDialog::AcceptSave);dialog.setNameFilter("Rhino 5 (*.3dm)");dialog.setDefaultSuffix("3dm");if(dialog.exec()!=QDialog::Accepted)return false;const auto files=dialog.selectedFiles();if(!files.isEmpty())path=files.front();}
    else {QFileDialog dialog(nullptr,"Export Selected Rhino 5 3DM");dialog.setOption(QFileDialog::DontUseNativeDialog);dialog.setAcceptMode(QFileDialog::AcceptSave);dialog.setNameFilter("Rhino 5 (*.3dm)");dialog.setDefaultSuffix("3dm");auto* choice=new QCheckBox("Geometry only (omit native blocks, source tables, retained records, history and userdata)",&dialog);choice->setObjectName("OM9GeometryOnlyExport");if(auto* grid=qobject_cast<QGridLayout*>(dialog.layout()))grid->addWidget(choice,grid->rowCount(),0,1,grid->columnCount());else dialog.layout()->addWidget(choice);auto* working=new QCheckBox("Working geometry (continue modeling)",&dialog);working->setObjectName("OM9ModelingExport");dialog.layout()->addWidget(working);QObject::connect(working,&QCheckBox::toggled,choice,[choice](bool checked){if(checked)choice->setChecked(false);});QObject::connect(choice,&QCheckBox::toggled,working,[working](bool checked){if(checked)working->setChecked(false);});if(dialog.exec()!=QDialog::Accepted)return false;geometryOnly=choice->isChecked();modeling=working->isChecked();const auto files=dialog.selectedFiles();if(!files.isEmpty())path=files.front();}
    if(path.isEmpty())return false;
    QString importMode="geometry";
    if(operation==1){bool accepted=false;auto choice=QInputDialog::getItem(nullptr,"3DM import mode","Choose how to use the imported geometry.",{"Preserve source data","Geometry only","Working geometry (continue modeling)"},0,false,&accepted);if(!accepted)return false;importMode=choice=="Working geometry (continue modeling)"?"modeling":choice=="Geometry only"?"geometry":"preserve";}
    if(!ready()||name!=App::GetApplication().getActiveDocument()->getName()||identity!=App::GetApplication().getActiveDocument()->Uid.getValueStr())return false;
    // Native menu/toolbar activation does not hold Python's GIL. Keep it for
    // every Python C API call, including error handling and reference cleanup.
    Base::PyGILStateLocker pythonLock;
    PyObject* module=PyImport_ImportModule("ThreeDm");if(!module){PyErr_Print();return false;}
    const auto bytes=path.toUtf8();PyObject* result=nullptr;
    if(operation==1){result=PyObject_CallMethod(module,"import_file","sOds",bytes.constData(),Py_None,0.0,importMode.toUtf8().constData());
        if(!result&&PyErr_ExceptionMatches(PyExc_RuntimeError)){
            PyObject *type,*value,*trace;PyErr_Fetch(&type,&value,&trace);auto* text=PyObject_Str(value);const char* message=text?PyUnicode_AsUTF8(text):nullptr;
            const bool unitless=message&&std::string(message).find("unitless/custom units")!=std::string::npos;
            Py_XDECREF(text);
            if(unitless){Py_XDECREF(type);Py_XDECREF(value);Py_XDECREF(trace);bool accepted=false;const double scale=QInputDialog::getDouble(nullptr,"3DM units","Millimeters per file unit",1,0.000001,1000000,6,&accepted);
                if(accepted&&ready()&&name==App::GetApplication().getActiveDocument()->getName())result=PyObject_CallMethod(module,"import_file","sOds",bytes.constData(),Py_None,scale,importMode.toUtf8().constData());
            }else PyErr_Restore(type,value,trace);
        }
    }else if(operation==6||operation==7)result=PyObject_CallMethod(module,operation==6?"export_session":"export_layer_selection","s",bytes.constData());
    else result=PyObject_CallMethod(module,"export_selection","sOO",bytes.constData(),geometryOnly?Py_True:Py_False,modeling?Py_True:Py_False);
    Py_DECREF(module);if(!result){if(PyErr_Occurred())PyErr_Print();return false;}Py_DECREF(result);return true;
}
