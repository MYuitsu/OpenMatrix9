// SPDX-License-Identifier: LGPL-2.1-or-later
#include "LayerClipboard.h"
#include <QGuiApplication>
#include <QClipboard>
#include <QMimeData>
#include <QImage>
#include <QThread>
#include <memory>
#include <stdexcept>
#ifdef _WIN32
#include <Windows.h>
#include <ole2.h>
#endif
namespace OpenMatrix9Gui::ThreeDm {
QString layerClipboardGeometryMime(){return QStringLiteral("application/x-qt-windows-mime;value=\"Rhino 5.0 3DM Clip global mem\"");}
QString layerClipboardMetadataMime(){return QStringLiteral("application/x-qt-windows-mime;value=\"OM9.LayerSession.v1\"");}
namespace {
void requireGuiThread(){auto app=QGuiApplication::instance();if(!app||QThread::currentThread()!=app->thread())throw std::runtime_error("Clipboard requires the GUI thread");}
void rustCheck(std::uint32_t code,const char* operation){if(code)throw std::runtime_error(std::string(operation)+": Rust layer code "+std::to_string(code));}
Om9LayerByteView bytes(const QByteArray& value){return {reinterpret_cast<const unsigned char*>(value.constData()),static_cast<std::size_t>(value.size())};}
#ifdef _WIN32
struct ClipboardBusy:std::runtime_error{ClipboardBusy():std::runtime_error("Windows clipboard is busy; try again") {}};
struct ClipboardLock {
    explicit ClipboardLock(int attempts=1){for(int attempt=0;attempt<attempts;++attempt){if(OpenClipboard(nullptr))return;if(attempt+1<attempts)QThread::msleep(5);}throw ClipboardBusy();}
    ~ClipboardLock(){CloseClipboard();}
};
struct GlobalReadLock {HGLOBAL handle;const void* pointer;explicit GlobalReadLock(HGLOBAL h):handle(h),pointer(GlobalLock(h)){if(!pointer)throw std::runtime_error("Cannot lock clipboard format");}~GlobalReadLock(){GlobalUnlock(handle);}};
QByteArray readFormat(UINT format,std::uint32_t kind){
    auto handle=GetClipboardData(format);if(!handle)throw std::runtime_error("Cannot read native clipboard format");
    const auto size=GlobalSize(handle);rustCheck(om9_layer_clipboard_check_size(kind,size),"Clipboard format size");
    if(size==0)return {};
    GlobalReadLock locked(handle);return QByteArray(static_cast<const char*>(locked.pointer),static_cast<qsizetype>(size));
}
LayerClipboardCapture captureRawPair(){
    const auto geometryFormat=RegisterClipboardFormatW(L"Rhino 5.0 3DM Clip global mem");
    const auto metadataFormat=RegisterClipboardFormatW(L"OM9.LayerSession.v1");
    if(!geometryFormat||!metadataFormat)throw std::runtime_error("Cannot register clipboard formats");
    for(int attempt=0;attempt<2;++attempt){
        LayerClipboardCapture result;std::uint32_t after=0;
        {
            ClipboardLock locked;result.sequence=GetClipboardSequenceNumber();
            if(!IsClipboardFormatAvailable(geometryFormat))throw std::runtime_error("Clipboard has no Rhino5 geometry");
            result.geometry=readFormat(geometryFormat,1);
            result.metadataPresent=IsClipboardFormatAvailable(metadataFormat)!=FALSE;
            if(result.metadataPresent)result.metadata=readFormat(metadataFormat,2);
            after=GetClipboardSequenceNumber();
        }
        if(result.sequence!=0&&result.sequence==after&&after==GetClipboardSequenceNumber())return result;
    }
    throw std::runtime_error("Clipboard changed during paired capture; try again");
}
void verifyPublishedPair(const QByteArray& geometry,const QByteArray& metadata,bool present){
    for(int attempt=0;attempt<5;++attempt){
        try{
            auto actual=captureRawPair();
            if(actual.geometry!=geometry||actual.metadataPresent!=present||(present&&actual.metadata!=metadata))throw std::runtime_error("Native clipboard publication did not retain the exact format pair");
            return;
        }catch(const ClipboardBusy&){if(attempt==4)throw;QThread::msleep(5);}
    }
}
std::unique_ptr<QMimeData> backupClipboard(QClipboard* clipboard){
    auto backup=std::make_unique<QMimeData>();const auto old=clipboard->mimeData();if(!old)return backup;
    const auto formats=old->formats();std::size_t total=0;
    rustCheck(om9_layer_clipboard_check_backup(0,static_cast<std::size_t>(formats.size())),"Clipboard backup formats");
    for(const auto& name:formats){
        const auto value=old->data(name);
        total+=static_cast<std::size_t>(value.size());
        rustCheck(om9_layer_clipboard_check_backup(total,static_cast<std::size_t>(formats.size())),"Clipboard backup bytes");
        backup->setData(name,value);
    }
    if(old->hasImage()){
        const auto image=old->imageData();
        const auto nativeImage=qvariant_cast<QImage>(image);
        total+=static_cast<std::size_t>(nativeImage.sizeInBytes());
        rustCheck(om9_layer_clipboard_check_backup(total,static_cast<std::size_t>(formats.size())),"Clipboard backup image");
        backup->setImageData(image);
    }
    return backup;
}
#endif
}
LayerClipboardCapture captureLayerClipboard(){
    requireGuiThread();
#ifdef _WIN32
    auto result=captureRawPair();std::uint64_t handle=0;
    rustCheck(om9_layer_clipboard_receive(bytes(result.geometry),bytes(result.metadata),result.metadataPresent?1:0,&handle,&result.info),"Clipboard payload validation");
    result.snapshot=NativeLayerSnapshot(handle);return result;
#else
    throw std::runtime_error("Rhino5 native clipboard requires Windows");
#endif
}
Om9LayerClipboardInfo publishLayerClipboard(const QByteArray& geometry,std::uint64_t sourceSnapshot,std::uint32_t scope,const LayerClipboardFlush& flush){
    requireGuiThread();
#ifdef _WIN32
    QByteArray metadata;
    if(sourceSnapshot){
        std::uint64_t handle=0;rustCheck(om9_layer_clipboard_prepare(sourceSnapshot,scope,bytes(geometry),&handle),"Clipboard prepare");
        struct Binding {std::uint64_t value;~Binding(){om9_layer_clipboard_free(value);}} binding{handle};
        std::size_t needed=0;const auto query=om9_layer_clipboard_bytes(handle,nullptr,0,&needed);
        if(query!=15)rustCheck(query,"Clipboard metadata size query");
        metadata=QByteArray(static_cast<qsizetype>(needed),Qt::Uninitialized);
        rustCheck(om9_layer_clipboard_bytes(handle,reinterpret_cast<unsigned char*>(metadata.data()),static_cast<std::size_t>(metadata.size()),&needed),"Clipboard metadata copy");
    }
    // Header/version facts come from Rust; source0 requires this preflight too.
    std::uint64_t unused=0;Om9LayerClipboardInfo info{};
    rustCheck(om9_layer_clipboard_receive(bytes(geometry),{nullptr,0},0,&unused,&info),"Clipboard geometry preflight");
    NativeLayerSnapshot headerSnapshot(unused);
    if(sourceSnapshot){info.evidence=2;info.scope=scope;info.metadata_length=static_cast<std::uint64_t>(metadata.size());}
    {ClipboardLock available(5);}
    auto clipboard=QGuiApplication::clipboard();const auto before=GetClipboardSequenceNumber();
    auto previous=backupClipboard(clipboard);
    if(before==0||before!=GetClipboardSequenceNumber())throw std::runtime_error("Clipboard changed during backup; try again");
    auto mime=std::make_unique<QMimeData>();mime->setData(layerClipboardGeometryMime(),geometry);
    if(sourceSnapshot)mime->setData(layerClipboardMetadataMime(),metadata);
    clipboard->setMimeData(mime.release());const auto published=GetClipboardSequenceNumber();
    try{
        verifyPublishedPair(geometry,metadata,sourceSnapshot!=0);
        const auto code=flush?flush():static_cast<long>(OleFlushClipboard());
        if(code!=0)throw std::runtime_error("Cannot retain clipboard data after application exit");
        // Flush may materialize formats and change the sequence. Verify in that
        // case, while a later foreign owner must never be claimed as success.
        if(GetClipboardSequenceNumber()!=published)verifyPublishedPair(geometry,metadata,sourceSnapshot!=0);
        return info;
    }catch(...){
        if(om9_layer_clipboard_rollback_owned(before,published,GetClipboardSequenceNumber())){
            clipboard->setMimeData(previous.release());
            if(OleFlushClipboard()!=S_OK)throw std::runtime_error("Clipboard publication failed and previous data could not be retained");
        }
        throw;
    }
#else
    (void)geometry;(void)sourceSnapshot;(void)scope;(void)flush;
    throw std::runtime_error("Rhino5 native clipboard requires Windows");
#endif
}
}
