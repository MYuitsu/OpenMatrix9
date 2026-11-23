// SPDX-License-Identifier: LGPL-2.1-or-later
pub(crate) const MAX_BYTES: usize = 512 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Om9,
    Rhino3dm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Shortcut {
    Copy,
    Copy3dm,
    Paste,
}
