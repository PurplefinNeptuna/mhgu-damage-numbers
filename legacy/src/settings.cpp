#include "settings.h"
#include "sdk_fs.h"
#include <forge/fs.h>

namespace damage_numbers {
static bool pending=true;
static const char* storage_status="Default settings; not saved";
static const char* paths[]={"savedata:/damage_numbers.ini","savedata:/damage_numbers_backup.ini"};
class SaveMount {
    bool mounted=forge_fs_mountSaveData();
public:
    ~SaveMount() { if (mounted) forge_fs_unmountSaveData(); }
    explicit operator bool() const { return mounted; }
};
static bool read_slot(unsigned slot,SettingsText& text,Settings& decoded) {
    uint32_t handle=0;
    if (dn_fs_open(&handle,paths[slot],1)) return false;
    int64_t size=0;
    bool ok=!dn_fs_size(&size,handle) && size>0 && size<int64_t(sizeof(text.bytes));
    if (ok) {
        text.size=unsigned(size);
        ok=!dn_fs_read(handle,0,text.bytes,text.size) && decode_settings(text.bytes,text.size,decoded);
    }
    dn_fs_close(handle); return ok;
}
static bool write_slot(unsigned slot,const SettingsText& text) {
    uint32_t handle=0;
    if (dn_fs_open(&handle,paths[slot],2)) {
        if (dn_fs_create(paths[slot],text.size) || dn_fs_open(&handle,paths[slot],2)) return false;
    }
    const uint32_t flush=1;
    const bool written=!dn_fs_resize(handle,text.size) &&
        !dn_fs_write(handle,0,text.bytes,text.size,&flush) && !dn_fs_flush(handle);
    dn_fs_close(handle);
    SettingsText verified{}; Settings decoded{};
    return written && read_slot(slot,verified,decoded) && text.size==verified.size &&
        !std::memcmp(text.bytes,verified.bytes,text.size);
}
bool settings_load(Settings& out) {
    const FsAbortGuard guard;
    const SaveMount mount;
    if (!mount) { storage_status="Waiting for player save storage; settings apply for this session"; return false; }
    pending=false;
    SettingsText text{};
    if (read_slot(0,text,out)) { storage_status="Saved settings loaded"; return true; }
    if (read_slot(1,text,out)) { storage_status="Settings recovered from backup"; return true; }
    storage_status="Default settings; no valid saved settings"; return false;
}
bool settings_save(const Settings& value) {
    if (!valid(value)) return false;
    const FsAbortGuard guard;
    const SaveMount mount;
    if (!mount) { storage_status="Player save storage unavailable; changes remain in this session"; return false; }
    pending=false;
    SettingsText previous{}; Settings decoded{};
    // Preserve the previous valid primary before replacing it. If the primary
    // is already damaged, leave the existing recovery copy untouched.
    if (read_slot(0,previous,decoded) && !write_slot(1,previous)) {
        storage_status="Settings backup failed; previous settings retained"; return false;
    }
    const SettingsText text=encode_settings(value);
    if (!write_slot(0,text) || dn_fs_commit("savedata")) {
        storage_status="Settings save failed; changes remain in this session"; return false;
    }
    // Forge releases its counted save mount (and performs its normal commit).
    storage_status="Settings saved beside Better Input"; return true;
}
bool settings_pending() { return pending; }
const char* settings_status() { return storage_status; }
void settings_changed() { pending=false; storage_status="Changes apply now; Save settings keeps them after restart"; }
}
