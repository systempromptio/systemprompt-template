// Only the mounted report frame may request these two read-only tools.
window.addEventListener('message', event => {
  const frame = document.getElementById('frame');
  const message = event.data;
  if (!frame || event.source !== frame.contentWindow || message?.method !== 'tools/call') return;
  const target = event.source;
  if (!['admin_report'].includes(message.params?.name)) {
    target.postMessage({ jsonrpc: '2.0', id: message.id, error: { code: -32601, message: 'Reporting tool not allowed' } }, '*');
    return;
  }
  request('tools/call', message.params).then(
    result => target.postMessage({ jsonrpc: '2.0', id: message.id, result }, '*'),
    error => target.postMessage({ jsonrpc: '2.0', id: message.id, error: { code: -32603, message: error.message } }, '*'),
  );
});
