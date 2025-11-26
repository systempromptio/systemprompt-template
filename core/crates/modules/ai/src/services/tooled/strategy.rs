use crate::models::tools::{CallToolResult, ToolCall};

#[derive(Debug)]
pub enum ResponseStrategy {
    ContentProvided {
        content: String,
        tool_calls: Vec<ToolCall>,
        tool_results: Vec<CallToolResult>,
    },
    ArtifactsProvided {
        tool_calls: Vec<ToolCall>,
        tool_results: Vec<CallToolResult>,
    },
    ToolsOnly {
        tool_calls: Vec<ToolCall>,
        tool_results: Vec<CallToolResult>,
    },
}

impl ResponseStrategy {
    pub fn from_response(
        content: String,
        tool_calls: Vec<ToolCall>,
        tool_results: Vec<CallToolResult>,
    ) -> Self {
        if !content.trim().is_empty() {
            Self::ContentProvided {
                content,
                tool_calls,
                tool_results,
            }
        } else if !tool_calls.is_empty() && !tool_results.is_empty() {
            if Self::has_valid_artifacts(&tool_results) {
                Self::ArtifactsProvided {
                    tool_calls,
                    tool_results,
                }
            } else {
                Self::ToolsOnly {
                    tool_calls,
                    tool_results,
                }
            }
        } else {
            Self::ContentProvided {
                content,
                tool_calls,
                tool_results,
            }
        }
    }

    fn has_valid_artifacts(tool_results: &[CallToolResult]) -> bool {
        tool_results
            .iter()
            .any(|result| result.structured_content.is_some() && result.is_error != Some(true))
    }
}
