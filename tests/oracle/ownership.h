// Resolve hunter and palico ownership through checked reads of native tables.
// Unrecognized or unreadable objects remain Unknown for filtering.

#pragma once
#include "model.h"

namespace damage_numbers {
// Checked memory reads are supplied by the native integration or host fixture.
// Cells are main-relative. Reads never change engine state.
template <class Memory> class Ownership {
    Memory &m;

    uint32_t players() const {
        return m.cell(0x1885b58);
    }

public:
    constexpr explicit Ownership(Memory &memory) : m(memory) {
    }

    // Use the active session index to select one of the four hunter-table cells.
    uint32_t local_player() const {
        const uint32_t session = m.cell(0x188ca48), table = players();
        if (!m.readable(session, 0x34) || !m.readable(table, 0x2c))
            return 0;
        const uint32_t index = m.word(session + 0x30);
        if (index >= 4)
            return 0;
        const uint32_t p = m.word(table + 0x1c + 4 * index);
        return m.readable(p, 4) ? p : 0;
    }

    Owner resolve(uint8_t kind, uint32_t object) const {
        if ((kind != 0 && kind != 3) || !m.readable(object, 4))
            return Owner::Unknown;
        const uint32_t local = local_player(), table = players();
        if (!local || !m.readable(table, 0x2c))
            return Owner::Unknown;
        if (object == local)
            return Owner::LocalPlayer;
        for (unsigned i = 0; i < 4; ++i)
            if (object == m.word(table + 0x1c + 4 * i))
                return Owner::RemotePlayer;
        if (kind != 3 || !m.readable(object, 0x3ac8) || m.word(object + 0x13c8) != 0)
            return Owner::Unknown;
        // Native 0x285c6c obtains a palico's master from its current work
        // pointer +1, then calls 0x274ce4 (player table +1c+4*master).
        const uint32_t work = m.word(object + 0x3ac4);
        if (!m.readable(work, 2))
            return Owner::Unknown;
        const uint8_t master = m.byte(work + 1);
        if (master >= 4)
            return Owner::Unknown;
        const uint32_t hunter = m.word(table + 0x1c + 4 * master);
        if (!m.readable(hunter, 4))
            return Owner::Unknown;
        return hunter == local ? Owner::LocalPalico : Owner::RemotePalico;
    }

    // Resolve a direct hunter/palico owner first; projectile records instead use
    // the secondary source kind/object fields.
    Owner attack(uint32_t hit) const {
        if (!m.readable(hit, 0xb8))
            return Owner::Unknown;
        const uint8_t primary = m.byte(hit + 0x10);
        if (primary == 0 || primary == 3)
            return resolve(primary, m.word(hit + 0xc));
        return resolve(m.byte(hit + 0x6e), m.word(hit + 0x70));
    }
};
} // namespace damage_numbers
