import { initEditForm } from '../services/list-page.js';

const init = () => {
  initEditForm('skill-form', {
    buildBody: (form, formData) => {
      const tagsRaw = formData.get('tags') || '';
      const tags = tagsRaw.split(',').map(t => t.trim()).filter(Boolean);
      return {
        name: formData.get('name'),
        description: formData.get('description') || '',
        content: formData.get('content') || '',
        tags,
      };
    },
  });
};

init();
