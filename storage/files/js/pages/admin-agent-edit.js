import { initEditForm } from '../services/list-page.js';

const init = () => {
  initEditForm('agent-form', {
    buildBody: (form, formData) => ({
      name: formData.get('name'),
      description: formData.get('description') || '',
      system_prompt: formData.get('system_prompt') || '',
      enabled: form.querySelector('[name="enabled"]').checked,
    }),
  });
};

init();
