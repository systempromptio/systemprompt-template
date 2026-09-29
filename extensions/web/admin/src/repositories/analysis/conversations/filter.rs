//! The enumerated narrowings of the conversations page: the judge-verdict
//! band a row must fall in, the deterministic flag it must carry, the column
//! it sorts by and the dimension the breakdown groups by. Each is a closed
//! vocabulary bound as text and matched by a `CASE` arm in `page.sql`.

/// Narrowing by the judge's single completion score.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgedFilter {
    Judged,
    Unjudged,
    Low,
    High,
}

impl JudgedFilter {
    pub const ALL: [(Self, &'static str); 4] = [
        (Self::Judged, "Judged"),
        (Self::Unjudged, "Not judged yet"),
        (Self::Low, "Completion below 50"),
        (Self::High, "Completion 80 and above"),
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Judged => "judged",
            Self::Unjudged => "unjudged",
            Self::Low => "low",
            Self::High => "high",
        }
    }

    #[must_use]
    pub fn parse_judged_filter(value: Option<&str>) -> Option<Self> {
        match value {
            Some("judged") => Some(Self::Judged),
            Some("unjudged") => Some(Self::Unjudged),
            Some("low") => Some(Self::Low),
            Some("high") => Some(Self::High),
            _ => None,
        }
    }
}

/// Narrowing by something the record shows the conversation did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagFilter {
    Errors,
    Denied,
    Tools,
    Skills,
    Safety,
}

impl FlagFilter {
    pub const ALL: [(Self, &'static str); 5] = [
        (Self::Errors, "Had failed requests"),
        (Self::Denied, "Had a denied tool call"),
        (Self::Tools, "Used tools"),
        (Self::Skills, "Invoked a skill"),
        (Self::Safety, "Had a safety finding"),
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Errors => "errors",
            Self::Denied => "denied",
            Self::Tools => "tools",
            Self::Skills => "skills",
            Self::Safety => "safety",
        }
    }

    #[must_use]
    pub fn parse_flag_filter(value: Option<&str>) -> Option<Self> {
        match value {
            Some("errors") => Some(Self::Errors),
            Some("denied") => Some(Self::Denied),
            Some("tools") => Some(Self::Tools),
            Some("skills") => Some(Self::Skills),
            Some("safety") => Some(Self::Safety),
            _ => None,
        }
    }
}

/// The column the rows sort by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FactSort {
    #[default]
    Activity,
    Turns,
    Tokens,
    Cost,
    Tools,
    Errors,
    Latency,
    Active,
    Duration,
    Completion,
}

impl FactSort {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Turns => "turns",
            Self::Tokens => "tokens",
            Self::Cost => "cost",
            Self::Tools => "tools",
            Self::Errors => "errors",
            Self::Latency => "latency",
            Self::Active => "active",
            Self::Duration => "duration",
            Self::Completion => "completion",
        }
    }

    #[must_use]
    pub fn parse_fact_sort(value: Option<&str>) -> Self {
        match value {
            Some("turns") => Self::Turns,
            Some("tokens") => Self::Tokens,
            Some("cost") => Self::Cost,
            Some("tools") => Self::Tools,
            Some("errors") => Self::Errors,
            Some("latency") => Self::Latency,
            Some("active") => Self::Active,
            Some("duration") => Self::Duration,
            Some("completion") => Self::Completion,
            _ => Self::Activity,
        }
    }
}

/// The dimension the breakdown table groups the filtered set by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BreakdownBy {
    Category,
    // Why: the default while no judge labels conversations — every row
    // would otherwise sit in one "unjudged" intent bucket.
    #[default]
    Model,
    Client,
    Group,
    Project,
    User,
    Skill,
    Outcome,
}

impl BreakdownBy {
    pub const ALL: [Self; 8] = [
        Self::Model,
        Self::Category,
        Self::Client,
        Self::Group,
        Self::Project,
        Self::User,
        Self::Skill,
        Self::Outcome,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Category => "category",
            Self::Model => "model",
            Self::Client => "client",
            Self::Group => "group",
            Self::Project => "project",
            Self::User => "user",
            Self::Skill => "skill",
            Self::Outcome => "outcome",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Category => "By intent",
            Self::Model => "By model",
            Self::Client => "By client",
            Self::Group => "By group",
            Self::Project => "By project",
            Self::User => "By person",
            Self::Skill => "By skill",
            Self::Outcome => "By outcome",
        }
    }

    #[must_use]
    pub fn from_breakdown_param(value: Option<&str>) -> Self {
        match value {
            Some("category") => Self::Category,
            Some("client") => Self::Client,
            Some("group") => Self::Group,
            Some("project") => Self::Project,
            Some("user") => Self::User,
            Some("skill") => Self::Skill,
            Some("outcome") => Self::Outcome,
            _ => Self::Model,
        }
    }
}
