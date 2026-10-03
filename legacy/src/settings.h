#pragma once
#include "model.h"
#include <cstring>

namespace damage_numbers {
// Two alternating, independently checked 32-byte SD records. A failed write
// cannot invalidate the last accepted slot. No native game pointers on disk.
struct StoredSettings { uint32_t words[8]; };
static_assert(sizeof(StoredSettings)==32);
inline uint32_t settings_checksum(const StoredSettings& record) {
    uint32_t hash=2166136261u;
    for (unsigned i=0;i<7;++i) for (unsigned b=0;b<4;++b) {
        hash^=(record.words[i]>>(8*b))&255u; hash*=16777619u;
    }
    return hash;
}
inline StoredSettings encode_settings(const Settings& s,uint32_t generation) {
    StoredSettings record{{0x444e554d,1,generation,
        uint32_t(s.enabled)|(uint32_t(s.monsters)<<1)|(uint32_t(s.objects)<<2)|
        (uint32_t(s.bounce_gray)<<4),0,0,0,0}};
    std::memcpy(&record.words[4],&s.scale,4);
    std::memcpy(&record.words[5],&s.lifetime,4);
    record.words[7]=settings_checksum(record); return record;
}
inline bool decode_settings(const StoredSettings& record,Settings& out) {
    if (record.words[0]!=0x444e554d || record.words[1]!=1 ||
        (record.words[3]&~0x17u) || record.words[6] ||
        record.words[7]!=settings_checksum(record)) return false;
    Settings next{};
    next.enabled=record.words[3]&1; next.monsters=record.words[3]&2;
    next.objects=record.words[3]&4; next.bounce_gray=record.words[3]&16;
    std::memcpy(&next.scale,&record.words[4],4);
    std::memcpy(&next.lifetime,&record.words[5],4);
    if (!valid(next)) return false;
    out=next; return true;
}
inline bool newer_settings(uint32_t a,uint32_t b) { return int32_t(a-b)>0; }
bool settings_load(Settings& out);
bool settings_save(const Settings& value);
void settings_changed();
const char* settings_status();
}
