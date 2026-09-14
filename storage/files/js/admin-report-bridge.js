// Deterministic report calls from the sandboxed MCP Apps artifact to its viewer.
if (!window.cowork && window.parent !== window) {
  const pending = new Map();
  let sequence = 0;
  window.cowork = { callMcpTool: (tool, args) => new Promise((resolve, reject) => {
    const name = tool.replace(/^mcp__systemprompt__/, '');
    if (!['admin_report'].includes(name)) {
      reject(new Error('This reporting artifact only supports read-only report tools'));
      return;
    }
    const id = 'admin-read-' + (++sequence);
    const timer = setTimeout(() => { pending.delete(id); reject(new Error('MCP report timed out')); }, 180000);
    pending.set(id, { resolve, reject, timer });
    window.parent.postMessage({ jsonrpc: '2.0', id, method: 'tools/call', params: { name, arguments: args } }, '*');
  }) };
  window.addEventListener('message', event => {
    if (event.source !== window.parent || !pending.has(event.data?.id)) return;
    const item = pending.get(event.data.id);
    pending.delete(event.data.id);
    clearTimeout(item.timer);
    if (event.data.error) item.reject(new Error(event.data.error.message || 'MCP report failed'));
    else item.resolve(event.data.result);
  });
  let scheduled = false;
  new ResizeObserver(() => {
    if (scheduled) return;
    scheduled = true;
    requestAnimationFrame(() => {
      scheduled = false;
      window.parent.postMessage({ jsonrpc: '2.0', method: 'ui/notifications/size-changed', params: {
        width: document.documentElement.scrollWidth, height: document.documentElement.scrollHeight,
      } }, '*');
    });
  }).observe(document.documentElement);
}
