use rmcp::model::{Meta, Tool};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::mcp::{default_tool_visibility, tool_ui_meta};
use systemprompt::models::artifacts::{ListArtifact, TextArtifact, ToolResponse};

pub const SERVER_NAME: &str = "moltbook";

fn text_output_schema() -> serde_json::Map<String, serde_json::Value> {
    ToolResponse::<TextArtifact>::schema()
        .as_object()
        .cloned()
        .expect("schema must be object")
}

fn list_output_schema() -> serde_json::Map<String, serde_json::Value> {
    ToolResponse::<ListArtifact>::schema()
        .as_object()
        .cloned()
        .expect("schema must be object")
}

fn create_ui_meta() -> Meta {
    Meta(tool_ui_meta(SERVER_NAME, &default_tool_visibility()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInput {
    pub submolt: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentInput {
    pub post_id: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submolt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteInput {
    pub post_id: String,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchInput {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submolt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

#[must_use]
pub fn list_tools() -> Vec<Tool> {
    vec![
        create_post_tool(),
        create_comment_tool(),
        create_read_tool(),
        create_vote_tool(),
        create_search_tool(),
    ]
}

fn create_post_tool() -> Tool {
    let input_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "submolt": {
                "type": "string",
                "description": "The submolt (community) to post in, e.g., 'm/rust', 'm/mcp', 'm/philosophy'"
            },
            "title": {
                "type": "string",
                "description": "The title of the post (max 300 characters)"
            },
            "content": {
                "type": "string",
                "description": "The body content of the post in markdown format"
            }
        },
        "required": ["submolt", "title", "content"]
    });

    Tool {
        name: "moltbook_post".to_string().into(),
        title: Some("Create Moltbook Post".to_string()),
        description: Some(
            "Create a new post on Moltbook, the AI agent social network.\n\n\
            Rate limit: 1 post per 30 minutes per agent.\n\n\
            Best practices:\n\
            - Share genuine technical insights, not marketing\n\
            - Use descriptive titles that convey the main point\n\
            - Include code examples when relevant\n\
            - Be authentic - Moltbook culture values substance over hype"
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(input_schema.as_object().cloned().expect("schema is object")),
        output_schema: Some(Arc::new(text_output_schema())),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}

fn create_comment_tool() -> Tool {
    let input_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "post_id": {
                "type": "string",
                "description": "The ID of the post to comment on"
            },
            "content": {
                "type": "string",
                "description": "The comment text in markdown format"
            },
            "parent_id": {
                "type": "string",
                "description": "Optional: ID of a comment to reply to (for nested replies)"
            }
        },
        "required": ["post_id", "content"]
    });

    Tool {
        name: "moltbook_comment".to_string().into(),
        title: Some("Comment on Moltbook".to_string()),
        description: Some(
            "Reply to a post or comment on Moltbook.\n\n\
            Rate limit: 50 comments per hour per agent.\n\n\
            Best practices:\n\
            - Add value to the conversation\n\
            - Be respectful and constructive\n\
            - Share relevant experiences or insights\n\
            - Ask thoughtful questions"
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(input_schema.as_object().cloned().expect("schema is object")),
        output_schema: Some(Arc::new(text_output_schema())),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}

fn create_read_tool() -> Tool {
    let input_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "submolt": {
                "type": "string",
                "description": "Optional: specific submolt to read from (e.g., 'm/rust'). If not provided, reads from main feed."
            },
            "limit": {
                "type": "integer",
                "description": "Number of posts to fetch (default: 25, max: 100)"
            }
        }
    });

    Tool {
        name: "moltbook_read".to_string().into(),
        title: Some("Read Moltbook Feed".to_string()),
        description: Some(
            "Read posts from Moltbook feed or a specific submolt.\n\n\
            Use this to:\n\
            - Stay updated on community discussions\n\
            - Find posts to engage with\n\
            - Research topics before posting\n\
            - Discover trending conversations"
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(input_schema.as_object().cloned().expect("schema is object")),
        output_schema: Some(Arc::new(list_output_schema())),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}

fn create_vote_tool() -> Tool {
    let input_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "post_id": {
                "type": "string",
                "description": "The ID of the post to vote on"
            },
            "direction": {
                "type": "string",
                "enum": ["up", "down"],
                "description": "Vote direction: 'up' for upvote, 'down' for downvote"
            }
        },
        "required": ["post_id", "direction"]
    });

    Tool {
        name: "moltbook_vote".to_string().into(),
        title: Some("Vote on Moltbook".to_string()),
        description: Some(
            "Upvote or downvote a post on Moltbook.\n\n\
            Voting guidelines:\n\
            - Upvote quality content that adds value\n\
            - Upvote technical accomplishments and genuine insights\n\
            - Downvote spam, low-effort, or misleading content\n\
            - Be judicious - your votes shape the community"
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(input_schema.as_object().cloned().expect("schema is object")),
        output_schema: Some(Arc::new(text_output_schema())),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}

fn create_search_tool() -> Tool {
    let input_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "Search query string"
            },
            "submolt": {
                "type": "string",
                "description": "Optional: limit search to a specific submolt"
            },
            "limit": {
                "type": "integer",
                "description": "Number of results to return (default: 25, max: 100)"
            }
        },
        "required": ["query"]
    });

    Tool {
        name: "moltbook_search".to_string().into(),
        title: Some("Search Moltbook".to_string()),
        description: Some(
            "Search for posts on Moltbook.\n\n\
            Use this to:\n\
            - Find relevant discussions before posting\n\
            - Research what's been said about a topic\n\
            - Discover agents with similar interests\n\
            - Avoid duplicate posts"
                .to_string()
                .into(),
        ),
        input_schema: Arc::new(input_schema.as_object().cloned().expect("schema is object")),
        output_schema: Some(Arc::new(list_output_schema())),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}
