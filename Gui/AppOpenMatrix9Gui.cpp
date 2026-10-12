#include "CoreNotes.h"
#include "Workbench.h"
#include "GumballGeometry.h"
#include "GumballController.h"
#include "GumballFrameObjects.h"
#include "GumballAssociativeScale.h"
#include "HistoryFeature.h"
#include "EditSpecialTypes.h"
#include "SurfaceHistory.h"
#include "BuilderHistory.h"
#include "CageFeature.h"
#include "CoreUnits.h"
#include "CoreObjects.h"
#include "CoreRebuild.h"
#include "CurveParameterPoint.h"
#include "CoreGeometryContract.h"
#include "CoreCPlanes.h"

#include <Base/Console.h>
#include <Base/Interpreter.h>
#include <Base/PyObjectBase.h>
#include <Base/Tools.h>

#include <Gui/Application.h>


void CreateOpenMatrix9Commands();
void AddThreeDmMethods(PyObject*);
void AddLayerDocumentMethods(PyObject*);
void AddModelingSnapMethods(PyObject*);
void AddSnapQueryMethods(PyObject*);
void AddModelingCurveMethods(PyObject*);
void AddThreeDmRegistryMethods(PyObject*);


namespace OpenMatrix9Gui
{

class Module final :
    public Py::ExtensionModule<Module>
{
public:

    Module()
        : Py::ExtensionModule<Module>(
            "OpenMatrix9Gui"
        )
    {
        initialize(
            "OpenMatrix9 GUI module"
        );
    }
};


PyObject* initModule()
{
    return
        Base::Interpreter().addModule(
            new Module
        );
}

} // namespace OpenMatrix9Gui


PyMOD_INIT_FUNC(OpenMatrix9Gui)
{
    if (!Gui::Application::Instance)
    {
        PyErr_SetString(
            PyExc_ImportError,
            "OpenMatrix9Gui cannot be loaded "
            "without FreeCAD GUI."
        );

        PyMOD_Return(nullptr);
    }

    PyObject* module =
        OpenMatrix9Gui::initModule();
    // Register the Part-derived document type only after its base module has
    // initialized. FreeCAD restores OpenMatrix9Gui::SurfaceHistory by importing
    // this module even when the workbench has not been selected in this session.
    PyObject* part = PyImport_ImportModule("Part");
    if (!part)
    {
        PyMOD_Return(nullptr);
    }
    Py_DECREF(part);
    OpenMatrix9Gui::GumballScaleCompound::init();
    OpenMatrix9Gui::SurfaceHistory::init();
    OpenMatrix9Gui::CoreNotes::registerTypes();
    AddThreeDmMethods(module);
    AddLayerDocumentMethods(module);
    AddModelingSnapMethods(module);
    AddSnapQueryMethods(module);
    AddModelingCurveMethods(module);
    AddThreeDmRegistryMethods(module);
    OpenMatrix9Gui::initializeHistoryTypes();
    OpenMatrix9Gui::initializeCurveParameterTypes();
    OpenMatrix9Gui::addCurveParameterMethods(module);
    OpenMatrix9Gui::initializeUnitTypes();
    OpenMatrix9Gui::addUnitMethods(module);
    OpenMatrix9Gui::addObjectSnapshotMethods(module);
    OpenMatrix9Gui::addGumballGeometryMethods(module);
    OpenMatrix9Gui::initializeGumballFrameTypes();
    OpenMatrix9Gui::addGumballFrameObjectMethods(module);
    OpenMatrix9Gui::GumballController::instance().addMethods(module);
    OpenMatrix9Gui::CoreRebuild::addDeliveryTestMethods(module);
    OpenMatrix9Gui::addGeometryContractMethods(module);
    OpenMatrix9Gui::addCPlaneMethods(module);
    OpenMatrix9Gui::initializeEditSpecialTypes();
    OpenMatrix9Gui::initializeBuilderHistoryTypes();
    AddBuilderHistoryMethods(module);
    OpenMatrix9Gui::initializeCageTypes();
    OpenMatrix9Gui::AddCageMethods(module);

    Base::Console().message(
        "Loading OpenMatrix9 GUI...\n"
    );

    // Register native FreeCAD workbench.
    OpenMatrix9Gui::Workbench::init();

    // Commands are described by Rust.
    CreateOpenMatrix9Commands();

    Base::Console().message(
        "OpenMatrix9 GUI loaded.\n"
    );

    PyMOD_Return(module);
}
