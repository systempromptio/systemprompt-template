// The two collapsible controls in the top bar: the actions menu that holds the
// header's icons at narrow widths, and the install menu behind the plugin
// counter. Both are `.is-open` toggles in CSS with nothing to set that class
// on them until this module runs and binds the triggers below.

// Why: look each element up by the id its template actually writes. These were
// queried as `sp-topbar__actions` and `sp-install-menu`, while layout.hbs and
// install-widget.hbs write `header-actions` and `install-menu`, so neither
// control was ever wired and the install button did nothing at any width.
const ACTIONS_ID = 'header-actions';
const INSTALL_ID = 'install-menu';

// Why: stopPropagation keeps the document listener that closes every open menu
// from immediately undoing the class this toggle has just set on the root.
const bindToggle = (root, triggerSelector) => {
  const trigger = root?.querySelector(triggerSelector);
  if (!trigger) return;
  trigger.addEventListener('click', (event) => {
    event.stopPropagation();
    const open = root.classList.toggle('is-open');
    trigger.setAttribute('aria-expanded', open ? 'true' : 'false');
  });
};

export const initHeaderActions = () => {
  bindToggle(document.getElementById(ACTIONS_ID), '.sp-topbar__actions-toggle');
  bindToggle(document.getElementById(INSTALL_ID), '.sp-install-trigger');
};
