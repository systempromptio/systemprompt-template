//! Whether the judge runs on its own. The profile's `judge.automatic`
//! switches every background judge spend; when it is off (development,
//! local), the Analysis pages draw the buttons that ask for a verdict, and
//! when it is on (production) verdicts simply arrive and the pages show
//! only the circles.

use crate::env::AdminEnv;

pub(crate) fn automatic(env: &AdminEnv) -> bool {
    env.profile().judge_automatic
}
