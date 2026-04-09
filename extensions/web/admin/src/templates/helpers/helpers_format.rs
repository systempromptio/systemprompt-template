use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext};

#[derive(Debug, Clone, Copy)]
pub struct FormatDateHelper;
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
            }
            Err(_) => {
                out.write(val)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RelativeTimeHelper;
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
                    "just now".to_string()
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
            }
            Err(_) => {
                out.write(val)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InitialsHelper;
impl HelperDef for InitialsHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("?");
        if name.is_empty() || name == "?" {
            out.write("?")?;
            return Ok(());
        }
        let initials: String = name
            .split(|c: char| c.is_whitespace() || c == '@' || c == '.' || c == '_' || c == '-')
            .filter(|s| !s.is_empty())
            .take(2)
            .filter_map(|s| s.chars().next())
            .flat_map(char::to_uppercase)
            .collect();
        out.write(if initials.is_empty() { "?" } else { &initials })?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TruncateHelper;
impl HelperDef for TruncateHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let val = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
        let max = h
            .param(1)
            .and_then(|v| v.value().as_u64())
            .map_or(60, |v| usize::try_from(v).unwrap_or(60));
        if val.len() <= max {
            out.write(val)?;
        } else {
            let truncated: String = val.chars().take(max).collect();
            out.write(&truncated)?;
            out.write("...")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JsonHelper;
impl HelperDef for JsonHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let val = h
            .param(0)
            .map_or(serde_json::Value::Null, |v| v.value().clone());
        let json_str = serde_json::to_string_pretty(&val).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to serialize value to JSON for template helper");
            "null".to_string()
        });
        let escaped = json_str
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        out.write(&escaped)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConcatHelper;
impl HelperDef for ConcatHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let mut result = String::new();
        for param in h.params() {
            match param.value() {
                serde_json::Value::String(s) => result.push_str(s),
                serde_json::Value::Number(n) => result.push_str(&n.to_string()),
                serde_json::Value::Bool(b) => result.push_str(&b.to_string()),
                serde_json::Value::Null => {}
                other => result.push_str(&other.to_string()),
            }
        }
        out.write(&result)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ToLowerCaseHelper;
impl HelperDef for ToLowerCaseHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let val = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
        out.write(&val.to_lowercase())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ToUpperCaseHelper;
impl HelperDef for ToUpperCaseHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let val = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
        out.write(&val.to_uppercase())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CssVersionHelper;
impl HelperDef for CssVersionHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        _h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        use std::sync::OnceLock;
        static VERSION: OnceLock<String> = OnceLock::new();
        let v = VERSION.get_or_init(|| {
            let path = std::env::current_dir()
                .unwrap_or_else(|e| {
                    tracing::debug!(error = %e, "Failed to get current directory for CSS version helper");
                    std::path::PathBuf::from(".")
                })
                .join("storage/files/css/css-manifest.json");
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|j| j.get("version")?.as_str().map(String::from))
                .unwrap_or_else(|| "0".to_string())
        });
        out.write(v)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DefaultHelper;
impl HelperDef for DefaultHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let val = h.param(0).map(handlebars::PathAndJson::value);
        let fallback = h.param(1).and_then(|v| v.value().as_str()).unwrap_or("");
        let is_truthy = match val {
            Some(serde_json::Value::Null) | None => false,
            Some(serde_json::Value::String(s)) => !s.is_empty(),
            Some(serde_json::Value::Bool(b)) => *b,
            _ => true,
        };
        if is_truthy {
            if let Some(v) = val {
                match v {
                    serde_json::Value::String(s) => out.write(s)?,
                    other => out.write(&other.to_string())?,
                }
            }
        } else {
            out.write(fallback)?;
        }
        Ok(())
    }
}
