-- Add client_message_id column for optimistic message reconciliation
ALTER TABLE task_messages ADD COLUMN IF NOT EXISTS client_message_id TEXT;

-- Create index for efficient lookups by client_message_id
CREATE INDEX IF NOT EXISTS idx_task_messages_client_id ON task_messages(client_message_id) WHERE client_message_id IS NOT NULL;
