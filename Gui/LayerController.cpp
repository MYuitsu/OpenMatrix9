// SPDX-License-Identifier: LGPL-2.1-or-later
#include "LayerController.h"
#include "LayerDocumentAdapter.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <App/PropertyStandard.h>
#include <Base/Console.h>
#include <Gui/Application.h>
#include <Gui/Control.h>
#include <Gui/Document.h>
#include <Gui/Selection/Selection.h>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <set>
#include <stdexcept>

namespace OpenMatrix9Gui {
namespace {
std::vector<std::string> selected(App::Document& doc) {
    std::set<std::string> unique;
    for(const auto& item:Gui::Selection().getSelection(doc.getName())) {
        auto* object=item.pObject;
        if(!object || object->getDocument()!=&doc || isLayerStorageObject(object))continue;
        auto* property=object->getPropertyByName("OM9LayerObjectId");
        auto* id=dynamic_cast<App::PropertyString*>(property);
        if(property && !id)throw std::runtime_error("Invalid native layer selection identity");
        unique.emplace(id?id->getValue():object->getNameInDocument());
    }
    return {unique.begin(),unique.end()};
}
bool editable(App::Document& doc) {
    auto* gui=Gui::Application::Instance->getDocument(&doc);
    return gui && !gui->getInEdit() && om9AlterDocument(&doc)
        && !doc.hasPendingTransaction() && om9BookedTransaction(doc)==0
        && om9GlobalTransaction()==0 && !doc.isPerformingTransaction();
}
QString quoted(const QString& path) {
    const auto json=QString::fromUtf8(QJsonDocument(QJsonArray{path}).toJson(QJsonDocument::Compact));
    return json.mid(1,json.size()-2);
}
// Own only a layer-only JSON frame and value identity. No retained native pointers.
class PanelCache {
public:
    PanelCache() {
        auto& app=App::GetApplication();
        created=app.signalNewObject.connect([this](const App::DocumentObject&){dirty=true;});
        removed=app.signalDeletedObject.connect([this](const App::DocumentObject&){dirty=true;});
        changed=app.signalChangedObject.connect([this](const App::DocumentObject&,const App::Property&){dirty=true;});
        deleted=app.signalDeleteDocument.connect([this](const App::Document&){dirty=true;key.clear();frame={};});
        undo=app.signalUndoDocument.connect([this](const App::Document&){dirty=true;});
        redo=app.signalRedoDocument.connect([this](const App::Document&){dirty=true;});
    }
    QJsonObject get(App::Document& doc) {
        const auto current=QString::fromUtf8(doc.getName())+":"+QString::fromStdString(doc.Uid.getValueStr());
        if(dirty || key!=current) {
            const auto json=layerDocumentPanel(doc);
            QJsonParseError error;
            const auto parsed=QJsonDocument::fromJson(QByteArray(json.data(),int(json.size())),&error);
            if(error.error!=QJsonParseError::NoError || !parsed.isObject())throw std::runtime_error("Invalid Rust panel frame");
            frame=parsed.object();key=current;dirty=false;
        }
        return frame;
    }
private:
    QString key;QJsonObject frame;bool dirty=true;
    fastsignals::scoped_connection created,removed,changed,deleted,undo,redo;
};
PanelCache& cache(){static PanelCache value;return value;}
}
QString LayerController::panel() {
    QJsonObject frame;
    if(auto* doc=App::GetApplication().getActiveDocument()) {
        try {
            frame=cache().get(*doc);
            frame.insert("editable",editable(*doc));
            frame.insert("selection_count",int(selected(*doc).size()));
        } catch(const std::exception& error) {
            frame.insert("editable",false);frame.insert("error",QString::fromUtf8(error.what()));
        }
    }
    return QString::fromUtf8(QJsonDocument(frame).toJson(QJsonDocument::Compact));
}
bool LayerController::submit(const QString& text) {
    const auto input=text.trimmed();
    if(input.section(' ',0,0).compare("Layer",Qt::CaseInsensitive)!=0)return false;
    try {
        auto* doc=App::GetApplication().getActiveDocument();
        if(!doc)throw std::runtime_error("No active layer document");
        layerDocumentText(*doc,input.toStdString(),selected(*doc));
    } catch(const std::exception& error) {Base::Console().error("OM9 Layer: %s\n",error.what());}
    return true;
}
void LayerController::setCurrent(const QString& path){submit("Layer Current "+quoted(path));}
void LayerController::assignSelection(const QString& path){submit("Layer Assign "+quoted(path));}
void LayerController::setLocked(const QString& path,bool value){submit("Layer Lock "+quoted(path)+(value?" On":" Off"));}
void LayerController::setVisible(const QString& path,bool value){submit("Layer Visible "+quoted(path)+(value?" On":" Off"));}
}
