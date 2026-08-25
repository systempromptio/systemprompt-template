//! Budget-health classification shared by every page that renders a budget
//! pill, so a customer that reads as "at risk" on one page reads the same on
//! all of them.

const WARN_PCT: i64 = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BudgetState {
    Ok,
    Warn,
    Over,
}

impl BudgetState {
    pub(crate) const fn from_pct(pct: i64) -> Self {
        if pct >= 100 {
            Self::Over
        } else if pct >= WARN_PCT {
            Self::Warn
        } else {
            Self::Ok
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Over => "over",
        }
    }
}
