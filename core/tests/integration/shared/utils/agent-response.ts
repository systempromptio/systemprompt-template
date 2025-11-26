export interface A2AResponse {
  status?: {
    task?: {
      status?: {
        message?: {
          role: string;
          parts?: Array<{ kind: string; text?: string; data?: any }>;
          messageId?: string;
          taskId?: string;
          contextId?: string;
        };
        timestamp?: string;
      };
      history?: Array<any>;
      artifacts?: Array<any>;
      kind?: string;
    };
  };
  error?: {
    code: number;
    message: string;
    data?: any;
  };
}

export function extractTextFromResponse(response: A2AResponse): string | null {
  const parts = response?.status?.task?.status?.message?.parts || [];
  const textParts = parts.filter(p => p.kind === 'text' && p.text);
  return textParts.map(p => p.text).join(' ') || null;
}

export function extractAllTextParts(response: A2AResponse): Array<{ kind: string; text?: string }> {
  return response?.status?.task?.status?.message?.parts?.filter(p => p.kind === 'text') || [];
}

export function extractArtifactsFromResponse(response: A2AResponse): any[] {
  return response?.status?.task?.artifacts || [];
}

export function extractHistoryFromResponse(response: A2AResponse): any[] {
  return response?.status?.task?.history || [];
}

export function hasError(response: A2AResponse | undefined): boolean {
  return !!response?.error;
}

export function getErrorMessage(response: A2AResponse | undefined): string | null {
  return response?.error?.message || null;
}

export function getErrorCode(response: A2AResponse | undefined): number | null {
  return response?.error?.code || null;
}

export function getMessageId(response: A2AResponse): string | null {
  return response?.status?.task?.status?.message?.messageId || null;
}

export function getTaskId(response: A2AResponse): string | null {
  return response?.status?.task?.status?.message?.taskId || null;
}

export function getContextId(response: A2AResponse): string | null {
  return response?.status?.task?.status?.message?.contextId || null;
}

export function getMessageRole(response: A2AResponse): string | null {
  return response?.status?.task?.status?.message?.role || null;
}

export function getTimestamp(response: A2AResponse): string | null {
  return response?.status?.task?.status?.timestamp || null;
}

export function hasToolErrors(response: A2AResponse): boolean {
  const artifacts = extractArtifactsFromResponse(response);
  return artifacts.some(a => a.metadata?.artifact_type === 'tool_error');
}

export function getToolErrors(response: A2AResponse): any[] {
  const artifacts = extractArtifactsFromResponse(response);
  return artifacts.filter(a => a.metadata?.artifact_type === 'tool_error');
}

export function extractToolResults(response: A2AResponse): any[] {
  const artifacts = extractArtifactsFromResponse(response);
  return artifacts.filter(a => a.metadata?.artifact_type === 'tool_result');
}

export function extractDataArtifacts(response: A2AResponse): any[] {
  const artifacts = extractArtifactsFromResponse(response);
  return artifacts.filter(a => a.parts?.[0]?.kind === 'data');
}
