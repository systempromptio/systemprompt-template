//! String and text manipulation helpers.

use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext};

#[derive(Debug, Clone, Copy)]
pub(crate) struct InitialsHelper;
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

// Why: the avatar stylesheet defines exactly this many `--sp-avatar-tone-N`
// tokens.
const AVATAR_TONES: u64 = 12;

// Why: FNV-1a over the normalised name is stable across pages and renders, so
// the same person always gets the same colour with no state to keep.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AvatarToneHelper;
impl HelperDef for AvatarToneHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let name = h.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
        let hash = name
            .trim()
            .to_lowercase()
            .bytes()
            .fold(0xcbf2_9ce4_8422_2325u64, |acc, b| {
                (acc ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
            });
        out.write(&(hash % AVATAR_TONES).to_string())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TruncateHelper;
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
pub(crate) struct ConcatHelper;
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
            // JSON: required by the handlebars HelperDef trait contract
            match param.value() {
                serde_json::Value::String(s) => result.push_str(s),
                serde_json::Value::Number(n) => result.push_str(&n.to_string()),
                serde_json::Value::Bool(b) => result.push_str(&b.to_string()),
                serde_json::Value::Null => {},
                other => result.push_str(&other.to_string()),
            }
        }
        out.write(&result)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ToLowerCaseHelper;
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
pub(crate) struct ToUpperCaseHelper;
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
pub(crate) struct ShortIdHelper;
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
