#include "ModelingCurveEditorDialog.h"
#include "ModelingCurveEditor.h"
#include "CurveBasisTransfer.h"
#include <App/Application.h>
#include <App/Document.h>
#include <App/DocumentObject.h>
#include <Gui/Application.h>
#include <Gui/MainWindow.h>
#include <Gui/WorkbenchManager.h>
#include <QDialog>
#include <QFormLayout>
#include <QTableWidget>
#include <QSpinBox>
#include <QDoubleSpinBox>
#include <QCheckBox>
#include <QLabel>
#include <QDialogButtonBox>
#include <QTimer>
#include <QPointer>
#include <Standard_Failure.hxx>
#include <fastsignals/signal.h>
namespace OpenMatrix9Gui {
namespace {
class Editor final:public QDialog {
    std::string document,name;QByteArray signature;std::uint64_t handle=0;
    QSpinBox* degree;QCheckBox* periodic;QTableWidget *poles,*knots;QLabel* error;QDoubleSpinBox *first,*last;
    bool committing=false;std::vector<fastsignals::scoped_connection> connections;
    Om9WitnessInput witness(App::Document& doc,const QByteArray& sig)const {auto* object=doc.getObject(name.c_str());return {std::uint64_t(reinterpret_cast<std::uintptr_t>(&doc)),object?std::uint64_t(object->getID())+1:0,0,reinterpret_cast<const std::uint8_t*>(sig.constData()),std::size_t(sig.size())};}
    QDoubleSpinBox* number(double value,QWidget* parent){auto* n=new QDoubleSpinBox(parent);n->setDecimals(15);n->setRange(-1e9,1e9);n->setValue(value);return n;}
    void invalidated(){if(!committing){om9_phase2_session_drop(handle);handle=0;reject();}}
    void acceptDraft() {
        try {auto* doc=App::GetApplication().getDocument(document.c_str());if(!doc||doc!=App::GetApplication().getActiveDocument()||Gui::WorkbenchManager::instance()->activeName()!="OpenMatrix9Workbench"){invalidated();return;}
            const auto current=readModelingCurveBasis(*doc,name);const auto sig=current.value("signature").toString().toUtf8();auto w=witness(*doc,sig);phase2Require(om9_phase2_session_check(handle,&w),handle);
            CurveBasisTransfer model(handle);auto value=model.json();value["degree"]=degree->value();value["periodic"]=periodic->isChecked();QJsonArray p,weights,k,m;
            for(int row=0;row<poles->rowCount();++row){QJsonArray xyz;for(int axis=0;axis<3;++axis)xyz.append(static_cast<QDoubleSpinBox*>(poles->cellWidget(row,axis))->value());p.append(xyz);weights.append(static_cast<QDoubleSpinBox*>(poles->cellWidget(row,3))->value());}
            for(int row=0;row<knots->rowCount();++row){k.append(static_cast<QDoubleSpinBox*>(knots->cellWidget(row,0))->value());m.append(static_cast<QSpinBox*>(knots->cellWidget(row,1))->value());}
            value["poles"]=p;value["weights"]=weights;value["knots"]=k;value["multiplicities"]=m;value["first"]=first->value();value["last"]=last->value();
            CurveBasisTransfer decoded(value);auto raw=decoded.input();phase2Require(om9_phase2_session_replace(handle,&raw),handle);
            value=CurveBasisTransfer(handle).json();value["signature"]=QString::fromUtf8(signature);committing=true;
            try{editModelingCurve(*doc,name,{value});}catch(...){committing=false;throw;}committing=false;accept();
        }catch(const Standard_Failure& e){error->setText(QString::fromUtf8(e.GetMessageString()));}
        catch(const std::exception& e){error->setText(QString::fromUtf8(e.what()));}
    }
public:
    void done(int result) override {
        om9_phase2_session_drop(handle);
        handle=0;
        setProperty("om9RustSession",qulonglong(0));
        setObjectName(QString());
        QDialog::done(result);
    }
    Editor(App::Document& doc,const std::string& object):QDialog(Gui::getMainWindow()),document(doc.getName()),name(object) {
        const auto original=readModelingCurveBasis(doc,name);signature=original.value("signature").toString().toUtf8();auto w=witness(doc,signature);CurveBasisTransfer decoded(original);auto raw=decoded.input();handle=om9_phase2_session_create(&w,&raw);phase2Require(handle!=0);
        // Subsequent widget/model reads come from the independent Rust-owned draft.
        const auto model=CurveBasisTransfer(handle).json();setObjectName("OM9CurveCVDialog");setProperty("om9RustSession",qulonglong(handle));setWindowTitle("Curve CV Editor");setAttribute(Qt::WA_DeleteOnClose);
        auto* layout=new QFormLayout(this);degree=new QSpinBox(this);degree->setObjectName("OM9CurveCVDegree");degree->setRange(1,25);degree->setValue(model.value("degree").toInt());layout->addRow("Degree",degree);
        periodic=new QCheckBox(this);periodic->setObjectName("OM9CurveCVPeriodic");periodic->setChecked(model.value("periodic").toBool());layout->addRow("Periodic",periodic);
        first=number(model.value("first").toDouble(),this);first->setObjectName("OM9CurveCVFirst");layout->addRow("First parameter",first);
        last=number(model.value("last").toDouble(),this);last->setObjectName("OM9CurveCVLast");layout->addRow("Last parameter",last);
        const auto points=model.value("poles").toArray(),weights=model.value("weights").toArray();poles=new QTableWidget(points.size(),4,this);poles->setObjectName("OM9CurveCVPoles");poles->setHorizontalHeaderLabels({"X","Y","Z","Weight"});
        for(int row=0;row<points.size();++row){const auto xyz=points[row].toArray();for(int axis=0;axis<3;++axis)poles->setCellWidget(row,axis,number(xyz[axis].toDouble(),poles));poles->setCellWidget(row,3,number(weights[row].toDouble(),poles));}layout->addRow(poles);
        const auto values=model.value("knots").toArray(),mults=model.value("multiplicities").toArray();knots=new QTableWidget(values.size(),2,this);knots->setObjectName("OM9CurveCVKnots");knots->setHorizontalHeaderLabels({"Knot","Multiplicity"});
        for(int row=0;row<values.size();++row){knots->setCellWidget(row,0,number(values[row].toDouble(),knots));auto* m=new QSpinBox(knots);m->setRange(1,26);m->setValue(mults[row].toInt());knots->setCellWidget(row,1,m);}layout->addRow(knots);
        error=new QLabel(this);error->setObjectName("OM9CurveCVError");error->setWordWrap(true);layout->addRow(error);auto* buttons=new QDialogButtonBox(QDialogButtonBox::Ok|QDialogButtonBox::Cancel,this);layout->addRow(buttons);connect(buttons,&QDialogButtonBox::accepted,this,[this]{acceptDraft();});connect(buttons,&QDialogButtonBox::rejected,this,&QDialog::reject);
        auto& app=App::GetApplication();connections.emplace_back(app.signalUndoDocument.connect([this](const App::Document& d){if(document==d.getName())invalidated();}));connections.emplace_back(app.signalRedoDocument.connect([this](const App::Document& d){if(document==d.getName())invalidated();}));
        connections.emplace_back(app.signalActiveDocument.connect([this](const App::Document& d){if(document!=d.getName())invalidated();}));
        connections.emplace_back(app.signalDeleteDocument.connect([this](const App::Document& d){if(document==d.getName())invalidated();}));
        connections.emplace_back(app.signalDeletedObject.connect([this](const App::DocumentObject& o){auto* d=o.getDocument();const auto* objectName=o.getNameInDocument();if(d&&objectName&&document==d->getName()&&name==objectName)invalidated();}));
        connections.emplace_back(Gui::Application::Instance->signalActivateWorkbench.connect([this](const char* workbench){if(!workbench||std::string(workbench)!="OpenMatrix9Workbench")invalidated();}));
        auto* timer=new QTimer(this);timer->setInterval(100);connect(timer,&QTimer::timeout,this,[this]{if(committing)return;try{auto* d=App::GetApplication().getActiveDocument();if(!d||document!=d->getName()||Gui::WorkbenchManager::instance()->activeName()!="OpenMatrix9Workbench"){invalidated();return;}const auto current=readModelingCurveBasis(*d,name);const auto sig=current.value("signature").toString().toUtf8();const auto w=witness(*d,sig);if(!om9_phase2_session_check(handle,&w))invalidated();}catch(...){invalidated();}});timer->start();resize(600,640);
    }
    ~Editor()override{om9_phase2_session_drop(handle);}
};
}
void showModelingCurveEditor(App::Document& doc,const std::string& name){
    static QPointer<Editor> current;
    if(current)current->reject();
    current=new Editor(doc,name);
    current->show();
}
}
