//! Optional: the settings in Purple-lib's Mod Options and a help topic per setting on that page's
//! own help. Purple-lib calls the plugin's `purplelib_on_register` with its table of functions at
//! start-up; without Purple-lib nothing calls it, this module does nothing and the Plugin UI panel
//! stays the way to change the settings.
//!
//! Purple-lib calls `get` and `set` inside the game's update, so they only touch the lock-free
//! `runtime::SETTINGS`. The INI is written on the render thread, which picks up `SAVE`.
use crate::runtime::{EDITED, SETTINGS};
use core::cell::UnsafeCell;
use core::ffi::{c_char, c_void};
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use damage_numbers_core::Settings;
use damage_numbers_core::purple_lib::{self as api, Api, Help, LANGUAGES, OptionRow, Options, ROW_TEXT};

/// Save the INI on the render thread's next frame.
pub static SAVE: AtomicBool = AtomicBool::new(false);
/// A saved row changed since the page opened; `closed` turns it into `SAVE`.
static DIRTY: AtomicBool = AtomicBool::new(false);
/// Purple-lib's table, kept from a successful registration; it stays valid for the whole run.
static API: AtomicPtr<Api> = AtomicPtr::new(core::ptr::null_mut());

/// Whether the Options window shows this page now and its own help is not covering it; false without
/// Purple-lib. Before API version 4 the help cannot be told apart, so the page counts as shown. Any
/// thread.
pub fn shown() -> bool {
    let api = API.load(Ordering::Acquire);
    if api.is_null() {
        return false;
    }
    // SAFETY: only Purple-lib's process-lifetime table is stored; each field is read through the raw
    // pointer, and `help_shown` only when the table is version 4 or later and so contains it.
    unsafe { ((*api).options_shown)(&OPTIONS.0) && ((*api).version < api::HELP_SHOWN_VERSION || !((*api).help_shown)(&OPTIONS.0)) }
}

extern "C" fn title(_: *mut c_void, _: u32) -> *const c_char {
    api::TITLE.as_ptr()
}

/// Label pointer arrays per language and row, filled once in `register` before Purple-lib can call
/// `row` (Purple-lib keeps the pointers).
struct Labels(UnsafeCell<[[[*const c_char; 3]; api::ROWS as usize]; LANGUAGES]>);
// SAFETY: written once before registration, read-only afterwards.
unsafe impl Sync for Labels {}
static LABELS: Labels = Labels(UnsafeCell::new([[[core::ptr::null(); 3]; api::ROWS as usize]; LANGUAGES]));

extern "C" fn row(_: *mut c_void, language: u32, index: u32, out: *mut OptionRow) -> bool {
    let Some(text) = ROW_TEXT.get(index as usize) else { return false };
    if out.is_null() {
        return false;
    }
    let language = api::language_index(language);
    let shape = api::shape(index);
    let labels = api::labels(index, language);
    // SAFETY: filled before registration and never written again.
    let pointers = unsafe { &(*LABELS.0.get())[language][index as usize] };
    // SAFETY: Purple-lib passes a writable, zeroed `OptionRow`.
    unsafe {
        (*out).name = text.name[language].as_ptr();
        (*out).help = text.help[language].as_ptr();
        (*out).kind = shape.kind;
        (*out).flags = shape.flags;
        if labels.is_empty() {
            (*out).value_count = shape.count;
            (*out).first = shape.first;
            (*out).step = shape.step;
            if shape.kind == api::ROW_NUMBERS {
                (*out).suffix = api::PERCENT.as_ptr();
            }
        } else {
            (*out).labels = pointers.as_ptr();
            (*out).label_count = labels.len() as u32;
        }
    }
    true
}

extern "C" fn get(_: *mut c_void, index: u32) -> u32 {
    api::value(SETTINGS.load(), index)
}

/// Applied at once; the render thread copies the shared settings into the Model next frame.
extern "C" fn set(_: *mut c_void, index: u32, value: u32) {
    if let Some(s) = api::with_value(SETTINGS.load(), index, value)
        && SETTINGS.store(s)
    {
        EDITED.store(true, Ordering::Release);
        if api::saved(index) {
            DIRTY.store(true, Ordering::Release);
        }
    }
}

