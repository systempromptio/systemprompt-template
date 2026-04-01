import { attachFilterHandlers } from '../utils/form.js';
import { renderReview, restoreStepState } from './admin-plugin-wizard-review.js';

const TOTAL_STEPS = 7;

const getTemplate = (id) => {
  const tpl = document.getElementById(id);
  return tpl ? tpl.content.cloneNode(true) : document.createDocumentFragment();
};

const STEP_LABELS = ['Basic Info', 'Skills', 'Agents', 'MCP Servers', 'Hooks', 'Roles & Access', 'Review'];

const buildStepBadge = (i, state) => {
  const isActive = i === state.step;
  const isDone = i < state.step;
  const div = document.createElement('div');
  div.className = 'wizard-step-badge' + (isActive ? ' is-active' : '') + (isDone ? ' is-done' : '');
  const num = document.createElement('span');
  num.className = 'wizard-step-num';
  num.textContent = i;
  const label = document.createElement('span');
  label.textContent = STEP_LABELS[i - 1];
  div.append(num);
  div.append(label);
  return div;
};

const renderStepIndicator = (state) => {
  const container = document.getElementById('wizard-step-indicator');
  if (container) {
    container.textContent = '';
    const steps = document.createElement('div');
    steps.className = 'wizard-steps';
    for (let i = 1; i <= TOTAL_STEPS; i++) steps.append(buildStepBadge(i, state));
    container.append(steps);
  }
};

const makeNavBtn = (id, label, cls) => {
  const btn = document.createElement('button');
  btn.type = 'button';
  btn.className = cls;
  btn.id = id;
  btn.textContent = label;
  return btn;
};

const renderNav = (state) => {
  const nav = document.getElementById('wizard-nav');
  if (nav) {
    nav.textContent = '';
    const wrapper = document.createElement('div');
    wrapper.className = 'wizard-nav';
    if (state.step > 1) wrapper.append(makeNavBtn('wizard-prev', 'Previous', 'btn btn-secondary'));
    if (state.step < TOTAL_STEPS) wrapper.append(makeNavBtn('wizard-next', 'Next', 'btn btn-primary'));
    if (state.step === TOTAL_STEPS) wrapper.append(makeNavBtn('wizard-create', 'Create Plugin', 'btn btn-primary'));
    nav.append(wrapper);
  }
};

const setField = (entry, name, value) => {
  const el = entry.querySelector('[name="' + name + '"]');
  if (el) el.value = value;
};

export const renderHooks = (state) => {
  const list = document.getElementById('hooks-list');
  if (list) {
    list.textContent = '';
    for (const hook of state.hooks) {
      const frag = getTemplate('tpl-hook-entry');
      const entry = frag.querySelector('.hook-entry');
      if (entry) {
        setField(entry, 'hook_event', hook.event || 'PostToolUse');
        setField(entry, 'hook_matcher', hook.matcher || '*');
        setField(entry, 'hook_command', hook.command || '');
        const ac = entry.querySelector('[name="hook_async"]');
        if (ac) ac.checked = !!hook.async;
      }
      list.append(frag);
    }
  }
};

export const renderStep = (state, root) => {
  const contentEl = document.getElementById('wizard-step-content');
  if (contentEl) {
    contentEl.textContent = '';
    if (state.step === 7) {
      contentEl.append(getTemplate('tpl-step-7'));
      renderReview(state);
    } else if (state.step === 5) {
      contentEl.append(getTemplate('tpl-step-5'));
      renderHooks(state);
    } else {
      contentEl.append(getTemplate('tpl-step-' + state.step));
      restoreStepState(state, root);
    }
    renderStepIndicator(state);
    renderNav(state);
    attachFilterHandlers(contentEl);
  }
};

export { TOTAL_STEPS };
