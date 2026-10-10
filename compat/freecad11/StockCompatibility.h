// Native API compatibility for the official FreeCAD 1.1.4 host only.
#pragma once
#include <FCConfig.h>
// OM9 uses UTF-8 string literals throughout its Qt adapter. This is a source
// conversion option, not a Qt binary ABI change.
#undef QT_NO_CAST_FROM_ASCII
#include <windows.h>
// Legacy Win32 distance modifiers collide with CAD variables named near/far.
#undef near
#undef far
#include <fastsignals/signal.h>
#include <App/Application.h>
#include <App/Document.h>
#include <Gui/Control.h>
#include <QFileInfo>
#include <string>
#include <Standard_Handle.hxx>
namespace occ { template<class T> using handle=opencascade::handle<T>; }

inline int om9GlobalTransaction() {
    int id=0;App::GetApplication().getActiveTransaction(&id);return id;
}
inline int om9BookedTransaction(const App::Document& doc) {
    if(doc.hasPendingTransaction())return doc.getTransactionID(true);
    return App::GetApplication().getActiveDocument()==&doc?om9GlobalTransaction():0;
}
inline int om9OpenTransaction(App::Document& doc,const std::string& name) {
    doc.openTransaction(name.c_str());
    const int id=om9GlobalTransaction();
    if(id==0)return 0;
    return id;
}
inline bool om9ClosingTransaction(const App::Document& doc) {
    // FreeCAD 1.1 clears the global ID before emitting its before-close signal.
    // Scope the older host's callback to its active modeling document.
    return doc.hasPendingTransaction() && App::GetApplication().getActiveDocument()==&doc;
}
inline void om9CommitTransaction(App::Document& doc) {
    if(doc.hasPendingTransaction())doc.commitTransaction();
    else if(App::GetApplication().getActiveDocument()==&doc)App::GetApplication().closeActiveTransaction(false,om9GlobalTransaction());
}
inline void om9AbortTransaction(App::Document& doc) {
    if(doc.hasPendingTransaction())doc.abortTransaction();
    else if(App::GetApplication().getActiveDocument()==&doc)App::GetApplication().closeActiveTransaction(true,om9GlobalTransaction());
}
inline bool om9ReadOnlyFile(const App::Document& doc) {
    const QFileInfo file(QString::fromUtf8(doc.FileName.getValue()));
    return file.exists()&&!file.isWritable();
}
inline bool om9AlterDocument(const App::Document*) {return Gui::Control().isAllowedAlterDocument();}
inline bool om9AlterView(const App::Document*) {return Gui::Control().isAllowedAlterView();}
inline bool om9AlterSelection(const App::Document*) {return Gui::Control().isAllowedAlterSelection();}
