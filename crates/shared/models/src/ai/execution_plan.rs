//! Execution Plan Types for Plan-Based Agentic Execution
//!
//! Simplified model: PLAN → EXECUTE → RESPOND
//!
//! - PLAN: AI outputs tool_calls[] or direct_response with template references
//! - EXECUTE: Sequential tool execution with template resolution
//! - RESPOND: AI generates response with full context
//!
//! Template syntax: `$N.output.field.path` references output from tool at index
//! N

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Result of the planning phase
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlanningResult {
    /// No tools needed - respond directly
    DirectResponse { content: String },
    /// Tools needed - execute these calls in order
    ToolCalls {
        reasoning: String,
        calls: Vec<PlannedToolCall>,
    },
}

impl PlanningResult {
    pub fn direct_response(content: impl Into<String>) -> Self {
        Self::DirectResponse {
            content: content.into(),
        }
    }

    pub fn tool_calls(reasoning: impl Into<String>, calls: Vec<PlannedToolCall>) -> Self {
        Self::ToolCalls {
            reasoning: reasoning.into(),
            calls,
        }
    }

    pub const fn is_direct(&self) -> bool {
        matches!(self, Self::DirectResponse { .. })
    }

    pub const fn is_tool_calls(&self) -> bool {
        matches!(self, Self::ToolCalls { .. })
    }

    pub fn tool_count(&self) -> usize {
        match self {
            Self::DirectResponse { .. } => 0,
            Self::ToolCalls { calls, .. } => calls.len(),
        }
    }
}

/// A single planned tool call with fully resolved arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedToolCall {
    pub tool_name: String,
    pub arguments: Value,
}

impl PlannedToolCall {
    pub fn new(tool_name: impl Into<String>, arguments: Value) -> Self {
        Self {
            tool_name: tool_name.into(),
            arguments,
        }
    }
}

/// Result of executing a single tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub tool_name: String,
    pub arguments: Value,
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}

impl ToolCallResult {
    pub const fn success(tool_name: String, arguments: Value, output: Value, duration_ms: u64) -> Self {
        Self {
            tool_name,
            arguments,
            success: true,
            output,
            error: None,
            duration_ms,
        }
    }

    pub fn failure(
        tool_name: String,
        arguments: Value,
        error: impl Into<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            tool_name,
            arguments,
            success: false,
            output: Value::Null,
            error: Some(error.into()),
            duration_ms,
        }
    }
}

/// State after plan execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionState {
    pub results: Vec<ToolCallResult>,
    pub halted: bool,
    pub halt_reason: Option<String>,
}

