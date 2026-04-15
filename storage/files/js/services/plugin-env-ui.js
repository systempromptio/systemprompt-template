export const mergeDefsWithValues = (defs, stored) => {
  const merged = [];
  const storedMap = {};
  for (const v of stored) storedMap[v.var_name] = v;
  for (const def of defs) {
    const existing = storedMap[def.name];
    merged.push({
      name: def.name, description: def.description || '',
      required: def.required !== false, secret: def.secret || false,
      example: def.example || '', value: existing ? existing.var_value : '',
      fromDef: true,
    });
    delete storedMap[def.name];
  }
  for (const key of Object.keys(storedMap)) {
    const s = storedMap[key];
    merged.push({
      name: s.var_name, description: '', required: false,
      secret: s.is_secret, example: '', value: s.var_value, fromDef: false,
    });
  }
  return merged;
};

export const buildVarListEl = (vars) => {
  const container = document.createElement('div');
  if (!vars.length) {
    const empty = document.createElement('div');
    empty.className = 'empty-state env-empty-state';
    const p = document.createElement('p');
    p.textContent = 'No environment variables defined for this plugin.';
    empty.append(p);
    container.append(empty);
    return container;
  }
  for (let i = 0; i < vars.length; i++) {
    const v = vars[i];
    const group = document.createElement('div');
    group.className = 'form-group';
    const label = document.createElement('label');
    label.textContent = v.name;
    if (v.required) {
      const reqBadge = document.createElement('span');
      reqBadge.className = 'badge badge-red';
      reqBadge.textContent = 'required';
      label.append(document.createTextNode(' '));
      label.append(reqBadge);
    }
    if (v.secret) {
      const secBadge = document.createElement('span');
      secBadge.className = 'badge badge-gray';
      secBadge.textContent = 'secret';
      label.append(document.createTextNode(' '));
      label.append(secBadge);
    }
    group.append(label);
    if (v.description) {
      const desc = document.createElement('p');
      desc.className = 'env-field-desc';
      desc.textContent = v.description;
      group.append(desc);
    }
    const input = document.createElement('input');
    input.type = v.secret ? 'password' : 'text';
    input.className = 'plugin-env-input';
    input.setAttribute('data-var-index', String(i));
    input.setAttribute('data-var-name', v.name);
    input.setAttribute('data-is-secret', v.secret ? '1' : '0');
    input.value = v.value;
    input.placeholder = v.example || '';
    group.append(input);
    container.append(group);
  }
  return container;
};
