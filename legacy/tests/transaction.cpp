#include "transaction.h"
#include <array>
#include <cassert>
#include <cstdio>
#include <map>
#include <vector>

struct Fake {
    std::map<uint32_t, uint32_t> memory;
    std::array<uint32_t, 1024> arena{};
    unsigned op = 0, fail_at = 0, allocations = 0, writes = 0, closes = 0;
    unsigned prepare_count = 0;
    bool owned = false, executable = false, restore_failure = false, close_failure = false;
    bool race = false;
    bool ok() { return ++op != fail_at; }
    uint32_t read(uint32_t a) { return memory.at(a); }
    uint32_t rw_base() const { return 0x800000; }
    uint32_t rx_base() const { return 0x900000; }
    bool owns_arena() const { return owned; }
    bool create(unsigned n) {
        assert(n == 4096 && !owned && allocations == 0);
        if (!ok()) return false;
        owned = true; ++allocations; return true;
    }
    bool close() {
        ++closes;
        if (close_failure || !ok()) return false;
        owned = executable = false; return true;
    }
    bool prepare_slot(unsigned off, const uint32_t* words) {
        assert(owned && !executable && off % 16 == 0 && off < 4096);
        if (!ok()) return false;
        ++prepare_count;
        for (unsigned i = 0; i < 4; ++i) arena[off / 4 + i] = words[i];
        return true;
    }
    bool make_executable() {
        if (!ok()) return false;
        executable = true;
        if (race) memory[0x1100] = 0xe3a00000;
        return true;
    }
    bool write_pair(uint32_t a, uint32_t first, uint32_t second) {
        assert(owned && executable && prepare_count == 3);
        ++writes;
        const bool restoring = first != 0xe51ff004;
        if (restoring && restore_failure) return false;
        memory[a + 4] = second; // Simulate failure after a partial target write.
        if (!ok()) return false;
        memory[a] = first; return true;
    }
    bool sync_target(uint32_t, unsigned n) {
        assert(n == 8);
        return !(restore_failure && writes > 3) && ok();
    }
};
constexpr damage_numbers::HookSpec recipes[3] = {
    {0x1000, 0x3000, {0xe92d4ff0, 0xe24dd014}},
    {0x1100, 0x3100, {0xe92d4830, 0xe1a04000}},
    {0x1200, 0x3200, {0xe92d4070, 0xe3010444}}
};
Fake make_fake() {
    Fake f;
    for (const auto& h : recipes) {
        f.memory[h.target] = h.expected[0]; f.memory[h.target + 4] = h.expected[1];
    }
    return f;
}
void baseline(const Fake& f) {
    for (const auto& h : recipes) {
        assert(f.memory.at(h.target) == h.expected[0]);
        assert(f.memory.at(h.target + 4) == h.expected[1]);
    }
}
int main() {
    using R = damage_numbers::InstallResult;
    unsigned checked = 0;
    auto f = make_fake();
    damage_numbers::Transaction<Fake> t(f);
    assert(t.install(recipes, 3, nullptr, 0, true) == R::Active);
    assert(t.is_active() && f.allocations == 1 && f.prepare_count == 3);
    const unsigned successful_ops = f.op;
    for (unsigned i = 0; i < 3; ++i) {
        assert(t.original(i) == f.rx_base() + 16 * i);
        assert(f.arena[4 * i] == recipes[i].expected[0]);
        assert(f.arena[4 * i + 1] == recipes[i].expected[1]);
        assert(f.arena[4 * i + 2] == 0xe51ff004);
        assert(f.arena[4 * i + 3] == recipes[i].target + 8);
    }
    assert(t.original(3) == 0);
    assert(t.remove(false) == R::NotQuiescent && t.is_active());
    assert(t.remove(true) == R::NotPrepared && !t.is_active() && !f.owned);
    baseline(f); ++checked;
    // Every preparation, partial publication and target-cache failure.
    for (unsigned fail = 1; fail <= successful_ops; ++fail) {
        auto x = make_fake(); x.fail_at = fail;
        damage_numbers::Transaction<Fake> tx(x);
        assert(tx.install(recipes, 3, nullptr, 0, true) != R::Active);
        assert(!tx.is_active() && !x.owned);
        assert(tx.original(0) == 0);
        baseline(x); ++checked;
    }
    // Rejection happens before any allocation or write.
    for (unsigned scenario = 0; scenario < 8; ++scenario) {
        auto x = make_fake(); damage_numbers::Transaction<Fake> tx(x);
        std::array<damage_numbers::HookSpec, 3> hs{recipes[0], recipes[1], recipes[2]};
        uint32_t occupied = hs[1].target;
        if (scenario == 0) hs[0].target |= 1;
        if (scenario == 1) hs[1].target = hs[0].target + 4;
        if (scenario == 2) hs[0].detour |= 1;
        if (scenario == 3) hs[0].expected[0] ^= 1;
        if (scenario == 4) { hs[0].expected[1] = 0xe59f0034; x.memory[0x1004] = hs[0].expected[1]; }
        if (scenario == 7) hs[0].detour = hs[0].target;
        assert(tx.install(hs.data(), 3, scenario == 5 ? &occupied : nullptr,
                          scenario == 5 ? 1 : 0, scenario != 6) != R::Active);
        assert(x.allocations == 0 && x.writes == 0); ++checked;
    }
    { // Change after initial validation: earlier published hook is restored.
        auto x = make_fake(); x.race = true; damage_numbers::Transaction<Fake> tx(x);
        assert(tx.install(recipes, 3, nullptr, 0, true) == R::SignatureMismatch);
        assert(x.memory[0x1000] == recipes[0].expected[0]);
        assert(x.memory[0x1100] == 0xe3a00000); // Preserve external change.
        assert(!x.owned); ++checked;
    }
    { // Rollback failure cannot be represented as successful vanilla fallback.
        auto x = make_fake(); damage_numbers::Transaction<Fake> tx(x);
        assert(tx.install(recipes, 3, nullptr, 0, true) == R::Active);
        x.restore_failure = true;
        assert(tx.remove(true) == R::RestoreFailure && x.owned && !tx.is_active());
        assert(x.closes == 0);
        for(unsigned i=0;i<3;++i) assert(tx.original(i)==x.rx_base()+16*i);
        ++checked;
        x.restore_failure = false;
        assert(tx.remove(true) == R::NotPrepared && !x.owned); baseline(x);
    }
    { // Cleanup failure retains ownership and disables reinstallation.
        auto x = make_fake(); x.fail_at = 2; x.close_failure = true;
        damage_numbers::Transaction<Fake> tx(x);
        assert(tx.install(recipes, 3, nullptr, 0, true) == R::CleanupFailure);
        assert(x.owned && !tx.is_active()); baseline(x); ++checked;
        assert(tx.install(recipes, 3, nullptr, 0, true) == R::InvalidRecipe);
        x.close_failure = false;
        assert(tx.remove(true) == R::NotPrepared && !x.owned);
    }
    for (uint32_t op : {0xe59f0000u, 0xeb000000u, 0xe12fff1eu, 0xe8bd8000u,
                        0xe1a0000fu, 0xe1a0f000u, 0xe92da000u, 0x192d4000u,
                        0xe1a0f2a1u, 0xe201f01fu})
        assert(!damage_numbers::relocatable(op));
    assert(damage_numbers::relocatable(0xe1a022a1u) && damage_numbers::relocatable(0xe201101fu));
    std::printf("Transactional hook tests passed: %u scenarios; one arena; exact rollback; unsafe relocation rejected.\n", checked);
}
