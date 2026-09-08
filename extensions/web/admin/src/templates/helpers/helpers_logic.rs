//! Comparison and conditional Handlebars helpers.

use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct EqHelper;
impl HelperDef for EqHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<handlebars::ScopedJson<'rc>, RenderError> {
        // JSON: required by the handlebars HelperDef trait contract
        let a = h.param(0).map(handlebars::PathAndJson::value);
        let b = h.param(1).map(handlebars::PathAndJson::value);
        let equal = match (a, b) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        };
        Ok(handlebars::ScopedJson::Derived(serde_json::Value::Bool(
            equal,
        )))
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct GtHelper;
impl HelperDef for GtHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<handlebars::ScopedJson<'rc>, RenderError> {
        let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
        let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
        // JSON: required by the handlebars HelperDef trait contract
        Ok(handlebars::ScopedJson::Derived(serde_json::Value::Bool(
            a > b,
        )))
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct NotHelper;
impl HelperDef for NotHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<handlebars::ScopedJson<'rc>, RenderError> {
        let val = h.param(0).map(handlebars::PathAndJson::value);
        // JSON: required by the handlebars HelperDef trait contract
        let is_falsy = match val {
            None | Some(serde_json::Value::Null | serde_json::Value::Bool(false)) => true,
            Some(serde_json::Value::String(s)) => s.is_empty(),
            Some(serde_json::Value::Number(n)) => n.as_f64() == Some(0.0),
            Some(serde_json::Value::Array(a)) => a.is_empty(),
            _ => false,
        };
        // JSON: required by the handlebars HelperDef trait contract
        Ok(handlebars::ScopedJson::Derived(serde_json::Value::Bool(
            is_falsy,
        )))
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AddHelper;
impl HelperDef for AddHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let a = h.param(0).and_then(|v| v.value().as_i64()).unwrap_or(0);
        let b = h.param(1).and_then(|v| v.value().as_i64()).unwrap_or(0);
        out.write(&(a + b).to_string())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SubHelper;
impl HelperDef for SubHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let a = h.param(0).and_then(|v| v.value().as_i64()).unwrap_or(0);
        let b = h.param(1).and_then(|v| v.value().as_i64()).unwrap_or(0);
        out.write(&(a - b).to_string())?;
        Ok(())
    }
}

// Why: the sidebar marks one link per section active, and several links stand
// for more than one page id — Requests owns both the list and the detail. Done
// with `eq` that is a nest of three `{{#if}}` blocks per link, which is what
// made the old sidebar unreadable. `{{navActive page "requests"
// "request-detail"}}` emits the whole attribute pair, or nothing.
#[derive(Debug, Clone, Copy)]
pub(super) struct NavActiveHelper;
impl HelperDef for NavActiveHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        // JSON: required by the handlebars HelperDef trait contract
        let current = h.param(0).and_then(|p| p.value().as_str()).unwrap_or("");
        let matched = h
            .params()
            .iter()
            .skip(1)
            .filter_map(|p| p.value().as_str())
            .any(|candidate| candidate == current);
        if matched {
            out.write(" class=\"is-active\" aria-current=\"page\"")?;
        }
        Ok(())
    }
}

// Why: a sidebar link that owns sub-pages has three states, not two. On its
// own page it is the active leaf; on a record it owns it is the expanded
// ancestor, and the record itself is drawn beneath it. `{{navState page "users"
// "user-detail"}}` reads the first id as the leaf and the rest as children.
#[derive(Debug, Clone, Copy)]
pub(super) struct NavStateHelper;
impl HelperDef for NavStateHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        // JSON: required by the handlebars HelperDef trait contract
        let current = h.param(0).and_then(|p| p.value().as_str()).unwrap_or("");
        let leaf = h.param(1).and_then(|p| p.value().as_str()).unwrap_or("");
        if current == leaf {
            out.write(" class=\"is-active\" aria-current=\"page\"")?;
            return Ok(());
        }
        let is_child = h
            .params()
            .iter()
            .skip(2)
            .filter_map(|p| p.value().as_str())
            .any(|candidate| candidate == current);
        if is_child {
            out.write(" class=\"is-ancestor\" aria-expanded=\"true\"")?;
        }
        Ok(())
    }
}
