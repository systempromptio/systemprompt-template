export const escapeHtml = (str) => {
  if (!str) {
    return '';
  } else {
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;');
  }
};

export const truncate = (str, max = 60) => {
  if (!str) {
    return '';
  } else {
    return str.length <= max ? str : str.substring(0, max) + '...';
  }
};

export const showLoading = (el, msg = 'Loading...') => {
  el.textContent = '';
  const wrapper = document.createElement('div');
  wrapper.className = 'loading-spinner';
  const spinner = document.createElement('div');
  spinner.className = 'spinner';
  const p = document.createElement('p');
  p.textContent = msg;
  wrapper.append(spinner);
  wrapper.append(p);
  el.append(wrapper);
};

export const showEmpty = (el, msg) => {
  el.textContent = '';
  const wrapper = document.createElement('div');
  wrapper.className = 'empty-state';
  const p = document.createElement('p');
  p.textContent = msg;
  wrapper.append(p);
  el.append(wrapper);
};

export const announceToScreenReader = (message) => {
  const region = document.querySelector('[data-live-region]');
  if (region != null) {
    region.textContent = '';
    requestAnimationFrame(() => { region.textContent = message; });
  }
};
