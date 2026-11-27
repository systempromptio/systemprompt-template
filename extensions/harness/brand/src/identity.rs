//! The table of every (brand, channel) build and the identity this binary was
//! built as.

use std::path::PathBuf;

use crate::{Brand, CONFIG_DIR_NAME, Channel, VERSION, local_data_base};

/// Everything that names one (brand, channel) build to the user, the OS and an
/// instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identity {
    brand: Brand,
    channel: Channel,
    display_name: &'static str,
    app_id: &'static str,
    scheme: &'static str,
    version: &'static str,
    config_dir_name: &'static str,
}

macro_rules! entry {
    ($brand:ident, $channel:ident, $product:literal, $app_id:literal, $scheme:literal,
     $suffix:literal, $id_suffix:literal, $scheme_suffix:literal) => {
        Identity {
            brand: Brand::$brand,
            channel: Channel::$channel,
            display_name: concat!($product, $suffix),
            app_id: concat!($app_id, $id_suffix),
            scheme: concat!($scheme, $scheme_suffix),
            version: VERSION,
            config_dir_name: CONFIG_DIR_NAME,
        }
    };
}

macro_rules! brand_rows {
    ($brand:ident, $product:literal, $app_id:literal, $scheme:literal) => {
        [
            entry!($brand, Internal, $product, $app_id, $scheme, "", "", ""),
            entry!(
                $brand, Canary, $product, $app_id, $scheme, " Canary", ".canary", "-canary"
            ),
            entry!(
                $brand, Dev, $product, $app_id, $scheme, " Dev", ".dev", "-dev"
            ),
        ]
    };
}

static TABLE: [[Identity; 3]; 2] = [
    brand_rows!(
        Systemprompt,
        "systemprompt Harness",
        "io.systemprompt.harness",
        "sp-harness"
    ),
    brand_rows!(
        Astound,
        "Astound Work",
        "com.astounddigital.work",
        "astound-work"
    ),
];

#[must_use]
pub fn identity() -> &'static Identity {
    Identity::of(Brand::CURRENT, Channel::CURRENT)
}

impl Identity {
    pub const ALL: [(Brand, Channel); 6] = [
        (Brand::Systemprompt, Channel::Internal),
        (Brand::Systemprompt, Channel::Canary),
        (Brand::Systemprompt, Channel::Dev),
        (Brand::Astound, Channel::Internal),
        (Brand::Astound, Channel::Canary),
        (Brand::Astound, Channel::Dev),
    ];

    #[must_use]
    pub fn of(brand: Brand, channel: Channel) -> &'static Self {
        &TABLE[brand.index()][channel.index()]
    }

    #[must_use]
    pub const fn brand(&self) -> Brand {
        self.brand
    }

    #[must_use]
    pub const fn channel(&self) -> Channel {
        self.channel
    }

    #[must_use]
    pub const fn product(&self) -> &'static str {
        self.brand.product()
    }

    #[must_use]
    pub const fn vendor(&self) -> &'static str {
        self.brand.vendor()
    }

    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        self.display_name
    }

    #[must_use]
    pub const fn app_id(&self) -> &'static str {
        self.app_id
    }

    #[must_use]
    pub const fn scheme(&self) -> &'static str {
        self.scheme
    }

    #[must_use]
    pub const fn version(&self) -> &'static str {
        self.version
    }

    #[must_use]
    pub const fn channel_label(&self) -> Option<&'static str> {
        self.channel.label()
    }

    #[must_use]
    pub const fn default_instance(&self) -> &'static str {
        self.brand.default_instance()
    }

    #[must_use]
    pub const fn governance_label(&self) -> &'static str {
        self.brand.governance_label()
    }

    #[must_use]
    pub const fn config_dir_name(&self) -> &'static str {
        self.config_dir_name
    }

    #[must_use]
    pub fn data_dir_relative(&self) -> PathBuf {
        PathBuf::from(self.vendor()).join(self.display_name)
    }

    #[must_use]
    pub fn data_root(&self) -> PathBuf {
        local_data_base().join(self.data_dir_relative())
    }

    #[must_use]
    pub fn device_label(&self, hostname: &str) -> String {
        format!("{} on {}", self.display_name, hostname.trim())
    }
}
