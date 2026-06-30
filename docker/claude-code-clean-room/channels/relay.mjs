#!/usr/bin/env node
// Throwaway instance-id -> channel-port routing shim for the Stage 2 A->B demo.
//
// This stands in for the gateway's real message-bus spine (event_outbox +
// Postgres LISTEN/NOTIFY + POST /api/v1/webhook/broadcast, addressed by
// bridge_sessions). The spike proves the last mile (a push reaches a listening
// Claude Code instance and wakes it) with the simplest possible router; the real
// bus is the documented follow-up.
//
//   POST /register {id, port}  -> remember which channel HTTP port an instance owns
//   POST /send {to, text}      -> look up `to`'s port, forward {content: text} to
//                                 that instance's webhook-channel listener
//
// Config via env:
//   RELAY_PORT   listen port (default 8790)
//   RELAY_LOG    log file (default /tmp/relay.log)
import http from 'node:http';
import fs from 'node:fs';

const PORT = Number(process.env.RELAY_PORT || 8790);
const LOG = process.env.RELAY_LOG || '/tmp/relay.log';

const routes = new Map(); // id -> port

function log(msg) {
  const line = `[relay ${new Date().toISOString()}] ${msg}\n`;
  process.stderr.write(line);
  try {
    fs.appendFileSync(LOG, line);
  } catch {
    /* best effort */
  }
}

function readJson(req) {
  return new Promise((resolve, reject) => {
    let body = '';
    req.on('data', (c) => {
      body += c;
    });
    req.on('end', () => {
      try {
        resolve(body ? JSON.parse(body) : {});
      } catch (e) {
        reject(e);
      }
    });
    req.on('error', reject);
  });
}

const server = http.createServer(async (req, res) => {
  try {
    if (req.method === 'POST' && req.url === '/register') {
      const { id, port } = await readJson(req);
      if (!id || !port) {
        res.writeHead(400).end('id and port required\n');
        return;
      }
      routes.set(String(id), Number(port));
      log(`register id=${id} -> port=${port}  (routes: ${JSON.stringify([...routes])})`);
      res.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify({ ok: true }));
      return;
    }
    if (req.method === 'POST' && req.url === '/send') {
      const { to, text } = await readJson(req);
      const port = routes.get(String(to));
      if (!port) {
        log(`send to unknown id=${to} (known: ${JSON.stringify([...routes.keys()])})`);
        res.writeHead(404).end(`unknown id ${to}\n`);
        return;
      }
      log(`send to=${to} (port ${port}) text=${JSON.stringify(text)}`);
      const r = await fetch(`http://127.0.0.1:${port}/`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ content: text, meta: { from: 'relay' } }),
      });
      log(`forwarded to id=${to} -> channel responded ${r.status}`);
      res.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify({ ok: true, forwarded: r.status }));
      return;
    }
    res.writeHead(404).end('not found\n');
  } catch (e) {
    log(`error: ${e.stack || e.message}`);
    res.writeHead(500).end('error\n');
  }
});

try {
  fs.writeFileSync(LOG, '');
} catch {
  /* best effort */
}
server.listen(PORT, '127.0.0.1', () => log(`relay up on 127.0.0.1:${PORT}`));
