pub mod core_stats;
pub mod events;
pub mod queries;
pub mod session;
pub mod subject_analysis;

pub use core_stats::CoreStatsRepository;
pub use events::EventsRepository;
pub use session::{SessionMigrationResult, SessionRecord, SessionRepository};

pub use queries::AnalyticsQueryRepository;

pub type AnalyticsSessionRepository = SessionRepository;
pub type EventRepository = EventsRepository;
