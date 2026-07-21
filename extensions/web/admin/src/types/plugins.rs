//! Plugin listing value types.

pub(super) const fn default_true() -> bool {
    true
}

pub(super) const fn default_port() -> u16 {
    5000
}

pub(super) fn default_version() -> String {
    "0.1.0".to_owned()
}

pub(super) fn default_internal() -> String {
    "internal".to_owned()
}

pub(super) fn default_external() -> String {
    "external".to_owned()
}
