// Mock Forge mounts and SDK file calls to check primary-INI loading, save
// verification, commit failures, and balanced resources without touching real saves.

// Execute production persistence against a recording filesystem, without game saves.
#include <cassert>
#include <string>
#include <cstring>
#include <cstdio>
#include <cstdarg>
#include "../src/settings.cpp"
static std::string primary, backup = "untouched legacy backup";
static bool exists = false, available = true;
static unsigned failure = 0, mounts = 0, handles = 0, guards = 0, accesses = 0, commits = 0;

bool forge_fs_mountSaveData() {
    if (!available)
        return false;
    ++mounts;
    return true;
}

void forge_fs_unmountSaveData() {
    assert(mounts == 1);
    --mounts;
}

void forge_log_warn(const char *, ...) {
}

void forge_log_info(const char *, ...) {
}

extern "C" {
void dn_fs_disable_abort(uint32_t *p) {
    *p = 42;
    ++guards;
}

void dn_fs_restore_context(uint32_t p) {
    assert(p == 42 && guards == 1);
    --guards;
}

uint32_t dn_fs_open(uint32_t *h, const char *p, int mode) {
    assert(std::strcmp(p, "savedata:/damage_numbers.ini") == 0);
    ++accesses;
    if (!exists)
        return 1;
    assert(!handles);
    *h = ++handles;
    if (mode == 1 && failure == 7) {
        --handles;
        return 1;
    }
    return 0;
}

void dn_fs_close(uint32_t h) {
    assert(h == 1 && handles == 1);
    --handles;
}

uint32_t dn_fs_size(int64_t *n, uint32_t) {
    *n = primary.size();
    return failure == 8;
}

uint32_t dn_fs_read(uint32_t, int64_t, void *p, uint32_t n) {
    std::memcpy(p, primary.data(), n);
    if (failure == 5 && n)
        static_cast<char *>(p)[0] = '!';
    return failure == 4;
}

uint32_t dn_fs_create(const char *p, int64_t n) {
    assert(std::strcmp(p, "savedata:/damage_numbers.ini") == 0);
    ++accesses;
    if (failure == 9)
        return 1;
    exists = true;
    primary.resize(n);
    return 0;
}

uint32_t dn_fs_resize(uint32_t, int64_t n) {
    primary.resize(n);
    return failure == 1;
}

uint32_t dn_fs_write(uint32_t, int64_t, const void *p, uint32_t n, const uint32_t *f) {
    assert(*f == 1);
    primary.assign(static_cast<const char *>(p), failure == 2 ? 1 : n);
    return failure == 2;
}

uint32_t dn_fs_flush(uint32_t) {
    return failure == 3;
}

uint32_t dn_fs_commit(const char *p) {
    assert(std::strcmp(p, "savedata") == 0);
    ++commits;
    return failure == 6;
}
}

static void balanced() {
    assert(!mounts && !handles && !guards && backup == "untouched legacy backup");
}

int main() {
    using namespace damage_numbers;
    Settings session{};
    session.style = VisualStyle::Rise;
    session.scale = 1.5f;
    assert(settings_save(session));
    balanced();
    assert(exists && primary.find("version=") == std::string::npos);
    Settings restarted{};
    assert(settings_load(restarted) && restarted.style == VisualStyle::Rise &&
           restarted.scale == 1.5f);
    balanced();
    primary = "[DamageNumbers]\nversion=2\nenabled=0\nstyle=rise\nscale=1.75\nlifetime=1.5\n";
    Settings legacy{};
    assert(settings_load(legacy) && !legacy.enabled && legacy.style == VisualStyle::Rise &&
           legacy.scale == 1.75f && legacy.lifetime == 1.5f);
    balanced();
    assert(settings_save(legacy) && primary.find("version=") == std::string::npos);
    balanced();
    Settings migrated{};
    assert(settings_load(migrated) && !migrated.enabled && migrated.style == VisualStyle::Rise &&
           migrated.scale == 1.75f && migrated.lifetime == 1.5f);
    balanced();
    session.style = VisualStyle::Classic;
    assert(settings_save(session) && settings_save(session));
    balanced();
    for (failure = 1; failure <= 9; ++failure) {
        exists = failure != 9;
        assert(!settings_save(session));
        balanced();
        assert(session.style == VisualStyle::Classic && std::strstr(settings_status(), "session"));
    }
    failure = 0;
    available = false;
    assert(!settings_save(session));
    balanced();
    available = true;
    exists = true;
    primary = "invalid primary";
    Settings defaults{};
    assert(!settings_load(defaults) && defaults.style == VisualStyle::Wilds);
    balanced();
    exists = false;
    assert(!settings_load(defaults));
    balanced();
    auto invalid = session;
    invalid.scale = 99;
    auto before = accesses;
    assert(!settings_save(invalid) && accesses == before);
    balanced();
    assert(settings_save(session));
    Settings again{};
    assert(settings_load(again) && again.style == VisualStyle::Classic);
    balanced();
    std::printf(
        "PASS: production primary-only persistence, versionless restart/repeated saves, existing version-2 migration, ignored backup, invalid input and nine storage failures (%u accesses, %u commits)\n",
        accesses, commits);
}
