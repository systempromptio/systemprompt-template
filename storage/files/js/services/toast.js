import '../components/sp-toast.js';

let toastEl = null;

const getToast = () => {
  if (!toastEl) {
    toastEl = document.createElement('sp-toast');
    document.body.append(toastEl);
  }
  return toastEl;
};

export const showToast = (message, type = 'info') => {
  getToast().show(message, type);
};
