#include "MatrixSidebar.h"
#include "RustBridge.h"
#include "CoreSnaps.h"
#include "CoreKeyboard.h"
#include <QMainWindow>
#include <QScrollArea>
#include <QToolButton>
#include <QAction>
#include <QMenu>
#include <QLabel>
#include <QComboBox>
#include <QFrame>
#include <QSettings>
#include <QDir>
#include <QTimer>
#include <QSignalBlocker>
#include <QPainter>
#include <QLayout>
#include <QHBoxLayout>
#include <QVBoxLayout>
#include <QGridLayout>
#include <QRegularExpression>
#include <algorithm>
#include <limits>
#include <utility>

namespace {
class TitleButton final : public QToolButton {
public:
    using QToolButton::QToolButton;
protected:
    void paintEvent(QPaintEvent*) override {
        QPainter painter(this);painter.setPen(Qt::black);
        painter.drawText(rect().adjusted(1,0,0,0),Qt::AlignLeft|Qt::AlignVCenter,text());
    }
};
class GroupButton final : public QToolButton {
public:
    explicit GroupButton(const QColor& dot,QWidget* parent):QToolButton(parent),dot(dot){}
    QSize sizeHint() const override {return QSize(fontMetrics().horizontalAdvance(text())+(dot.isValid()?9:3),16);}
protected:
    void paintEvent(QPaintEvent*) override {
        QPainter p(this);p.setRenderHint(QPainter::Antialiasing);
        if(isChecked()){p.fillRect(rect(),QColor("#45577d"));p.setPen(QColor("#829dce"));p.drawRect(rect().adjusted(0,0,-1,-1));}
        int x=1;if(dot.isValid()){QRadialGradient gradient(4,7,4);gradient.setColorAt(0,Qt::white);gradient.setColorAt(.4,dot);gradient.setColorAt(1,dot.darker(180));p.setPen(Qt::NoPen);p.setBrush(gradient);p.drawEllipse(QRectF(1,4,6,6));x=8;}
        p.setPen(Qt::white);p.drawText(rect().adjusted(x,0,0,0),Qt::AlignLeft|Qt::AlignVCenter,text());
    }
private:QColor dot;
};
// Compact wrapping layout keeps every selector reachable at small widths/high DPI.
class FlowLayout final : public QLayout {
public:
    explicit FlowLayout(QWidget* parent):QLayout(parent) {setContentsMargins(0,0,0,0);setSpacing(1);}
    ~FlowLayout() override {while(auto* item=takeAt(0)) delete item;}
    void addItem(QLayoutItem* item) override {items.push_back(item);}
    int count() const override {return int(items.size());}
    QLayoutItem* itemAt(int i) const override {return i>=0 && i<count()?items[std::size_t(i)]:nullptr;}
    QLayoutItem* takeAt(int i) override {if(i<0 || i>=count())return nullptr;auto* v=items[std::size_t(i)];items.erase(items.begin()+i);return v;}
    Qt::Orientations expandingDirections() const override {return {};}
    bool hasHeightForWidth() const override {return true;}
    int heightForWidth(int w) const override {return arrange(QRect(0,0,w,0),false);}
    QSize sizeHint() const override {return minimumSize();}
    QSize minimumSize() const override {QSize size;for(auto* i:items)size=size.expandedTo(i->minimumSize());return size;}
    void setGeometry(const QRect& rect) override {QLayout::setGeometry(rect);arrange(rect,true);}
private:
    int arrange(const QRect& rect,bool set) const {
        int x=rect.x(),y=rect.y(),row=0;
        for(auto* item:items) {
            const QSize size=item->sizeHint();
            if(x>rect.x() && x+size.width()>rect.right()+1) {x=rect.x();y+=row+spacing();row=0;}
            if(set)item->setGeometry(QRect(QPoint(x,y),size));
            x+=size.width()+spacing();row=std::max(row,size.height());
        }
        return y-rect.y()+row;
    }
    std::vector<QLayoutItem*> items;
};
void clear(QWidget* body) {while(auto* item=body->layout()->takeAt(0)){delete item->widget();delete item;}}
QIcon coloredIcon(const QPixmap& pixmap) {
    QIcon icon(pixmap);
    // Disabled is an execution state; keep the supplied artwork in full color.
    icon.addPixmap(pixmap,QIcon::Disabled,QIcon::Off);
    icon.addPixmap(pixmap,QIcon::Disabled,QIcon::On);
    return icon;
}
QIcon fallbackIcon() {
    QPixmap image(24,24);image.fill(Qt::transparent);
    QPainter p(&image);p.setRenderHint(QPainter::Antialiasing);p.setPen(QPen(QColor("#6ea1d7"),1.5));
    p.drawRoundedRect(QRectF(3,3,18,18),2,2);p.drawLine(5,18,18,5);p.drawEllipse(QPointF(9,9),2,2);
    return coloredIcon(image);
}
QString color(const QString& name) {
    static const QHash<QString,QString> colors={{"Yellow","#efda48"},{"Cyan","#72d9e8"},{"Red","#e97777"},{"Green","#8bc78b"},{"Blue","#6c96eb"},{"Magenta","#d885c9"},{"Orange","#e9a86b"},{"Grey","#dddddd"},{"Purple","#b896d7"}};
    return colors.value(name,"#b0c6e4");
}
}
using namespace OpenMatrix9Gui;
MatrixSidebar::MatrixSidebar(QMainWindow* window,const QString& resourceRoot,HostCallbacks host)
    :QDockWidget("OpenMatrix9",window),window(window),resourceRoot(resourceRoot),host(std::move(host)) {
    setObjectName("OpenMatrix9Sidebar");setAllowedAreas(Qt::LeftDockWidgetArea|Qt::RightDockWidgetArea);
    setFeatures(QDockWidget::NoDockWidgetFeatures);setTitleBarWidget(new QWidget(this));
    setMinimumWidth(280);
    QFont sidebarFont("Verdana");sidebarFont.setPointSizeF(8.25);setFont(sidebarFont);
    setStyleSheet("QDockWidget,QScrollArea,QWidget#OM9Content{background:#333333;color:#eeeeee;} QWidget#OM9PanelBody{background:#606060;} QToolButton{border:1px solid transparent;color:#eeeeee;background:transparent;padding:0px;} QToolButton:hover{border:1px solid #bfd0ee;background:#526e99;} QToolButton:checked{border:1px solid #afc4eb;background:#718fd1;} QToolButton:disabled{color:#eeeeee;} QLabel{color:#eeeeee;} QComboBox{background:#555555;color:white;border:1px solid #999999;font-size:10px;} QComboBox:disabled{color:white;} QFrame#OM9ProjectSlot,QFrame#OM9ProjectCategory{border:1px solid #888888;background:#333333;} QScrollBar{background:#555555;} QScrollBar:vertical{width:11px;} QScrollBar:horizontal{height:11px;}");
    auto* scroll=new QScrollArea(this);scroll->setWidgetResizable(true);scroll->setFrameShape(QFrame::NoFrame);
    auto* content=new QWidget(scroll);content->setObjectName("OM9Content");
    auto* layout=new QVBoxLayout(content);layout->setContentsMargins(0,0,0,0);layout->setSpacing(2);
    history=new QWidget(content);history->setStyleSheet("background:#646464;");new FlowLayout(history);
    layout->addWidget(section(0,"ICON HISTORY",history));

    auto* main=new QWidget(content);auto* mainLayout=new QVBoxLayout(main);mainLayout->setContentsMargins(1,0,1,0);mainLayout->setSpacing(1);
    std::vector<QLayout*> selectorFlows;
    for(int r=0;r<3;++r){auto* row=new QWidget(main);row->setObjectName(QString("OM9SelectorRow%1").arg(r));QLayout* flow=r==0?static_cast<QLayout*>(new QHBoxLayout(row)):static_cast<QLayout*>(new FlowLayout(row));flow->setContentsMargins(0,0,0,0);flow->setSpacing(0);selectorFlows.push_back(flow);mainLayout->addWidget(row);}
    for(std::size_t g=0;g<om9_sidebar_group_count();++g) {
        auto* b=new GroupButton(g<6?QColor():QColor(color(QString::fromUtf8(om9_sidebar_group_color(g)))),main);b->setObjectName(QString("OM9Group%1").arg(g));
        const QString title=QString::fromUtf8(om9_sidebar_group_title(g));
        b->setText(title=="Clayoo"?"SubD":title=="Emboss"?"Art":title=="Settings"?"Setting":title);b->setToolTip(title);b->setCheckable(true);
        b->setFixedHeight(16);if(g==5)static_cast<QHBoxLayout*>(selectorFlows[0])->addStretch();selectorFlows[std::min(g/6,std::size_t(2))]->addWidget(b);
        connect(b,&QToolButton::clicked,this,[this,g]{if(om9_sidebar_group_kind(g)==2)om9_sidebar_reset();else om9_sidebar_select_group(g);refreshState();});
    }
    grid=new QWidget(main);new FlowLayout(grid);mainLayout->addWidget(grid);grid->setMinimumHeight(51);
    auto* quick=new QWidget(main);auto* quickFlow=new FlowLayout(quick);
    for(std::size_t i=0;i<om9_sidebar_quick_count();++i)quickFlow->addWidget(commandButton(om9_sidebar_quick_command(i),quick,QString("OM9Quick%1").arg(i)));
    mainLayout->addWidget(quick);
    auto* workspace=new QWidget(main);workspace->setObjectName("OM9WorkspaceRow");
    auto* workspaceFlow=new FlowLayout(workspace);workspaceFlow->setSpacing(1);
    const QStringList workspaceLabels={"All","None","Delete","Fit all","Fit sel"};
    for(std::size_t i=0;i<5;++i) {
        auto command=om9_workspace_command(i);auto* button=commandButton(command,workspace,
            QString("OM9Workspace")+QString::fromUtf8(om9_command_icon(command)));
        button->setIcon(QIcon());button->setText(workspaceLabels[int(i)]);button->setFixedSize(i==2?45:43,21);
        button->setStyleSheet("QToolButton{border:1px solid #939393;background:#555555;font-size:10px;} QToolButton:hover{background:#526e99;} QToolButton:disabled{color:#aaaaaa;}");
        workspaceFlow->addWidget(button);
    }
    auto* views=new QToolButton(workspace);views->setObjectName("OM9WorkspaceViews");views->setText("Views");views->setFixedSize(48,21);
    views->setStyleSheet("QToolButton{border:1px solid #939393;background:#555555;font-size:10px;} QToolButton:disabled{color:#aaaaaa;}");
    auto* viewMenu=new QMenu(views);views->setMenu(viewMenu);views->setPopupMode(QToolButton::InstantPopup);
    for(std::size_t i=5;i<om9_workspace_command_count();++i) {
        auto command=om9_workspace_command(i);auto* action=viewMenu->addAction(QString::fromUtf8(om9_command_menu_text(command)));
        action->setObjectName(QString("OM9Workspace")+QString::fromUtf8(om9_command_icon(command)));
        action->setProperty("om9Command",QVariant::fromValue<qulonglong>(command));
        connect(action,&QAction::triggered,this,[this,command]{invoke(command);});
    }
    workspaceFlow->addWidget(views);mainLayout->addWidget(workspace);
    layout->addWidget(section(1,"MAIN MENU",main));

    const auto iconButton=[this](QWidget* parent,const QString& key,const QString& name,const QSize& size=QSize(23,23)){
        auto* b=new QToolButton(parent);b->setObjectName(name);b->setProperty("om9IconKey",key);
        const QSize iconSize=size-QSize(1,1);b->setIcon(iconForKey(key,iconSize));b->setIconSize(iconSize);b->setFixedSize(size);b->setEnabled(false);b->setToolTip(key+" — chưa hỗ trợ");
        const auto command=findCommand(key.toUtf8().constData());
        if(command<om9_command_count()){b->setProperty("om9Command",QVariant::fromValue<qulonglong>(command));connect(b,&QToolButton::clicked,this,[this,command]{invoke(command);});}
        return b;
    };
    const auto textButton=[](QWidget* parent,const QString& text,const QSize& size){auto* b=new QToolButton(parent);b->setText(text);b->setFixedSize(size);b->setEnabled(false);b->setToolTip(text+" — chưa hỗ trợ");b->setStyleSheet("QToolButton{border:1px solid #979797;background:#555555;font-size:10px;}");return b;};
    const auto separator=[](QWidget* parent){auto* label=new QLabel("────────────── ● ──────────────",parent);label->setAlignment(Qt::AlignCenter);label->setFixedHeight(13);label->setStyleSheet("color:#a8a8a0;background:#333333;font-size:10px;");return label;};

    auto* display=new QWidget(content);auto* displayLayout=new QVBoxLayout(display);displayLayout->setContentsMargins(1,1,1,1);displayLayout->setSpacing(1);
    auto* viewRow=new QWidget(display);auto* viewFlow=new FlowLayout(viewRow);viewFlow->setSpacing(1);
    const QStringList displayToggles={"GridON","PreviewCutterON","PreviewShadeON","ShadeSelectedOnlyON","GVGemView_1","GVSurfaceView_1"};
    for(int i=0;i<displayToggles.size();++i)viewFlow->addWidget(iconButton(viewRow,displayToggles[i],QString("OM9DisplayToggle%1").arg(i)));
    viewFlow->addItem(new QSpacerItem(5,23));
    const QStringList displayKeys={"Dial_Wireframe","Dial_Shaded","Dial_Ghosted","Dial_Tech Shade","Dial_Working Render"};
    for(int i=0;i<displayKeys.size();++i)viewFlow->addWidget(iconButton(viewRow,displayKeys[i],QString("OM9Display%1").arg(i)));
    displayLayout->addWidget(viewRow);
    auto* comboRow=new QWidget(display);auto* combos=new QHBoxLayout(comboRow);combos->setContentsMargins(0,0,0,0);combos->setSpacing(2);
    for(int i=0;i<2;++i){auto* combo=new QComboBox(comboRow);combo->setObjectName(QString("OM9DisplayCombo%1").arg(i));combo->addItems(i==0?QStringList{"Coarse Mesh","Medium Mesh","Fine Mesh","Super Fine Mesh"}:QStringList{"Wireframe","Shaded","Ghosted","Technical","Working Render"});if(i==1)combo->setCurrentIndex(1);combo->setEnabled(false);combo->setFixedHeight(18);combos->addWidget(combo,1);}
    displayLayout->addWidget(comboRow);layout->addWidget(section(2,"DISPLAY",display));

    auto* snaps=new QWidget(content);auto* snapsLayout=new QVBoxLayout(snaps);snapsLayout->setContentsMargins(1,1,1,1);snapsLayout->setSpacing(1);
    auto* snapRow=new QWidget(snaps);auto* snapFlow=new FlowLayout(snapRow);
    const QStringList snapLabels={"Perp","Tan","Int","Point","Near","Quad","Cen","Mid","End"};
    const QStringList snapKeys={"ToolsObjectSnapPerpendicularTo","ToolsObjectSnapTangentTo","ToolsObjectSnapIntersection","ToolsObjectSnapPoint","ToolsObjectSnapNear","ToolsObjectSnapQuadrant","ToolsObjectSnapCenter","ToolsObjectSnapMidpoint","ToolsObjectSnapEnd"};
    for(int i=0;i<snapKeys.size();++i)snapFlow->addWidget(iconButton(snapRow,snapKeys[i],"OM9Snap"+snapLabels[i]));
    auto* osnap=textButton(snapRow,"I",QSize(50,23));osnap->setObjectName("OM9OsnapMaster");osnap->setCheckable(true);osnap->setToolTip("O-Snap On/Off");connect(osnap,&QToolButton::clicked,this,[]{CoreSnaps::submit("Osnap Toggle");});osnap->setStyleSheet("QToolButton{background:#789de9;border:1px solid #c5d7ff;color:#162f54;}QToolButton:checked{background:#b6d584;}");snapFlow->addWidget(osnap);snapsLayout->addWidget(snapRow);
    auto* stepRow=new QWidget(snaps);auto* stepFlow=new FlowLayout(stepRow);
    const QStringList stepKeys={"SnapOnSurface","SnapOnPolysurface","SnapBetween","OrthoSnapON","PlanarSnapON","ProjectSnapON"};
    for(int i=0;i<stepKeys.size();++i)stepFlow->addWidget(iconButton(stepRow,stepKeys[i],QString("OM9SnapExtra%1").arg(i)));
    for(const auto& step:QStringList{"0.1","0.25","0.5","1.0"}){auto* b=textButton(stepRow,step,QSize(26,20));if(step=="0.5")b->setStyleSheet("background:#789de9;border:1px solid #c5d7ff;font-size:10px;");stepFlow->addWidget(b);}
    stepFlow->addWidget(iconButton(stepRow,"GridSnapON","OM9GridSnap"));snapsLayout->addWidget(stepRow);layout->addWidget(section(3,"SNAPS",snaps));

    auto* info=new QWidget(content);auto* infoLayout=new QVBoxLayout(info);infoLayout->setContentsMargins(1,1,1,1);infoLayout->setSpacing(1);
    const std::vector<QStringList> infoKeys={
        {"RhinoOptions","ObjectProperties","GVObjectInfo","AllObjectInfo","CommandHistory","ProjectNotes","SuperSelect","InfoSettingsSmartTargetsGumballON","InfoSettingsGumballAlignment","InfoSettingsRelocateGumball","InfoSettingsGumballON"},
        {"InfoSettingsViewportTabsToggle","InfoSettingsDisplayProperties","InfoSettingsBoxEdit","InfoSettingsLibraries","InfoSettingsSelectionFilter","RhinoSmartTrackON","InfoSettingsDesignReport","RhinoHistoryON","GVHistoryUpdateON","GVHistoryRecordON","InfoSettingsGVClearHistory"}};
    for(int r=0;r<2;++r){auto* row=new QWidget(info);auto* flow=new FlowLayout(row);for(int i=0;i<infoKeys[r].size();++i){if(i==7)flow->addItem(new QSpacerItem(5,23));flow->addWidget(iconButton(row,infoKeys[r][i],QString("OM9InfoIcon%1").arg(r*11+i)));}infoLayout->addWidget(row);}
    layout->addWidget(section(4,"INFO & SETTINGS",info));

    auto* layers=new QWidget(content);auto* layersLayout=new QVBoxLayout(layers);layersLayout->setContentsMargins(1,1,1,1);layersLayout->setSpacing(1);
    auto* lights=new QWidget(layers);auto* lightsLayout=new QHBoxLayout(lights);lightsLayout->setContentsMargins(0,0,0,0);lightsLayout->setSpacing(1);auto* lightsLabel=new QLabel("Lights",lights);lightsLabel->setFixedWidth(51);lightsLayout->addWidget(lightsLabel);
    lightsLayout->addWidget(iconButton(lights,"LayerArrow","OM9LightsArrow",QSize(15,15)));auto* lightColor=new QLabel(lights);lightColor->setFixedSize(21,15);lightColor->setStyleSheet("background:white;border:1px solid #bbbbbb;");lightsLayout->addWidget(lightColor);
    lightsLayout->addWidget(iconButton(lights,"LayerLock","OM9LightsLock",QSize(18,15)));lightsLayout->addWidget(iconButton(lights,"LayerVisibility","OM9LightsVisibility",QSize(26,15)));lightsLayout->addStretch();lightsLayout->addWidget(textButton(lights,"Hide ◉",QSize(55,15)));lightsLayout->addWidget(textButton(lights,"Show ◉",QSize(58,15)));layersLayout->addWidget(lights);
    QStringList layerNames={"Metal 01","Metal 02","Metal 03","Metal 04","Gem 01","Gem 02","Gem 03","Gem 04","User 01","User 02","User 03","User 04","Heads","Finger","Cutting","Creation"};
    QStringList swatches={"#229987","#319f49","#70c64b","#a0cf78","#598bc2","#4f80cb","#7aa7ce","#accae4","#e23434","#70c133","#3468ff","#777777","#851ca1","#ad423c","#ca7135","#dfb126"};
    for(int n:{17,18,19,20,25,26,27,28,21,22,23,24,29,30,31,32})layerNames.append(QString("User %1").arg(n));
    swatches+=QStringList{"#df9098","#dabb7c","#bd8735","#a5c14b","#47c7aa","#72c6cb","#607cab","#4d626a","#57b397","#586076","#c6bbb7","#6c5578","#57536b","#1e555b","#0c343f","#1b3555"};
    for(int block=0;block<2;++block){if(block)layersLayout->addWidget(separator(layers));auto* rows=new QWidget(layers);auto* gridLayout=new QGridLayout(rows);gridLayout->setContentsMargins(0,0,0,0);gridLayout->setSpacing(1);
        for(int j=0;j<16;++j){int i=block*16+j;auto* row=new QWidget(rows);auto* rowLayout=new QHBoxLayout(row);rowLayout->setContentsMargins(0,0,0,0);rowLayout->setSpacing(0);
            auto* label=new QLabel(layerNames[i],row);label->setObjectName(QString("OM9LayerName%1").arg(i));label->setFixedSize(51,15);label->setStyleSheet(i==0?"font-size:10px;background:#759df0;":"font-size:10px;");rowLayout->addWidget(label);
            rowLayout->addWidget(iconButton(row,"LayerArrow",QString("OM9LayerArrow%1").arg(i),QSize(15,15)));auto* swatch=new QLabel(row);swatch->setFixedSize(21,15);swatch->setStyleSheet("background:"+swatches[i]+";border:1px solid #bbbbbb;");rowLayout->addWidget(swatch);
            rowLayout->addWidget(iconButton(row,"LayerLock",QString("OM9LayerLock%1").arg(i),QSize(18,15)));auto* visibility=textButton(row,"I",QSize(26,15));visibility->setObjectName(QString("OM9LayerVisibility%1").arg(i));visibility->setStyleSheet("background:#789de9;border:1px solid #c5d7ff;color:#162f54;");rowLayout->addWidget(visibility);gridLayout->addWidget(row,j%8,j/8);}
        layersLayout->addWidget(rows);}
    layersLayout->addWidget(separator(layers));layout->addWidget(section(5,"LAYERS",layers));

    auto* projects=new QWidget(content);auto* projectLayout=new QVBoxLayout(projects);projectLayout->setContentsMargins(1,1,1,1);projectLayout->setSpacing(1);
    auto* mainProjects=new QWidget(projects);auto* mainProjectsLayout=new QHBoxLayout(mainProjects);mainProjectsLayout->setContentsMargins(0,0,0,0);mainProjectsLayout->setSpacing(2);
    auto* listPanel=new QWidget(mainProjects);listPanel->setFixedWidth(90);auto* listLayout=new QVBoxLayout(listPanel);listLayout->setContentsMargins(0,0,0,0);listLayout->setSpacing(1);
    auto* list=new QScrollArea(listPanel);list->setFixedHeight(111);list->setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOn);list->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);auto* emptyList=new QWidget(list);emptyList->setFixedSize(70,130);emptyList->setStyleSheet("background:#555555;");list->setWidget(emptyList);listLayout->addWidget(list);
    auto* listActions=new QWidget(listPanel);auto* listActionsLayout=new QHBoxLayout(listActions);listActionsLayout->setContentsMargins(0,0,0,0);listActionsLayout->setSpacing(1);listActionsLayout->addWidget(iconButton(listActions,"ProjectAdd","OM9ProjectNew",QSize(24,15)));listActionsLayout->addWidget(iconButton(listActions,"ProjectIn","OM9ProjectLoad",QSize(24,15)));listActionsLayout->addWidget(textButton(listActions,"Mngr",QSize(40,15)));listLayout->addWidget(listActions);mainProjectsLayout->addWidget(listPanel);
    auto* cardsScroll=new QScrollArea(mainProjects);cardsScroll->setWidgetResizable(false);cardsScroll->setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOff);cardsScroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOn);auto* cards=new QWidget(cardsScroll);auto* cardsLayout=new QHBoxLayout(cards);cardsLayout->setContentsMargins(0,0,0,0);cardsLayout->setSpacing(2);
    for(int i=1;i<=3;++i){auto* frame=new QFrame(cards);frame->setObjectName("OM9ProjectSlot");frame->setFixedSize(78,109);auto* l=new QVBoxLayout(frame);l->setContentsMargins(2,1,2,1);l->setSpacing(1);auto* top=new QHBoxLayout;top->addWidget(new QLabel(QString::number(i),frame));top->addStretch();top->addWidget(iconButton(frame,"ProjectOut",QString("OM9ProjectOut%1").arg(i),QSize(15,12)));l->addLayout(top);l->addStretch();auto* actions=new QHBoxLayout;actions->setSpacing(0);for(const auto& key:QStringList{"ProjectIn","ProjectSave","ProjectDelete"})actions->addWidget(iconButton(frame,key,key+QString::number(i),QSize(24,15)));l->addLayout(actions);cardsLayout->addWidget(frame);}
    cards->setFixedSize(238,111);cardsScroll->setWidget(cards);cardsScroll->setFixedHeight(128);mainProjectsLayout->addWidget(cardsScroll,1);projectLayout->addWidget(mainProjects);projectLayout->addWidget(separator(projects));
    auto* categories=new QWidget(projects);auto* categoriesLayout=new QHBoxLayout(categories);categoriesLayout->setContentsMargins(0,0,0,0);categoriesLayout->setSpacing(2);
    for(const auto& name:QStringList{"Master","Creation","Parts","Render","Output"}){auto* frame=new QFrame(categories);frame->setObjectName("OM9ProjectCategory");frame->setMinimumHeight(76);auto* l=new QVBoxLayout(frame);l->setContentsMargins(1,1,1,1);l->setSpacing(0);auto* title=new QLabel(name,frame);title->setStyleSheet("font-size:9px;");l->addWidget(title);l->addStretch();auto* actions=new QHBoxLayout;actions->setContentsMargins(0,0,0,0);actions->setSpacing(0);for(const auto& key:QStringList{"ProjectIn","ProjectSave","ProjectDelete"})actions->addWidget(iconButton(frame,key,name+key,QSize(17,15)));l->addLayout(actions);categoriesLayout->addWidget(frame,1);}
    projectLayout->addWidget(categories);projectLayout->addWidget(separator(projects));layout->addWidget(section(6,"PROJECTS",projects));layout->addStretch(1);
    scroll->setWidget(content);setWidget(scroll);window->addDockWidget(Qt::LeftDockWidgetArea,this);
    auto* restore=new QAction(tr("Khôi phục các bảng"),this);restore->setObjectName("OM9RestorePanels");
    addAction(restore);setContextMenuPolicy(Qt::ActionsContextMenu);
    connect(restore,&QAction::triggered,this,[this]{om9_sidebar_reset();refreshState();});
    timer=new QTimer(this);timer->setInterval(350);connect(timer,&QTimer::timeout,this,&MatrixSidebar::refreshAvailability);
    refreshState();hide();
}
QWidget* MatrixSidebar::section(int index,const QString& title,QWidget* body) {
    body->setObjectName("OM9PanelBody");
    auto* wrapper=new QWidget(this);auto* layout=new QVBoxLayout(wrapper);layout->setContentsMargins(0,0,0,0);layout->setSpacing(0);
    auto* bar=new QWidget(wrapper);bar->setStyleSheet("background:qlineargradient(x1:0,y1:0,x2:0,y2:1,stop:0 #dbd9ce,stop:0.35 #b9b6a8,stop:1 #8f8d84);color:black;");auto* barLayout=new QHBoxLayout(bar);barLayout->setContentsMargins(1,0,1,0);barLayout->setSpacing(0);
    auto* label=new TitleButton(bar);label->setObjectName(QString("OM9SectionTitle%1").arg(index));label->setText(title);label->setStyleSheet("color:black;font-weight:bold;border:0;");label->setSizePolicy(QSizePolicy::Expanding,QSizePolicy::Fixed);label->setFixedHeight(15);
    auto* close=new QToolButton(bar);close->setObjectName(QString("OM9SectionClose%1").arg(index));close->setText("×");close->setFixedSize(14,15);close->setStyleSheet("color:#ad3232;border:1px solid #747474;");
    connect(label,&QToolButton::clicked,this,[this,index]{auto f=om9_sidebar_section_flags(index);om9_sidebar_set_section(index,true,(f&2)==0);refreshState();});
    connect(close,&QToolButton::clicked,this,[this,index]{auto f=om9_sidebar_section_flags(index);om9_sidebar_set_section(index,false,(f&2)!=0);refreshState();});
    barLayout->addWidget(label);barLayout->addWidget(close);layout->addWidget(bar);layout->addWidget(body);sections.push_back(wrapper);bodies.push_back(body);return wrapper;
}
std::size_t MatrixSidebar::findCommand(const char* icon) const {
    for(std::size_t i=0;i<om9_command_count();++i)if(QString::fromUtf8(om9_command_icon(i))==QLatin1String(icon))return i;
    return std::numeric_limits<std::size_t>::max();
}
QString MatrixSidebar::tooltip(std::size_t command) const {
    const char* raw=om9_command_icon(command);if(!raw)return {};
    QSettings ini(QDir(resourceRoot).filePath("menu/icons.ini"),QSettings::IniFormat);
    return ini.value(QString::fromUtf8(raw)+"/tooltip",QString::fromUtf8(om9_command_tooltip(command))).toString();
}
QIcon MatrixSidebar::iconForCommand(std::size_t command) {
    const char* raw=om9_command_icon(command);if(!raw)return fallbackIcon();
    return iconForKey(QString::fromUtf8(raw));
}
QIcon MatrixSidebar::iconForKey(const QString& key,const QSize& size) {
    QSettings ini(QDir(resourceRoot).filePath("menu/icons.ini"),QSettings::IniFormat);ini.beginGroup(key);
    const auto status=ini.value("status").toString();if(status!="resolved"&&status!="fallback")return fallbackIcon();
    const bool direct=ini.contains("image");
    const QString relative=ini.value(direct?"image":"atlas").toString();
    if(QDir::isAbsolutePath(relative)||relative.contains(".."))return fallbackIcon();
    if(!atlases.contains(relative))atlases.insert(relative,QImage(QDir(resourceRoot).filePath(relative)));
    const auto image=atlases.value(relative);
    if(direct)return image.isNull()?fallbackIcon():coloredIcon(QPixmap::fromImage(image.scaled(size,Qt::KeepAspectRatio,Qt::SmoothTransformation)));
    const QRect rect(ini.value("x").toInt(),ini.value("y").toInt(),ini.value("width").toInt(),ini.value("height").toInt());
    if(image.isNull()||rect.isEmpty()||!image.rect().contains(rect))return fallbackIcon();
    return coloredIcon(QPixmap::fromImage(image.copy(rect)));
}
QToolButton* MatrixSidebar::commandButton(std::size_t command,QWidget* parent,const QString& name) {
    auto* b=new QToolButton(parent);b->setObjectName(name);b->setProperty("om9Command",QVariant::fromValue<qulonglong>(command));b->setFixedSize(25,25);b->setIconSize(QSize(24,24));b->setIcon(iconForCommand(command));
    b->setProperty("om9IconKey",QString::fromUtf8(om9_command_icon(command)));
    connect(b,&QToolButton::clicked,this,[this,command]{invoke(command);});return b;
}
void MatrixSidebar::invoke(std::size_t command) {
    if(host.available && host.available(command) && host.execute)om9_sidebar_record_execution(command,host.execute(command));
    populateHistory();refreshAvailability();
}
void MatrixSidebar::populateGrid() {
    clear(grid);auto g=om9_sidebar_selected_group();
    for(std::size_t i=0;i<om9_sidebar_group_item_count(g);++i){auto command=om9_sidebar_group_item_command(g,i);grid->layout()->addWidget(commandButton(command,grid,QString("OM9Command%1").arg(command)));}
}
void MatrixSidebar::populateHistory() {
    clear(history);
    renderedHistory.clear();
    history->layout()->addWidget(commandButton(findCommand("Undo"),history,"OM9Undo"));history->layout()->addWidget(commandButton(findCommand("Redo"),history,"OM9Redo"));
    for(std::size_t i=0;i<om9_sidebar_history_count();++i) {
        auto command=om9_sidebar_history_command(i);renderedHistory.push_back(command);
        history->layout()->addWidget(commandButton(command,history,QString("OM9History%1").arg(i)));
    }
    history->setMinimumHeight(28);
}
void MatrixSidebar::refreshState() {
    for(std::size_t i=0;i<sections.size();++i){auto flags=om9_sidebar_section_flags(i);sections[i]->setVisible((flags&1)!=0);bodies[i]->setVisible((flags&2)==0);}
    for(std::size_t g=0;g<om9_sidebar_group_count();++g)findChild<QToolButton*>(QString("OM9Group%1").arg(g))->setChecked(g==om9_sidebar_selected_group());
    populateGrid();populateHistory();refreshAvailability();
}
void MatrixSidebar::refreshAvailability() {
    std::vector<std::size_t> currentHistory;
    for(std::size_t i=0;i<om9_sidebar_history_count();++i)currentHistory.push_back(om9_sidebar_history_command(i));
    if(currentHistory!=renderedHistory)populateHistory();
    for(auto* button:findChildren<QToolButton*>())if(button->property("om9Command").isValid()) {
        const auto command=std::size_t(button->property("om9Command").toULongLong());
        bool enabled=host.available && host.available(command);button->setEnabled(enabled);
        if(CoreSnaps::handles(command)){
            const QSignalBlocker blocker(button);button->setCheckable(true);button->setChecked(CoreSnaps::checked(command));
        }
        if(QString::fromUtf8(om9_command_id(command))=="Ortho"){
            const QSignalBlocker blocker(button);button->setCheckable(true);button->setChecked(CoreKeyboard::checked(command));
        }
        button->setToolTip(tooltip(command)+(enabled?QString():QString(" — chưa hỗ trợ hoặc chưa có tài liệu")));
    }
    bool viewsAvailable=false;
    for(auto* action:findChildren<QAction*>())if(action->property("om9Command").isValid()) {
        const auto command=std::size_t(action->property("om9Command").toULongLong());
        const bool enabled=host.available && host.available(command);action->setEnabled(enabled);
        viewsAvailable|=enabled;
    }
    findChild<QToolButton*>("OM9WorkspaceViews")->setEnabled(viewsAvailable);
}
void MatrixSidebar::activate() {
    if(active)return;active=true;alteredDocks.clear();
    for(auto* dock:window->findChildren<QDockWidget*>(QString(),Qt::FindDirectChildrenOnly))if(dock!=this && window->dockWidgetArea(dock)==Qt::LeftDockWidgetArea){alteredDocks.emplace_back(dock,dock->isVisible());dock->hide();}
    show();window->resizeDocks({this},{300},Qt::Horizontal);refreshState();timer->start();
}
void MatrixSidebar::deactivate() {
    if(!active)return;timer->stop();hide();
    for(const auto& [dock,visible]:alteredDocks)if(dock)dock->setVisible(visible);
    alteredDocks.clear();active=false;
}
