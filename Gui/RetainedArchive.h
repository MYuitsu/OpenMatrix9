// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <filesystem>
#include <string>
#include <vector>
struct _object;
namespace OpenMatrix9Gui {
struct RetainedArchiveRecord {
    std::filesystem::path file;
    std::string sourceUuid,sourceClass,importNamespace;
    double scaleMm=0;
    std::vector<unsigned char> archiveBytes; // owned exact bytes checked against archive hash
};
// Resolve exactly one source container and verify its payload/manifest/identity.
// Typed decoders consume archiveBytes without reopening file (original metadata).
// The returned snapshot remains immutable; caller performs typed decoding before
// entering a native document transaction.
RetainedArchiveRecord verifiedRetainedArchive(_object* source);
}
