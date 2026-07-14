//! Presentation helpers: JSON rendering, asset versioning, governance color
//! mapping, and value fallbacks.

use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext};

#[derive(Debug, Clone, Copy)]
pub(crate) struct JsonHelper;
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
            "null".to_owned()
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
pub(crate) struct CssVersionHelper;
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
            // Why: a missing/corrupt CSS manifest at template-render time falls
            // back to "0" — the helper exists to bust browser caches, not to
            // halt rendering. Errors are absorbed deliberately.
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|j| j.get("version")?.as_str().map(String::from))
                .unwrap_or_else(|| "0".to_owned())
        });
        out.write(v)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GovernanceColorHelper;
impl HelperDef for GovernanceColorHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let decision = h
            .param(0)
            .and_then(|v| v.value().as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let color = match decision.as_str() {
            "allow" | "pass" | "ok" => "success",
            "flag" | "warn" | "warning" | "review" => "warning",
            "deny" | "block" | "denied" | "fail" | "error" => "danger",
            _ => "neutral",
        };
        out.write(color)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DefaultHelper;
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
