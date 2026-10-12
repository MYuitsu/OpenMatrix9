// SPDX-License-Identifier: LGPL-2.1-or-later
#include <Python.h>
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/PropertyStandard.h>
#include <Base/Exception.h>
#include <memory>
#include <string>
namespace {
template<class D> constexpr bool available=requires{typename D::PropertyReplayTransaction;};
template<class D> PyObject* edit(D& doc,App::DocumentObject& child,bool accept){
 if constexpr(!available<D>){PyErr_SetString(PyExc_NotImplementedError,"Native Document property replay scope unavailable");return nullptr;}
 else{
  typename D::PropertyReplayTransaction scope(doc);bool accepted=false,rolled=false,complete=false;std::string error;
  auto reject=[&](const char* message){error=message;rolled=scope.rollback();
   if(rolled){try{child.template getPropertyByName<App::PropertyString>("Output")->setValue("");complete=scope.completeErrorState();}
    catch(const Base::Exception& e){error+="; cleanup: ";error+=e.what();}
    catch(const std::exception& e){error+="; cleanup: ";error+=e.what();}}
  };
  try{
   child.Label.setValue("replayed-label");
   auto* output=child.template getPropertyByName<App::PropertyString>("Output");
   if(!output)throw std::runtime_error("Missing native test Output property");
   output->setValue("replayed-output");
   scope.verify();
   if(accept){accepted=scope.commit();complete=accepted;}
   else{throw std::runtime_error("StaleGeometry: explicit native validation rejection");}
  }catch(const Base::Exception& e){reject(e.what());}
   catch(const std::exception& e){reject(e.what());}
   catch(...){reject("KernelFailure: unknown native replay exception");}
  return Py_BuildValue("{s:O,s:O,s:O,s:s}","accepted",accepted?Py_True:Py_False,"rollback",rolled?Py_True:Py_False,"complete",complete?Py_True:Py_False,"error",error.c_str());
 }
}
PyObject* hasScope(PyObject*,PyObject*){return PyBool_FromLong(available<App::Document>);}
PyObject* run(PyObject*,PyObject* args){const char *docName,*childName;int accept;
 if(!PyArg_ParseTuple(args,"ssp",&docName,&childName,&accept))return nullptr;
 auto* doc=App::GetApplication().getDocument(docName);auto* child=doc?doc->getObject(childName):nullptr;
 if(!child){PyErr_SetString(PyExc_ValueError,"Live test owner and child required");return nullptr;}
 try{return edit(*doc,*child,accept!=0);}
 catch(const Base::Exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
 catch(const std::exception& e){PyErr_SetString(PyExc_RuntimeError,e.what());return nullptr;}
 catch(...){PyErr_SetString(PyExc_RuntimeError,"Unknown native replay scope exception");return nullptr;}
}
PyObject* budget(PyObject*,PyObject* args){
 const char *docName,*childName;Py_ssize_t records,memory;int capture;
 if(!PyArg_ParseTuple(args,"ssnnp",&docName,&childName,&records,&memory,&capture))return nullptr;
 if(records<0||memory<0){PyErr_SetString(PyExc_ValueError,"Nonnegative replay budgets required");return nullptr;}
 auto* doc=App::GetApplication().getDocument(docName);auto* child=doc?doc->getObject(childName):nullptr;
 if(!child){PyErr_SetString(PyExc_ValueError,"Live test owner and child required");return nullptr;}
 bool entered=false,verified=false,restored=false;std::string error;
 try{
  App::Document::PropertyReplayTransaction scope(*doc,static_cast<std::size_t>(records),static_cast<std::size_t>(memory));entered=true;
  try{
   if(capture)child->Label.setValue("budget-probe-label");
   scope.verify();verified=true;
  }catch(const Base::Exception& e){error=e.what();}
   catch(const std::exception& e){error=e.what();}
  restored=scope.rollback()&&scope.completeErrorState();
 }catch(const Base::Exception& e){error=e.what();}
  catch(const std::exception& e){error=e.what();}
  catch(...){error="KernelFailure: unknown budget probe exception";}
 return Py_BuildValue("{s:O,s:O,s:O,s:s}","entered",entered?Py_True:Py_False,"verified",verified?Py_True:Py_False,"restored",restored?Py_True:Py_False,"error",error.c_str());
}
PyMethodDef methods[]={{"available",hasScope,METH_NOARGS,nullptr},{"run",run,METH_VARARGS,nullptr},{"budget",budget,METH_VARARGS,nullptr},{nullptr,nullptr,0,nullptr}};
PyModuleDef module={PyModuleDef_HEAD_INIT,"OM9ReplayScope",nullptr,-1,methods};
}
PyMODINIT_FUNC PyInit_OM9ReplayScope(){return PyModule_Create(&module);}
