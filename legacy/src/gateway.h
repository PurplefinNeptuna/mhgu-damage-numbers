// C++ view of the audited gateway.S stack frame and its callback contracts.
// The assembly file stays pinned; these offsets let callbacks inspect saved state.

#pragma once
#include <cstddef>
#include <cstdint>

namespace damage_numbers {
// Stack image shared with gateway.S. Native AArch32 routines use VFP argument
// registers even though the plugin is compiled with softfp. Keep their bits.
// SAVE pushes r0-r12/lr, then d0-d15 and d16-d31, then CPSR/FPSCR, and
// reserves eight bytes. Since the stack grows downward, this structure lists
// those fields in reverse group order. caller_sp() recovers the entry sp.
// RESTORE stores the helper's continuation in target and reloads saved state
// before branching there. It restores NZCVQ/FPSCR; GE is dead only at the
// eight audited sites described in GE_PRESERVATION_AUDIT.md.
struct RawFrame {
    uint32_t target, spare, cpsr, fpscr;
    uint64_t high_d[16], low_d[16];
    uint32_t r[14]; // r0-r12, lr

    uint32_t caller_sp() const {
        return uint32_t(uintptr_t(this) + sizeof(*this));
    }
};

static_assert(sizeof(RawFrame) == 328);
static_assert(offsetof(RawFrame, cpsr) == 8 && offsetof(RawFrame, fpscr) == 12);
static_assert(offsetof(RawFrame, high_d) == 16 && offsetof(RawFrame, low_d) == 144);
static_assert(offsetof(RawFrame, r) == 272);
static_assert(offsetof(RawFrame, r) + 13 * sizeof(uint32_t) == 324);
} // namespace damage_numbers

// Implementations must always return an existing original trampoline. Around
// entry redirects saved lr only after storing the caller in a native TLS scope.
extern "C" uint32_t dn_gateway_enter(uint32_t, damage_numbers::RawFrame *);
extern "C" uint32_t dn_gateway_leave(damage_numbers::RawFrame *);
extern "C" uint32_t dn_gateway_tap(uint32_t, damage_numbers::RawFrame *);
