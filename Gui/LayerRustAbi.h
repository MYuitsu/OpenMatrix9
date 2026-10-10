// SPDX-License-Identifier: LGPL-2.1-or-later
#pragma once
#include <cstddef>
#include <cstdint>

// Schema 1. UTF-8 spans are readable only for a call; result bytes are copied
// into caller-owned buffers without NUL. No Rust/native internal pointer escapes.
struct Om9LayerByteView { const unsigned char* data; std::size_t len; };
struct Om9LayerView {
    Om9LayerByteView id, parent, name, path;
    unsigned char rgb[3], locked, visible;
    std::int8_t persistent_locked, persistent_visible;
    unsigned char reserved;
};
struct Om9LayerObjectView {
    Om9LayerByteView id, layer;
    unsigned char rgb[3], locked, visible, color_source, reserved[2];
};
struct Om9LayerLegacyObjectView {
    Om9LayerByteView id,path;
    unsigned char rgb[3],locked,visible,path_present,reserved[2];
};
static_assert(sizeof(Om9LayerLegacyObjectView)==40);
struct Om9LayerSnapshotView {
    std::uint32_t version, reserved;
    Om9LayerByteView document;
    std::uint64_t generation;
    Om9LayerByteView active;
    const Om9LayerView* layers;
    std::size_t layer_count;
    const Om9LayerObjectView* objects;
    std::size_t object_count;
};
struct Om9LayerEffective { unsigned char rgb[3], locked, visible, selectable, snap_eligible, reserved; };
struct Om9LayerCounts { std::size_t layer_count, object_count; std::uint64_t generation; std::size_t active_layer_index; };
struct Om9LayerInfo {
    unsigned char rgb[3], locked, visible;
    std::int8_t persistent_locked, persistent_visible;
    unsigned char reserved;
};
struct Om9LayerObjectInfo { unsigned char rgb[3], locked, visible, color_source, reserved[2]; };
struct Om9LayerClipboardInfo {
    std::uint32_t version, evidence, scope, geometry_version;
    std::uint64_t geometry_length, metadata_length;
};
struct Om9LayerGeometryBindingView { Om9LayerByteView physical, source; };
static_assert(sizeof(Om9LayerClipboardInfo) == 32);
static_assert(sizeof(Om9LayerEffective) == 8);
static_assert(sizeof(Om9LayerInfo) == 8);
static_assert(sizeof(Om9LayerObjectInfo) == 8);

