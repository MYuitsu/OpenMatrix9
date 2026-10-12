// SPDX-License-Identifier: LGPL-2.1-or-later
#include "LayerClipboard.h"
#include "ThreeDmArchive.h"
#include <QApplication>
#include <QClipboard>
#include <QFile>
#include <QMimeData>
#include <QJsonDocument>
#include <QJsonObject>
#include <QUuid>
#include <QThread>
#include <QImage>
#include <thread>
#include <atomic>
#include <Windows.h>
#include <ole2.h>
#include <memory>
#include <iostream>
using namespace OpenMatrix9Gui::ThreeDm;
namespace {
int checks=0;
void check(bool value,const char* text){++checks;if(!value)throw std::runtime_error(text);}
struct RestoreClipboard {
    std::unique_ptr<QMimeData> previous{new QMimeData};DWORD owned=0;
    RestoreClipboard(){auto old=QApplication::clipboard()->mimeData();if(old){for(const auto& name:old->formats())previous->setData(name,old->data(name));if(old->hasImage())previous->setImageData(old->imageData());}}
    void remember(){owned=GetClipboardSequenceNumber();}
    ~RestoreClipboard(){if(owned&&owned==GetClipboardSequenceNumber()){QApplication::clipboard()->setMimeData(previous.release());OleFlushClipboard();}}
};
void setPair(const QByteArray& geometry,const QByteArray& metadata){auto mime=new QMimeData;mime->setData(layerClipboardGeometryMime(),geometry);mime->setData(layerClipboardMetadataMime(),metadata);QApplication::clipboard()->setMimeData(mime);}
template<class F>bool rejects(F&& work){try{work();}catch(const std::exception&){return true;}return false;}
bool nativeTextEquals(const QString& expected){
    for(int attempt=0;attempt<20;++attempt){
        if(OpenClipboard(nullptr)){
            QString text;bool readable=false;auto handle=GetClipboardData(CF_UNICODETEXT);
            if(handle){const auto capacity=GlobalSize(handle)/sizeof(wchar_t);auto pointer=static_cast<const wchar_t*>(GlobalLock(handle));
                if(pointer){std::size_t count=0;while(count<capacity&&pointer[count]!=0)++count;if(count<capacity){text=QString::fromWCharArray(pointer,static_cast<qsizetype>(count));readable=true;}GlobalUnlock(handle);}}
            CloseClipboard();if(readable&&text==expected)return true;
        }
        QThread::msleep(5);
    }
    return false;
}
}
int main(int argc,char** argv){QApplication app(argc,argv);RestoreClipboard restore;try{
    const auto directory=std::filesystem::path(OM9_LAYER_FIXTURES);
    auto source=readArchive(directory/"layer-source.3dm");std::vector<NativeObjectLayerRow> rows;for(const auto& item:source.items)rows.push_back(*item.ownState);
    const auto token=QUuid::createUuid().toString(QUuid::WithoutBraces);
    auto snapshot=createNativeLayerSnapshot(source.layerTable,rows,token.toStdString(),99);
    QFile file(QString::fromStdWString((directory/"layer-session.3dm").wstring()));check(file.open(QIODevice::ReadOnly),"Read actual 3DM fixture");const auto geometry=file.readAll();file.close();
    auto info=publishLayerClipboard(geometry,snapshot.get(),2);restore.remember();
    check(info.evidence==2&&info.scope==2,"Publish both formats with extended evidence");
    auto captured=captureLayerClipboard();
    check(captured.geometry==geometry&&captured.metadataPresent,"Exact geometry and metadata paired capture");
    check(captured.info.evidence==2&&captured.snapshot.get()!=0,"Owned Rust received snapshot");
    Om9LayerCounts counts{};check(om9_layer_snapshot_counts(captured.snapshot.get(),&counts)==0&&counts.layer_count==4&&counts.object_count==2,"Full empty palette survives clipboard");
    auto prepared=readArchive(directory/"layer-session.3dm");applyExtendedLayerOverlay(prepared,captured.snapshot.get());
    check(prepared.layerTable.activeId==source.layerTable.activeId&&!prepared.items[0].ownState->locked&&prepared.items[0].locked,"Exact native tag overlay after paired clipboard");
    const auto metadata=captured.metadata;
    std::atomic<bool> offThreadCapture=false,offThreadPublish=false;
    std::thread worker([&]{offThreadCapture=rejects([]{captureLayerClipboard();});offThreadPublish=rejects([&]{publishLayerClipboard(geometry,snapshot.get(),2);});});worker.join();
    check(offThreadCapture&&offThreadPublish,"Native clipboard rejects operations away from GUI thread");
    auto wrong=geometry;wrong[wrong.size()-1]^=1;setPair(wrong,metadata);restore.remember();
    check(rejects([]{captureLayerClipboard();}),"Mismatched digest rejects extended payload");
    auto unknown=QJsonDocument::fromJson(metadata).object();unknown["version"]=2;setPair(geometry,QJsonDocument(unknown).toJson(QJsonDocument::Compact));restore.remember();
    check(rejects([]{captureLayerClipboard();}),"Unknown metadata version rejects without downgrade");
    setPair(geometry,QByteArray{});restore.remember();check(rejects([]{captureLayerClipboard();}),"Present empty metadata never native-only downgrade");
    publishLayerClipboard(geometry,0,0);restore.remember();auto nativeOnly=captureLayerClipboard();
    check(!nativeOnly.metadataPresent&&nativeOnly.info.evidence==1&&nativeOnly.snapshot.get()==0,"Native-only clipboard reduced evidence explicit");
    const auto previous="previous-"+token;QApplication::clipboard()->setText(previous);restore.remember();
    int flushCalls=0;
    check(rejects([&]{publishLayerClipboard(geometry,snapshot.get(),1,[&]{++flushCalls;return static_cast<long>(E_FAIL);});})&&flushCalls==1,"Actual Qt publication with flush failure rejects");restore.remember();
    check(nativeTextEquals(previous),"Owned failure restores original text clipboard");
    const auto newer="newer-owner-"+token;
    flushCalls=0;
    check(rejects([&]{publishLayerClipboard(geometry,snapshot.get(),1,[&]{++flushCalls;QApplication::clipboard()->setText(newer);return static_cast<long>(E_FAIL);});})&&flushCalls==1,"Foreign replacement during flush failure rejects");restore.remember();
    check(nativeTextEquals(newer),"Failure never overwrites newer owner");
    const auto newerSuccess="newer-success-"+token;
    flushCalls=0;
    check(rejects([&]{publishLayerClipboard(geometry,snapshot.get(),1,[&]{++flushCalls;QApplication::clipboard()->setText(newerSuccess);return static_cast<long>(S_OK);});})&&flushCalls==1,"New owner during successful flush cannot be claimed as our publication");restore.remember();
    check(nativeTextEquals(newerSuccess),"Successful flush with foreign replacement preserves new owner");
    check(rejects([&]{publishLayerClipboard(QByteArray(32,'x'),snapshot.get(),1);}),"Invalid geometry rejects before publication");
    check(nativeTextEquals(newerSuccess),"Invalid preflight keeps previous clipboard");
    publishLayerClipboard(geometry,snapshot.get(),2);restore.remember();auto priorPair=captureLayerClipboard();
    flushCalls=0;
    check(rejects([&]{publishLayerClipboard(geometry,snapshot.get(),1,[&]{++flushCalls;return static_cast<long>(E_FAIL);});})&&flushCalls==1,"Failed replacement of prior extended clipboard rejects");restore.remember();
    auto restoredPair=captureLayerClipboard();check(restoredPair.geometry==priorPair.geometry&&restoredPair.metadata==priorPair.metadata&&restoredPair.info.scope==2,"Both original formats restored exactly");
    QImage previousImage(16,12,QImage::Format_ARGB32);previousImage.fill(qRgb(12,23,244));QApplication::clipboard()->setImage(previousImage);restore.remember();
    flushCalls=0;
    check(rejects([&]{publishLayerClipboard(geometry,snapshot.get(),1,[&]{++flushCalls;return static_cast<long>(E_FAIL);});})&&flushCalls==1,"Failed replacement of image clipboard rejects");restore.remember();
    bool imageRestored=false;for(int i=0;i<20;++i){if(QApplication::clipboard()->image()==previousImage){imageRestored=true;break;}QThread::msleep(5);}
    check(imageRestored,"Original clipboard image restored");
    std::cout<<"Layer clipboard PASS; checks="<<checks<<"; actual_3dm_fixtures=1\n";return 0;
}catch(const std::exception& error){std::cerr<<"Layer clipboard FAIL after "<<checks<<" checks: "<<error.what()<<'\n';return 1;}}
