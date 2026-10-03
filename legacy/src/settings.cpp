#include "settings.h"
#include "sdk_fs.h"

namespace damage_numbers {
static bool mounted;
static uint32_t generation;
static unsigned selected=1;
static const char* storage_status="Default settings; not saved";
static const char* paths[]={"dnum:/forge/mhgu-damage-numbers-a.cfg",
                            "dnum:/forge/mhgu-damage-numbers-b.cfg"};
static bool mount_sd() {
    if (mounted) return true;
    if (dn_fs_mount_sd("dnum")) {
        storage_status="SD storage unavailable; settings apply for this session";
        return false;
    }
    mounted=true; return true;
}
static bool read_slot(unsigned slot,StoredSettings& out,Settings& decoded) {
    uint32_t handle=0;
    if (dn_fs_open(&handle,paths[slot],1)) return false;
    int64_t size=0;
    const bool ok=!dn_fs_size(&size,handle) && size==int64_t(sizeof(out)) &&
        !dn_fs_read(handle,0,&out,sizeof(out)) && decode_settings(out,decoded);
    dn_fs_close(handle); return ok;
}
bool settings_load(Settings& out) {
    const FsAbortGuard guard;
    if (!mount_sd()) return false;
    StoredSettings records[2]{}; Settings decoded[2]{};
    const bool first=read_slot(0,records[0],decoded[0]);
    const bool second=read_slot(1,records[1],decoded[1]);
    if (!first && !second) { storage_status="Default settings; no valid saved settings"; return false; }
    selected=second && (!first || newer_settings(records[1].words[2],records[0].words[2])) ? 1 : 0;
    generation=records[selected].words[2]; out=decoded[selected];
    storage_status="Saved settings loaded"; return true;
}
bool settings_save(const Settings& value) {
    const FsAbortGuard guard;
    if (!valid(value) || !mount_sd()) return false;
    int type=-1;
    if (dn_fs_entry(&type,"dnum:/forge")) {
        if (dn_fs_directory("dnum:/forge")) { storage_status="Could not create settings folder; changes remain in this session"; return false; }
    } else if (type!=0) { storage_status="Settings folder is unavailable; changes remain in this session"; return false; }
    const unsigned slot=1-selected;
    const StoredSettings record=encode_settings(value,generation+1);
    uint32_t handle=0;
    if (dn_fs_open(&handle,paths[slot],2)) {
        if (dn_fs_create(paths[slot],sizeof(record)) || dn_fs_open(&handle,paths[slot],2)) {
            storage_status="Could not open settings file; changes remain in this session"; return false;
        }
    }
    const uint32_t flush=1;
    const bool written=!dn_fs_resize(handle,sizeof(record)) &&
        !dn_fs_write(handle,0,&record,sizeof(record),&flush) && !dn_fs_flush(handle);
    dn_fs_close(handle);
    StoredSettings verified{}; Settings decoded{};
    if (!written || !read_slot(slot,verified,decoded) || std::memcmp(&record,&verified,sizeof(record))) {
        storage_status="Settings save failed; previous saved settings retained"; return false;
    }
    selected=slot; generation=record.words[2]; storage_status="Settings saved to SD card"; return true;
}
const char* settings_status() { return storage_status; }
void settings_changed() { storage_status="Changes apply now; Save settings keeps them after restart"; }
}
