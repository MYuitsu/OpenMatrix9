// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include "CoreLayers.h"
#include "CoreCurveNativeInput.h"
#include <Python.h>
#include <QJsonObject>
#include <QByteArray>
#include <array>
#include <vector>
#include <string>
#include <cstdint>
namespace App {class Document;}
namespace OpenMatrix9Gui {
// GUI-thread bridge only. All expected geometry/metadata are owned values;
// document/feature pointers are resolved and checked afresh after callbacks.
class CurvePublication {
public:
    CurvePublication(App::Document&,const std::string& command,
        const std::string& construction,const std::string& units,const CircleLayerSnapshot&);
    ~CurvePublication(){abort();}
    App::Document& document() const;
    void check() const;
    void open();
    void prepare(PyObject* expectedShape,const std::string& name,const QJsonObject& metadata);
    void watch(PyObject* object,const std::string& name);
    void commit();
    void abort() noexcept;
private:
    struct Output {
        std::string name;long id;QByteArray digest;int orientation;
        std::array<double,16> frame;QJsonObject metadata;
    };
    void checkOutput(const Output&) const;
    const void* identity;
    std::string documentName,documentUid,command,units;
    std::uint64_t generation,revision,context;
    CircleLayerSnapshot layer;
    std::vector<std::shared_ptr<const CurveNativeSource>> sources;
    std::vector<Output> outputs;
    bool opened=false;
    int transactionId=0;
    bool transactionLocked=false;
};
}
