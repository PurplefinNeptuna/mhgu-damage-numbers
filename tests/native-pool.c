// Execute the pinned Forge allocator with host JIT/TLS/cache doubles. ARM
// barriers are omitted by the runner; this is allocation evidence, not gameplay.
#include "mock.h"
#include <assert.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "native-hook.h"

static unsigned created, closed, flushed, invalidated;
static bool fail_create, wrong_type;
Result jitCreate(Jit* jit, size_t bytes) {
    if (fail_create) return 1;
    jit->rw_addr = calloc(1, bytes);
    assert(jit->rw_addr);
    jit->rx_addr = jit->rw_addr;
    jit->size = bytes;
    jit->type = wrong_type ? 0 : JitType_CodeMemory;
    ++created;
    return 0;
}
Result jitClose(Jit* jit) { free(jit->rw_addr); ++closed; return 0; }
Result nnosAllocateTlsSlot(TlsSlot* slot, void* callback) { *slot = 1; return 0; }
void nnosSetTlsValue(TlsSlot slot, void* value) {}
void* nnosGetTlsValue(TlsSlot slot) { return NULL; }
void forge_nnosInitializeMutex(void* mutex, bool recursive) {}
void forge_nnosLockMutex(void* mutex) {}
void forge_nnosUnlockMutex(void* mutex) {}
void forge_log_error(const char* format, ...) {}
void armDCacheFlush(void* address, size_t bytes) { ++flushed; }
void armICacheInvalidate(void* address, size_t bytes) { ++invalidated; }

#include "native-hook-host.c"

// Clear the static fixture allocator between independent cases. Production
// never reclaims these chunks; no reset function is added to shipped code.
static void reset(void) {
    for (int i = 0; i < s_jitChunkCount; ++i) jitClose(&s_jitChunks[i].jit);
    assert(closed == created);
    memset(s_jitChunks, 0, sizeof(s_jitChunks));
    s_jitChunkCount = 0;
    created = closed = flushed = invalidated = 0;
    fail_create = wrong_type = false;
}
int main(void) {
    assert(forge_hook_init() == 0);
    /* Production installs eleven ordinary hooks (hooks.rs COUNT). */
    uint32_t targets[11][2];
    for (unsigned i = 0; i < 11; ++i) {
        targets[i][0] = 0xe92d4010;
        targets[i][1] = 0xe1a04000;
        void* original = NULL;
        Hook hook = forge_hook_create(targets[i], (void*)(uintptr_t)0x30000000, &original);
        assert(original && hook.original == original);
        assert(targets[i][0] == 0xe51ff004 && targets[i][1] == 0x30000000);
        uint32_t* trampoline = original;
        assert(trampoline[0] == 0xe92d4010 && trampoline[1] == 0xe1a04000);
        assert(trampoline[2] == 0xe51ff004);
        assert(trampoline[3] == (uint32_t)(uintptr_t)(targets[i] + 2));
    }
    assert(created == 1 && s_jitChunkCount == 1 && s_jitChunks[0].used == 176);
    assert(flushed == 22 && invalidated == 22);
    reset();

    // JIT creation failure and the unsupported fallback leave targets untouched.
    for (unsigned mode = 0; mode < 2; ++mode) {
        fail_create = mode == 0;
        wrong_type = mode == 1;
        uint32_t target[2] = {0xe92d4010, 0xe1a04000};
        void* original = (void*)(uintptr_t)1;
        Hook hook = forge_hook_create(target, (void*)(uintptr_t)0x30000000, &original);
        assert(!original && !hook.original);
        assert(target[0] == 0xe92d4010 && target[1] == 0xe1a04000);
        assert(s_jitChunkCount == 0 && flushed == 0);
        reset();
    }
    // Fill both real chunk capacities; the next hook must fail before its branch.
    for (unsigned i = 0; i < 2048; ++i) {
        uint32_t target[2] = {0xe92d4010, 0xe1a04000};
        void* original = NULL;
        forge_hook_create(target, (void*)(uintptr_t)0x30000000, &original);
        assert(original);
    }
    assert(created == 2 && s_jitChunkCount == 2);
    uint32_t target[2] = {0xe92d4010, 0xe1a04000};
    void* original = (void*)(uintptr_t)1;
    forge_hook_create(target, (void*)(uintptr_t)0x30000000, &original);
    assert(!original && target[0] == 0xe92d4010 && target[1] == 0xe1a04000);
    reset();
    puts("PASS: native pool shares 176 bytes for eleven hooks; allocation, fallback and capacity failures preserve targets");
}
