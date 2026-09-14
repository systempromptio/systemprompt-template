// Verify the running client container's isolation without contacting a provider.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const net = require('node:net');
assert.notEqual(process.getuid(), 0, 'client must not run as root');
assert.equal(fs.existsSync('/var/run/docker.sock'), false, 'Docker socket exposed');
for (const name of ['ANTHROPIC_API_KEY', 'OPENAI_API_KEY', 'GEMINI_API_KEY']) {
  assert.equal(process.env[name], undefined, `${name} exposed`);
}
assert.throws(() => fs.writeFileSync('/home/tester/escape-probe', 'probe'),
  {code: 'EROFS'}, 'root filesystem must be read-only');
const status = fs.readFileSync('/proc/self/status', 'utf8');
assert.match(status, /^CapEff:\s+0+$/m, 'capabilities remain enabled');
assert.match(status, /^NoNewPrivs:\s+1$/m, 'privilege escalation remains enabled');
const connection = net.connect({host: '192.0.2.1', port: 443});
connection.setTimeout(1000);
connection.once('connect', () => {
  connection.destroy();
  throw new Error('unexpected external connectivity');
});
connection.once('error', error => {
  assert.ok(['ENETUNREACH', 'EHOSTUNREACH'].includes(error.code), error.message);
  console.log('Client isolation verified: non-root, read-only, no capabilities, no credentials, no egress.');
});
connection.once('timeout', () => {
  connection.destroy();
  throw new Error('egress isolation could not be verified');
});
