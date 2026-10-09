// SPDX-License-Identifier: LGPL-2.1-or-later
#include "CoreUnits.h"
#include <App/Application.h>
#include <App/Document.h>
#include <Base/Exception.h>
#include <Base/Interpreter.h>
#include <Gui/Control.h>
#include <QApplication>
#include <QThread>
#include <limits>
#include <stdexcept>
#include <vector>

// Rust owns schema/validation/conversion. This adapter owns only FreeCAD state.
struct OM9UnitContext {
    std::uint64_t revision;
    double model_mm, page_mm, absolute_mm, relative_ratio, angular_rad;
};
extern "C" bool om9_units_validate(const OM9UnitContext*);
extern "C" bool om9_units_decode(const char*, OM9UnitContext*);
extern "C" std::size_t om9_units_encode(const OM9UnitContext*, char*, std::size_t);
extern "C" bool om9_units_change_scale(const OM9UnitContext*, const OM9UnitContext*, unsigned, double*);

namespace OpenMatrix9Gui {
PROPERTY_SOURCE(OpenMatrix9Gui::UnitSettings, App::DocumentObject)
UnitSettings::UnitSettings() {
    ADD_PROPERTY_TYPE(Context, (""), "OpenMatrix9", App::Prop_ReadOnly,
        "Versioned explicit units/tolerance context; use setUnitContext for transactional changes");
}
void initializeUnitTypes() { UnitSettings::init(); }
namespace {
UnitSettings* settings(const App::Document& doc) {
    UnitSettings* result=nullptr;
    for (auto* object:doc.getObjects()) if (auto* value=dynamic_cast<UnitSettings*>(object)) {
        if (result) throw std::runtime_error("Duplicate OpenMatrix9 unit contexts");
        result=value;
    }
    return result;
}
OM9UnitContext decode(const std::string& raw) {
    OM9UnitContext result{};
    if (!om9_units_decode(raw.c_str(), &result)) throw std::runtime_error("Invalid or unsupported document unit context");
    return result;
}
App::Document& document(const char* name) {
    if (!qApp || QThread::currentThread()!=qApp->thread()) throw std::runtime_error("Document units require the GUI thread");
    auto* doc=App::GetApplication().getDocument(name);
    if (!doc || doc->testStatus(App::Document::Restoring)) throw std::runtime_error("Document is missing or restoring");
    return *doc;
}
template<class F> PyObject* api(F&& run) {
    try { return run(); }
    catch(const Base::Exception& error) { PyErr_SetString(PyExc_RuntimeError,error.what()); }
    catch(const std::exception& error) { PyErr_SetString(PyExc_ValueError,error.what()); }
    catch(...) { PyErr_SetString(PyExc_RuntimeError,"Native unit context operation failed"); }
    return nullptr;
}
PyObject* asDict(const std::string& raw) {
    if (raw.empty()) Py_RETURN_NONE;
    auto context=decode(raw);
    return Py_BuildValue("{s:i,s:K,s:d,s:d,s:d,s:d,s:d,s:s}","schema_version",1,
        "revision",static_cast<unsigned long long>(context.revision),"model_mm",context.model_mm,
        "page_mm",context.page_mm,"absolute_mm",context.absolute_mm,"relative_ratio",context.relative_ratio,
        "angular_rad",context.angular_rad,"snapshot",raw.c_str());
}
PyObject* getAPI(PyObject*,PyObject* args) {
    const char* name=nullptr;
    if (!PyArg_ParseTuple(args,"s",&name)) return nullptr;
    return api([&] {return asDict(unitContextSnapshot(document(name)));});
}
PyObject* setAPI(PyObject*,PyObject* args) {
    const char* name=nullptr; OM9UnitContext next{}; unsigned mode=0;
    if (!PyArg_ParseTuple(args,"sddddd|I",&name,&next.model_mm,&next.page_mm,&next.absolute_mm,
        &next.relative_ratio,&next.angular_rad,&mode)) return nullptr;
    return api([&]() -> PyObject* {
        auto& doc=document(name);
        if (!Gui::Control().isAllowedAlterDocument(&doc) || doc.hasPendingTransaction())
            throw std::runtime_error("Unit changes require an editable document without an active transaction");
        const auto previous=unitContextSnapshot(doc);
        next.revision=previous.empty()?1:decode(previous).revision;
        if (!previous.empty()) {
            if (next.revision==std::numeric_limits<std::uint64_t>::max()) throw std::runtime_error("Unit revision exhausted");
            ++next.revision;
        }
        if (!om9_units_validate(&next) || mode>1) throw std::runtime_error("Invalid unit/tolerance context or scale mode");
        double scale=1;
        if (mode==1) {
            if (previous.empty()) throw std::runtime_error("Declare the source units before preserving declared numbers");
            auto old=decode(previous);
            if (!om9_units_change_scale(&old,&next,mode,&scale)) throw std::runtime_error("Invalid unit change plan");
            // Unknown dimensions cannot safely be scaled by property-name guesses.
            if (scale!=1) for (auto* object:doc.getObjects()) if (!dynamic_cast<UnitSettings*>(object))
                throw std::runtime_error(std::string("Unit scaling has no dimensional adapter for object: ")+object->getNameInDocument());
        }
        const auto count=om9_units_encode(&next,nullptr,0);
        if (!count || count>1024) throw std::runtime_error("Invalid encoded unit context");
        std::vector<char> buffer(count);
        if (om9_units_encode(&next,buffer.data(),buffer.size())!=count) throw std::runtime_error("Cannot encode unit context");
        const std::string raw(buffer.data());
        decode(raw); // Serialization must be readable before any document mutation.
        doc.openTransaction("OpenMatrix9 document units");
        try {
            auto* target=settings(doc);
            if (!target) target=dynamic_cast<UnitSettings*>(doc.addObject("OpenMatrix9Gui::UnitSettings","OM9Units"));
            if (!target) throw std::runtime_error("Cannot create document unit context");
            target->Context.setValue(raw);
            doc.commitTransaction();
        } catch(...) { doc.abortTransaction(); throw; }
        return asDict(raw);
    });
}
}
std::string unitContextSnapshot(const App::Document& doc) {
    auto* context=settings(doc);
    if (!context) return {};
    std::string raw=context->Context.getValue();
    decode(raw); // Malformed/future data must never fall back silently to mm.
    return raw;
}
double unitInputScale(const App::Document& doc) {
    const auto raw=unitContextSnapshot(doc);
    return raw.empty()?1.0:decode(raw).model_mm;
}
void addUnitContextProvenance(App::DocumentObject& object,const std::string& raw) {
    if (raw.empty()) return;
    auto* property=dynamic_cast<App::PropertyString*>(object.getPropertyByName("OM9UnitContext"));
    if (!property) property=dynamic_cast<App::PropertyString*>(object.addDynamicProperty("App::PropertyString","OM9UnitContext","OpenMatrix9","Input units/tolerances snapshot; solver thresholds remain per operation",App::Prop_ReadOnly));
    if (!property) throw std::runtime_error("Cannot store unit context provenance");
    property->setValue(raw);
}
void addUnitMethods(PyObject* module) {
    static PyMethodDef methods[]={
        {"unitContext",getAPI,METH_VARARGS,"Read explicit units/tolerances by document name; None means legacy per-command units."},
        {"setUnitContext",setAPI,METH_VARARGS,"Set explicit model_mm,page_mm,absolute_mm,relative_ratio,angular_rad, optional mode (0 physical,1 declared). Atomic; unknown object scaling rejects."},
        {nullptr,nullptr,0,nullptr}};
    if (PyModule_AddFunctions(module,methods)<0) throw Base::RuntimeError("Cannot register unit context API");
}
}
