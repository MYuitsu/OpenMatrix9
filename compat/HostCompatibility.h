// Keep the adapter on the supported API of each matching FreeCAD SDK.
#pragma once
#if defined(OM9_STOCK_FREECAD)
#include "freecad11/StockCompatibility.h"
#else
#include <App/Document.h>
#include <Gui/Control.h>
#include <string>
inline int om9OpenTransaction(App::Document& doc, const std::string& name) {
    return doc.openTransaction(name);
}
inline void om9CommitTransaction(App::Document& doc) { doc.commitTransaction(); }
inline void om9AbortTransaction(App::Document& doc) { doc.abortTransaction(); }
inline bool om9AlterDocument(App::Document* doc) {
    return Gui::Control().isAllowedAlterDocument(doc);
}
inline bool om9AlterView(App::Document* doc) {
    return Gui::Control().isAllowedAlterView(doc);
}
inline bool om9AlterSelection(App::Document* doc) {
    return Gui::Control().isAllowedAlterSelection(doc);
}
#endif
