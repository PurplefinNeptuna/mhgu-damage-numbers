//! Execute the production derives and vendored config parser/writer against memory-backed MtFile I/O.
extern crate alloc;
extern crate self as forge;
pub use forge_macros::{IniDeserialize, IniSerialize};
pub mod config {
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}
#[path = "../../../src/config.rs"]
pub mod settings;
thread_local! {static FILES:std::cell::RefCell<std::collections::BTreeMap<String,Vec<u8>>>=Default::default();static MOUNTS:std::cell::Cell<u32>=const{std::cell::Cell::new(0)};}
pub mod fs {
    pub fn file_exists(path: &str) -> bool {
        super::FILES.with(|files| files.borrow().contains_key(path))
    }
    pub mod savedata {
        pub fn mount_point() -> &'static str {
            "savedata"
        }
        pub struct ScopedMount;
        impl ScopedMount {
            pub fn new() -> Self {
                super::super::MOUNTS.with(|v| v.set(v.get() + 1));
                Self
            }
        }
        impl Drop for ScopedMount {
            fn drop(&mut self) {
                super::super::MOUNTS.with(|v| v.set(v.get() - 1));
            }
        }
    }
}
pub mod mt {
    pub mod object {
        pub trait Object {
            fn destroy(&mut self);
        }
    }
    pub mod fs {
        use core::{ffi::CStr, fmt};
        pub enum OpenMode {
            Read,
            Write,
        }
        pub struct MtFile {
            path: String,
            data: Vec<u8>,
            write: bool,
        }
        impl MtFile {
            pub fn open(path: &CStr, mode: OpenMode) -> Result<&'static mut Self, &'static str> {
                let path = path.to_str().unwrap().to_owned();
                let write = matches!(mode, OpenMode::Write);
                let data = if write {
                    Vec::new()
                } else {
                    crate::FILES
                        .with(|files| files.borrow().get(&path).cloned())
                        .ok_or("missing file")?
                };
                Ok(Box::leak(Box::new(Self { path, data, write })))
            }
            pub fn size(&self) -> usize {
                self.data.len()
            }
            pub fn read(&mut self, out: &mut [u8]) -> usize {
                out.copy_from_slice(&self.data);
                out.len()
            }
            pub fn close(&mut self) {
                if self.write {
                    crate::FILES.with(|files| {
                        files
                            .borrow_mut()
                            .insert(self.path.clone(), self.data.clone())
                    });
                }
            }
        }
        impl fmt::Write for MtFile {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.data.extend_from_slice(s.as_bytes());
                Ok(())
            }
        }
        impl super::object::Object for MtFile {
            fn destroy(&mut self) { /* The fixture keeps its stream allocation until test process exit. */
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use damage_numbers_core::{Settings, Style};
    fn clear() {
        FILES.with(|files| files.borrow_mut().clear());
        assert_eq!(MOUNTS.with(|v| v.get()), 0);
    }
    #[test]
    fn missing_config_is_created_by_upstream_load() {
        clear();
        let loaded = settings::load().unwrap();
        assert_eq!(loaded, Settings::default());
        let data = FILES.with(|files| files.borrow()["savedata:/damage_numbers_rs.ini"].clone());
        let text = String::from_utf8(data).unwrap();
        assert!(text.contains("enabled = true") && text.contains("lifetime = 0.9"));
        assert_eq!(MOUNTS.with(|v| v.get()), 0);
    }
    #[test]
    fn save_round_trips_all_settings_and_styles() {
        clear();
        for style in [Style::Wilds, Style::Classic, Style::Rise] {
            let value = Settings {
                enabled: false,
                style,
                scale: 2.75,
                lifetime: 1.8,
            };
            settings::save(value).unwrap();
            assert_eq!(settings::load().unwrap(), value);
        }
        assert_eq!(MOUNTS.with(|v| v.get()), 0);
    }
    #[test]
    fn invalid_or_missing_fields_use_gameplay_defaults() {
        clear();
        FILES.with(|files| {
            files.borrow_mut().insert(
                "savedata:/damage_numbers_rs.ini".into(),
                b"[DamageNumbers]\nscale=NaN\nlifetime=99\nstyle=invalid\n".to_vec(),
            )
        });
        assert_eq!(settings::load().unwrap(), Settings::default());
    }
    #[test]
    fn restore_defaults_then_save_uses_upstream_writer() {
        clear();
        settings::save(Settings {
            enabled: false,
            style: Style::Rise,
            scale: 3.0,
            lifetime: 2.0,
        })
        .unwrap();
        settings::save(Settings::default()).unwrap();
        assert_eq!(settings::load().unwrap(), Settings::default());
    }
    #[test]
    fn scalar_validation_rejects_nonfinite_and_out_of_range() {
        for value in ["NaN", "inf", "-1", "0", "3.1"] {
            assert!(value.parse::<settings::Scale>().is_err());
        }
        for value in ["NaN", "0.3", "2.1"] {
            assert!(value.parse::<settings::Lifetime>().is_err());
        }
    }
}
