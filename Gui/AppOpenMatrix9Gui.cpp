#include "Workbench.h"

#include <Base/Console.h>
#include <Base/Interpreter.h>
#include <Base/PyObjectBase.h>
#include <Base/Tools.h>

#include <Gui/Application.h>


void CreateOpenMatrix9Commands();
void AddThreeDmMethods(PyObject*);
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
    AddThreeDmMethods(module);
    AddThreeDmRegistryMethods(module);

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
