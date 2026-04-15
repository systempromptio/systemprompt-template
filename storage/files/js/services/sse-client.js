export const createSSEClient = (url, handlers, opts = {}) => {
  let source = null;
  let reconnectTimer = null;
  const reconnectDelay = opts.reconnectDelay || 3000;

  const connect = () => {
    source = new EventSource(url);

    source.onopen = () => {
      handlers.onConnect?.();
    };

    source.onerror = () => {
      source.close();
      handlers.onDisconnect?.();
      reconnectTimer = setTimeout(connect, reconnectDelay);
    };

    for (const [event, handler] of Object.entries(handlers.events || {})) {
      source.addEventListener(event, (e) => {
        try {
          const data = JSON.parse(e.data);
          handler(data);
        } catch (_e) {
          handler(e.data);
        }
      });
    }
  };

  const disconnect = () => {
    clearTimeout(reconnectTimer);
    source?.close();
    source = null;
  };

  connect();

  return { disconnect };
};
