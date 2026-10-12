// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <QString>
namespace OpenMatrix9Gui {
// Host lifetimes/selection only. Rust resolves paths, plans and guards changes.
class LayerController {
public:
    static QString panel();
    static bool submit(const QString&);
    static void setCurrent(const QString&);
    static void assignSelection(const QString&);
    static void setLocked(const QString&,bool);
    static void setVisible(const QString&,bool);
};
}
