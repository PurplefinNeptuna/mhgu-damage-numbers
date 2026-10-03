#include "mailbox.h"
#include <atomic>
#include <cstdio>
#include <cstdlib>
#include <thread>

struct Packet { uint32_t serial, owner, fields[14]; };
static Packet packet(uint32_t serial, uint32_t owner) {
    Packet p{serial, owner, {}};
    for (unsigned i=0; i<14; ++i) p.fields[i] = serial ^ (owner*0x12345678u) ^ (i*0x76543210u);
    return p;
}
static bool correct(const Packet& p, unsigned owner) {
    if (!p.serial || p.owner != owner) return false;
    const auto expected = packet(p.serial, owner);
    for (unsigned i=0; i<14; ++i) if (p.fields[i] != expected.fields[i]) return false;
    return true;
}
static void require(bool ok) { if (!ok) std::abort(); }
int main() {
    damage_numbers::Mailbox<Packet, 4> simple;
    Packet p{};
    require(!simple.read(p));
    for (unsigned i=1; i<=12; ++i) simple.publish(packet(i, 0));
    for (unsigned i=9; i<=12; ++i) require(simple.read(p) && p.serial == i && correct(p, 0));
    require(!simple.read(p) && simple.overwritten == 8);
    damage_numbers::Mailbox<Packet, 4> wrapped(0xfffffffc);
    for (unsigned i=1; i<=12; ++i) wrapped.publish(packet(i, 0));
    for (unsigned i=9; i<=12; ++i) require(wrapped.read(p) && p.serial == i && correct(p, 0));
    require(!wrapped.read(p) && wrapped.overwritten == 8);
    constexpr unsigned workers = 8, iterations = 20000;
    damage_numbers::Mailbox<Packet, 64> queues[workers];
    std::atomic<bool> start{false}, done[workers]{};
    std::thread threads[workers];
    for (unsigned owner=0; owner<workers; ++owner) {
        threads[owner] = std::thread([&, owner] {
            while (!start.load(std::memory_order_acquire)) std::this_thread::yield();
            for (unsigned i=1; i<=iterations; ++i) queues[owner].publish(packet(i, owner));
            done[owner].store(true, std::memory_order_release);
        });
    }
    start.store(true, std::memory_order_release);
    uint32_t last[workers]{}, received[workers]{};
    bool running = true;
    while (running) {
        running = false;
        for (unsigned owner=0; owner<workers; ++owner) {
            // Read completion before draining. A producer can finish during the
            // drain; one more pass must observe its last published packet.
            const bool complete = done[owner].load(std::memory_order_acquire);
            for (unsigned i=0; i<64; ++i) {
                if (!queues[owner].read(p)) break;
                require(correct(p, owner) && p.serial > last[owner]);
                last[owner] = p.serial; ++received[owner];
            }
            running = running || !complete || last[owner] != iterations;
        }
        std::this_thread::yield();
    }
    unsigned total = 0;
    for (unsigned owner=0; owner<workers; ++owner) {
        threads[owner].join();
        require(last[owner] == iterations && received[owner]+queues[owner].overwritten == iterations);
        total += received[owner];
    }
    std::printf("Passed mailbox overflow and 8-producer stress: %u untorn packets received from %u published\n", total, workers*iterations);
}
