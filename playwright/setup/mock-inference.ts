// Deterministic mock inference upstream for the e2e suite.
//
// Speaks the Anthropic wire on POST /v1/messages and echoes the last user
// message back, so a full gateway round-trip (auth -> governance -> dispatch
// -> audit -> cost) runs against real traffic without a live provider. The
// gateway reaches it via the `e2e-mock` provider in the local profile
// (endpoint http://127.0.0.1:18091/v1, surface backend, model
// e2e-mock-sonnet).
//
// Run standalone (`npx tsx setup/mock-inference.ts`) or let global-setup
// spawn it; either way it is idempotent — if the port is already held by a
// live mock (GET /health answers), the second start exits quietly.
import { createServer } from 'node:http';

export const MOCK_PORT = 18091;

type WireMessage = { role: string; content: unknown };

function lastUserText(messages: WireMessage[]): string {
  for (let i = messages.length - 1; i >= 0; i -= 1) {
    const m = messages[i];
    if (m.role === 'user') {
      if (typeof m.content === 'string') return m.content;
      if (Array.isArray(m.content)) {
        const texts = m.content
          .filter((b): b is { type: string; text: string } => b?.type === 'text')
          .map((b) => b.text);
        if (texts.length > 0) return texts.join('\n');
      }
    }
  }
  return '';
}

export function startMockInference(): Promise<void> {
  const server = createServer((req, res) => {
    if (req.method === 'GET' && req.url === '/health') {
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end('{"mock":"inference"}');
      return;
    }
    if (req.method !== 'POST' || !req.url?.endsWith('/messages')) {
      res.writeHead(404, { 'content-type': 'application/json' });
      res.end('{"error":"mock-inference: unknown route"}');
      return;
    }
    let body = '';
    req.on('data', (c) => {
      body += c;
    });
    req.on('end', () => {
      let parsed: { model?: string; messages?: WireMessage[] };
      try {
        parsed = JSON.parse(body);
      } catch {
        res.writeHead(400, { 'content-type': 'application/json' });
        res.end('{"error":"invalid json"}');
        return;
      }
      const prompt = lastUserText(parsed.messages ?? []);
      const text = `Mock reply: ${prompt}`.slice(0, 4000);
      const inputTokens = Math.max(1, Math.ceil(body.length / 4));
      const outputTokens = Math.max(1, Math.ceil(text.length / 4));
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(
        JSON.stringify({
          id: `msg_mock_${Date.now()}`,
          type: 'message',
          role: 'assistant',
          model: parsed.model ?? 'e2e-mock-sonnet',
          content: [{ type: 'text', text }],
          stop_reason: 'end_turn',
          stop_sequence: null,
          usage: { input_tokens: inputTokens, output_tokens: outputTokens },
        }),
      );
    });
  });

  return new Promise((resolve, reject) => {
    server.once('error', (err: NodeJS.ErrnoException) => {
      if (err.code === 'EADDRINUSE') {
        resolve();
      } else {
        reject(err);
      }
    });
    server.listen(MOCK_PORT, '127.0.0.1', () => {
      console.log(`mock inference listening on 127.0.0.1:${MOCK_PORT}`);
      resolve();
    });
  });
}

if (require.main === module) {
  startMockInference().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
