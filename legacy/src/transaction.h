#pragma once
#include <stdint.h>
#include <stddef.h>

namespace damage_numbers {
constexpr unsigned arena_bytes = 4096, slot_bytes = 16, max_hooks = 32;
struct HookSpec {
    uint32_t target, detour;
    uint32_t expected[2];
};
enum class InstallResult {
    NotPrepared, Active, InvalidRecipe, NotQuiescent, SignatureMismatch,
    UnsafeInstruction, AllocationFailure, PreparationFailure,
    PublicationFailure, RestoreFailure, CleanupFailure
};

// Deliberately narrow whitelist: displaced instructions execute unchanged.
// Other instructions require a separately verified relocation implementation.
inline bool relocatable(uint32_t op) {
    // Exact additional instructions verified for damage/camera research sites.
    if (op == 0xed2d8b02u || op == 0xed2d8b0au ||
        op == 0xe28db018u || op == 0xe28db01cu ||
        op == 0xe5c40000u || op == 0xecbd8b0au) return true;
    if ((op >> 28) != 14) return false; // unconditional ARM only
    // Exact PC-independent words from the pinned command getter. No generic
    // data-processing relocation permission is added by this exception.
    if (op == 0xe1a022a1u || op == 0xe201101fu) return true;
    if ((op & 0xffff0000u) == 0xe92d0000u)
        return (op & 0x8000u) == 0 && (op & 0x2000u) == 0 && (op & 0xffffu) != 0;
    if ((op & 0xfffff000u) == 0xe24dd000u) return true; // sub sp, sp, #imm
    if ((op & 0xffff0ff0u) == 0xe1a00000u)
        return ((op >> 12) & 15) != 15 && (op & 15) != 15; // mov rd, rm
    if ((op & 0xfff00000u) == 0xe3000000u)
        return ((op >> 12) & 15) != 15; // movw rd, immediate
    return false;
}
inline bool overlaps(uint32_t a, unsigned n, uint32_t b, unsigned m) {
    return uint64_t(a) < uint64_t(b) + m && uint64_t(b) < uint64_t(a) + n;
}

// Platform owns one CodeMemory arena. Installation/removal is permitted only
// at a verified boundary where no thread executes any target/trampoline.
// A failed restoration retains the arena: never free code still reachable.
template<class Platform> class Transaction {
    Platform& p;
    HookSpec journal[max_hooks]{};
    unsigned touched = 0;
    bool arena_owned = false;
    bool active = false;
    InstallResult result = InstallResult::NotPrepared;
    bool matches(const HookSpec& h) {
        return p.read(h.target) == h.expected[0] &&
               p.read(h.target + 4) == h.expected[1];
    }
    InstallResult cleanup(InstallResult cause) {
        if (arena_owned && !p.close()) return result = InstallResult::CleanupFailure;
        arena_owned = false;
        return result = cause;
    }
    InstallResult restore(InstallResult cause) {
        active = false;
        bool good = true;
        // Restore every journal entry, even after one restore/sync fails.
        for (unsigned i = touched; i != 0; --i) {
            const auto& h = journal[i - 1];
            const bool w = p.write_pair(h.target, h.expected[0], h.expected[1]);
            const bool s = p.sync_target(h.target, 8);
            good = w && s && matches(h) && good;
        }
        if (!good) return result = InstallResult::RestoreFailure;
        touched = 0;
        return cleanup(cause);
    }
public:
    constexpr explicit Transaction(Platform& platform) : p(platform) {}
    bool is_active() const { return active; }
    InstallResult status() const { return result; }
    uint32_t original(unsigned i) const {
        // A failed rollback can leave a published detour reachable even while
        // remapping is inactive. Its original must remain callable together
        // with the retained executable arena; never return a null trampoline.
        return arena_owned && i < touched ? p.rx_base() + i * slot_bytes : 0;
    }
    InstallResult install(const HookSpec* hooks, unsigned count,
                          const uint32_t* occupied, unsigned occupied_count,
                          bool quiescent) {
        if (active || arena_owned || touched || !hooks || !count || count > max_hooks ||
            (occupied_count && !occupied)) return result = InstallResult::InvalidRecipe;
        if (!quiescent) return result = InstallResult::NotQuiescent;
        for (unsigned i = 0; i < count; ++i) {
            const auto& h = hooks[i];
            if (!h.target || !h.detour || ((h.target | h.detour) & 3) ||
                uint64_t(h.target) + 8 >= 0x100000000ull)
                return result = InstallResult::InvalidRecipe;
            if (overlaps(h.target, 8, h.detour, 4)) return result = InstallResult::InvalidRecipe;
            for (unsigned j = 0; j < i; ++j)
                if (overlaps(h.target, 8, hooks[j].target, 8))
                    return result = InstallResult::InvalidRecipe;
            for (unsigned j = 0; j < occupied_count; ++j)
                if (overlaps(h.target, 8, occupied[j], 8))
                    return result = InstallResult::InvalidRecipe;
            if (!matches(h)) return result = InstallResult::SignatureMismatch;
            if (!relocatable(h.expected[0]) || !relocatable(h.expected[1]))
                return result = InstallResult::UnsafeInstruction;
        }
        if (!p.create(arena_bytes)) {
            // Failed preparation may itself fail to release a partial mapping.
            arena_owned = p.owns_arena();
            return cleanup(InstallResult::AllocationFailure);
        }
        arena_owned = true;
        if (!p.rw_base() || !p.rx_base() || ((p.rw_base() | p.rx_base()) & 4095u) ||
            uint64_t(p.rw_base()) + arena_bytes > 0x100000000ull ||
            uint64_t(p.rx_base()) + arena_bytes > 0x100000000ull)
            return cleanup(InstallResult::PreparationFailure);
        for (unsigned i = 0; i < count; ++i) {
            const auto& h = hooks[i];
            if (overlaps(h.target, 8, p.rw_base(), arena_bytes) ||
                overlaps(h.target, 8, p.rx_base(), arena_bytes))
                return cleanup(InstallResult::InvalidRecipe);
            journal[i] = h;
            const uint32_t words[4] = {h.expected[0], h.expected[1], 0xe51ff004u, h.target + 8};
            if (!p.prepare_slot(i * slot_bytes, words))
                return cleanup(InstallResult::PreparationFailure);
        }
        // Complete all executable preparation before any target mutation.
        if (!p.make_executable()) return cleanup(InstallResult::PreparationFailure);
        for (unsigned i = 0; i < count; ++i) {
            const auto& h = journal[i];
            if (!matches(h)) return restore(InstallResult::SignatureMismatch);
            // Include this entry before the first write, including partial failure.
            touched = i + 1;
            if (!p.write_pair(h.target, 0xe51ff004u, h.detour) ||
                !p.sync_target(h.target, 8) || p.read(h.target) != 0xe51ff004u ||
                p.read(h.target + 4) != h.detour)
                return restore(InstallResult::PublicationFailure);
        }
        active = true;
        return result = InstallResult::Active;
    }
    InstallResult remove(bool quiescent) {
        if (!quiescent) return result = InstallResult::NotQuiescent;
        return restore(InstallResult::NotPrepared);
    }
};
}
