use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
};

#[derive(Debug, Clone, Copy)]
pub struct EqHelper;
impl HelperDef for EqHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<handlebars::ScopedJson<'rc>, RenderError> {
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
pub struct GtHelper;
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
        Ok(handlebars::ScopedJson::Derived(serde_json::Value::Bool(
            a > b,
        )))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NotHelper;
impl HelperDef for NotHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<handlebars::ScopedJson<'rc>, RenderError> {
        let val = h.param(0).map(handlebars::PathAndJson::value);
        let is_falsy = match val {
            None | Some(serde_json::Value::Null | serde_json::Value::Bool(false)) => true,
            Some(serde_json::Value::String(s)) => s.is_empty(),
            Some(serde_json::Value::Number(n)) => n.as_f64() == Some(0.0),
            Some(serde_json::Value::Array(a)) => a.is_empty(),
            _ => false,
        };
        Ok(handlebars::ScopedJson::Derived(serde_json::Value::Bool(
            is_falsy,
        )))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AddHelper;
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
pub struct SubHelper;
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
