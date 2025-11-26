use crate::models::a2a::{
    Artifact, DataPart, Message, Part, Task, TaskState, TaskStatus, TextPart,
};
use rmcp::model::{Content, RawContent};
use serde_json::json;
use systemprompt_identifiers::{ContextId, TaskId};
use systemprompt_models::a2a::{agent_names, ArtifactMetadata, TaskMetadata};
use uuid::Uuid;

fn extract_text_from_content(content: &[Content]) -> String {
    content
        .iter()
        .filter_map(|c| match &c.raw {
            RawContent::Text(text_content) => Some(text_content.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn content_to_json(content: &[Content]) -> serde_json::Value {
    let items: Vec<serde_json::Value> = content
        .iter()
        .map(|c| match &c.raw {
            RawContent::Text(text_content) => json!({"type": "text", "text": text_content.text}),
            RawContent::Image(image_content) => {
                json!({"type": "image", "data": image_content.data, "mimeType": image_content.mime_type})
            }
            RawContent::ResourceLink(resource) => {
                json!({"type": "resource", "uri": resource.uri, "mimeType": resource.mime_type})
            }
            _ => json!({"type": "unknown"}),
        })
        .collect();
    json!(items)
}

#[derive(Debug)]
pub struct TaskBuilder {
    task_id: TaskId,
    context_id: ContextId,
    state: TaskState,
    response_text: String,
    message_id: String,
    user_message: Option<Message>,
    artifacts: Vec<Artifact>,
}

impl TaskBuilder {
    pub fn new(context_id: ContextId) -> Self {
        Self {
            task_id: TaskId::generate(),
            context_id,
            state: TaskState::Completed,
            response_text: String::new(),
            message_id: Uuid::new_v4().to_string(),
            user_message: None,
            artifacts: Vec::new(),
        }
    }

    pub fn with_task_id(mut self, task_id: TaskId) -> Self {
        self.task_id = task_id;
        self
    }

    pub fn with_state(mut self, state: TaskState) -> Self {
        self.state = state;
        self
    }

    pub fn with_response_text(mut self, text: String) -> Self {
        self.response_text = text;
        self
    }

    pub fn with_message_id(mut self, message_id: String) -> Self {
        self.message_id = message_id;
        self
    }

    pub fn with_user_message(mut self, message: Message) -> Self {
        self.user_message = Some(message);
        self
    }

    pub fn with_artifacts(mut self, artifacts: Vec<Artifact>) -> Self {
        self.artifacts = artifacts;
        self
    }

    pub fn build(self) -> Task {
        let agent_message = Message {
            role: "agent".to_string(),
            parts: vec![Part::Text(TextPart {
                text: self.response_text.clone(),
            })],
            message_id: self.message_id.clone(),
            task_id: Some(self.task_id.clone()),
            context_id: self.context_id.clone(),
            kind: "message".to_string(),
            metadata: None,
            extensions: None,
            reference_task_ids: None,
        };

        let history = if let Some(user_msg) = self.user_message {
            Some(vec![
                user_msg,
                Message {
                    role: "agent".to_string(),
                    parts: vec![Part::Text(TextPart {
                        text: self.response_text.clone(),
                    })],
                    message_id: Uuid::new_v4().to_string(),
                    task_id: Some(self.task_id.clone()),
                    context_id: self.context_id.clone(),
                    kind: "message".to_string(),
                    metadata: None,
                    extensions: None,
                    reference_task_ids: None,
                },
            ])
        } else {
            None
        };

        Task {
            id: self.task_id.clone(),
            context_id: self.context_id.clone(),
            kind: "task".to_string(),
            status: TaskStatus {
                state: self.state,
                message: Some(agent_message),
                timestamp: Some(chrono::Utc::now()),
            },
            history,
            artifacts: if self.artifacts.is_empty() {
                None
            } else {
                Some(self.artifacts)
            },
            metadata: None,
        }
    }
}

pub fn build_completed_task(
    task_id: TaskId,
    context_id: ContextId,
    response_text: String,
    user_message: Message,
    artifacts: Vec<Artifact>,
) -> Task {
    TaskBuilder::new(context_id)
        .with_task_id(task_id)
        .with_state(TaskState::Completed)
        .with_response_text(response_text)
        .with_user_message(user_message)
        .with_artifacts(artifacts)
        .build()
}

pub fn build_canceled_task(task_id: TaskId, context_id: ContextId) -> Task {
    TaskBuilder::new(context_id)
        .with_task_id(task_id)
        .with_state(TaskState::Canceled)
        .with_response_text("Task was canceled.".to_string())
        .build()
}

pub fn build_mock_task(task_id: TaskId) -> Task {
    let mock_context_id = ContextId::generate();
    TaskBuilder::new(mock_context_id)
        .with_task_id(task_id)
        .with_state(TaskState::Completed)
        .with_response_text("Task completed successfully.".to_string())
        .build()
}

pub fn build_multiturn_task(
    context_id: ContextId,
    task_id: TaskId,
    user_message: Message,
    tool_calls: Vec<systemprompt_core_ai::ToolCall>,
    tool_results: Vec<systemprompt_core_ai::CallToolResult>,
    final_response: String,
    total_iterations: usize,
) -> Task {
    let ctx_id = context_id.clone();

    let mut history = Vec::new();

    history.push(user_message.clone());

    let mut iteration = 1;
    let mut call_idx = 0;

    while call_idx < tool_calls.len() {
        let iteration_calls: Vec<_> = tool_calls
            .iter()
            .skip(call_idx)
            .take_while(|_| call_idx < tool_calls.len())
            .cloned()
            .collect();

        if iteration_calls.is_empty() {
            break;
        }

        history.push(Message {
            role: "agent".to_string(),
            parts: vec![Part::Text(TextPart {
                text: format!("Executing {} tool(s)...", iteration_calls.len()),
            })],
            message_id: Uuid::new_v4().to_string(),
            task_id: Some(task_id.clone()),
            context_id: ctx_id.clone(),
            kind: "message".to_string(),
            metadata: Some(json!({
                "iteration": iteration,
                "tool_calls": iteration_calls.iter().map(|tc| {
                    json!({"id": tc.ai_tool_call_id.as_ref(), "name": tc.name})
                }).collect::<Vec<_>>()
            })),
            extensions: None,
            reference_task_ids: None,
        });

        let results_text = iteration_calls
            .iter()
            .enumerate()
            .filter_map(|(idx, call)| {
                let result_idx = call_idx + idx;
                tool_results.get(result_idx).map(|r| {
                    let content_text = extract_text_from_content(&r.content);
                    format!("Tool '{}' result: {}", call.name, content_text)
                })
            })
            .collect::<Vec<_>>()
            .join("\n");

        history.push(Message {
            role: "user".to_string(),
            parts: vec![Part::Text(TextPart { text: results_text })],
            message_id: Uuid::new_v4().to_string(),
            task_id: Some(task_id.clone()),
            context_id: ctx_id.clone(),
            kind: "message".to_string(),
            metadata: Some(json!({
                "iteration": iteration,
                "tool_results": true
            })),
            extensions: None,
            reference_task_ids: None,
        });

        call_idx += iteration_calls.len();
        iteration += 1;
    }

    history.push(Message {
        role: "agent".to_string(),
        parts: vec![Part::Text(TextPart {
            text: final_response.clone(),
        })],
        message_id: Uuid::new_v4().to_string(),
        task_id: Some(task_id.clone()),
        context_id: ctx_id.clone(),
        kind: "message".to_string(),
        metadata: Some(json!({
            "iteration": iteration,
            "final_synthesis": true
        })),
        extensions: None,
        reference_task_ids: None,
    });

    let artifacts: Vec<Artifact> = tool_results
        .iter()
        .enumerate()
        .filter_map(|(idx, result)| {
            let tool_call = tool_calls.get(idx)?;
            let tool_name = &tool_call.name;
            let call_id = tool_call.ai_tool_call_id.as_ref();
            let is_error = result.is_error?;

            let mut data_map = serde_json::Map::new();
            data_map.insert("call_id".to_string(), json!(call_id));
            data_map.insert("tool_name".to_string(), json!(tool_name));
            data_map.insert("output".to_string(), content_to_json(&result.content));
            data_map.insert(
                "status".to_string(),
                json!(if is_error { "error" } else { "success" }),
            );

            Some(Artifact {
                artifact_id: Uuid::new_v4().to_string(),
                name: Some(format!("tool_execution_{}", idx + 1)),
                description: Some(format!("Result from tool: {}", tool_name)),
                parts: vec![Part::Data(DataPart { data: data_map })],
                extensions: vec![],
                metadata: ArtifactMetadata::new(
                    "tool_execution".to_string(),
                    ctx_id.clone(),
                    task_id.clone(),
                )
                .with_mcp_execution_id(call_id.to_string())
                .with_tool_name(tool_name.to_string())
                .with_execution_index(idx),
            })
        })
        .collect();

    Task {
        id: task_id.clone(),
        context_id: ctx_id.clone(),
        kind: "task".to_string(),
        status: TaskStatus {
            state: TaskState::Completed,
            message: Some(Message {
                role: "agent".to_string(),
                parts: vec![Part::Text(TextPart {
                    text: final_response.clone(),
                })],
                message_id: Uuid::new_v4().to_string(),
                task_id: Some(task_id.clone()),
                context_id: ctx_id.clone(),
                kind: "message".to_string(),
                metadata: None,
                extensions: None,
                reference_task_ids: None,
            }),
            timestamp: Some(chrono::Utc::now()),
        },
        history: Some(history),
        artifacts: if artifacts.is_empty() {
            None
        } else {
            Some(artifacts)
        },
        metadata: Some(
            TaskMetadata::new_agent_message(agent_names::SYSTEM.to_string())
                .with_extension("total_iterations".to_string(), json!(total_iterations))
                .with_extension("total_tools_called".to_string(), json!(tool_calls.len())),
        ),
    }
}