/// Only a change to a saved row writes the INI; the preview rows are session-only.
extern "C" fn closed(_: *mut c_void) {
    if DIRTY.swap(false, Ordering::AcqRel) {
        SAVE.store(true, Ordering::Release);
    }
}

extern "C" fn value_text(_: *mut c_void, language: u32, index: u32, value: u32) -> *const c_char {
    if index != api::ROW_DURATION {
        return core::ptr::null();
    }
    api::duration_text(api::language_index(language), value).map_or(core::ptr::null(), |t| t.as_ptr())
}

/// Restore defaults. Purple-lib calls `closed` only after a `set`, so this asks for the save itself.
extern "C" fn action(_: *mut c_void, index: u32) {
    if index == api::ROW_RESTORE && SETTINGS.store(Settings::default()) {
        EDITED.store(true, Ordering::Release);
        DIRTY.store(false, Ordering::Release);
        SAVE.store(true, Ordering::Release);
    }
}

// A topic's context is its row index.
extern "C" fn help_title(context: *mut c_void, language: u32) -> *const c_char {
    ROW_TEXT[context as usize].name[api::language_index(language)].as_ptr()
}
extern "C" fn help_page(context: *mut c_void, language: u32, index: u32) -> *const c_char {
    api::HELP_TEXT[context as usize][api::language_index(language)]
        .get(index as usize)
        .map_or(core::ptr::null(), |p| p.as_ptr())
}

/// Read-only records of static data; no context is ever dereferenced.
struct Shared<T>(T);
// SAFETY: see above.
unsafe impl<T> Sync for Shared<T> {}
static OPTIONS: Shared<Options> = Shared(Options {
    size: core::mem::size_of::<Options>() as u32,
    context: core::ptr::null_mut(),
    title: Some(title),
    row_count: api::ROWS,
    row: Some(row),
    get: Some(get),
    set: Some(set),
    closed: Some(closed),
    value_text: Some(value_text),
    action: Some(action),
});
const fn topic(row: usize) -> Help {
    Help {
        size: core::mem::size_of::<Help>() as u32,
        context: row as *mut c_void,
        category: api::HELP_CATEGORY,
        page_count: api::HELP_PAGES[row],
        title: Some(help_title),
        page: Some(help_page),
        open: None,
    }
}
static HELP: Shared<[Help; api::TOPICS]> = Shared([topic(0), topic(1), topic(2), topic(3), topic(4), topic(5), topic(6)]);

pub enum Registered {
    /// Purple-lib is older than `API_VERSION`; nothing was registered.
    OlderVersion(u32),
    Refused { options: bool, help: bool },
    Done,
    /// A second call: the first one's registration stands.
    Repeated,
}

/// Register through Purple-lib's table, from `purplelib_on_register`: the one time Purple-lib
/// accepts records, during its first update.
///
/// # Safety
/// `purple` is Purple-lib's non-null table, valid for the whole run; it may end before `help_shown`.
pub unsafe fn register(purple: *const Api) -> Registered {
    // SAFETY: the caller's contract; only fields every version 3 table has are read here.
    let (version, add_options, add_set_help) = unsafe { ((*purple).version, (*purple).add_options, (*purple).add_set_help) };
    static CALLED: AtomicBool = AtomicBool::new(false);
    // A later Purple-lib still reads these records, by their `size`; an older one lacks the API.
    if version < api::API_VERSION {
        return Registered::OlderVersion(version);
    }
    // Purple-lib keeps the label pointers after the first call, so the table is filled only once.
    if CALLED.swap(true, Ordering::AcqRel) {
        return Registered::Repeated;
    }
    // SAFETY: nothing reads the table before the registration below, and this runs once.
    let table = unsafe { &mut *LABELS.0.get() };
    for (language, rows) in table.iter_mut().enumerate() {
        for (row, slots) in rows.iter_mut().enumerate() {
            for (slot, label) in slots.iter_mut().zip(api::labels(row as u32, language)) {
                *slot = label.as_ptr();
            }
        }
    }
    // The page's help names the page, so it goes in after the page; topics show in this order.
    let options = add_options(&OPTIONS.0);
    if options {
        API.store(purple.cast_mut(), Ordering::Release);
    }
    let help = options && HELP.0.iter().all(|topic| add_set_help(&OPTIONS.0, topic));
    if options && help { Registered::Done } else { Registered::Refused { options, help } }
}
