// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "LayerExchangeAdapter.h"
#include <QByteArray>
#include <QString>
#include <functional>
namespace OpenMatrix9Gui::ThreeDm {
struct LayerClipboardCapture {
    QByteArray geometry,metadata;
    bool metadataPresent=false;
    std::uint32_t sequence=0;
    Om9LayerClipboardInfo info{};
    NativeLayerSnapshot snapshot;
};
QString layerClipboardGeometryMime();
QString layerClipboardMetadataMime();
// GUI thread only. Both HGLOBAL formats are copied under one clipboard lock;
// Rust validates bounds/identity after unlocking. SDK parsing remains required.
LayerClipboardCapture captureLayerClipboard();
// A source snapshot0 explicitly publishes native-only; scope1/2 otherwise.
// Flush injection is for native failure tests; default invokes real OleFlushClipboard.
using LayerClipboardFlush=std::function<long()>;
Om9LayerClipboardInfo publishLayerClipboard(const QByteArray& geometry,
    std::uint64_t sourceSnapshot,std::uint32_t scope,const LayerClipboardFlush& flush={});
}
