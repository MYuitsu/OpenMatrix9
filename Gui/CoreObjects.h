// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <Python.h>
#include <array>
#include <cstdint>
#include <string>
#include <vector>
namespace App { class Document; class DocumentObject; }
namespace OpenMatrix9Gui {
// Fixed-width Rust ABI, never an address. Generations are process-session scoped.
struct ObjectStamp {
    std::uint64_t document=0, object=0, generation=0, revision=0, epoch=0;
};
struct ObjectSnapshot {
    ObjectStamp stamp;
    std::string objectName, subelement, documentUid, unitContext;
    std::array<double,16> world{};
    std::vector<ObjectStamp> ancestors;
    bool captured() const { return stamp.document != 0; }
};
// GUI-thread native capture. No document mutation or transaction; no raw host
// pointers are retained by Rust. A subelement is valid only at this revision.
ObjectSnapshot captureObject(App::Document&, const std::string& name,
                             const std::string& subelement={});
// GUI-thread process identity; changes on close/reopen even with the same Uid.
std::uint64_t documentSession(App::Document&);
ObjectStamp objectIdentity(App::DocumentObject&);
bool validateObjectIdentity(App::Document&,const std::string&,const ObjectStamp&);
void verifyObject(App::Document&, const ObjectSnapshot&);
void addObjectSnapshotMethods(PyObject* module);
}
