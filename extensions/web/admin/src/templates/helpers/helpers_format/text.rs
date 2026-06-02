//! String and text manipulation helpers.

use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext};

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
pub struct ShortIdHelper;
impl HelperDef for ShortIdHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let val = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
        let n: usize = h
            .param(1)
            .and_then(|v| v.value().as_u64())
            .map_or(12, |v| v as usize);
        let s: String = val.chars().take(n).collect();
        out.write(&s)?;
        Ok(())
    }
}
