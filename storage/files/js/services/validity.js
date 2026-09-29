// The one translation every "Expires" date picker on the console needs: a
// `<input type="date">` value is a calendar day, the API wants an RFC 3339
// instant. A blank picker means open-ended, sent as null so a stale window
// on the row is cleared rather than kept.

export const validUntilFrom = (input) => {
  const day = input?.value?.trim();
  if (!day) return null;
  // Why: the end of the chosen day, in the browser's zone, so "expires on the
  // 30th" holds for the whole of the 30th rather than lapsing the night before.
  const at = new Date(`${day}T23:59:59`);
  return Number.isNaN(at.getTime()) ? null : at.toISOString();
};

// Why: the earliest and latest days a picker may offer, as the `min`/`max`
// attributes want them (local calendar days).
export const localDay = (date) => {
  const pad = (n) => String(n).padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
};

export const dayAfter = (days) => {
  const at = new Date();
  at.setDate(at.getDate() + days);
  return localDay(at);
};
