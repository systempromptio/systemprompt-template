const KIND_LABEL = {
  request: 'Request',
  trace: 'Trace',
  session: 'Session',
  context: 'Conversation',
};

const setExpanded = (list, input, isOpen) => {
  list.hidden = !isOpen;
  input.setAttribute('aria-expanded', String(isOpen));
  if (!isOpen) input.removeAttribute('aria-activedescendant');
};

const buildItem = (tpl, match, index) => {
  const item = tpl.content.firstElementChild.cloneNode(true);
  item.id = `admin-header-search-opt-${index}`;
  item.dataset.url = match.url;
  item.querySelector('.sp-topbar__search-kind').textContent = KIND_LABEL[match.kind] ?? match.kind;
  item.querySelector('.sp-topbar__search-id').textContent = match.id;
  return item;
};

export const createSearchList = ({ list, input, tpl }) => {
  let active = -1;

  const options = () => [...list.querySelectorAll('[role="option"]')];

  const highlight = (index) => {
    const opts = options();
    if (opts.length === 0) return;
    active = (index + opts.length) % opts.length;
    for (const [i, opt] of opts.entries()) {
      const isActive = i === active;
      opt.setAttribute('aria-selected', String(isActive));
      opt.classList.toggle('is-active', isActive);
      if (isActive) {
        input.setAttribute('aria-activedescendant', opt.id);
        opt.scrollIntoView({ block: 'nearest' });
      }
    }
  };

  const close = () => {
    active = -1;
    list.replaceChildren();
    setExpanded(list, input, false);
  };

  const show = (matches) => {
    active = -1;
    list.replaceChildren(...matches.map((m, i) => buildItem(tpl, m, i)));
    setExpanded(list, input, matches.length > 0);
  };

  const activeUrl = () => options()[active]?.dataset.url ?? null;

  const move = (delta) => highlight(active + delta);

  const isOpen = () => !list.hidden;

  return { show, close, move, activeUrl, isOpen };
};
