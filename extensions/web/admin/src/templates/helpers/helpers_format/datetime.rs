//! Date and time formatting helpers.

use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext};

#[derive(Debug, Clone, Copy)]
pub(crate) struct FormatDateHelper;
impl HelperDef for FormatDateHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let val = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("-");
        if val == "-" || val.is_empty() {
            out.write("-")?;
            return Ok(());
        }
        match chrono::DateTime::parse_from_rfc3339(val).or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M:%S%.f")
                .map(|dt| dt.and_utc().fixed_offset())
        }) {
            Ok(dt) => {
                let local_dt = dt.with_timezone(&chrono::Local);
                let formatted = local_dt.format("%b %d, %Y %I:%M %p").to_string();
                out.write(&formatted)?;
            },
            Err(_) => {
                out.write(val)?;
            },
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RelativeTimeHelper;
impl HelperDef for RelativeTimeHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let val = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("-");
        if val == "-" || val.is_empty() {
            out.write("-")?;
            return Ok(());
        }
        match chrono::DateTime::parse_from_rfc3339(val).or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M:%S%.f")
                .map(|dt| dt.and_utc().fixed_offset())
        }) {
            Ok(dt) => {
                let now = chrono::Utc::now();
                let diff = now.signed_duration_since(dt);
                let mins = diff.num_minutes();
                let result = if mins < 1 {
                    "just now".to_owned()
                } else if mins < 60 {
                    format!("{mins}m ago")
                } else {
                    let hours = diff.num_hours();
                    if hours < 24 {
                        format!("{hours}h ago")
                    } else {
                        let days = diff.num_days();
                        if days < 30 {
                            format!("{days}d ago")
                        } else {
                            dt.with_timezone(&chrono::Local)
                                .format("%b %d, %Y %I:%M %p")
                                .to_string()
                        }
                    }
                };
                out.write(&result)?;
            },
            Err(_) => {
                out.write(val)?;
            },
        }
        Ok(())
    }
}
