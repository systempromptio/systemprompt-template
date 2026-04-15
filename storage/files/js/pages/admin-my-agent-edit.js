import { initEditForm } from '../services/list-page.js';

const init = () => {
  initEditForm('my-agent-form', {
    buildBody: (form, formData) => ({
      agent_id: formData.get('agent_id'),
      name: formData.get('name'),
      description: formData.get('description') || '',
      system_prompt: formData.get('system_prompt') || '',
    }),
  });
};

init();
