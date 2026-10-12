// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <QJsonObject>
#include <cstddef>
#include <cstdint>
#include <stdexcept>
namespace App {class DocumentObject;}
namespace Mesh {class MeshObject;}
namespace OpenMatrix9Gui {
// Caller holds GUI-thread native ownership and revalidates its source snapshot.
QJsonObject discreteSnapshot(App::DocumentObject&,bool mesh);
// Same checked reader for a live exact native MeshObject before publication.
QJsonObject meshSnapshot(const Mesh::MeshObject&);
struct MeshInputBudget {std::size_t nodes=0,bytes=0;};
struct MeshInputFailure:std::runtime_error {
    std::uint32_t status;
    MeshInputFailure(std::uint32_t code,const char* message):std::runtime_error(message),status(code){}
};
// Existing Rust count/value/index/owner/byte policy before a native mesh copy.
// Caller adds these counts to its cumulative request budget when applicable.
MeshInputBudget meshInputBudget(const Mesh::MeshObject&);
}
