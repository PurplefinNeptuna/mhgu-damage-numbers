#pragma once
#include <stdint.h>
#include <math.h>

namespace damage_numbers {
enum class Source : uint8_t { Attack, Monster, Object, Tick };
enum class Owner : uint8_t { LocalPlayer, LocalPalico, RemotePlayer, RemotePalico, Unknown };
enum class Critical : uint8_t { None, Positive, Negative };
enum class Tint : uint8_t { White, Gray, Weak };

struct Vec3 { float x, y, z; };
struct HitKey {
    uint32_t epoch, serial, target, application;
    bool operator==(const HitKey& b) const {
        return epoch == b.epoch && serial == b.serial && target == b.target && application == b.application;
    }
};

// One native, finalized impact. Accompanying elemental damage is already in
// total_damage; HZV describes only the physical component. No game pointers.
struct Event {
    HitKey key;
    Vec3 position;
    int32_t total_damage;
    int16_t physical_hzv; // -1 when physical HZV is absent or unavailable.
    Source source;
    Owner owner;
    Critical critical;
    bool bounced;
    float captured_at;
};

struct Settings {
    bool enabled = true;
    bool monsters = false;
    bool objects = false;
    bool ticks = false;
    bool bounce_gray = true;
    float scale = 1.0f;
    float lifetime = 0.9f;
};

inline bool finite(float n) { return isfinite(n); }
inline bool valid(Vec3 p) { return finite(p.x) && finite(p.y) && finite(p.z); }
inline bool valid(const Settings& s) {
    return finite(s.scale) && s.scale >= 0.5f && s.scale <= 3.0f &&
           finite(s.lifetime) && s.lifetime >= 0.4f && s.lifetime <= 2.0f;
}
inline bool visible(const Event& e, const Settings& s) {
    if (!s.enabled || !valid(s) || e.total_damage <= 0 || !valid(e.position) ||
        !finite(e.captured_at) || !e.key.serial || !e.key.target || e.physical_hzv < -1 ||
        unsigned(e.owner)>unsigned(Owner::Unknown) ||
        unsigned(e.critical)>unsigned(Critical::Negative)) return false;
    switch (e.source) {
    case Source::Attack: return e.owner == Owner::LocalPlayer || e.owner == Owner::LocalPalico;
    case Source::Monster: return s.monsters;
    case Source::Object: return s.objects;
    case Source::Tick: return s.ticks;
    }
    return false;
}
inline Tint tint(const Event& e, const Settings& s) {
    if (e.source == Source::Tick || (e.bounced && s.bounce_gray)) return Tint::Gray;
    return e.physical_hzv >= 45 ? Tint::Weak : Tint::White;
}
inline Critical marker(const Event& e) {
    return e.source == Source::Tick ? Critical::None : e.critical;
}
inline uint32_t rgb(Tint c) {
    switch (c) {
    case Tint::Gray: return 0x9a9a9a;
    case Tint::Weak: return 0xffbf3f;
    default: return 0xffffff;
    }
}

// Serialized access is supplied by the integration layer; never take a game
// lock or spin in a native detour. Queue overflow discards oldest events.
template<unsigned Capacity> class Queue {
    static_assert(Capacity > 0);
    Event items[Capacity]{};
    unsigned head = 0, count = 0;
public:
    uint32_t dropped = 0;
    unsigned size() const { return count; }
    void clear() { head = count = 0; }
    void push(const Event& e) {
        if (count == Capacity) { head = (head + 1) % Capacity; --count; ++dropped; }
        items[(head + count) % Capacity] = e;
        ++count;
    }
    bool pop(Event& e) {
        if (!count) return false;
        e = items[head]; head = (head + 1) % Capacity; --count; return true;
    }
    bool bounce(const HitKey& key) {
        for (unsigned i=0;i<count;++i) {
            Event& e=items[(head+i)%Capacity];
            if (e.key==key) { e.bounced=true; return true; }
        }
        return false;
    }
};

struct Number { Event event; };
class Model {
    Queue<256> queue;
    Number numbers[128]{};
    HitKey recent[512]{};
    unsigned count = 0, recent_count = 0, recent_next = 0;
    uint32_t current_epoch = 0;
    Settings settings{};
public:
    uint32_t duplicates = 0, filtered = 0, active_drops = 0;
    unsigned size() const { return count; }
    unsigned pending() const { return queue.size(); }
    uint32_t queue_drops() const { return queue.dropped; }
    const Number& at(unsigned i) const { return numbers[i]; }
    const Settings& config() const { return settings; }
    void reset(uint32_t epoch) {
        current_epoch = epoch; count = recent_count = recent_next = 0; queue.clear();
    }
    bool configure(const Settings& next) {
        if (!valid(next)) return false;
        settings = next;
        // A disabled option must not leave an already-visible source behind.
        for (unsigned i=0; i<count;) {
            if (visible(numbers[i].event, settings)) { ++i; continue; }
            for (unsigned j=i+1;j<count;++j) numbers[j-1]=numbers[j];
            --count;
        }
        const unsigned pending = queue.size();
        for (unsigned i=0;i<pending;++i) {
            Event e{};
            queue.pop(e);
            if (visible(e,settings)) queue.push(e);
        }
        return true;
    }
    bool capture(const Event& e) {
        if (e.key.epoch != current_epoch || !visible(e,settings)) { ++filtered; return false; }
        for (unsigned i=0;i<recent_count;++i) if (recent[i] == e.key) { ++duplicates; return false; }
        recent[recent_next] = e.key;
        recent_next = (recent_next+1)%512;
        if (recent_count<512) ++recent_count;
        queue.push(e); return true;
    }
    bool bounce(const HitKey& key) {
        if (key.epoch!=current_epoch) return false;
        if (queue.bounce(key)) return true;
        for (unsigned i=0;i<count;++i) if (numbers[i].event.key==key) {
            numbers[i].event.bounced=true; return true;
        }
        return false;
    }
    void update(float now, bool gameplay_valid) {
        if (!gameplay_valid || !finite(now) || !settings.enabled) {
            count=recent_count=recent_next=0; queue.clear(); return;
        }
        for (unsigned i=0;i<count;) {
            const float age=now-numbers[i].event.captured_at;
            if (age>=0 && age<settings.lifetime && visible(numbers[i].event,settings)) { ++i; continue; }
            for (unsigned j=i+1;j<count;++j) numbers[j-1]=numbers[j];
            --count;
        }
        Event e{};
        while (queue.pop(e)) {
            const float age=now-e.captured_at;
            if (e.key.epoch!=current_epoch || !visible(e,settings) || age<0 || age>=settings.lifetime) continue;
            if (count==128) {
                for (unsigned i=1;i<count;++i) numbers[i-1]=numbers[i];
                --count; ++active_drops;
            }
            numbers[count++].event=e;
        }
    }
};
}
