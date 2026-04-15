export const buildEnvItem = (def, stored) => {
  const hasValue = stored?.var_value && stored.var_value !== '';
  const item = document.createElement('div');
  item.className = 'detail-item';
  const info = document.createElement('div');
  info.className = 'detail-item-info';
  const nameRow = document.createElement('div');
  nameRow.className = 'detail-item-name';
  const code = document.createElement('code');
  code.className = 'env-code-label';
  code.textContent = def.name;
  nameRow.append(code);
  nameRow.append(document.createTextNode(' '));
  const valBadge = document.createElement('span');
  valBadge.className = hasValue ? 'badge badge-green' : 'badge badge-red';
  valBadge.textContent = hasValue ? 'configured' : 'not set';
  nameRow.append(valBadge);
  if (def.required !== false && !hasValue) {
    const reqBadge = document.createElement('span');
    reqBadge.className = 'badge badge-yellow';
    reqBadge.textContent = 'required';
    nameRow.append(document.createTextNode(' '));
    nameRow.append(reqBadge);
  }
  if (def.secret) {
    const secBadge = document.createElement('span');
    secBadge.className = 'badge badge-gray';
    secBadge.textContent = 'secret';
    nameRow.append(document.createTextNode(' '));
    nameRow.append(secBadge);
  }
  info.append(nameRow);
  const desc = document.createElement('div');
  desc.className = 'detail-item-desc env-item-desc';
  if (def.description) desc.textContent = def.description;
  if (hasValue) {
    const masked = stored.is_secret ? '--------' : stored.var_value;
    const valSpan = document.createElement('span');
    valSpan.className = 'env-val-mono';
    valSpan.textContent = masked;
    desc.append(document.createTextNode(' '));
    desc.append(valSpan);
  }
  info.append(desc);
  item.append(info);
  return item;
};

export const buildOverviewRow = (label, valueEl) => {
  const lbl = document.createElement('span');
  lbl.className = 'config-overview-label';
  lbl.textContent = label;
  const val = document.createElement('span');
  val.className = 'config-overview-value';
  val.append(valueEl);
  return [lbl, val];
};
