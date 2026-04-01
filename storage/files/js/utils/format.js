export const formatDate = (iso) => {
  if (!iso) {
    return '-';
  } else {
    const d = new Date(iso);
    return d.toLocaleDateString('en-US', {
      year: 'numeric', month: 'short', day: 'numeric',
      hour: '2-digit', minute: '2-digit'
    });
  }
};

export const formatRelativeTime = (iso) => {
  if (!iso) {
    return '-';
  } else {
    const diff = Date.now() - new Date(iso).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) {
      return 'just now';
    } else if (mins < 60) {
      return mins + 'm ago';
    } else {
      const hours = Math.floor(mins / 60);
      if (hours < 24) {
        return hours + 'h ago';
      } else {
        const days = Math.floor(hours / 24);
        if (days < 30) {
          return days + 'd ago';
        } else {
          return formatDate(iso);
        }
      }
    }
  }
};

export const formatTimeAgo = (el) => {
  const iso = el.getAttribute('data-time');
  if (iso) {
    el.textContent = formatRelativeTime(iso);
  }
};

export const initTimeAgo = (root = document) => {
  for (const el of root.querySelectorAll('[data-time]')) {
    formatTimeAgo(el);
  }
};
