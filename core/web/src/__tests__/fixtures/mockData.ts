import type { Artifact } from '@/types/artifact'

const MOCK_CONTEXT_ID = 'mock-context'
const MOCK_TASK_ID = 'mock-task'
const MOCK_CREATED_AT = new Date().toISOString()

export const mockTableArtifact: Artifact = {
  artifactId: 'table-1',
  name: 'User Analytics',
  description: 'User activity summary for the last 7 days',
  parts: [
    {
      kind: 'text',
      text: 'Found 5 users in the system',
    },
    {
      kind: 'data',
      data: {
        items: [
          {
            user_id: 'alice@example.com',
            total_sessions: 42,
            total_requests: 156,
            total_ai_requests: 89,
            total_tokens: 125430,
            total_cost: 12.54,
            avg_response_time: 234.5,
            total_tasks: 12,
            total_messages: 89,
            active_days: 7,
            total_errors: 3,
            avg_success_rate: 96.2,
          },
          {
            user_id: 'bob@example.com',
            total_sessions: 28,
            total_requests: 92,
            total_ai_requests: 52,
            total_tokens: 87320,
            total_cost: 8.73,
            avg_response_time: 189.3,
            total_tasks: 8,
            total_messages: 52,
            active_days: 5,
            total_errors: 1,
            avg_success_rate: 98.9,
          },
          {
            user_id: 'charlie@example.com',
            total_sessions: 15,
            total_requests: 45,
            total_ai_requests: 28,
            total_tokens: 42150,
            total_cost: 4.22,
            avg_response_time: 312.1,
            total_tasks: 5,
            total_messages: 28,
            active_days: 3,
            total_errors: 0,
            avg_success_rate: 100.0,
          },
          {
            user_id: 'diana@example.com',
            total_sessions: 67,
            total_requests: 234,
            total_ai_requests: 145,
            total_tokens: 198750,
            total_cost: 19.88,
            avg_response_time: 276.8,
            total_tasks: 18,
            total_messages: 145,
            active_days: 7,
            total_errors: 5,
            avg_success_rate: 96.6,
          },
          {
            user_id: 'eve@example.com',
            total_sessions: 11,
            total_requests: 38,
            total_ai_requests: 22,
            total_tokens: 35400,
            total_cost: 3.54,
            avg_response_time: 201.4,
            total_tasks: 4,
            total_messages: 22,
            active_days: 2,
            total_errors: 0,
            avg_success_rate: 100.0,
          },
        ],
      } satisfies Record<string, unknown>,
    },
  ],
  extensions: ['https://systemprompt.io/extensions/artifact-rendering/v1'],
  metadata: {
    artifact_type: 'table',
    context_id: MOCK_CONTEXT_ID,
    task_id: MOCK_TASK_ID,
    created_at: MOCK_CREATED_AT,
    rendering_hints: {
      columns: [
        'user_id',
        'total_sessions',
        'total_requests',
        'total_ai_requests',
        'total_tokens',
        'total_cost',
        'avg_response_time',
        'total_tasks',
        'total_messages',
        'active_days',
        'total_errors',
        'avg_success_rate',
      ],
      sortable_columns: ['total_sessions', 'total_requests', 'total_tokens', 'total_cost', 'active_days'],
      default_sort: {
        column: 'total_sessions',
        order: 'desc' as const,
      },
      filterable: true,
      page_size: 10,
      column_types: {
        user_id: 'string' as const,
        total_sessions: 'integer' as const,
        total_requests: 'integer' as const,
        total_ai_requests: 'integer' as const,
        total_tokens: 'integer' as const,
        total_cost: 'currency' as const,
        avg_response_time: 'number' as const,
        total_tasks: 'integer' as const,
        total_messages: 'integer' as const,
        active_days: 'integer' as const,
        total_errors: 'integer' as const,
        avg_success_rate: 'percentage' as const,
      },
    },
    source: 'mcp_tool',
  },
}

export const mockChartArtifact: Artifact = {
  artifactId: 'chart-1',
  name: 'Daily Activity Trend',
  description: 'User activity over the last 7 days',
  parts: [
    {
      kind: 'text',
      text: 'Daily activity trend for the last 7 days',
    },
    {
      kind: 'data',
      data: {
        labels: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
        datasets: [
          {
            label: 'Sessions',
            data: [45, 52, 48, 61, 55, 38, 42],
            color: '#4CAF50',
          },
          {
            label: 'Requests',
            data: [167, 189, 178, 212, 198, 145, 156],
            color: '#2196F3',
          },
          {
            label: 'Tokens (in thousands)',
            data: [125, 142, 135, 156, 148, 108, 118],
            color: '#FF9800',
          },
        ],
      },
    },
  ],
  extensions: ['https://systemprompt.io/extensions/artifact-rendering/v1'],
  metadata: {
    artifact_type: 'chart',
    context_id: MOCK_CONTEXT_ID,
    task_id: MOCK_TASK_ID,
    created_at: MOCK_CREATED_AT,
    rendering_hints: {
      chart_type: 'line' as const,
      title: 'Daily Activity Trend',
      x_axis: {
        label: 'Day of Week',
        type: 'category' as const,
      },
      y_axis: {
        label: 'Count',
        type: 'linear' as const,
      },
      series: [
        { name: 'Sessions', color: '#4CAF50' },
        { name: 'Requests', color: '#2196F3' },
        { name: 'Tokens', color: '#FF9800' },
      ],
    },
    source: 'mcp_tool',
  },
}

