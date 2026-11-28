/*
 * WebTiger Core: the C ABI every shell links (docs/features/000-master-plan.md §5.5).
 *
 * The Apple shell imports it as the module WebTigerCore and links the static
 * library; the Windows shell includes it from C++ and links webtiger_core.dll.
 * Android doesn't include it: its JNI glue is Rust inside the same library
 * (src/android.rs, the `jni` feature), over these functions. It is written by
 * hand and kept honest by Core/crates/wt-ffi/tests/abi.rs, which checks every
 * declaration and constant here against the Rust exports.
 *
 * Messages are protobuf bytes of the contract in Core/proto/webtiger/core/v1:
 * an Envelope (envelope.proto) both ways, a CoreConfig (config.proto) to
 * start. Rules: nothing panics or throws across this boundary; changes within
 * a contract major only add; no address, title or typed text ever appears in a
 * log or an error.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at https://mozilla.org/MPL/2.0/.
 */
#ifndef WEBTIGER_CORE_H
#define WEBTIGER_CORE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* The contract this header goes with. wt_core_version() returns the library's
 * as (major << 16) | minor; a shell refuses to start on a major mismatch, and
 * puts its own in CoreConfig. */
#define WT_CONTRACT_MAJOR 1u
#define WT_CONTRACT_MINOR 57u

/* What wt_core_send, wt_core_stop and the fast path return. */
#define WT_OK 0
/* No running core has this handle: never started, or already stopped. */
#define WT_UNKNOWN_CORE 1
/* A NULL pointer where bytes or a structure were required. */
#define WT_NULL_ARGUMENT 2
/* A message longer than WT_MAX_MESSAGE_LEN. It was not read. */
#define WT_TOO_LARGE 3
/* The bytes are not an Envelope. */
#define WT_MALFORMED 4
/* wt_core_stop called from inside the sink: it would wait for itself. */
#define WT_WRONG_THREAD 5
/* A panic was caught inside the library. Nothing unwound into the caller. */
#define WT_PANIC 6
/* A fast-path argument that is not what it must be (an address that is not
 * UTF-8). The verdict written is the safe one. */
#define WT_BAD_ARGUMENT 7

/* The longest message either way: 4 MiB. */
#define WT_MAX_MESSAGE_LEN 4194304u

/* How many cores may run at once in one process. A shell runs one. */
#define WT_MAX_CORES 64u

/* Receives every envelope the core sends: Responses, Events and HostRequests.
 * Called on the core's own thread, one call at a time, in order. The bytes are
 * valid only during the call: copy them, hand them to your own thread, and
 * return. Do not call wt_core_stop from inside it. */
typedef void (*wt_sink)(void *context, const uint8_t *envelope, size_t len);

/* The contract the library implements: (major << 16) | minor. */
uint32_t wt_core_version(void);

/* Starts a core from an encoded CoreConfig and returns its handle, never 0.
 * On a refusal it returns 0, after delivering one Event to the sink:
 * CoreStatus STATE_FATAL with a code ("start.bad_config",
 * "start.contract_major", "start.no_engine", "start.data_dir", "store.open",
 * "store.newer", "start.too_many_cores"). On success the first envelope the
 * sink receives is CoreStatus STATE_READY. `sink` and `context` must stay
 * valid until wt_core_stop returns for the handle. */
uint64_t wt_core_start(const uint8_t *config, size_t len, wt_sink sink, void *context);

/* Hands the core an encoded Envelope: a Request or a HostResponse. Returns at
 * once; the answer arrives at the sink. Callable from any thread. */
int32_t wt_core_send(uint64_t core, const uint8_t *envelope, size_t len);

/* Stops a core and waits for its thread to end. The sink is not called after
 * this returns. Requests still waiting on the shell are dropped unanswered. */
int32_t wt_core_stop(uint64_t core);

/*
 * The fast path (000 §5.5): synchronous, and callable from any thread,
 * including an engine's own network and navigation threads. No call queues
 * behind the core's own thread, waits for the store, or waits for a shell:
 * each answers from what the core holds for it. The navigation check neither
 * locks nor allocates. Guard's check (064) parses the two addresses and looks
 * rules up: it allocates, and decisions are made one at a time behind a lock
 * of the blocker's own, held for the fraction of a microsecond one takes. The
 * storage check (071) parses its three addresses: it allocates, and locks
 * nothing.
 */