impl ExecutionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_result(&mut self, result: ToolCallResult) {
        if !result.success && !self.halted {
            self.halted = true;
            self.halt_reason = result.error.clone();
        }
        self.results.push(result);
    }

    pub fn successful_results(&self) -> Vec<&ToolCallResult> {
        self.results.iter().filter(|r| r.success).collect()
    }

    pub fn failed_results(&self) -> Vec<&ToolCallResult> {
        self.results.iter().filter(|r| !r.success).collect()
    }

    pub fn total_duration_ms(&self) -> u64 {
        self.results.iter().map(|r| r.duration_ms).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRef {
    pub tool_index: usize,
    pub field_path: Vec<String>,
}

impl TemplateRef {
    pub fn parse(template: &str) -> Option<Self> {
        let re = Regex::new(r"^\$(\d+)\.output\.(.+)$").ok()?;
        let caps = re.captures(template)?;

        let tool_index = caps.get(1)?.as_str().parse().ok()?;
        let path = caps.get(2)?.as_str();
        let field_path = path.split('.').map(String::from).collect();

        Some(Self {
            tool_index,
            field_path,
        })
    }

    pub fn format(&self) -> String {
        format!("${}.output.{}", self.tool_index, self.field_path.join("."))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanValidationError {
    pub tool_index: usize,
    pub argument: String,
    pub template: String,
    pub error: ValidationErrorKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationErrorKind {
    InvalidTemplateSyntax,
    IndexOutOfBounds {
        referenced_index: usize,
        max_valid_index: usize,
    },
    SelfReference,
    ForwardReference {
        referenced_index: usize,
    },
    FieldNotFound {
        tool_name: String,
        field: String,
        available_fields: Vec<String>,
    },
    NoOutputSchema {
        tool_name: String,
    },
}

impl std::fmt::Display for PlanValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error {
            ValidationErrorKind::InvalidTemplateSyntax => {
                write!(
                    f,
                    "Tool {}: Invalid template syntax '{}' for argument '{}'",
                    self.tool_index, self.template, self.argument
                )
            },
            ValidationErrorKind::IndexOutOfBounds {
                referenced_index,
                max_valid_index,
            } => {
                write!(
                    f,
                    "Tool {}: Template '{}' references tool {} but only tools 0-{} are available",
                    self.tool_index, self.template, referenced_index, max_valid_index
                )
            },
            ValidationErrorKind::SelfReference => {
                write!(
                    f,
                    "Tool {}: Template '{}' cannot reference itself",
                    self.tool_index, self.template
                )
            },
            ValidationErrorKind::ForwardReference { referenced_index } => {
                write!(
                    f,
                    "Tool {}: Template '{}' references tool {} which hasn't executed yet",
                    self.tool_index, self.template, referenced_index
                )
            },
            ValidationErrorKind::FieldNotFound {
                tool_name,
                field,
                available_fields,
            } => {
                write!(
                    f,
                    "Tool {}: Template '{}' references field '{}' but tool '{}' outputs: [{}]",
                    self.tool_index,
                    self.template,
                    field,
                    tool_name,
                    available_fields.join(", ")
                )
            },
            ValidationErrorKind::NoOutputSchema { tool_name } => {
                write!(
                    f,
                    "Tool {}: Template '{}' references '{}' which has no output schema",
                    self.tool_index, self.template, tool_name
                )
            },
        }
    }
}

impl std::error::Error for PlanValidationError {}

#[derive(Debug, Clone, Copy)]
pub struct TemplateValidator;

impl TemplateValidator {
    pub fn find_templates_in_value(value: &Value) -> Vec<String> {
        let mut templates = Vec::new();
        Self::collect_templates(value, &mut templates);
        templates
    }

    fn collect_templates(value: &Value, templates: &mut Vec<String>) {
        match value {
            Value::String(s) if s.starts_with('$') && s.contains(".output.") => {
                templates.push(s.clone());
            },
            Value::Array(arr) => {
                for v in arr {
                    Self::collect_templates(v, templates);
                }
            },
            Value::Object(obj) => {
                for v in obj.values() {
                    Self::collect_templates(v, templates);
                }
            },
            _ => {},
        }
    }

    pub fn validate_plan(
        calls: &[PlannedToolCall],
        tool_output_schemas: &[(String, Option<Value>)],
    ) -> Result<(), Vec<PlanValidationError>> {
        let mut errors = Vec::new();

        for (tool_index, call) in calls.iter().enumerate() {
            let templates = Self::find_templates_in_value(&call.arguments);

            for template in templates {
                if let Some(template_ref) = TemplateRef::parse(&template) {
                    if template_ref.tool_index == tool_index {
                        errors.push(PlanValidationError {
                            tool_index,
                            argument: Self::find_argument_for_template(&call.arguments, &template),
                            template: template.clone(),
                            error: ValidationErrorKind::SelfReference,
                        });
                        continue;
                    }

                    if template_ref.tool_index > tool_index {
                        errors.push(PlanValidationError {
                            tool_index,
                            argument: Self::find_argument_for_template(&call.arguments, &template),
                            template: template.clone(),
                            error: ValidationErrorKind::ForwardReference {
                                referenced_index: template_ref.tool_index,
                            },
                        });
                        continue;
                    }

                    if template_ref.tool_index >= tool_output_schemas.len() {
                        errors.push(PlanValidationError {
                            tool_index,
                            argument: Self::find_argument_for_template(&call.arguments, &template),
                            template: template.clone(),
                            error: ValidationErrorKind::IndexOutOfBounds {
                                referenced_index: template_ref.tool_index,
                                max_valid_index: tool_output_schemas.len().saturating_sub(1),
                            },
                        });
                        continue;
                    }

                    let (ref_tool_name, ref_output_schema) =
                        &tool_output_schemas[template_ref.tool_index];

                    if let Some(schema) = ref_output_schema {
                        if let Some(first_field) = template_ref.field_path.first() {
                            let available_fields = Self::get_schema_fields(schema);
                            if !available_fields.contains(first_field) {
                                errors.push(PlanValidationError {
                                    tool_index,
                                    argument: Self::find_argument_for_template(
                                        &call.arguments,
                                        &template,
                                    ),
                                    template: template.clone(),
                                    error: ValidationErrorKind::FieldNotFound {
                                        tool_name: ref_tool_name.clone(),
                                        field: first_field.clone(),
                                        available_fields,
                                    },
                                });
                            }
                        }
                    } else {
                        errors.push(PlanValidationError {
                            tool_index,
                            argument: Self::find_argument_for_template(&call.arguments, &template),
                            template: template.clone(),
                            error: ValidationErrorKind::NoOutputSchema {
                                tool_name: ref_tool_name.clone(),
                            },
                        });
                    }
                } else {
                    errors.push(PlanValidationError {
                        tool_index,
                        argument: Self::find_argument_for_template(&call.arguments, &template),
                        template: template.clone(),
                        error: ValidationErrorKind::InvalidTemplateSyntax,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn find_argument_for_template(value: &Value, template: &str) -> String {
        if let Value::Object(obj) = value {
            for (key, val) in obj {
                if let Value::String(s) = val {
                    if s == template {
                        return key.clone();
                    }
                }
                let nested = Self::find_argument_for_template(val, template);
                if !nested.is_empty() {
                    return format!("{key}.{nested}");
                }
            }
        }
        String::new()
    }

    fn get_schema_fields(schema: &Value) -> Vec<String> {
        schema
            .get("properties")
            .and_then(|p| p.as_object())
            .map(|obj| obj.keys().cloned().collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TemplateResolver;

impl TemplateResolver {
    pub fn resolve_arguments(arguments: &Value, results: &[ToolCallResult]) -> Value {
        Self::resolve_value(arguments, results)
    }

    fn resolve_value(value: &Value, results: &[ToolCallResult]) -> Value {
        match value {
            Value::String(s) if s.starts_with('$') && s.contains(".output.") => {
                Self::resolve_template(s, results)
            },
            Value::Array(arr) => Value::Array(
                arr.iter()
                    .map(|v| Self::resolve_value(v, results))
                    .collect(),
            ),
            Value::Object(obj) => Value::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), Self::resolve_value(v, results)))
                    .collect(),
            ),
            _ => value.clone(),
        }
    }

    fn resolve_template(template: &str, results: &[ToolCallResult]) -> Value {
        let Some(template_ref) = TemplateRef::parse(template) else {
            return Value::String(template.to_string());
        };

        let Some(result) = results.get(template_ref.tool_index) else {
            return Value::Null;
        };

        Self::get_nested_value(&result.output, &template_ref.field_path)
    }

    fn get_nested_value(value: &Value, path: &[String]) -> Value {
        let mut current = value;
        for segment in path {
            match current.get(segment) {
                Some(v) => current = v,
                None => return Value::Null,
            }
        }
        current.clone()
    }
}
