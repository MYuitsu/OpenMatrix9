#include "ThreeDmStaging.h"
#include <QCoreApplication>
#include <QFile>
#include <QJsonDocument>
#include <Standard_Failure.hxx>
using namespace OpenMatrix9Gui::ThreeDm;
int main(int argc,char** argv){
    QCoreApplication app(argc,argv);
    try{
        if(app.arguments().size()!=3)throw ExchangeError("Expected request and result paths");
        QFile request(app.arguments()[1]);if(!request.open(QIODevice::ReadOnly)||request.size()>8*1024*1024)throw ExchangeError("Invalid worker request size");
        QJsonParseError error;auto json=QJsonDocument::fromJson(request.readAll(),&error);
        if(error.error!=QJsonParseError::NoError||!json.isObject())throw ExchangeError("Invalid worker request JSON");
        auto row=json.object();std::set<std::string> subset;
        for(auto value:row["ids"].toArray()){auto id=value.toString();if(id.size()!=36||!subset.insert(id.toStdString()).second)throw ExchangeError("Invalid worker subset");}
        auto model=readArchiveSubset(std::filesystem::path(row["source"].toString().toStdWString()),row["scale"].toDouble(),row["members"].toBool(),subset);
        QJsonArray rows;int index=0;auto staging=std::filesystem::path(row["staging"].toString().toStdWString());
        for(auto& item:model.items){auto value=encodeExchangeItem(item,staging,index++);value["tolerance"]=model.tolerance;rows.append(value);}
        if(rows.size()!=subset.size())throw ExchangeError("Worker subset count changed");
        auto bytes=QJsonDocument(QJsonObject{{"rows",rows}}).toJson(QJsonDocument::Compact);
        QFile output(app.arguments()[2]);if(!output.open(QIODevice::WriteOnly|QIODevice::NewOnly)||output.write(bytes)!=bytes.size())throw ExchangeError("Cannot write worker result");
        output.close();return 0;
    }catch(const Standard_Failure& error){fprintf(stderr,"%s\n",error.GetMessageString());}
     catch(const std::exception& error){fprintf(stderr,"%s\n",error.what());}
    return 1;
}
