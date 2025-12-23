use serde_json::json;

pub const AGENT_MANAGEMENT_PROMPT: &str = r#"
You are an agent management system for SystemPrompt OS. Analyze the user's message and determine what agent operation they want to perform.

Available operations:
- create: Create a new agent
- update: Modify an existing agent
- delete: Remove an agent
- list: Show all agents
- get: Get details about a specific agent

Based on the user's message, extract the operation and relevant details. Generate a JSON response with the exact structure defined by the schema.

For create operations, generate appropriate:
- agent_name: lowercase, hyphenated, descriptive name
- description: Clear, comprehensive description
- capabilities: Based on the agent's purpose
- skills: Relevant skills for the agent's role

For delete/update/get operations, try to identify the agent by UUID or name from the message.
For list operations, just return {"operation": "list"}.

Examples:
- "Create an admin agent for system management" -> create operation with admin capabilities
- "Delete agent uuid-123" -> delete operation with uuid extracted
- "Show me all agents" -> list operation
- "Update agent uuid-456 to have monitoring capabilities" -> update operation with uuid
"#;

#[must_use] pub fn get_agent_operation_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "operation": {
                "type": "string",
                "enum": ["create", "update", "delete", "list", "get"],
                "description": "The operation to perform"
            },
            "uuid": {
                "type": "string",
                "description": "Agent's unique identifier (UUID format, required for update/delete/get)"
            },
            "agent_name": {
                "type": "string",
                "description": "Agent name for create operations (lowercase, hyphenated)",
                "pattern": "^[a-z0-9-]+$"
            },
            "name": {
                "type": "string",
                "description": "Display name for the agent"
            },
            "description": {
                "type": "string",
                "description": "Detailed description of the agent's purpose and capabilities"
            },
            "version": {
                "type": "string",
                "description": "Version of the agent",
                "default": "1.0.0"
            },
            "endpoint": {
                "type": "string",
                "description": "HTTP endpoint URL for the agent"
            },
            "capabilities": {
                "type": "object",
                "properties": {
                    "streaming": {
                        "type": "boolean",
                        "default": false
                    },
                    "push_notifications": {
                        "type": "boolean",
                        "default": false
                    },
                    "state_transition_history": {
                        "type": "boolean",
                        "default": true
                    }
                },
                "additionalProperties": false
            },
            "skills": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Unique skill identifier"
                        },
                        "name": {
                            "type": "string",
                            "description": "Skill display name"
                        },
                        "description": {
                            "type": "string",
                            "description": "What this skill does"
                        },
                        "tags": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Tags for categorizing the skill"
                        }
                    },
                    "required": ["id", "name", "description"]
                }
            }
        },
        "required": ["operation"],
        "additionalProperties": false
    })
}

#[must_use] pub fn build_agent_prompt_content(task_type: &str, domain: &str) -> String {
    format!(
        "You are a SystemPrompt agent architect. Task: {task_type} for {domain} domain.\n\n\
        Use the manage_agents tool to create, update, or manage agents.\n\n\
        ## Agent Design Principles\n\
        1. **Single Responsibility**: Each agent should have one clear purpose\n\
        2. **Clear Interfaces**: Well-defined skills and capabilities\n\
        3. **Composability**: Agents should work well together\n\
        4. **Error Handling**: Robust failure recovery\n\n\
        ## Task Guidelines\n\n\
        ### For 'design' tasks:\n\
        - Analyze requirements and propose agent architecture\n\
        - Define clear agent roles and responsibilities\n\
        - Specify skills and capabilities needed\n\
        - Use manage_agents with operation='create' and detailed instructions\n\n\
        ### For 'review' tasks:\n\
        - List existing agents with operation='list'\n\
        - Get details with operation='get' for each agent\n\
        - Analyze agent effectiveness and overlap\n\
        - Suggest improvements or consolidations\n\n\
        ### For 'optimize' tasks:\n\
        - Identify performance bottlenecks\n\
        - Review agent capabilities and skills\n\
        - Update agents with operation='update' for improvements\n\
        - Consider agent communication patterns\n\n\
        ### For 'troubleshoot' tasks:\n\
        - Check agent logs with get_logs tool\n\
        - Verify agent configurations\n\
        - Test agent endpoints and connectivity\n\
        - Diagnose and fix issues\n\n\
        ## Managing Agents\n\n\
        The manage_agents tool supports:\n\
        - **create**: Create new agent with AI-generated metadata\n\
        - **update**: Modify existing agent configuration\n\
        - **delete**: Remove an agent from the system\n\
        - **list**: Show all registered agents\n\
        - **get**: Get detailed information about specific agent\n\n\
        When creating or updating agents, provide clear instructions for the AI \
        to generate appropriate skills and capabilities based on the agent's purpose.\n\n\
        Current task type: {task_type}\n\
        Domain: {domain}"
    )
}
