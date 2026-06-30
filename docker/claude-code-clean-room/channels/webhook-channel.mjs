#!/usr/bin/env node
// Throwaway custom Claude Code "channel" for the gateway-push viability spike.
//
// A channel is an MCP **stdio** server that Claude Code spawns. It declares the
// experimental `claude/channel` capability, opens a local HTTP listener, and on
// an inbound POST emits an MCP notification `notifications/claude/channel`
// `{content, meta}`. That surfaces in the session as
// `<channel source="webhook" ...>content</channel>` and is documented to wake an
// idle model autonomously — IF the model's auth path is Anthropic-blessed.
//
// The whole question of this spike is whether that wake fires when Claude Code
// is pointed at our gateway via ANTHROPIC_BASE_URL (undocumented, may be gated
// with a SILENT drop). So this server logs every notification it emits to a file
// + stderr: if the log shows "emitted" but the model never acts, that is the
// gated/silent-drop outcome.
//
// Config via env:
//   CHANNEL_HTTP_PORT   HTTP listener port (default 8788)
//   CHANNEL_LOG         log file path (default /tmp/webhook-channel.log)
//   CHANNEL_OUTFILE     file the model is instructed to write the body to
//                       (default /tmp/from-channel.txt)
//   CHANNEL_SOURCE      server name -> <channel source="..."> (default webhook)
//   RELAY_URL           (Stage 2) if set, POST /register {id,port} on startup
//   INSTANCE_ID         (Stage 2) this instance's id for relay routing
//
// Built on the MCP TypeScript SDK (@modelcontextprotocol/sdk). The container is
// node 22 (no Bun), so this is plain ESM run with `node`.
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import http from 'node:http';
import fs from 'node:fs';

const PORT = Number(process.env.CHANNEL_HTTP_PORT || 8788);
const LOG = process.env.CHANNEL_LOG || '/tmp/webhook-channel.log';
const OUTFILE = process.env.CHANNEL_OUTFILE || '/tmp/from-channel.txt';
const SOURCE = process.env.CHANNEL_SOURCE || 'webhook';
const RELAY_URL = process.env.RELAY_URL || '';
const INSTANCE_ID = process.env.INSTANCE_ID || '';

// MCP transport owns stdout — every diagnostic MUST go to stderr + the log file.
function log(msg) {
  const line = `[webhook-channel ${new Date().toISOString()}] ${msg}\n`;
  process.stderr.write(line);
  try {
    fs.appendFileSync(LOG, line);
  } catch {
    /* best effort */
  }
}

// The channel's instructions are injected into the model's system prompt. Make
// the desired side effect unambiguous and autonomous so an idle wake produces an
// observable artifact (the OUTFILE write) we can assert on.
const INSTRUCTIONS =
  `You are connected to a push channel named "${SOURCE}". ` +
  `When a message arrives it appears as <channel source="${SOURCE}" ...>BODY</channel>. ` +
  `The instant you see such a channel event, autonomously and without asking, use the Write tool ` +
  `to write the EXACT text of BODY (the channel content, verbatim, nothing else) to the file ${OUTFILE}. ` +
  `Overwrite the file. Do not explain, do not wait for confirmation — just write the file immediately.`;

const server = new Server(
  { name: SOURCE, version: '0.0.1' },
  {
    capabilities: {
      // Presence of this experimental capability is what registers the channel
      // listener. One-way push only for this spike — no tools, no permission relay.
      experimental: { 'claude/channel': {} },
    },
    instructions: INSTRUCTIONS,
  },
);

server.oninitialized = () => {
  log('MCP session initialized by client (Claude Code connected the channel)');
};

// Emit a channel event into the live Claude Code session.
async function emit(content, meta) {
  await server.notification({
    method: 'notifications/claude/channel',
    params: { content, ...(meta ? { meta } : {}) },
  });
  log(`notification emitted -> content=${JSON.stringify(content)} meta=${JSON.stringify(meta || {})}`);
}

// Local HTTP listener: POST / with {content, meta?} pushes a channel event.
const httpServer = http.createServer((req, res) => {
  if (req.method !== 'POST') {
    res.writeHead(405).end('POST only\n');
    return;
  }
  let body = '';
  req.on('data', (c) => {
    body += c;
  });
  req.on('end', async () => {
    let payload;
    try {
      payload = body ? JSON.parse(body) : {};
    } catch (e) {
      log(`bad request body: ${e.message}`);
      res.writeHead(400).end('invalid JSON\n');
      return;
    }
    const content = typeof payload.content === 'string' ? payload.content : JSON.stringify(payload);
    const meta = payload.meta && typeof payload.meta === 'object' ? payload.meta : undefined;
    log(`HTTP POST received -> ${JSON.stringify(payload)}`);
    try {
      await emit(content, meta);
      res.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify({ ok: true }));
    } catch (e) {
      log(`emit failed: ${e.stack || e.message}`);
      res.writeHead(500).end('emit failed\n');
    }
  });
});

async function registerWithRelay() {
  if (!RELAY_URL || !INSTANCE_ID) return;
  try {
    const r = await fetch(`${RELAY_URL.replace(/\/$/, '')}/register`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ id: INSTANCE_ID, port: PORT }),
    });
    log(`registered with relay ${RELAY_URL} as id=${INSTANCE_ID} port=${PORT} (status ${r.status})`);
  } catch (e) {
    log(`relay registration failed: ${e.message}`);
  }
}

async function main() {
  // Fresh log per run so readiness/silent-drop detection is unambiguous.
  try {
    fs.writeFileSync(LOG, '');
  } catch {
    /* best effort */
  }
  await server.connect(new StdioServerTransport());
  log(`MCP stdio server connected (source="${SOURCE}", outfile=${OUTFILE})`);
  await new Promise((resolve, reject) => {
    httpServer.once('error', reject);
    httpServer.listen(PORT, '127.0.0.1', resolve);
  });
  log(`HTTP listener up on 127.0.0.1:${PORT}`);
  await registerWithRelay();
}

main().catch((e) => {
  log(`fatal: ${e.stack || e.message}`);
  process.exit(1);
});
