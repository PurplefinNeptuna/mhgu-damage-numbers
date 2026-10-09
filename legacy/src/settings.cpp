// Persist only savedata:/damage_numbers.ini using Forge's counted save mount.
// Save operations validate, flush, read back, and commit; guards balance resources.

#include "log_policy.h"
#include <forge/log.h>
#include "settings.h"
#include "sdk_fs.h"
#include <forge/fs.h>

namespace damage_numbers {
static bool pending = true;
static const char *storage_status = "Default settings; not saved";
static const char *path = "savedata:/damage_numbers.ini";

// RAII releases exactly the counted Forge mount acquired by this object.
class SaveMount {
    bool mounted = forge_fs_mountSaveData();

public:
    ~SaveMount() {
        if (mounted)
            forge_fs_unmountSaveData();
    }

    explicit operator bool() const {
        return mounted;
    }
};

// Read a bounded primary file and accept it only after INI validation.
// Every successfully opened handle is closed on this path.
static bool read_primary(SettingsText &text, Settings &decoded) {
    uint32_t handle = 0;
    if (dn_fs_open(&handle, path, 1))
        return false;
    int64_t size = 0;
    bool ok = !dn_fs_size(&size, handle) && size > 0 && size < int64_t(sizeof(text.bytes));
    if (ok) {
        text.size = unsigned(size);
        ok = !dn_fs_read(handle, 0, text.bytes, text.size) &&
             decode_settings(text.bytes, text.size, decoded);
    }
    dn_fs_close(handle);
    return ok;
}

// Resize/write/flush the primary, then compare its readback byte-for-byte.
// A failed write may have changed disk contents; session settings stay separate.
static bool write_primary(const SettingsText &text) {
    uint32_t handle = 0;
    if (dn_fs_open(&handle, path, 2)) {
        if (dn_fs_create(path, text.size) || dn_fs_open(&handle, path, 2))
            return false;
    }
    const uint32_t flush = 1;
    const bool written = !dn_fs_resize(handle, text.size) &&
                         !dn_fs_write(handle, 0, text.bytes, text.size, &flush) &&
                         !dn_fs_flush(handle);
    dn_fs_close(handle);
    SettingsText verified{};
    Settings decoded{};
    return written && read_primary(verified, decoded) && text.size == verified.size &&
           !std::memcmp(text.bytes, verified.bytes, text.size);
}

// Retry mount availability while pending. Once storage is available, missing
// or invalid primary contents leave the caller's existing/default value intact.
bool settings_load(Settings &out) {
    const FsAbortGuard guard;
    const SaveMount mount;
    if (!mount) {
        storage_status = "Waiting for player save storage; settings apply for this session";
        return false;
    }
    pending = false;
    SettingsText text{};
    if (read_primary(text, out)) {
        storage_status = "Saved settings loaded";
        return true;
    }
    storage_status = "Default settings; no valid saved settings";
    return false;
}

// Validate before writing and commit only verified primary bytes. Failure
// reasons are logged once; guards restore the FS context and release the mount.
bool settings_save(const Settings &value) {
    static mod_log::Once failures;
    static Settings saved{};
    static bool have_saved;

    struct FailureReport {
        mod_log::Once &failures;
        unsigned reason = 0;

        ~FailureReport() {
            if (reason && failures.claim(reason))
                forge_log_warn("[damage-numbers] %s; load a player save and retry Save settings",
                               storage_status);
        }
    } report{failures};

    if (!valid(value))
        return false;
    const SettingsText text = encode_settings(value);
    const FsAbortGuard guard;
    const SaveMount mount;
    if (!mount) {
        storage_status = "Player save storage unavailable; changes remain in this session";
        report.reason = 1;
        return false;
    }
    pending = false;
    if (!write_primary(text) || dn_fs_commit("savedata")) {
        storage_status = "Settings save failed; changes remain in this session";
        report.reason = 3;
        return false;
    }
    // Forge releases its counted save mount (and performs its normal commit).
    storage_status = "Saved damage_numbers.ini in MHGU's save folder";
    if (!have_saved || value.enabled != saved.enabled || value.style != saved.style ||
        value.scale != saved.scale || value.lifetime != saved.lifetime)
        forge_log_info("[damage-numbers] Settings saved");
    saved = value;
    have_saved = true;
    return true;
}

bool settings_pending() {
    return pending;
}

const char *settings_status() {
    return storage_status;
}

void settings_changed() {
    pending = false;
    storage_status = "Changes apply now; Save settings keeps them after restart";
}
} // namespace damage_numbers
