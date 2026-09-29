//! The closed vocabularies of the Tools and Artifacts pages: the state a
//! call must be in, the artifact kind a result must have, the column the
//! rows sort by and the dimension the breakdown groups by. Each is bound as
//! text and matched by a `CASE` arm in `page.sql`.

/// Where a call is in its life: asked for, run, run and failed, or run
/// without a request that asked for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolState {
    Intended,
    Executed,
    Failed,
    Unattested,
}

impl ToolState {
    pub const ALL: [(Self, &'static str); 4] = [
        (Self::Executed, "Executed"),
        (Self::Failed, "Failed"),
        (Self::Intended, "Intended only"),
        (Self::Unattested, "Unattested"),
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Intended => "intended",
            Self::Executed => "executed",
            Self::Failed => "failed",
            Self::Unattested => "unattested",
        }
    }

    // Why: lint-ok: unused-pub — the /admin/tools page parses its query with
    // it; that page lands with Stage 3 phase 9 (tools, artifacts).
    #[must_use]
    pub fn parse_tool_state(value: Option<&str>) -> Option<Self> {
        match value {
            Some("intended") => Some(Self::Intended),
            Some("executed") => Some(Self::Executed),
            Some("failed") => Some(Self::Failed),
            Some("unattested") => Some(Self::Unattested),
            _ => None,
        }
    }
}

/// The one artifact rule's four answers (schema 46 `artifact_kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    File,
    Card,
    Ui,
    Body,
}

impl ArtifactKind {
    pub const ALL: [Self; 4] = [Self::File, Self::Card, Self::Ui, Self::Body];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Card => "card",
            Self::Ui => "ui",
            Self::Body => "body",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Card => "Card",
            Self::Ui => "UI",
            Self::Body => "Body",
        }
    }

    // Why: the glyph is the kind's name in `components/icon`.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        self.as_str()
    }

    // Why: lint-ok: unused-pub — the /admin/tools page parses its query with
    // it; that page lands with Stage 3 phase 9 (tools, artifacts).
    #[must_use]
    pub fn parse_artifact_kind(value: Option<&str>) -> Option<Self> {
        match value {
            Some("file") => Some(Self::File),
            Some("card") => Some(Self::Card),
            Some("ui") => Some(Self::Ui),
            Some("body") => Some(Self::Body),
            _ => None,
        }
    }
}

/// The column the rows sort by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolSort {
    #[default]
    Time,
    Duration,
    Tool,
    Size,
}

impl ToolSort {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Time => "time",
            Self::Duration => "duration",
            Self::Tool => "tool",
            Self::Size => "size",
        }
    }

    // Why: lint-ok: unused-pub — the /admin/tools page parses its query with
    // it; that page lands with Stage 3 phase 9 (tools, artifacts).
    #[must_use]
    pub fn parse_tool_sort(value: Option<&str>) -> Self {
        match value {
            Some("duration") => Self::Duration,
            Some("tool") => Self::Tool,
            Some("size") => Self::Size,
            _ => Self::Time,
        }
    }
}

/// The dimension the breakdown table groups the filtered set by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolBreakdownBy {
    #[default]
    Tool,
    Server,
    User,
    Client,
    Skill,
    Kind,
}

impl ToolBreakdownBy {
    pub const ALL: [Self; 6] = [
        Self::Tool,
        Self::Server,
        Self::User,
        Self::Client,
        Self::Skill,
        Self::Kind,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tool => "tool",
            Self::Server => "server",
            Self::User => "user",
            Self::Client => "client",
            Self::Skill => "skill",
            Self::Kind => "kind",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Tool => "By tool",
            Self::Server => "By server",
            Self::User => "By person",
            Self::Client => "By client",
            Self::Skill => "By skill",
            Self::Kind => "By kind",
        }
    }

    // Why: the query parameter a bucket narrows the list by, so a breakdown
    // row links into the table filtered to itself.
    #[must_use]
    pub const fn param(self) -> &'static str {
        match self {
            Self::Tool => "tool",
            Self::Server => "server",
            Self::User => "user_id",
            Self::Client => "client",
            Self::Skill => "skill",
            Self::Kind => "artifact",
        }
    }

    // Why: lint-ok: unused-pub — the /admin/tools page parses its query with
    // it; that page lands with Stage 3 phase 9 (tools, artifacts).
    #[must_use]
    pub fn parse_tool_breakdown(value: Option<&str>) -> Self {
        match value {
            Some("server") => Self::Server,
            Some("user") => Self::User,
            Some("client") => Self::Client,
            Some("skill") => Self::Skill,
            Some("kind") => Self::Kind,
            _ => Self::Tool,
        }
    }
}
