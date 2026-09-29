//! The figures behind the Evaluation view: sums over `PluginEvalRow`s and
//! how each is read against a baseline. Rates and per-run averages are
//! computed here and nowhere else, so every table on the tab agrees.

use serde::Serialize;

use super::super::history::cost;
use crate::repositories::analysis::plugin_eval::PluginEvalRow;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub(crate) struct Figures {
    pub runs: i64,
    pub success: i64,
    pub completed: i64,
    pub cost: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub turns: i64,
    pub connector_calls: i64,
    pub failed_calls: i64,
    pub schema_errors: i64,
    pub access_errors: i64,
    pub upstream_errors: i64,
    pub bad_arguments: i64,
    pub timeouts: i64,
    pub writes: i64,
    pub repeated_calls: i64,
    pub placeholder_mentions: i64,
    pub tools_unavailable: i64,
}

impl Figures {
    pub(super) fn add(&mut self, r: &PluginEvalRow) {
        self.runs += 1;
        self.success += i64::from(r.success);
        self.completed += i64::from(r.completed);
        self.cost += r.cost;
        self.input_tokens += r.input_tokens;
        self.output_tokens += r.output_tokens;
        self.cache_read_tokens += r.cache_read_tokens;
        self.turns += r.turns;
        self.connector_calls += r.connector_calls;
        self.failed_calls += r.failed_calls;
        self.schema_errors += r.schema_errors;
        self.access_errors += r.access_errors;
        self.upstream_errors += r.upstream_errors;
        self.bad_arguments += r.bad_arguments;
        self.timeouts += r.timeouts;
        self.writes += r.writes;
        self.repeated_calls += r.repeated_calls;
        self.placeholder_mentions += r.placeholder_mentions;
        self.tools_unavailable += i64::from(r.tools_unavailable);
    }

    pub(super) fn success_rate(&self) -> f64 {
        pct(self.success, self.runs)
    }

    pub(super) fn avg_cost(&self) -> f64 {
        self.per_run(self.cost)
    }

    fn per_run(&self, total: i64) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            total as f64 / self.runs as f64
        }
    }
}

// Why: a figure is read with its change from the baseline; `tone` says
// whether that change is an improvement.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Measure {
    pub value: String,
    pub delta: Option<String>,
    pub tone: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Summary {
    pub runs: i64,
    pub success_rate: Measure,
    pub completed_rate: Measure,
    pub avg_cost: Measure,
    pub cost_per_success: Measure,
    pub avg_input_tokens: Measure,
    pub avg_output_tokens: Measure,
    pub avg_cache_read_tokens: Measure,
    pub avg_turns: Measure,
    pub avg_connector_calls: Measure,
    pub failed_calls: Measure,
    pub schema_errors: i64,
    pub access_errors: i64,
    pub upstream_errors: i64,
    pub bad_arguments: i64,
    pub timeouts: i64,
    pub writes: i64,
    pub repeated_calls: i64,
    pub placeholder_mentions: i64,
    pub tools_unavailable: i64,
}

fn pct(n: i64, d: i64) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 * 100.0 / d as f64
    }
}

// Why: lower is better for spend, size and errors; higher for success.
fn measure(
    value: f64,
    base: Option<f64>,
    fmt: fn(f64) -> String,
    higher_is_better: bool,
) -> Measure {
    let Some(b) = base else {
        return Measure {
            value: fmt(value),
            delta: None,
            tone: "",
        };
    };
    let d = value - b;
    let tone = if d.abs() < 1e-9 {
        "flat"
    } else if (d > 0.0) == higher_is_better {
        "good"
    } else {
        "bad"
    };
    let sign = if d > 0.0 {
        "+"
    } else if d < 0.0 {
        "−"
    } else {
        "±"
    };
    Measure {
        value: fmt(value),
        delta: Some(format!("{sign}{}", fmt(d.abs()))),
        tone,
    }
}

// Why: what a successful run costs once the failed runs' spend is counted
// against it; a plugin that fails half its runs doubles this figure.
fn per_success(f: &Figures) -> f64 {
    if f.success == 0 {
        0.0
    } else {
        f.cost as f64 / f.success as f64
    }
}

fn fmt_pct(v: f64) -> String {
    format!("{v:.0}%")
}
fn fmt_int(v: f64) -> String {
    format!("{v:.0}")
}
fn fmt_one(v: f64) -> String {
    format!("{v:.1}")
}
fn fmt_cost(v: f64) -> String {
    cost(v.round() as i64)
}

pub(super) fn summary(f: &Figures, base: Option<&Figures>) -> Summary {
    let b = |g: fn(&Figures) -> f64| base.filter(|x| x.runs > 0).map(g);
    Summary {
        runs: f.runs,
        success_rate: measure(
            pct(f.success, f.runs),
            b(|x| pct(x.success, x.runs)),
            fmt_pct,
            true,
        ),
        completed_rate: measure(
            pct(f.completed, f.runs),
            b(|x| pct(x.completed, x.runs)),
            fmt_pct,
            true,
        ),
        avg_cost: measure(f.per_run(f.cost), b(|x| x.per_run(x.cost)), fmt_cost, false),
        cost_per_success: measure(per_success(f), b(per_success), fmt_cost, false),
        avg_input_tokens: measure(
            f.per_run(f.input_tokens),
            b(|x| x.per_run(x.input_tokens)),
            fmt_int,
            false,
        ),
        avg_output_tokens: measure(
            f.per_run(f.output_tokens),
            b(|x| x.per_run(x.output_tokens)),
            fmt_int,
            false,
        ),
        avg_cache_read_tokens: measure(
            f.per_run(f.cache_read_tokens),
            b(|x| x.per_run(x.cache_read_tokens)),
            fmt_int,
            false,
        ),
        avg_turns: measure(
            f.per_run(f.turns),
            b(|x| x.per_run(x.turns)),
            fmt_one,
            false,
        ),
        avg_connector_calls: measure(
            f.per_run(f.connector_calls),
            b(|x| x.per_run(x.connector_calls)),
            fmt_one,
            false,
        ),
        failed_calls: measure(
            f.failed_calls as f64,
            b(|x| x.failed_calls as f64),
            fmt_int,
            false,
        ),
        schema_errors: f.schema_errors,
        access_errors: f.access_errors,
        upstream_errors: f.upstream_errors,
        bad_arguments: f.bad_arguments,
        timeouts: f.timeouts,
        writes: f.writes,
        repeated_calls: f.repeated_calls,
        placeholder_mentions: f.placeholder_mentions,
        tools_unavailable: f.tools_unavailable,
    }
}
