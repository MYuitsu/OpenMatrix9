// OM9-FILE-012: typed native values; no preview geometry is archive authority.
#include "ThreeDmHatchDialog.h"
#include "ThreeDmGeometry.h"
#include <Gui/MainWindow.h>
#include <QCheckBox>
#include <QComboBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QHeaderView>
#include <QJsonArray>
#include <QLabel>
#include <QLineEdit>
#include <QLocale>
#include <QPushButton>
#include <QTreeWidget>
#include <QVBoxLayout>
#include <algorithm>
#include <cmath>
namespace OpenMatrix9Gui::ThreeDm {
namespace {
QString label(const QString& key){
    static const QMap<QString,QString> names{{"cvs","Control vertices (homogeneous when rational)"},{"knots","Knots"},
        {"domain","Curve domain"},{"angle_domain","Angular domain (radians)"},{"plane","Plane frame and equation"},
        {"points","Points (mm)"},{"parameters","Parameters"},{"segment_parameters","Segment parameters"},
        {"radius","Radius (mm)"},{"order","Order"},{"segments","Native segments"}};
    return names.value(key,key);
}
QJsonValue replace(const QJsonValue& value,const QStringList& path,int position,const QJsonValue& leaf){
    if(position==path.size())return leaf;
    if(value.isArray()){auto a=value.toArray();bool ok=false;int i=path[position].toInt(&ok);if(!ok||i<0||i>=a.size())throw ExchangeError("Invalid loop field path");a[i]=replace(a[i],path,position+1,leaf);return a;}
    if(value.isObject()){auto o=value.toObject();if(!o.contains(path[position]))throw ExchangeError("Missing native loop field");o[path[position]]=replace(o[path[position]],path,position+1,leaf);return o;}
    throw ExchangeError("Invalid native loop field container");
}
QJsonValue at(QJsonValue value,const QStringList& path){
    for(const auto& part:path){
        if(value.isObject()){auto object=value.toObject();if(!object.contains(part))throw ExchangeError("Missing native loop field");value=object[part];}
        else if(value.isArray()){bool ok=false;int index=part.toInt(&ok);auto array=value.toArray();if(!ok||index<0||index>=array.size())throw ExchangeError("Invalid native row index");value=array[index];}
        else throw ExchangeError("Invalid native row container");
    }
    return value;
}
class Editor final:public QDialog {
    QJsonObject current;
    QTreeWidget* tree;QLabel* error;QComboBox* role;QLineEdit *radius,*x,*y;
    int nodes=0;
    void stage(const QJsonObject& proposed,const QString& selection={}){
        int count=0;
        const auto bounded=[&count](auto&& self,const QJsonValue& value,int depth)->void{
            if(depth>64||++count>65536)throw ExchangeError("Native loop exceeds the table editor's 65536 fields or depth64; use the bounded loop API");
            if(value.isArray())for(auto child:value.toArray())self(self,child,depth+1);
            else if(value.isObject())for(auto child:value.toObject())self(self,child,depth+1);
        };
        for(auto value:proposed["loops"].toArray()){
            auto loop=value.toObject();bounded(bounded,loop["source_index"],0);bounded(bounded,loop["curve"],0);
        }
        current=proposed;populate();
        if(!selection.isEmpty()){
            QTreeWidgetItemIterator iterator(tree);for(;*iterator;++iterator)if((*iterator)->data(0,Qt::UserRole).toString()==selection){
                tree->setCurrentItem(*iterator);tree->scrollToItem(*iterator);(*iterator)->setExpanded(true);break;
            }
        }
        error->clear();
    }
    void editRow(bool duplicate){try{
        auto selected=tree->currentItem();if(!selected)throw ExchangeError("Select a CV, knot, point, parameter or native segment row");
        const auto proposed=candidate();auto path=selected->data(0,Qt::UserRole).toString().split('/',Qt::SkipEmptyParts);
        QStringList container;int index=-1;
        for(int size=path.size();size>=1;--size){
            auto prefix=path.mid(0,size);const auto key=prefix.last();
            if(!QStringList{"cvs","knots","points","parameters","segments"}.contains(key))continue;
            auto value=at(proposed,prefix);if(!value.isArray())continue;
            const auto kind=at(proposed,prefix.mid(0,size-1)).toObject()["class_name"].toString();
            bool editable=(kind=="ON_NurbsCurve"&&(key=="cvs"||key=="knots"))
                ||(kind=="ON_PolylineCurve"&&(key=="points"||key=="parameters"))
                ||(kind=="ON_PolyCurve"&&(key=="segments"||key=="parameters"));
            if(!editable)throw ExchangeError("This native array has fixed cardinality; edit its values instead");
            auto values=value.toArray();
            if(path.size()==size)index=values.size()-1;
            else{bool ok=false;index=path[size].toInt(&ok);if(!ok)throw ExchangeError("Invalid native row selection");}
            container=prefix;break;
        }
        if(container.isEmpty())throw ExchangeError("Select a CV, knot, point, parameter or native segment row");
        auto values=at(proposed,container).toArray();if(index<0||index>=values.size())throw ExchangeError("Invalid native row selection");
        if(duplicate)values.insert(index+1,values[index]);
        else{if(values.size()<=1)throw ExchangeError("Keep at least one row; native validation checks complete cardinality before commit");values.removeAt(index);}
        const int selectedIndex=duplicate?index+1:std::min(index,int(values.size())-1);
        stage(replace(proposed,container,0,values).toObject(),"/"+container.join('/')+"/"+QString::number(selectedIndex));
    }catch(const std::exception& e){error->setText(e.what());}}
    void node(QTreeWidgetItem* parent,const QString& name,const QString& path,const QJsonValue& value,int depth){
        if(depth>64||++nodes>65536)throw ExchangeError("Native loop exceeds the table editor's 65536 fields or depth64; use the bounded loop API");
        auto item=new QTreeWidgetItem(parent,QStringList{label(name),{}});item->setData(0,Qt::UserRole,path);
        if(value.isDouble()){
            item->setText(1,QString::number(value.toDouble(),'g',17));
            if(name!="source_index"&&name!="dimension"&&name!="schema_version")item->setFlags(item->flags()|Qt::ItemIsEditable);
        }else if(value.isBool())item->setText(1,value.toBool()?tr("Yes"):tr("No"));
        else if(value.isString())item->setText(1,value.toString());
        else if(value.isNull())item->setText(1,tr("New native record"));
        else if(value.isArray()){auto a=value.toArray();for(int i=0;i<a.size();++i)node(item,QString::number(i),path+"/"+QString::number(i),a[i],depth+1);}
        else if(value.isObject()){
            auto o=value.toObject();for(auto it=o.begin();it!=o.end();++it)node(item,it.key(),path+"/"+it.key(),it.value(),depth+1);
        }
        item->setData(1,Qt::UserRole,item->text(1));
    }
    QJsonObject candidate()const{
        QJsonValue data=current;
        QTreeWidgetItemIterator iterator(tree);
        for(;*iterator;++iterator){auto item=*iterator;
            if(!(item->flags()&Qt::ItemIsEditable))continue;
            if(item->text(0)!=item->data(0,Qt::UserRole+1).toString())throw ExchangeError("Native field names are immutable");
            if(item->text(1)==item->data(1,Qt::UserRole).toString())continue;
            bool ok=false;double value=QLocale::c().toDouble(item->text(1),&ok);
            if(!ok||!std::isfinite(value))throw ExchangeError("Enter a finite number using a dot for the decimal separator");
            data=replace(data,item->data(0,Qt::UserRole).toString().split('/',Qt::SkipEmptyParts),0,value);
        }
        return data.toObject();
    }
    void populate(){
        tree->clear();nodes=0;auto loops=current["loops"].toArray();
        for(int i=0;i<loops.size();++i){auto loop=loops[i].toObject();
            auto root=new QTreeWidgetItem(tree,QStringList{tr("Boundary %1").arg(i+1),loop["curve"].toObject()["class_name"].toString()});
            root->setData(0,Qt::UserRole,QString("/loops/%1").arg(i));
            auto roleItem=new QTreeWidgetItem(root,QStringList{tr("Boundary role"),{}});
            auto choices=new QComboBox(tree);choices->addItems({tr("Outer"),tr("Inner")});choices->setCurrentIndex(loop["type"].toInt());
            tree->setItemWidget(roleItem,1,choices);
            QObject::connect(choices,&QComboBox::currentIndexChanged,this,[this,i](int value){try{current=candidate();auto a=current["loops"].toArray();auto row=a[i].toObject();row["type"]=value;a[i]=row;current["loops"]=a;}catch(const std::exception& e){error->setText(e.what());}});
            node(root,"source_index",QString("/loops/%1/source_index").arg(i),loop["source_index"],0);
            node(root,"curve",QString("/loops/%1/curve").arg(i),loop["curve"],0);
            root->setExpanded(true);
        }
        QTreeWidgetItemIterator iterator(tree);for(;*iterator;++iterator)(*iterator)->setData(0,Qt::UserRole+1,(*iterator)->text(0));
        tree->resizeColumnToContents(0);
    }
    void addCircle(){try{
        bool validR=false,validX=false,validY=false;double r=QLocale::c().toDouble(radius->text(),&validR),cx=QLocale::c().toDouble(x->text(),&validX),cy=QLocale::c().toDouble(y->text(),&validY);
        if(!validR||!validX||!validY||!std::isfinite(r)||r<=0||!std::isfinite(cx)||!std::isfinite(cy))throw ExchangeError("Circle needs a positive finite radius and finite center in mm");
        auto proposed=candidate();auto a=proposed["loops"].toArray();if(a.size()>=1024)throw ExchangeError("Native loop topology is limited to1024 boundaries");
        QJsonObject curve{{"class_name","ON_ArcCurve"},{"source_index",QJsonValue::Null},{"dimension",2},{"domain",QJsonArray{0,2*std::acos(-1.)}},
            {"angle_domain",QJsonArray{0,2*std::acos(-1.)}},{"radius",r},{"plane",QJsonArray{cx,cy,0,1,0,0,0,1,0,0,0,1,0,0,1,0}}};
        a.append(QJsonObject{{"type",role->currentIndex()},{"source_index",QJsonValue::Null},{"curve",curve}});proposed["loops"]=a;stage(proposed);
    }catch(const std::exception& e){error->setText(e.what());}}
    void removeLoop(){try{
        auto selected=tree->currentItem();if(!selected)throw ExchangeError("Select a boundary to remove");while(selected->parent())selected=selected->parent();int index=tree->indexOfTopLevelItem(selected);
        auto proposed=candidate();auto a=proposed["loops"].toArray();if(a.size()<=1)throw ExchangeError("A native Hatch requires at least one boundary");a.removeAt(index);proposed["loops"]=a;stage(proposed);
    }catch(const std::exception& e){error->setText(e.what());}}
public:
    Editor(const QJsonObject& input,const QString& initialError):QDialog(Gui::getMainWindow()),current(input){
        setObjectName("OM9HatchLoopEditor");setWindowTitle(tr("Edit native Hatch boundaries"));resize(850,650);
        auto layout=new QVBoxLayout(this);auto help=new QLabel(tr("Edit native values in the Value column. Coordinates and radius use millimeters; angles use radians. Source identities and native curve types are preserved."),this);help->setWordWrap(true);layout->addWidget(help);
        tree=new QTreeWidget(this);tree->setObjectName("loopFields");tree->setHeaderLabels({tr("Native field"),tr("Value")});tree->setEditTriggers(QAbstractItemView::DoubleClicked|QAbstractItemView::EditKeyPressed);layout->addWidget(tree);
        auto rows=new QHBoxLayout;
        auto duplicate=new QPushButton(tr("Duplicate selected row"),this);duplicate->setObjectName("duplicateLoopRow");rows->addWidget(duplicate);connect(duplicate,&QPushButton::clicked,this,[this]{editRow(true);});
        auto deleteRow=new QPushButton(tr("Remove selected row"),this);deleteRow->setObjectName("removeLoopRow");rows->addWidget(deleteRow);connect(deleteRow,&QPushButton::clicked,this,[this]{editRow(false);});layout->addLayout(rows);
        auto rowHelp=new QLabel(tr("Rows copy raw CV/knot/point/parameter/segment values. Edit paired arrays before OK; native validation checks the complete boundary."),this);rowHelp->setWordWrap(true);layout->addWidget(rowHelp);
        auto row=new QHBoxLayout;role=new QComboBox(this);role->addItems({tr("Outer"),tr("Inner")});role->setCurrentIndex(1);row->addWidget(role);
        for(auto entry:{std::pair<QLineEdit**,QString>{&radius,tr("Radius")},{&x,tr("Center X")},{&y,tr("Center Y")}}){row->addWidget(new QLabel(entry.second,this));*entry.first=new QLineEdit(entry.first==&radius?"0.5":"0",this);row->addWidget(*entry.first);}
        auto add=new QPushButton(tr("Add circle"),this);add->setObjectName("addCircle");row->addWidget(add);connect(add,&QPushButton::clicked,this,[this]{addCircle();});
        auto remove=new QPushButton(tr("Remove boundary"),this);remove->setObjectName("removeLoop");row->addWidget(remove);connect(remove,&QPushButton::clicked,this,[this]{removeLoop();});layout->addLayout(row);
        error=new QLabel(this);error->setObjectName("loopError");error->setWordWrap(true);error->setStyleSheet("color: #c62828");layout->addWidget(error);
        error->setText(initialError);
        auto buttons=new QDialogButtonBox(QDialogButtonBox::Ok|QDialogButtonBox::Cancel,this);layout->addWidget(buttons);
        connect(buttons,&QDialogButtonBox::rejected,this,&QDialog::reject);
        connect(buttons,&QDialogButtonBox::accepted,this,[this]{try{current=candidate();accept();}catch(const std::exception& e){error->setText(QString::fromUtf8(e.what()));}});
        populate();
    }
    QJsonObject value()const{return current;}
};
}
bool editHatchLoops(QJsonObject& value,const QString& error){
    if(value["schema_version"].toInt()!=1||!value["loops"].isArray()||value["loops"].toArray().isEmpty())throw ExchangeError("Unsupported native Hatch loop editor schema");
    // Detached values only. Host/native geometry preflight belongs after this
    // native call returns, with no dialog/widget frames alive on its stack.
    Editor dialog(value,error);if(dialog.exec()!=QDialog::Accepted)return false;value=dialog.value();return true;
}
}
