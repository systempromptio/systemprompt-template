const handlers = {
  click: [],
  change: [],
  keydown: [],
  input: []
};

export const on = (eventType, selector, handler, options) => {
  const entry = {
    selector,
    handler,
    exclusive: options?.exclusive || false
  };
  if (handlers[eventType]) {
    handlers[eventType].push(entry);
  }
};

const dispatch = (entries, e) => {
  for (const entry of entries) {
    const match = e.target.closest(entry.selector);
    if (match) {
      entry.handler(e, match);
      if (entry.exclusive) return true;
    }
  }
  return false;
};

let closeMenusFn = null;

export const setCloseMenus = (fn) => {
  closeMenusFn = fn;
};

export const initDelegation = () => {
  document.addEventListener('click', (e) => {
    const handled = dispatch(handlers.click, e);
    if (!handled && closeMenusFn) closeMenusFn();
  });
  document.addEventListener('change', (e) => dispatch(handlers.change, e));
  document.addEventListener('input', (e) => dispatch(handlers.input, e));
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && closeMenusFn) closeMenusFn();
    dispatch(handlers.keydown, e);
  });
};
