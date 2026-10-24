/* OM9 SDK v1. Native plugins are trusted code, not a sandbox.
 * Copyright OpenMatrix9 contributors. SPDX-License-Identifier: LGPL-2.1-or-later
 *
 * All messages are UTF-8 JSON, at most OM9_MAX_MESSAGE_BYTES. The callee sends
 * exactly one reply before returning, and the receiver copies the bytes during
 * the callback. Retaining contexts or data pointers is forbidden. Calls stay on
 * the invoking thread; no callback may throw/unwind across this interface.
 * Each allocator frees only its own allocations. Successful replies contain
 * the requested JSON value; OM9_STATUS_ERROR replies contain {code,message}.
 */
#ifndef OM9_SDK_H
#define OM9_SDK_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define OM9_ABI_VERSION UINT32_C(1)
#define OM9_MAX_MESSAGE_BYTES ((size_t)8 * 1024 * 1024)
#define OM9_STATUS_OK INT32_C(0)
#define OM9_STATUS_ERROR INT32_C(1)

typedef int32_t (*Om9ReplyFn)(void *context, const uint8_t *data, size_t len);
typedef int32_t (*Om9CallFn)(void *context, const uint8_t *data, size_t len,
                           void *reply_context, Om9ReplyFn reply);

typedef struct Om9HostApi {
    uint32_t abi_version;
    uint32_t struct_size;
    void *context;
    Om9CallFn call;
} Om9HostApi;

typedef int32_t (*Om9PluginEntryV1)(const Om9HostApi *host,
                                   const uint8_t *request, size_t request_len,
                                   void *reply_context, Om9ReplyFn reply);

/* Export this exact symbol from a native plugin. The Rust export_plugin! macro
 * supplies it. Keep the library loaded while any of its code can be called. */
#define OM9_PLUGIN_ENTRY_SYMBOL "om9_plugin_entry_v1"

#ifdef __cplusplus
}
#endif
#endif
