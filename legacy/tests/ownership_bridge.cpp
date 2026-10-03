#include "ownership.h"
using Read=uint32_t(*)(uint32_t,uint32_t);
struct Memory {
    Read read;
    uint32_t cell(uint32_t a) const { return read(a,4); }
    uint32_t word(uint32_t a) const { return read(a,4); }
    uint8_t byte(uint32_t a) const { return uint8_t(read(a,1)); }
    bool readable(uint32_t a,uint32_t n) const { return a&&read(a,n|0x80000000u); }
};
extern "C" uint32_t dn_local_player(Read read) {
    Memory m{read};return damage_numbers::Ownership<Memory>(m).local_player();
}
extern "C" uint32_t dn_owner(Read read,uint32_t hit) {
    Memory m{read};return uint32_t(damage_numbers::Ownership<Memory>(m).attack(hit));
}
