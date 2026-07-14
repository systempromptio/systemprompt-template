//! Numeric, currency, and percentage formatting helpers.

use handlebars::{Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext};

#[derive(Debug, Clone, Copy)]
pub(crate) struct FormatNumberHelper;
impl HelperDef for FormatNumberHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let n = h.param(0).and_then(|v| v.value().as_i64()).unwrap_or(0);
        let abs = n.unsigned_abs();
        let formatted = if abs >= 1_000_000_000 {
            format!("{:.1}B", n as f64 / 1_000_000_000.0)
        } else if abs >= 1_000_000 {
            format!("{:.1}M", n as f64 / 1_000_000.0)
        } else if abs >= 1_000 {
            let s = n.to_string();
            let bytes = s.as_bytes();
            let neg = bytes.first() == Some(&b'-');
            let digits = if neg { &bytes[1..] } else { bytes };
            let mut grouped = String::new();
            for (i, c) in digits.iter().enumerate() {
                if i > 0 && (digits.len() - i) % 3 == 0 {
                    grouped.push(',');
                }
                grouped.push(*c as char);
            }
            if neg { format!("-{grouped}") } else { grouped }
        } else {
            n.to_string()
        };
        out.write(&formatted)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FormatUsdHelper;
impl HelperDef for FormatUsdHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let micro = h.param(0).and_then(|v| v.value().as_i64());
        let formatted = micro.map_or_else(
            || "—".to_owned(),
            |m| {
                let usd = m as f64 / 1_000_000.0;
                if !usd.is_finite() {
                    "—".to_owned()
                } else if usd >= 100.0 {
                    format!("${usd:.0}")
                } else if usd >= 1.0 {
                    format!("${usd:.2}")
                } else if usd >= 0.01 {
                    format!("${usd:.3}")
                } else {
                    format!("${usd:.5}")
                }
            },
        );
        out.write(&formatted)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PercentHelper;
impl HelperDef for PercentHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let v = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
        let pct = v * 100.0;
        out.write(&format!("{pct:.1}%"))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DeltaPctHelper;
impl HelperDef for DeltaPctHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let curr = h.param(0).and_then(|v| v.value().as_i64());
        let prev = h.param(1).and_then(|v| v.value().as_i64());
        let formatted = match (curr, prev) {
            (Some(c), Some(p)) if p != 0 => {
                let pct = (c - p) as f64 / p as f64 * 100.0;
                if !pct.is_finite() {
                    String::new()
                } else if pct > 0.0 {
                    format!("+{pct:.0}% vs prev")
                } else {
                    format!("{pct:.0}% vs prev")
                }
            },
            _ => String::new(),
        };
        out.write(&formatted)?;
        Ok(())
    }
}
