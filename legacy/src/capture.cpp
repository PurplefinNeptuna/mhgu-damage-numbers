#include "capture.h"
#include "gateway.h"
#include "hitzone.h"
#include "mailbox.h"
#include "ownership.h"
#include "outcomes.h"
#include "transaction.h"
#include "signatures.h"
#include "../../modern-weapon-controls/src/native_platform.h"
#include <switch.h>
#include "sdk_tls.h"
#include <forge/log.h>
#include <stdarg.h>
#define CIMGUI_DEFINE_ENUMS_AND_STRUCTS
#include <cimgui.h>
extern "C" {
#include <forge/mem.h>
#include <forge/fs.h>
Result mwc_svc_query(MemoryInfo*, uint32_t*, uint64_t);
Result mwc_svc_get_info(uint64_t*, uint32_t, uint32_t, uint64_t);
void dn_gateway_0(); void dn_gateway_1(); void dn_gateway_2(); void dn_gateway_3();
void dn_gateway_4(); void dn_gateway_5(); void dn_gateway_6(); void dn_gateway_7();
void dn_gateway_after();
}

namespace damage_numbers {
static mwc::NativePlatform platform;
static Transaction<mwc::NativePlatform> transaction(platform);
static uint32_t base, active, claimed, consumer, patch_error;
static uint32_t camera_enabled, hook_count;
static unsigned hook_slot[8]{};
static uint32_t context_used[16]{};
static TlsSlot tls_slot = -1;
static const char* status = "Capture not initialized";
static uint32_t calls[8]{}, scope_drops, pool_drops, orphan_calls, excluded, bad_memory;
// One weak CAS attempt, no retry. Shared diagnostic counts are best-effort
// under contention; record payloads/serials do not use these counters.
static void increment(uint32_t& n) {
    uint32_t expected=__atomic_load_n(&n,__ATOMIC_RELAXED);
    __atomic_compare_exchange_n(&n,&expected,expected+1,true,__ATOMIC_RELAXED,__ATOMIC_RELAXED);
}
static uint32_t load(const uint32_t& n) { return __atomic_load_n(&n, __ATOMIC_RELAXED); }
static bool readable(uint32_t a, unsigned n) {
    if (!a || uint64_t(a)+n > 0x100000000ull) return false;
    MemoryInfo info{}; uint32_t page = 0;
    return !mwc_svc_query(&info, &page, a) && (info.perm&1) &&
           info.addr <= a && uint64_t(a)+n <= info.addr+info.size;
}
static uint32_t word(uint32_t a) { return *reinterpret_cast<volatile const uint32_t*>(a); }
static uint8_t byte(uint32_t a) { return *reinterpret_cast<volatile const uint8_t*>(a); }
struct Memory {
    bool readable(uint32_t a, unsigned n) const { return damage_numbers::readable(a,n); }
    uint32_t word(uint32_t a) const { return damage_numbers::word(a); }
    uint8_t byte(uint32_t a) const { return damage_numbers::byte(a); }
    uint32_t cell(uint32_t o) const { return readable(base+o,4) ? word(base+o) : 0; }
};
static const Memory memory;
static const Ownership<const Memory> ownership(memory);
static bool copy(uint32_t address, void* out, unsigned n) {
    if (!readable(address,n)) { increment(bad_memory); return false; }
    std::memcpy(out, reinterpret_cast<const void*>(address), n); return true;
}
struct FeedbackSnapshot {
    uint32_t attack, flags;
    uint8_t active, effect, critical, reserved;
};
struct Record {
    uint32_t kind, serial, actor, attack, monster, flags, damage_type, part, critical, effect;
    int32_t damage, hzv;
    Vec3 position;
    float view[16], projection[16];
    int32_t rect[4];
    FeedbackSnapshot feedback[2];
    uint32_t attack_counter;
    uint32_t source_tags;
};
static_assert(sizeof(Record)==236 && offsetof(Record,damage)==40 &&
              offsetof(Record,position)==48 && offsetof(Record,view)==60 &&
              offsetof(Record,projection)==124 && offsetof(Record,rect)==188 &&
              offsetof(Record,feedback)==204 && sizeof(FeedbackSnapshot)==12);
struct Scope {
    uint32_t kind, caller, sp, subject, monster, auxiliary;
    Owner owner;
    int16_t hzv;
    Vec3 position;
};
struct Context {
    uint32_t depth = 0, serial = 0;
    bool quarantine = false;
    Scope stack[64]{};
    Mailbox<Record,256> records;
};
static Context contexts[16];
static Context* context() {
    if (tls_slot < 0) return nullptr;
    void* p = nnosGetTlsValue(tls_slot);
    if (p == reinterpret_cast<void*>(1)) return nullptr;
    if (p) return static_cast<Context*>(p);
    unsigned i=16;
    for (unsigned candidate=0;candidate<16;++candidate) {
        uint32_t unused=0;
        if (__atomic_compare_exchange_n(&context_used[candidate],&unused,1,true,
                                       __ATOMIC_SEQ_CST,__ATOMIC_SEQ_CST)) {
            i=candidate; increment(claimed); break;
        }
    }
    if (i == 16) {
        nnosSetTlsValue(tls_slot, reinterpret_cast<void*>(1)); increment(pool_drops); return nullptr;
    }
    p = &contexts[i];
    nnosSetTlsValue(tls_slot, p);
    if (nnosGetTlsValue(tls_slot) != p) { increment(pool_drops); return nullptr; }
    return static_cast<Context*>(p);
}
static Scope* find(Context& c, unsigned kind) {
    for (unsigned i=c.depth; i; --i) if (c.stack[i-1].kind == kind) return &c.stack[i-1];
    return nullptr;
}
static void publish(Context& c, Record& r) {
    r.serial = ++c.serial; c.records.publish(r);
}
static uint32_t detours[] = {uint32_t(&dn_gateway_0),uint32_t(&dn_gateway_1),
    uint32_t(&dn_gateway_2),uint32_t(&dn_gateway_3),uint32_t(&dn_gateway_4),
    uint32_t(&dn_gateway_5),uint32_t(&dn_gateway_6),uint32_t(&dn_gateway_7)};
static uint32_t original(unsigned kind) {
    return kind < 8 && hook_slot[kind] < hook_count ? transaction.original(hook_slot[kind]) : 0;
}

extern "C" uint32_t dn_gateway_enter(uint32_t kind, RawFrame* frame) {
    const uint32_t original = damage_numbers::original(kind);
    if (!load(active)) return original;
    increment(calls[kind]);
    Context* c = context();
    if (!c) { increment(scope_drops); return original; }
    if (c->quarantine || c->depth == 64) {
        c->quarantine = true; increment(scope_drops); return original;
    }
    Scope& s = c->stack[c->depth++];
    s = {}; s.kind = kind; s.caller = frame->r[13]; s.sp = frame->caller_sp(); s.hzv = -1;
    if (kind == 0) {
        s.subject = frame->r[1]; s.monster = frame->r[2];
        s.owner = ownership.attack(s.subject);
        copy(frame->r[3], &s.position, sizeof(s.position));
        if (readable(s.sp+8,4)) s.auxiliary = word(s.sp+8);
    } else if (kind == 1) {
        s.subject = frame->r[0]; s.auxiliary = frame->r[1];
    } else if (kind == 2 || kind == 3) {
        s.subject = frame->r[0];
    } else if (kind == 6) {
        s.monster = frame->r[1];
    } else if (kind == 7) {
        s.subject = frame->r[1]; s.monster = frame->r[2];
        if (readable(s.sp+4,4)) s.auxiliary = word(s.sp+4);
    }
    frame->r[13] = uint32_t(&dn_gateway_after);
    return original;
}

extern "C" uint32_t dn_gateway_leave(RawFrame* frame) {
    Context* c = context();
    // Enter and leave run on the same native thread, with an owned TLS slot.
    // A missing return scope cannot be recovered by inventing a caller address.
    if (!c || !c->depth) __builtin_trap();
    const Scope s = c->stack[--c->depth];
    if (s.sp != frame->caller_sp()) __builtin_trap();
    if (frame->r[13] == uint32_t(&dn_gateway_after)) frame->r[13] = s.caller;
    if (c->quarantine) {
        if (!c->depth) c->quarantine = false;
        return s.caller;
    }
    if (!load(active)) return s.caller;
    if (s.kind == 1) {
        Scope* hit = find(*c,0);
        if (!hit || hit->monster != s.subject) { increment(orphan_calls); return s.caller; }
        if (hit->owner != Owner::LocalPlayer && hit->owner != Owner::LocalPalico) {
            increment(excluded); return s.caller;
        }
        if (!readable(hit->subject,0xb8)) { increment(bad_memory); return s.caller; }
        const uint32_t type = byte(hit->subject+0x3a);
        const int32_t amount = int32_t(frame->r[0]);
        if (!damage_outcome(uint8_t(type),amount)) { increment(excluded); return s.caller; }
        Record r{};
        r.kind = 1; r.actor = uint32_t(hit->owner); r.attack = hit->subject;
        r.monster = hit->monster; r.part = s.auxiliary; r.hzv = hit->hzv;
        r.damage = amount; r.damage_type = type; r.flags = word(hit->subject+0x30);
        r.critical = byte(hit->subject+0x54); r.effect = byte(hit->subject+6);
        r.source_tags=byte(hit->subject+0x10) | uint32_t(byte(hit->subject+0x6d))<<8 |
                      uint32_t(byte(hit->subject+0x6e))<<16;
        r.position = hit->position;
        publish(*c,r);
    } else if (s.kind == 2 && s.subject == ownership.local_player()) {
        Record r{}; r.kind = 2; r.actor = s.subject; r.effect = frame->r[0]&0xff;
        for (unsigned i=0;i<2;++i) {
            const uint32_t attack=s.subject+0x3020+i*0xb8;
            if (readable(attack,0xb8)) {
                r.feedback[i]={attack,word(attack+0x30),byte(attack+5),byte(attack+6),byte(attack+0x54),0};
            }
        }
        if (readable(s.subject+0x2522,2))
            std::memcpy(&r.attack_counter,reinterpret_cast<const void*>(s.subject+0x2522),2);
        publish(*c,r);
    } else if (s.kind == 7) {
        Scope* hit = find(*c,0);
        if (hit && hit->monster == s.monster && hit->subject == s.subject &&
            readable(s.subject+0x30,4) && (word(s.subject+0x30)&1)) {
            float factor = 0;
            if (copy(s.auxiliary,&factor,4)) {
                const float hzv = factor*100;
                if (finite(hzv) && hzv >= 0 && hzv <= 32767) hit->hzv = int16_t(hzv);
            }
        }
    }
    return s.caller;
}

extern "C" uint32_t dn_gateway_tap(uint32_t kind, RawFrame* frame) {
    const uint32_t original = damage_numbers::original(kind);
    if (!load(active)) return original;
    increment(calls[kind]);
    Context* c = context();
    if (!c || c->quarantine) return original;
    if (kind == 5) {
        Scope* hit = find(*c,0); Scope* hzv = find(*c,6);
        if (hit && hzv && hit->monster == hzv->monster) hit->hzv = melee_hitzone(*frame);
    } else if (kind == 4) {
        Scope* viewport = find(*c,3);
        if (viewport && readable(viewport->subject,0x28)) {
            const uint32_t camera = word(viewport->subject+0x24);
            if (readable(camera,4) && word(camera) == base+0x1730d0c) {
                Record r{}; r.kind = 3; r.actor = camera;
                if (copy(frame->r[1],r.view,64) && copy(frame->r[2],r.projection,64) &&
                    copy(frame->r[3],r.rect,16)) publish(*c,r);
            }
        }
    }
    return original;
}

void capture_install() {
    // Separate opt-in, absent from the default candidate package. No save mount.
    char flag[128]{};
    const char* mount = forge_fs_getRomFsMountPoint();
    unsigned n=0;
    if (!mount) { status = "Capture inactive: no RomFS mount"; return; }
    while (mount[n] && n<32) { flag[n]=mount[n]; ++n; }
    const char suffix[] = ":/nativeNX/forge/damage-numbers-capture.enabled";
    if (mount[n]) { status = "Capture inactive: invalid mount name"; return; }
    std::memcpy(flag+n,suffix,sizeof(suffix));
    if (!forge_fs_fileExists(flag)) { status = "Capture inactive: opt-in flag absent"; return; }
    const char camera_suffix[] = ":/nativeNX/forge/damage-numbers-camera.enabled";
    std::memcpy(flag+n,camera_suffix,sizeof(camera_suffix));
    camera_enabled = forge_fs_fileExists(flag);
    base = forge_mem_getMainTextAddr();
    uint64_t title=0;
    if (!base || base>0xfe000000u || mwc_svc_get_info(&title,18,CUR_PROCESS_HANDLE,0) ||
        title!=0x0100770008dd8000ull) { status = "Capture inactive: wrong program"; return; }
    for (const auto& s : signatures) {
        if (!readable(base+s.offset,sizeof(s.bytes))) { status = "Capture inactive: unreadable signature"; return; }
        for (unsigned i=0;i<sizeof(s.bytes);++i) if (byte(base+s.offset+i)!=s.bytes[i]) {
            status = "Capture inactive: build/code signature mismatch"; return;
        }
    }
    if (nnosAllocateTlsSlot(&tls_slot,nullptr) || tls_slot<0) {
        status = "Capture inactive: native TLS allocation failed"; return;
    }
    HookSpec recipe[8]{};
    for (unsigned i=0;i<8;++i) {
        hook_slot[i] = 8;
        // Leave both renderer entry points untouched in the recovery trial.
        if (!camera_enabled && (i == 3 || i == 4)) continue;
        uint32_t expected[2]; std::memcpy(expected,signatures[i].bytes,8);
        hook_slot[i] = hook_count;
        recipe[hook_count++] = {base+signatures[i].offset,detours[i],{expected[0],expected[1]}};
    }
    constexpr uint32_t offsets[] = {0x3de064,0x3af5b4,0x388ee8,0x5e8130,0x3850b0,0x7a3480,
        0x3da8fc,0xb8692c,0x887810,0x88784c,0x27e774,0x27ca28,0x678894,0x285e2c};
    uint32_t occupied[14];
    for (unsigned i=0;i<14;++i) occupied[i] = base+offsets[i];
    // Forge's boot callback runs before original_sApp_run. No hot installation
    // or removal is supported. Quest collision/viewport consumers start later.
    const auto result = transaction.install(recipe,hook_count,occupied,14,true);
    if (result!=InstallResult::Active) {
        status = result==InstallResult::RestoreFailure ?
            "ERROR: hook restoration failed; arena retained; restart required" :
            result==InstallResult::CleanupFailure ?
            "ERROR: arena cleanup failed; restart required" :
            "Capture inactive: hook transaction failed (restart required)";
        forge_log_warn("[damage-numbers] Capture transaction result %u",unsigned(result));
        return;
    }
    __atomic_store_n(&active,1,__ATOMIC_RELEASE);
    status = "Diagnostic capture active; floating numbers are pending validation";
    forge_log_info("[damage-numbers] Capture r3: %u hooks; camera capture %s; one private arena/TLS slot; hit log limit 256",hook_count,camera_enabled ? "enabled" : "off");
}
void capture_monitor() {
    if (!load(active)) return;
    for (unsigned i=0;i<8;++i) {
        if (hook_slot[i] >= hook_count) continue;
        const auto& s=signatures[i];
        bool valid = word(base+s.offset)==0xe51ff004u && word(base+s.offset+4)==detours[i];
        for (unsigned j=8;j<sizeof(s.bytes);++j) valid=valid && byte(base+s.offset+j)==s.bytes[j];
        if (!valid) {
            __atomic_store_n(&patch_error,1,__ATOMIC_RELEASE);
            __atomic_store_n(&active,0,__ATOMIC_RELEASE);
            forge_log_warn("[damage-numbers] External change at hook %u; capture disabled; restart required",i);
            return;
        }
    }
}
static Record recent[8]{}, last_camera{}, last_feedback{};
static unsigned recent_count, consume_cursor;
static uint32_t consumed, shown_player, captured_hits;
static constexpr unsigned log_limit=256, hits_per_consume=8, records_per_consume=64;
void capture_consume() {
    if (!load(active)) return;
    static uint32_t first;
    uint32_t unlogged=0;
    if (__atomic_compare_exchange_n(&first,&unlogged,1,true,__ATOMIC_RELAXED,__ATOMIC_RELAXED))
        forge_log_info("[damage-numbers] First render consumer entered");
    Context* current = context();
    if (!current) return;
    static uint32_t ready;
    unlogged=0;
    if (__atomic_compare_exchange_n(&ready,&unlogged,1,true,__ATOMIC_RELAXED,__ATOMIC_RELAXED))
        forge_log_info("[damage-numbers] Render TLS context ready");
    const uint32_t id=uint32_t(current), expected=0;
    uint32_t initial=expected;
    __atomic_compare_exchange_n(&consumer,&initial,id,true,__ATOMIC_SEQ_CST,__ATOMIC_SEQ_CST);
    if (load(consumer)!=id) return; // single consumer enforced across callbacks
    const uint32_t player=ownership.local_player();
    // Hit records own copied values only. Retain them across quest/village
    // changes; camera and feedback still belong to the current player context.
    if (player!=shown_player) { last_camera={}; last_feedback={}; shown_player=player; }
    unsigned empty=0, hits=0, records=0;
    while (empty<16 && hits<hits_per_consume && records<records_per_consume) {
        const unsigned i=consume_cursor;
        consume_cursor=(consume_cursor+1)%16;
        Record r{};
        if (!contexts[i].records.read(r)) { ++empty; continue; }
        empty=0; ++records; ++consumed;
        // Ownership was resolved by the producer. A queued hit remains valid
        // evidence even if the player disappears before the render callback.
        if (r.kind==1) {
            if (recent_count==8) {
                for (unsigned k=1;k<8;++k) recent[k-1]=recent[k];
                --recent_count;
            }
            recent[recent_count++]=r;
            ++hits; ++captured_hits;
            if (captured_hits<=log_limit) {
                forge_log_info("[damage-numbers] Hit %u: damage %d; HZV %d; crit %u; effect %u; part %u; owner %u; thread %u; serial %u; type %u; flags %08x; tags %06x; pos %.2f %.2f %.2f",
                    captured_hits,r.damage,r.hzv,r.critical,r.effect,r.part,r.actor,i,r.serial,
                    r.damage_type,r.flags,r.source_tags,double(r.position.x),double(r.position.y),double(r.position.z));
                if (captured_hits==log_limit)
                    forge_log_info("[damage-numbers] Hit log limit reached (256); panel capture continues until restart");
            }
        } else if (r.kind==2 && player) last_feedback=r;
        else if (r.kind==3 && player) last_camera=r;
    }
}
void capture_panel() {
    igTextUnformatted(load(patch_error) ? "Capture disabled: external code change; restart required" : status,nullptr);
    igTextUnformatted("Research capture: no world-space damage display or gameplay changes.",nullptr);
    if (!load(active)) return;
    igTextUnformatted(camera_enabled ? "Capture r3: camera capture enabled." :
        "Capture r3: camera hooks off; combat capture only.",nullptr);
    capture_consume();
    Context* current=context();
    if (!current || load(consumer)!=uint32_t(current)) {
        igTextUnformatted("Capture panel deferred: a different render thread owns the consumer.",nullptr);
        return;
    }
    igTextUnformatted("Shared counters are best-effort under contention; per-hit records remain separate.",nullptr);
    igText("Collision %u; normalization %u; feedback %u; viewport %u",load(calls[0]),load(calls[1]),load(calls[2]),load(calls[3]));
    igText("Camera uploads %u; HZV taps %u; melee queries %u; shot queries %u",load(calls[4]),load(calls[5]),load(calls[6]),load(calls[7]));
    igText("TLS claims %u; scope drops %u; pool drops %u; orphan contributions %u",load(claimed),load(scope_drops),load(pool_drops),load(orphan_calls));
    uint32_t overwritten=0;
    for (const auto& c : contexts) overwritten+=c.records.overwritten;
    igText("Old capture records discarded: %u",overwritten);
    igText("Excluded %u; read failures %u; records consumed %u",load(excluded),load(bad_memory),consumed);
    igText("Latest resolved feedback: %u (aggregate; not assigned to individual hits)",last_feedback.effect);
    for (const auto& f : last_feedback.feedback)
        igText("  Attack %08x: active %u; effect %u; crit %u; flags %08x",f.attack,f.active,f.effect,f.critical,f.flags);
    igText("  Native attack counter: %u",last_feedback.attack_counter);
    igText("Quest camera rectangle: %d %d %d %d",last_camera.rect[0],last_camera.rect[1],last_camera.rect[2],last_camera.rect[3]);
    igSeparator();
    igTextUnformatted("Last eight hits remain across quest/village transitions until restart.",nullptr);
    igText("Captured hits %u; first %u written to Eden log (maximum 256 per restart)",captured_hits,
        captured_hits<log_limit ? captured_hits : log_limit);
    for (unsigned i=0;i<recent_count;++i) {
        const auto& r=recent[i];
        igText("Hit %u: damage %d; HZV %d; crit %u; effect %u; part %u; owner %u",r.serial,r.damage,r.hzv,r.critical,r.effect,r.part,r.actor);
        igText("  Position %.2f %.2f %.2f; type %u; flags %08x; source tags %06x",r.position.x,r.position.y,r.position.z,r.damage_type,r.flags,r.source_tags);
    }
}
} // namespace damage_numbers
