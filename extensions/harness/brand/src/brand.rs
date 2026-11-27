//! The product line a build ships as, chosen by the `brand-astound` cargo
//! feature.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

/// The product line a build ships as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Brand {
    Systemprompt,
    Astound,
}

impl Brand {
    pub const ALL: [Self; 2] = [Self::Systemprompt, Self::Astound];

    #[cfg(not(feature = "brand-astound"))]
    pub const CURRENT: Self = Self::Systemprompt;
    #[cfg(feature = "brand-astound")]
    pub const CURRENT: Self = Self::Astound;

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Systemprompt => "systemprompt",
            Self::Astound => "astound",
        }
    }

    #[must_use]
    pub const fn product(self) -> &'static str {
        match self {
            Self::Systemprompt => "systemprompt Harness",
            Self::Astound => "Astound Work",
        }
    }

    #[must_use]
    pub const fn vendor(self) -> &'static str {
        match self {
            Self::Systemprompt => "systemprompt",
            Self::Astound => "Astound Digital",
        }
    }

    #[must_use]
    pub const fn default_instance(self) -> &'static str {
        match self {
            Self::Systemprompt => "https://systemprompt.io",
            Self::Astound => "https://systemprompt.astound.digital",
        }
    }

    #[must_use]
    pub const fn governance_label(self) -> &'static str {
        match self {
            Self::Systemprompt | Self::Astound => "Governance",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Systemprompt => 0,
            Self::Astound => 1,
        }
    }
}