/* What a request is for, as far as the engine says. */
#define WT_RESOURCE_OTHER 0u
#define WT_RESOURCE_DOCUMENT 1u
#define WT_RESOURCE_SUBDOCUMENT 2u
#define WT_RESOURCE_SCRIPT 3u
#define WT_RESOURCE_STYLESHEET 4u
#define WT_RESOURCE_IMAGE 5u
#define WT_RESOURCE_FONT 6u
#define WT_RESOURCE_MEDIA 7u
#define WT_RESOURCE_XHR 8u
#define WT_RESOURCE_WEBSOCKET 9u
#define WT_RESOURCE_PING 10u

/* A request a page is about to make. Addresses are UTF-8, not NUL-terminated,
 * and borrowed for the call. */
typedef struct wt_request {
    const uint8_t *url;
    size_t url_len;
    /* The address of the page (the top frame) that asks. */
    const uint8_t *source_url;
    size_t source_url_len;
    uint64_t web_view_id;
    uint32_t resource_type;
} wt_request;

#define WT_VERDICT_ALLOW 0u
#define WT_VERDICT_BLOCK 1u

typedef struct wt_verdict {
    uint32_t action;
} wt_verdict;

/* Guard's decision for one request (064): by the core's rules, may the page
 * at `source_url` fetch `url`? With no rules every request is allowed, as is
 * the top frame's own document whatever the rules. On any error the verdict
 * written is ALLOW: a blocker that fails must not break the page. */
int32_t wt_guard_check(uint64_t core, const wt_request *request, wt_verdict *out);

#define WT_NAV_MAIN_FRAME 1u
#define WT_NAV_USER_GESTURE 2u

/* A navigation a web view is about to make. */
typedef struct wt_navigation {
    const uint8_t *url;
    size_t url_len;
    uint64_t web_view_id;
    /* WT_NAV_* flags, or'ed. */
    uint32_t flags;
} wt_navigation;

/* Let the web view load it. */
#define WT_NAV_ALLOW 0u
/* Refuse it. */
#define WT_NAV_CANCEL 1u
/* Not a page: refuse it in the web view and hand the address to the system. */
#define WT_NAV_EXTERNAL 2u

typedef struct wt_nav_verdict {
    uint32_t action;
} wt_nav_verdict;

/* Whether a web view may make a navigation. On any error the verdict written
 * is CANCEL. */
int32_t wt_nav_check(uint64_t core, const wt_navigation *navigation, wt_nav_verdict *out);

/* A frame whose document is starting, in a page. Origins and the address are
 * UTF-8, not NUL-terminated, and borrowed for the call. */
typedef struct wt_frame {
    /* The frame's origin, as its own document has it ("https://example.com";
     * "null" for one that is nobody's). */
    const uint8_t *origin;
    size_t origin_len;
    /* The origin of the top frame above it, as the frame's document has it. */
    const uint8_t *top_origin;
    size_t top_origin_len;
    /* The address of the page (the top frame), as the shell knows it. */
    const uint8_t *page_url;
    size_t page_url_len;
    uint64_t web_view_id;
} wt_frame;

/* The frame has its site's own storage. */
#define WT_STORAGE_OWN 0u
/* The frame is kept from its site's storage: what it keeps goes with its
 * document. */
#define WT_STORAGE_APART 1u

typedef struct wt_storage_verdict {
    uint32_t action;
} wt_storage_verdict;

/* Whether a frame has its site's own storage (071): for an engine that does
 * not keep a third party's storage apart by itself, asked as the frame's
 * document starts. OWN for a frame of the page's own site, and for any frame
 * where Guard is off for the page (067). APART for a frame of another site;
 * and for any frame whose top frame is not of the origin of the page the
 * shell knows, since a shell learns of a new page a moment after its engine
 * has it. On any error the verdict written is APART: a third party is let at
 * its storage on the core's word alone. */
int32_t wt_storage_check(uint64_t core, const wt_frame *frame, wt_storage_verdict *out);

#ifdef __cplusplus
}
#endif

#endif
