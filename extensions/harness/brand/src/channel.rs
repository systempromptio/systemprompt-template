//! The release track a build ships on, chosen by the `channel-canary` /
//! `channel-dev` features.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

/// The release track a build ships on; each track installs beside the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Channel {
    Internal,
    Canary,
    Dev,
}

impl Channel {
    pub const ALL: [Self; 3] = [Self::Internal, Self::Canary, Self::Dev];

    // Why: cargo unifies features across a workspace build, so both channel
    // features can be on at once; the narrower channel wins rather than failing
    // the build.
    #[cfg(feature = "channel-dev")]
    pub const CURRENT: Self = Self::Dev;
    #[cfg(all(feature = "channel-canary", not(feature = "channel-dev")))]
    pub const CURRENT: Self = Self::Canary;
    #[cfg(not(any(feature = "channel-canary", feature = "channel-dev")))]
    pub const CURRENT: Self = Self::Internal;

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Internal => "internal",
            Self::Canary => "canary",
            Self::Dev => "dev",
        }
    }

    #[must_use]
    pub const fn label(self) -> Option<&'static str> {
        match self {
            Self::Internal => None,
            Self::Canary => Some("Canary"),
            Self::Dev => Some("Dev"),
        }
    }

    #[must_use]
    pub const fn app_id_suffix(self) -> &'static str {
        match self {
            Self::Internal => "",
            Self::Canary => ".canary",
            Self::Dev => ".dev",
        }
    }

    #[must_use]
    pub const fn scheme_suffix(self) -> &'static str {
        match self {
            Self::Internal => "",
            Self::Canary => "-canary",
            Self::Dev => "-dev",
        }
    }

    #[must_use]
    pub const fn display_suffix(self) -> &'static str {
        match self {
            Self::Internal => "",
            Self::Canary => " Canary",
            Self::Dev => " Dev",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Internal => 0,
            Self::Canary => 1,
            Self::Dev => 2,
        }
    }
}
