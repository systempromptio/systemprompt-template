//! The product line a build ships as.

/// The product line a build ships as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Brand {
    Systemprompt,
}

impl Brand {
    pub const ALL: [Self; 1] = [Self::Systemprompt];

    pub const CURRENT: Self = Self::Systemprompt;

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Systemprompt => "systemprompt",
        }
    }

    #[must_use]
    pub const fn product(self) -> &'static str {
        match self {
            Self::Systemprompt => "systemprompt Harness",
        }
    }

    #[must_use]
    pub const fn vendor(self) -> &'static str {
        match self {
            Self::Systemprompt => "systemprompt",
        }
    }

    #[must_use]
    pub const fn default_instance(self) -> &'static str {
        match self {
            Self::Systemprompt => "https://systemprompt.io",
        }
    }

    #[must_use]
    pub const fn governance_label(self) -> &'static str {
        match self {
            Self::Systemprompt => "Governance",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Systemprompt => 0,
        }
    }
}
