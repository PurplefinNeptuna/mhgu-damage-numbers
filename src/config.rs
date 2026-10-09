//! Upstream forge-rs INI derives and storage helpers. Loading may create defaults, as requested.
use core::{fmt, str::FromStr};
use damage_numbers_core::{Settings, Style};
use forge::config::{IniSerialize, ini::Writer};

/// Derives use each field's Default, so wrappers preserve the gameplay defaults on a missing or invalid key.
// This one-field wrapper gives INI parsing a true default. Rust's plain bool
// would default to false when the derive encounters a missing/invalid field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Enabled(pub bool);
impl Default for Enabled {
    fn default() -> Self {
        Self(true)
    }
}
impl FromStr for Enabled {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        s.parse::<bool>().map(Self).map_err(|_| ())
    }
}
impl fmt::Display for Enabled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
// Generate the same default, parser and formatter for each floating setting.
// FromStr is used when reading INI text; Display is used when writing it.
// Reject NaN/infinity and out-of-range values before they reach layout math.
macro_rules! bounded_setting {
    ($name:ident,$default:expr,$min:expr,$max:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name(pub f32);
        impl Default for $name {
            fn default() -> Self {
                Self($default)
            }
        }
        impl FromStr for $name {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, ()> {
                let value = s.parse::<f32>().map_err(|_| ())?;
                if value.is_finite() && ($min..=$max).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(())
                }
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}
// The arguments are type name, default, inclusive minimum and maximum.
// These limits match the settings sliders and Settings::valid in the core.
bounded_setting!(Scale, 1.0, 0.5, 3.0);
bounded_setting!(Lifetime, 0.9, 0.4, 2.0);
// The derives turn this typed structure into the DamageNumbers INI section.
// The section attribute starts the section; the following fields belong to it.
#[derive(Default, Clone, Copy, Debug, PartialEq, forge::IniSerialize, forge::IniDeserialize)]
pub struct Config {
    #[section("DamageNumbers")]
    pub enabled: Enabled,
    pub style: Style,
    pub scale: Scale,
    pub lifetime: Lifetime,
}
// Unwrap the INI-specific field types into the portable gameplay settings.
// The reverse conversion below wraps a settings snapshot for serialization.
impl From<Config> for Settings {
    fn from(c: Config) -> Self {
        Self {
            enabled: c.enabled.0,
            style: c.style,
            scale: c.scale.0,
            lifetime: c.lifetime.0,
        }
    }
}
impl From<Settings> for Config {
    fn from(s: Settings) -> Self {
        Self {
            enabled: Enabled(s.enabled),
            style: s.style,
            scale: Scale(s.scale),
            lifetime: Lifetime(s.lifetime),
        }
    }
}
pub const FILENAME: &str = "damage_numbers_rs.ini";
// Stock load reads the plugin-specific file or automatically writes defaults
// when needed. Consumer::load_config calls it once storage becomes available.
pub fn load() -> Result<Settings, alloc::string::String> {
    forge::config::load::<Config>(FILENAME).map(Into::into)
}
pub fn save(settings: Settings) -> Result<(), alloc::string::String> {
    // Keep the counted save-data mount alive until Writer finishes. The guard's
    // Drop releases it on both success and error when this function returns.
    // UI changes and Reset stay in memory until this explicit Save path runs.
    let _mount = forge::fs::savedata::ScopedMount::new();
    let path = alloc::format!("{}:/{}", forge::fs::savedata::mount_point(), FILENAME);
    Writer::write(&path, &Config::from(settings).to_ini())
}
