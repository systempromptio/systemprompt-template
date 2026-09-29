//! The two tool schemas one request carries, side by side.
//!
//! `ai_request_payloads.offered_tools` is what the client sent;
//! `prepared_tools` is the `tools` array the gateway put upstream, in the
//! provider's own wire shape. The shapes differ per provider (the flat
//! `input_schema` of Anthropic, `function.parameters` of `OpenAI`,
//! `function_declarations` of Gemini), so both sides are first read into one
//! [`WireTool`] and then paired by name. Nothing records the sanitizer's
//! decisions per request, so the Gemini rule hits are recomputed here from the
//! client schema with the same checker the sanitizer is tested against: a hit
//! is a shape Gemini would have refused as sent.

// JSON: tool schemas are protocol-boundary JSON Schema documents in three
// provider wire shapes; the diff reads them as written.
use serde_json::Value;
use systemprompt::models::schema::gemini_declaration_violations;

/// One tool in provider-neutral form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireTool {
    pub name: String,
    pub description: Option<String>,
    // JSON: the tool's input schema as it appeared on that side of the wire.
    pub schema: Value,
}

/// One tool as the client offered it and as the provider received it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSchemaPair {
    pub name: String,
    pub client: Option<WireTool>,
    pub provider: Option<WireTool>,
    // Why: the whole point of the view — a schema the sanitizer rewrote.
    pub changed: bool,
    // Why: the Gemini declaration rules the client schema breaks; empty for
    // a schema Gemini would take as sent, and only meaningful on a Gemini or
    // Vertex request.
    pub rule_hits: Vec<String>,
}

/// Reads a stored `tools` array in any of the three provider wire shapes.
// JSON: the `offered_tools` or `prepared_tools` array.
#[must_use]
pub fn wire_tools(tools: &Value) -> Vec<WireTool> {
    let Some(items) = tools.as_array() else {
        return Vec::new();
    };
    items.iter().flat_map(wire_tool_entries).collect()
}

// JSON: one entry of a tools array; Gemini nests several under one entry.
fn wire_tool_entries(entry: &Value) -> Vec<WireTool> {
    if let Some(declarations) = entry
        .get("function_declarations")
        .or_else(|| entry.get("functionDeclarations"))
        .and_then(Value::as_array)
    {
        return declarations
            .iter()
            .filter_map(|d| wire_tool(d, "parameters"))
            .collect();
    }
    if let Some(function) = entry.get("function") {
        return wire_tool(function, "parameters").into_iter().collect();
    }
    wire_tool(entry, "input_schema")
        .or_else(|| wire_tool(entry, "parameters"))
        .into_iter()
        .collect()
}

// JSON: one flat tool object with its schema under `schema_key`.
fn wire_tool(entry: &Value, schema_key: &str) -> Option<WireTool> {
    let name = entry.get("name")?.as_str()?.to_owned();
    let schema = entry.get(schema_key).cloned().unwrap_or(Value::Null);
    Some(WireTool {
        name,
        description: entry
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_owned),
        schema,
    })
}

// Why: Whether the served provider's declarations are checked by the Gemini
// rules.
#[must_use]
pub fn gemini_rules_apply(provider: &str) -> bool {
    let provider = provider.to_ascii_lowercase();
    provider.contains("gemini") || provider.contains("vertex") || provider.contains("google")
}

// Why: Pairs the client's tools with the provider's by name, in the client's
// order, with provider-only tools appended.
#[must_use]
pub fn pair_tools(
    client: &[WireTool],
    provider: &[WireTool],
    check_rules: bool,
) -> Vec<ToolSchemaPair> {
    let mut pairs: Vec<ToolSchemaPair> = client
        .iter()
        .map(|c| {
            let p = provider.iter().find(|p| p.name == c.name);
            ToolSchemaPair {
                name: c.name.clone(),
                changed: p.is_some_and(|p| p.schema != c.schema),
                rule_hits: if check_rules {
                    gemini_declaration_violations(&c.schema)
                } else {
                    Vec::new()
                },
                client: Some(c.clone()),
                provider: p.cloned(),
            }
        })
        .collect();
    for p in provider {
        if !client.iter().any(|c| c.name == p.name) {
            pairs.push(ToolSchemaPair {
                name: p.name.clone(),
                client: None,
                provider: Some(p.clone()),
                changed: true,
                rule_hits: Vec::new(),
            });
        }
    }
    pairs
}