export const mockCodeArtifact: Artifact = {
  artifactId: 'code-1',
  name: 'Generated API Endpoint',
  description: 'Rust API endpoint for user management',
  parts: [
    {
      kind: 'text',
      text: 'Generated Rust API endpoint',
    },
    {
      kind: 'data',
      data: {
        code: `pub async fn get_user_handler(
    Path(user_id): Path<i64>,
    State(repo): State<UserRepository>,
) -> impl IntoResponse {
    match repo.get_user(user_id).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}`,
      } satisfies Record<string, unknown>,
    },
  ],
  extensions: ['https://systemprompt.io/extensions/artifact-rendering/v1'],
  metadata: {
    artifact_type: 'code',
    context_id: MOCK_CONTEXT_ID,
    task_id: MOCK_TASK_ID,
    created_at: MOCK_CREATED_AT,
    rendering_hints: {
      language: 'rust',
      theme: 'oneDark',
      line_numbers: true,
      highlight_lines: [3, 4, 5],
    },
    source: 'mcp_tool',
  },
}

export const mockFormArtifact: Artifact = {
  artifactId: 'form-1',
  name: 'Create User Form',
  description: 'Form to create a new user',
  parts: [
    {
      kind: 'text',
      text: 'User creation form ready',
    },
    {
      kind: 'data',
      data: {
        form_type: 'create_user',
        endpoint: '/api/v1/core/users',
      } as Record<string, unknown>,
    },
  ],
  extensions: ['https://systemprompt.io/extensions/artifact-rendering/v1'],
  metadata: {
    artifact_type: 'form',
    context_id: MOCK_CONTEXT_ID,
    task_id: MOCK_TASK_ID,
    created_at: MOCK_CREATED_AT,
    rendering_hints: {
      submit_action: '/api/v1/core/users',
      submit_method: 'POST' as const,
      layout: 'vertical' as const,
      fields: [
        {
          name: 'username',
          type: 'text',
          label: 'Username',
          placeholder: 'Enter username',
          help_text: '3-50 alphanumeric characters',
          required: true,
        },
        {
          name: 'email',
          type: 'email',
          label: 'Email Address',
          placeholder: 'user@example.com',
          required: true,
        },
        {
          name: 'password',
          type: 'password',
          label: 'Password',
          help_text: 'Minimum 8 characters',
          required: true,
        },
        {
          name: 'role',
          type: 'select',
          label: 'User Role',
          required: true,
          options: [
            { value: 'admin', label: 'Administrator' },
            { value: 'user', label: 'Standard User' },
            { value: 'guest', label: 'Guest' },
          ],
          default: 'user',
        },
        {
          name: 'send_welcome_email',
          type: 'checkbox',
          label: 'Send welcome email',
          default: true,
        },
      ],
    },
    source: 'mcp_tool',
  },
}

export const mockTreeArtifact: Artifact = {
  artifactId: 'tree-1',
  name: 'System Status',
  description: 'Hierarchical system health status',
  parts: [
    {
      kind: 'text',
      text: 'System health status',
    },
    {
      kind: 'data',
      data: {
        name: 'SystemPrompt OS',
        status: 'healthy',
        children: [
          {
            name: 'API Server',
            status: 'healthy',
            metadata: {
              uptime: '5d 3h',
              requests: 15234,
            },
          },
          {
            name: 'Database',
            status: 'healthy',
            metadata: {
              connections: 12,
              queries: 892341,
            },
          },
          {
            name: 'Agents',
            status: 'warning',
            children: [
              {
                name: 'Agent-1',
                status: 'healthy',
                metadata: {
                  tasks: 42,
                  uptime: '2d 1h',
                },
              },
              {
                name: 'Agent-2',
                status: 'warning',
                metadata: {
                  error: 'High memory usage',
                  memory: '85%',
                },
              },
              {
                name: 'Agent-3',
                status: 'healthy',
                metadata: {
                  tasks: 28,
                  uptime: '1d 12h',
                },
              },
            ],
          },
          {
            name: 'MCP Servers',
            status: 'healthy',
            children: [
              {
                name: 'systemprompt-admin',
                status: 'healthy',
                metadata: {
                  tools: 7,
                  uptime: '3d 8h',
                },
              },
              {
                name: 'github-mcp',
                status: 'healthy',
                metadata: {
                  tools: 12,
                  uptime: '1d 4h',
                },
              },
            ],
          },
        ],
      },
    },
  ],
  extensions: ['https://systemprompt.io/extensions/artifact-rendering/v1'],
  metadata: {
    artifact_type: 'tree',
    context_id: MOCK_CONTEXT_ID,
    task_id: MOCK_TASK_ID,
    created_at: MOCK_CREATED_AT,
    rendering_hints: {
      expandable: true,
      default_expanded_levels: 2,
      show_icons: true,
      icon_map: {
        healthy: 'check-circle',
        warning: 'alert-triangle',
        error: 'x-circle',
        unknown: 'help-circle',
      },
    },
    source: 'mcp_tool',
  },
}

export const mockArtifacts = {
  table: mockTableArtifact,
  chart: mockChartArtifact,
  code: mockCodeArtifact,
  form: mockFormArtifact,
  tree: mockTreeArtifact,
}
