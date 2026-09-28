// What the dialog says a download will hold before the reader asks for it:
// the row and cell count fetched from `/preview`, or the sentence a record
// export shows instead, since a whole conversation has nothing to count.

const strong = (value) => {
  const el = document.createElement('strong');
  el.textContent = value.toLocaleString();
  return el;
};

export const renderPreview = (el, preview, kind, columns) => {
  el.replaceChildren();
  if (kind === 'transcripts') {
    el.append(strong(preview.rows), preview.rows === 1 ? ' conversation, in full' : ' conversations, in full');
  } else {
    el.append(strong(preview.rows), ` rows × ${columns} columns = `, strong(preview.rows * columns), ' cells');
  }
  if (preview.capped) {
    el.dataset.tone = 'warn';
    el.append(` — capped at ${preview.cap.toLocaleString()}; narrow the window or the filters for the rest.`);
  } else {
    delete el.dataset.tone;
  }
};

// A record export downloads straight from the dataset's own href; only the
// format decides which one.
export const documentHref = (dialog, datasetOption) => {
  const option = datasetOption(dialog);
  const format = new FormData(dialog.querySelector('[data-export-form]')).get('format');
  return format === 'markdown' ? option.dataset.hrefMarkdown : option.dataset.hrefJson;
};
