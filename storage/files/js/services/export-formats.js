// Which formats a dataset kind allows, and keeping the pick inside that set.
// A table leaves in any format; one conversation's record as JSON or
// Markdown; a set of records only as JSON Lines, one conversation per line.

const FORMATS = {
  table: ['csv', 'json', 'jsonl', 'markdown'],
  document: ['json', 'markdown'],
  transcripts: ['jsonl'],
};

export const formatInputs = (dialog) => [...dialog.querySelectorAll('input[name="format"]')];

// A format the new kind does not allow is replaced by that kind's first,
// so the dialog never sits on a pick the server would refuse.
export const showFormats = (dialog, kind, setHidden) => {
  const allowed = FORMATS[kind];
  const inputs = formatInputs(dialog);
  for (const input of inputs) setHidden(input.closest('label'), !allowed.includes(input.value));
  const checked = inputs.find((input) => input.checked);
  if (!allowed.includes(checked?.value)) {
    const first = inputs.find((input) => input.value === allowed[0]);
    if (first) first.checked = true;
  }
};