extern "C" {
// Version1 per-layer native presence witness; fields -1 unset, 0 false, 1 true.
std::uint32_t om9_layer_native_persistent_encode(unsigned char child,unsigned char locked,unsigned char visible,std::int8_t persistent_locked,std::int8_t persistent_visible,unsigned char*,std::size_t,std::size_t*);
std::uint32_t om9_layer_native_persistent_decode(unsigned char child,unsigned char locked,unsigned char visible,Om9LayerByteView,Om9LayerInfo*);
// Return 0 on success; LayerError values 1..17 otherwise. Create outputs reset
// to 0 on error. Every snapshot/plan/binding handle must use its own free function.
// Clipboard size kinds: 1 geometry, 2 metadata. Scope1 Selected, scope2 Session.
// Prepare stores only metadata; bytes queries/copies never rehash geometry.
std::uint32_t om9_layer_clipboard_check_size(std::uint32_t, std::size_t);
std::uint32_t om9_layer_clipboard_check_backup(std::size_t bytes, std::size_t formats);
unsigned char om9_layer_clipboard_rollback_owned(std::uint32_t before, std::uint32_t published, std::uint32_t current);
std::uint32_t om9_layer_clipboard_prepare(std::uint64_t, std::uint32_t, Om9LayerByteView, std::uint64_t*);
std::uint32_t om9_layer_clipboard_bytes(std::uint64_t, unsigned char*, std::size_t, std::size_t*);
std::uint32_t om9_layer_clipboard_free(std::uint64_t);
// Evidence1 native-only (snapshot0, scope0), evidence2 extended. Present metadata
// must match exact geometry bytes; no downgrade on error. Outputs reset on error.
// Native SDK parsing and object bindings are required before document mutation.
std::uint32_t om9_layer_clipboard_receive(Om9LayerByteView, Om9LayerByteView, unsigned char, std::uint64_t*, Om9LayerClipboardInfo*);
// Native facts from the digest-verified archive. Missing/duplicate tags reject.
// Returns a snapshot whose object IDs match the prepared physical geometry IDs.
std::uint32_t om9_layer_clipboard_bind_objects(std::uint64_t, const Om9LayerGeometryBindingView*, std::size_t, std::uint64_t*);
// Detached retained archive palette + exact selected object bindings. Destination
// snapshot contains no objects; returned snapshot owns source context/physical IDs.
std::uint32_t om9_layer_retained_overlay(std::uint64_t, std::uint64_t, const Om9LayerGeometryBindingView*, std::size_t, std::uint64_t*);
// Full palette plus exact portable object subset; original IDs/context retained.
std::uint32_t om9_layer_snapshot_subset(std::uint64_t,const Om9LayerByteView*,std::size_t,std::uint64_t*);
// Native-verified helper IDs only; semantic inventory, no geometry deletion.
// Preserves generation/palette/active and resets output on invalid input.
std::uint32_t om9_layer_snapshot_filter_metadata(std::uint64_t,const Om9LayerByteView*,std::size_t,std::uint64_t*);
std::uint32_t om9_layer_snapshot_create(const Om9LayerSnapshotView*, std::uint64_t*);
// Native archive compatibility only: normalize legacy embedded path leaf names.
// Canonical snapshots and JSON always validate strictly without implicit repair.
std::uint32_t om9_layer_snapshot_create_native(const Om9LayerSnapshotView*, std::uint64_t*);
std::uint32_t om9_layer_snapshot_free(std::uint64_t);
// Owned versioned JSON envelope, geometry-free. Unknown fields/version reject.
std::uint32_t om9_layer_snapshot_from_json(Om9LayerByteView, std::uint64_t*);
std::uint32_t om9_layer_snapshot_json(std::uint64_t, unsigned char*, std::size_t, std::size_t*);
std::uint32_t om9_layer_snapshot_effective(std::uint64_t, Om9LayerByteView, Om9LayerEffective*);
// Operation: 1 edit, 2 transform, 3 delete, 4 assign(destination layer ID), 5 export.
std::uint32_t om9_layer_snapshot_can_mutate(std::uint64_t, const Om9LayerByteView*, std::size_t, std::uint32_t, Om9LayerByteView);
std::uint32_t om9_layer_snapshot_counts(std::uint64_t, Om9LayerCounts*);
std::uint32_t om9_layer_snapshot_layer(std::uint64_t, std::size_t, Om9LayerInfo*);
std::uint32_t om9_layer_snapshot_object(std::uint64_t, std::size_t, Om9LayerObjectInfo*);
// kind 0 document/active (fields0/1), 1 layer ID/parent/name/path (fields0..3),
// 2 object ID/layer (fields0/1). BufferTooSmall returns required UTF-8 byte count.
std::uint32_t om9_layer_snapshot_text(std::uint64_t, std::uint32_t, std::size_t, std::uint32_t, unsigned char*, std::size_t, std::size_t*);
// Scope1 selected (count0 is palette-only), scope2 session requires count0.
std::uint32_t om9_layer_plan_receive(std::uint64_t, std::uint64_t, std::uint32_t, const Om9LayerByteView*, std::size_t, std::uint64_t*);
// Strict command JSON: set_current, assign, set_locked, set_visible, set_color.
// Returns the same owned before/after plan type as receive; no geometry in Rust.
std::uint32_t om9_layer_document_plan(std::uint64_t, Om9LayerByteView, std::uint64_t*);
std::uint32_t om9_layer_document_default(Om9LayerByteView,std::uint64_t*);
std::uint32_t om9_layer_document_legacy(Om9LayerByteView,const Om9LayerLegacyObjectView*,std::size_t,std::uint64_t*);
std::uint32_t om9_layer_document_reconcile(std::uint64_t,const Om9LayerLegacyObjectView*,std::size_t,std::uint64_t*);
std::uint32_t om9_layer_document_observed(std::uint64_t,const Om9LayerByteView*,std::size_t,std::uint64_t*);
// Geometry-free panel with cached Rust effective flags; caller owns UTF-8 bytes.
std::uint32_t om9_layer_document_panel(std::uint64_t,unsigned char*,std::size_t,std::size_t*);
std::uint32_t om9_layer_document_text(std::uint64_t,Om9LayerByteView,const Om9LayerByteView*,std::size_t,std::uint64_t*);
std::uint32_t om9_layer_plan_free(std::uint64_t);
std::uint32_t om9_layer_plan_validate(std::uint64_t, std::uint64_t);
std::uint32_t om9_layer_plan_after_snapshot(std::uint64_t, std::uint64_t*);
// kind1 layer mapping, kind2 object mapping; source input is an ID, never a path.
std::uint32_t om9_layer_plan_map(std::uint64_t, std::uint32_t, Om9LayerByteView, unsigned char*, std::size_t, std::size_t*);
}
