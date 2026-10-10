#include <QtTest/QtTest>
#include <QMainWindow>
#include <QToolButton>
#include <QAction>
#include <QLabel>
#include <QComboBox>
#include <QFrame>
#include <QSettings>
#include <QMenu>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include "MatrixSidebar.h"
#include "RustBridge.h"
#include "LayerRustAbi.h"
using namespace OpenMatrix9Gui;
class MatrixSidebarTest : public QObject {
    Q_OBJECT
private slots:
    void liveLayerRowsUseRustPaletteAndDispatchFullPaths() {
        const QByteArray id="sidebar-test";
        std::uint64_t snapshot=0;
        QCOMPARE(om9_layer_document_default({reinterpret_cast<const unsigned char*>(id.data()),std::size_t(id.size())},&snapshot),0U);
        std::size_t size=0;QCOMPARE(om9_layer_snapshot_json(snapshot,nullptr,0,&size),15U);
        QByteArray data(int(size),Qt::Uninitialized);
        QCOMPARE(om9_layer_snapshot_json(snapshot,reinterpret_cast<unsigned char*>(data.data()),size,&size),0U);
        QCOMPARE(om9_layer_snapshot_free(snapshot),0U);
        auto envelope=QJsonDocument::fromJson(data).object();auto state=envelope.value("snapshot").toObject();
        auto layers=state.value("layers").toArray();auto first=layers.at(0).toObject();first.insert("rgb",QJsonArray{201,202,203});layers[0]=first;
        auto custom=first;custom.insert("source_id","custom");custom.insert("name","Custom");custom.insert("path_components",QJsonArray{"Custom"});layers.append(custom);
        auto other=custom;other.insert("source_id","other");other.insert("name","Other");other.insert("path_components",QJsonArray{"Other"});layers.append(other);
        state.insert("layers",layers);envelope.insert("snapshot",state);data=QJsonDocument(envelope).toJson(QJsonDocument::Compact);
        QCOMPARE(om9_layer_snapshot_from_json({reinterpret_cast<const unsigned char*>(data.data()),std::size_t(data.size())},&snapshot),0U);
        QCOMPARE(om9_layer_document_panel(snapshot,nullptr,0,&size),15U);data.resize(int(size));
        QCOMPARE(om9_layer_document_panel(snapshot,reinterpret_cast<unsigned char*>(data.data()),size,&size),0U);
        QCOMPARE(om9_layer_snapshot_free(snapshot),0U);
        auto frame=QJsonDocument::fromJson(data).object();frame.insert("editable",true);frame.insert("selection_count",2);
        QString sent;
        QMainWindow window;MatrixSidebar sidebar(&window,QStringLiteral(OM9_RESOURCE_DIR),
            {{},{},{},[&](const QString& text){sent=text;return true;},[&]{return QString::fromUtf8(QJsonDocument(frame).toJson(QJsonDocument::Compact));}});
        auto* swatch=sidebar.findChild<QLabel*>("OM9LayerSwatch0");QVERIFY(swatch);QCOMPARE(swatch->property("om9LayerRGB").toString(),QString("#c9cacb"));
        auto* lock=sidebar.findChild<QToolButton*>("OM9LayerLock0");QVERIFY(lock->isEnabled());lock->click();QCOMPARE(sent,QString("Layer Lock \"Metal 01\" Toggle"));
        auto* assign=sidebar.findChild<QToolButton*>("OM9LayerArrow4");QVERIFY(assign->isEnabled());assign->click();QCOMPARE(sent,QString("Layer Assign \"Gem 01\""));
        auto* combo=sidebar.findChild<QComboBox*>("OM9LayerAll");QCOMPARE(combo->count(),34);
        combo->setCurrentIndex(combo->findData("Other"));sidebar.findChild<QToolButton*>("OM9LayerAllCurrent")->click();QCOMPARE(sent,QString("Layer Current \"Other\""));
        frame.insert("editable",false);sidebar.refreshAvailability();QVERIFY(!lock->isEnabled());QVERIFY(!assign->isEnabled());
        auto* label=sidebar.findChild<QLabel*>("OM9LayerName0");QVERIFY(!label->isEnabled());
        const auto before=sent;lock->click();QCOMPARE(sent,before);
    }
    void checkedHostStatesAndOsnapDispatchAreInjected() {
        QMainWindow window;QString sent;
        MatrixSidebar sidebar(&window,QStringLiteral(OM9_RESOURCE_DIR),
            {[](std::size_t){return true;},{},[](std::size_t)->std::optional<bool>{return true;},[&](const QString& text){sent=text;return true;},{}});
        auto* snap=sidebar.findChild<QToolButton*>("OM9SnapEnd");QVERIFY(snap->isCheckable());QVERIFY(snap->isChecked());
        sidebar.findChild<QToolButton*>("OM9OsnapMaster")->click();QCOMPARE(sent,QString("Osnap Toggle"));
    }
    void workspaceControlsDispatchAndRefreshAvailability() {
        QMainWindow window; bool available=false;std::size_t executed=std::size_t(-1);
        MatrixSidebar sidebar(&window,QStringLiteral(OM9_RESOURCE_DIR),
            {[&](std::size_t){return available;},[&](std::size_t command){executed=command;return true;}});
        auto* select=sidebar.findChild<QToolButton*>("OM9WorkspaceSelectAll");QVERIFY(select);
        QVERIFY(!select->isEnabled());select->click();QCOMPARE(executed,std::size_t(-1));
        auto* views=sidebar.findChild<QToolButton*>("OM9WorkspaceViews");QVERIFY(views);QVERIFY(views->menu());
        QCOMPARE(views->menu()->actions().size(),7);
        available=true;sidebar.refreshAvailability();QVERIFY(select->isEnabled());select->click();
        QCOMPARE(QString::fromUtf8(om9_command_icon(executed)),QString("SelectAll"));
        auto* top=sidebar.findChild<QAction*>("OM9WorkspaceViewTop");QVERIFY(top);QVERIFY(top->isEnabled());top->trigger();
        QCOMPARE(QString::fromUtf8(om9_command_icon(executed)),QString("ViewTop"));
        available=false;sidebar.refreshAvailability();QVERIFY(!top->isEnabled());QVERIFY(!views->isEnabled());
    }
    void referencePanelsHaveCompactRowsAndControls() {
        QMainWindow window;MatrixSidebar sidebar(&window,QStringLiteral(OM9_RESOURCE_DIR),{});
        QCOMPARE(sidebar.findChildren<QWidget*>(QRegularExpression("OM9SelectorRow[0-2]")).size(),3);
        QCOMPARE(sidebar.findChildren<QToolButton*>(QRegularExpression("OM9InfoIcon\\d+")).size(),22);
        QCOMPARE(sidebar.findChildren<QLabel*>(QRegularExpression("OM9LayerName\\d+")).size(),32);
        QCOMPARE(sidebar.findChildren<QToolButton*>(QRegularExpression("OM9LayerLock\\d+")).size(),32);
        QCOMPARE(sidebar.findChildren<QComboBox*>(QRegularExpression("OM9DisplayCombo\\d")).size(),2);
        QCOMPARE(sidebar.findChildren<QFrame*>("OM9ProjectCategory").size(),5);
        auto* label=sidebar.findChild<QToolButton*>("OM9Group10");QVERIFY(label);QCOMPARE(label->text(),QString("SubD"));
        auto* info=sidebar.findChild<QToolButton*>("OM9InfoIcon0");QVERIFY(info);QVERIFY(!info->isEnabled());
        auto* lock=sidebar.findChild<QToolButton*>("OM9LayerLock0");QVERIFY(lock);QVERIFY(!lock->isEnabled());
        window.setCentralWidget(new QWidget(&window));window.resize(1000,700);window.show();sidebar.activate();
        QTest::qWait(40);
        auto* firstDisplay=sidebar.findChild<QToolButton*>("OM9DisplayToggle0");
        auto* lastDisplay=sidebar.findChild<QToolButton*>("OM9Display4");
        QCOMPARE(lastDisplay->y(),firstDisplay->y());
        for(int r=0;r<2;++r){auto* first=sidebar.findChild<QToolButton*>(QString("OM9InfoIcon%1").arg(r*11));
            auto* last=sidebar.findChild<QToolButton*>(QString("OM9InfoIcon%1").arg(r*11+10));QCOMPARE(last->y(),first->y());}
        auto* extra=sidebar.findChild<QToolButton*>("OM9SnapExtra0");
        QCOMPARE(sidebar.findChild<QToolButton*>("OM9GridSnap")->y(),extra->y());
    }
    void disabledIconsStayColoredAndCannotExecute() {
        om9_sidebar_reset(); QMainWindow window; int executions=0;
        MatrixSidebar sidebar(&window, QStringLiteral(OM9_RESOURCE_DIR),
            {[](std::size_t){return false;}, [&executions](std::size_t){++executions;return true;}});
        auto* button=sidebar.findChild<QToolButton*>("OM9Command2");
        QVERIFY(button); QVERIFY(!button->isEnabled());
        QCOMPARE(button->icon().pixmap(24,24,QIcon::Disabled).toImage(),
                 button->icon().pixmap(24,24,QIcon::Normal).toImage());
        auto before=om9_sidebar_history_count();button->click();
        QCOMPARE(executions,0);QCOMPARE(om9_sidebar_history_count(),before);
        auto* snap=sidebar.findChild<QToolButton*>("OM9SnapEnd");QVERIFY(snap);
        QCOMPARE(snap->icon().pixmap(24,24,QIcon::Disabled).toImage(),snap->icon().pixmap(24,24,QIcon::Normal).toImage());
    }
    void displayButtonsUseInstalledModeIcons() {
        QMainWindow window;
        MatrixSidebar sidebar(&window, QStringLiteral(OM9_RESOURCE_DIR), {});
        auto* button=sidebar.findChild<QToolButton*>("OM9Display0");
        QVERIFY(button);
        QVERIFY(!button->isEnabled());
        QSettings assets(QStringLiteral(OM9_RESOURCE_DIR "/menu/icons.ini"),QSettings::IniFormat);
        QImage source(QStringLiteral(OM9_RESOURCE_DIR "/")+assets.value("Dial_Wireframe/image").toString());
        QVERIFY(!source.isNull());
        QCOMPARE(button->icon().pixmap(button->iconSize()).toImage().convertToFormat(QImage::Format_ARGB32),
                 source.scaled(button->iconSize(),Qt::KeepAspectRatio,Qt::SmoothTransformation).convertToFormat(QImage::Format_ARGB32));
    }
    void snapButtonsUseSelectedInstalledAssets() {
        QMainWindow window;
        MatrixSidebar sidebar(&window, QStringLiteral(OM9_RESOURCE_DIR), {});
        auto* button=sidebar.findChild<QToolButton*>("OM9SnapEnd");
        QVERIFY(button);
        QVERIFY(!button->isEnabled());
        QSettings assets(QStringLiteral(OM9_RESOURCE_DIR "/menu/icons.ini"),QSettings::IniFormat);
        QImage source(QStringLiteral(OM9_RESOURCE_DIR "/")+assets.value("ToolsObjectSnapEnd/image").toString());
        QVERIFY(!source.isNull());
        QCOMPARE(button->icon().pixmap(button->iconSize()).toImage().convertToFormat(QImage::Format_ARGB32),
                 source.scaled(button->iconSize(),Qt::KeepAspectRatio,Qt::SmoothTransformation).convertToFormat(QImage::Format_ARGB32));
    }
    void closedMainMenuCanBeRestored() {
        om9_sidebar_reset();QMainWindow window;
        MatrixSidebar sidebar(&window, QStringLiteral(OM9_RESOURCE_DIR), {});
        sidebar.findChild<QToolButton*>("OM9SectionClose1")->click();
        QCOMPARE(om9_sidebar_section_flags(1),0u);
        auto* restore=sidebar.findChild<QAction*>("OM9RestorePanels");QVERIFY(restore);
        restore->trigger();QCOMPARE(om9_sidebar_section_flags(1),1u);
    }
    void externalExecutionRefreshesHistory() {
        QMainWindow window;
        MatrixSidebar sidebar(&window, QStringLiteral(OM9_RESOURCE_DIR), {[](std::size_t){return true;}, [](std::size_t){return true;}});
        auto before=om9_sidebar_history_count();
        om9_sidebar_record_execution(0,true);
        sidebar.refreshAvailability();
        QCOMPARE(sidebar.findChildren<QToolButton*>(QRegularExpression("OM9History\\d+")).size(),int(std::min(before+1,std::size_t(20))));
    }
    void layoutSelectionAndReset() {
        om9_sidebar_reset(); QMainWindow window;
        MatrixSidebar sidebar(&window, QStringLiteral(OM9_RESOURCE_DIR), {[](std::size_t){return false;}, [](std::size_t){return false;}});
        QCOMPARE(sidebar.findChildren<QAbstractButton*>(QRegularExpression("OM9SectionTitle\\d")).size(),7);
        QCOMPARE(sidebar.findChildren<QToolButton*>(QRegularExpression("OM9Group\\d+")).size(),18);
        QCOMPARE(sidebar.findChildren<QToolButton*>(QRegularExpression("OM9Quick\\d+")).size(),11);
        auto* group=sidebar.findChild<QToolButton*>("OM9Group7"); QVERIFY(group); group->click();
        QCOMPARE(om9_sidebar_selected_group(),std::size_t(7));
        om9_sidebar_set_section(0,false,true); sidebar.refreshState();
        sidebar.findChild<QToolButton*>("OM9Group5")->click();
        QCOMPARE(om9_sidebar_section_flags(0),1u);
        for(auto* button:sidebar.findChildren<QToolButton*>(QRegularExpression("OM9Command\\d+"))) QVERIFY(!button->isEnabled());
    }
    void successFailureAndDockRestoration() {
        om9_sidebar_reset(); QMainWindow window;
        QDockWidget other("Tree",&window); window.addDockWidget(Qt::LeftDockWidgetArea,&other);
        bool success=false;
        MatrixSidebar sidebar(&window, QStringLiteral(OM9_RESOURCE_DIR), {[](std::size_t){return true;}, [&success](std::size_t){return success;}});
        window.resize(1000,800); window.show(); other.show();
        sidebar.activate(); QVERIFY(!other.isVisible());
        auto count=sidebar.findChildren<QWidget*>().size();
        for(int i=0;i<3;++i) {sidebar.deactivate(); QVERIFY(other.isVisible()); sidebar.activate();}
        QCOMPARE(sidebar.findChildren<QWidget*>().size(),count);
        auto* button=sidebar.findChild<QToolButton*>("OM9Command0"); QVERIFY(button);
        auto before=om9_sidebar_history_count(); button->click(); QCOMPARE(om9_sidebar_history_count(),before);
        success=true; button->click(); QCOMPARE(om9_sidebar_history_count(),std::min(before+1,std::size_t(20)));
        success=false; before=om9_sidebar_history_count(); button->click(); QCOMPARE(om9_sidebar_history_count(),before);
        sidebar.deactivate(); QVERIFY(other.isVisible());
    }
};
QTEST_MAIN(MatrixSidebarTest)
#include "MatrixSidebarTest.moc"
