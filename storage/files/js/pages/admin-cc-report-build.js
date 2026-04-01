export const buildReportHtml = () => {
  const report = document.querySelector('#tab-report .cc-report');
  if (report) {
    const clone = report.cloneNode(true);
    clone.querySelector('.cc-report-actions')?.remove();
    clone.querySelector('.cc-report-countdown')?.remove();
    const dateEl = clone.querySelector('.cc-report-date');
    const date = dateEl?.textContent?.trim() || new Date().toISOString().slice(0, 10);

    return `<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Daily Report \u2014 ${date} \u2014 systemprompt.io</title>
<style>
:root{--sp-brand:#6366f1;--sp-brand-light:#818cf8;--sp-brand-wash:#eef2ff;--sp-brand-dark:#4338ca;--sp-success:#16a34a;--sp-danger:#dc2626;--sp-neutral:#6b7280}
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,"Helvetica Neue",sans-serif;color:#1a1a2e;background:#fff;line-height:1.5}
.sp-report-brand{background:linear-gradient(135deg,#1e1b4b 0%,#312e81 50%,#4338ca 100%);padding:1.5rem 2rem;color:#fff}
.sp-report-brand-inner{max-width:900px;margin:0 auto;display:flex;align-items:center;justify-content:space-between}
.sp-report-brand-logo{display:flex;align-items:center;gap:0.75rem}
.sp-report-brand-mark{width:32px;height:32px;background:var(--sp-brand-light);border-radius:8px;display:flex;align-items:center;justify-content:center;font-weight:800;font-size:1rem;color:#fff}
.sp-report-brand-name{font-size:1rem;font-weight:700;letter-spacing:-0.02em}
.sp-report-brand-name span{opacity:0.6;font-weight:400}
.sp-report-brand-meta{text-align:right;font-size:0.75rem;opacity:0.7}
.sp-report-brand-meta strong{display:block;font-size:0.875rem;opacity:1;font-weight:600}
.sp-report-content{max-width:900px;margin:0 auto;padding:2rem}
h2{font-size:1.25rem;font-weight:700;color:#1e1b4b;margin-bottom:0.25rem}
h3{font-size:0.875rem;font-weight:600;text-transform:uppercase;letter-spacing:0.04em;color:var(--sp-brand-dark);margin-bottom:0.75rem;padding-left:0.75rem;border-left:3px solid var(--sp-brand)}
table{width:100%;border-collapse:collapse;font-size:0.875rem;margin-bottom:1.5rem;font-variant-numeric:tabular-nums}
thead{border-bottom:2px solid #e5e7eb}th{font-size:0.75rem;font-weight:600;color:var(--sp-neutral);text-transform:uppercase;letter-spacing:0.04em;padding:0.5rem 0.75rem;white-space:nowrap}
td{padding:0.5rem 0.75rem}tbody tr{border-bottom:1px solid #f3f4f6}tbody tr:nth-child(even){background:#fafaff}
.cc-report-header{display:flex;justify-content:space-between;align-items:baseline;padding-bottom:1rem;margin-bottom:1.25rem;border-bottom:2px solid var(--sp-brand-wash)}
.cc-report-title-group{display:flex;align-items:baseline;gap:0.75rem}
.cc-report-subtitle{font-size:0.75rem;color:#9ca3af;text-transform:uppercase;letter-spacing:0.04em}
.cc-report-header-right{display:flex;flex-direction:column;align-items:flex-end;gap:0.25rem}
.cc-report-date{font-size:0.875rem;font-weight:600;color:var(--sp-brand-dark)}
.cc-report-section{margin-bottom:1.5rem}.cc-report-th-metric{text-align:left;width:30%}
.cc-report-th-value,.cc-report-th-delta,.cc-report-th-global{text-align:right;width:12%}
.cc-report-th-global{color:var(--sp-brand)}.cc-report-metric{text-align:left;font-weight:500}
.cc-report-value{text-align:right;font-weight:700;font-family:ui-monospace,"Cascadia Code",monospace;font-size:0.75rem}
.cc-report-delta{text-align:right;font-size:0.75rem}
.cc-report-delta--positive{color:var(--sp-success)}.cc-report-delta--negative{color:var(--sp-danger)}.cc-report-delta--neutral{color:var(--sp-neutral)}
.cc-report-arrow{font-size:0.75em}
.cc-report-entity-grid{display:grid;grid-template-columns:repeat(5,1fr);gap:0.75rem;margin-bottom:1.25rem}
.cc-report-entity-card{display:flex;flex-direction:column;align-items:center;gap:0.25rem;background:var(--sp-brand-wash);border:1px solid #c7d2fe;border-radius:0.5rem;padding:0.75rem}
.cc-report-entity-value{font-size:1.125rem;font-weight:700;font-family:ui-monospace,monospace;color:var(--sp-brand-dark)}
.cc-report-entity-label{font-size:0.75rem;font-weight:600;color:var(--sp-neutral);text-transform:uppercase;letter-spacing:0.04em}
.cc-report-insights-grid{display:grid;grid-template-columns:repeat(2,1fr);gap:0.75rem}
.cc-report-insight{background:#f9fafb;border:1px solid #e5e7eb;border-left:3px solid var(--sp-brand-light);border-radius:0.5rem;padding:0.75rem 1rem}
.cc-report-insight--highlight{border-left-color:var(--sp-brand);background:var(--sp-brand-wash)}
.cc-report-insight-label{display:block;font-size:0.75rem;font-weight:600;color:var(--sp-brand);text-transform:uppercase;letter-spacing:0.05em;margin-bottom:0.25rem}
.cc-report-insight-text{font-size:0.875rem;color:#374151;margin:0;line-height:1.5}
.cc-quality-gauge,.cc-goal-badge,.cc-tag-badge{font-size:0.75rem}
.cc-tag-badges{display:flex;gap:0.25rem;flex-wrap:wrap}
.cc-tag-badge{background:var(--sp-brand-wash);color:var(--sp-brand-dark);border-radius:0.25rem;padding:0.1rem 0.4rem;font-size:0.7rem;border:1px solid #c7d2fe}
.cc-report-history-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:1rem}
.cc-report-history-item{display:flex;flex-direction:column;align-items:center;gap:0.5rem;background:#f9fafb;border:1px solid #e5e7eb;border-radius:0.5rem;padding:0.75rem}
.cc-report-history-label{font-size:0.75rem;font-weight:600;color:var(--sp-neutral);text-transform:uppercase}
.cc-report-sparkline polyline{fill:none;stroke:var(--sp-brand);stroke-width:1.5}.cc-report-sparkline circle{fill:var(--sp-brand)}
.cc-report-streak{font-size:0.75rem;font-weight:600;color:var(--sp-brand)}
.cc-report-session-title{font-weight:500;word-break:break-word}
.cc-report-session-outcome{font-size:0.75rem;color:#555;max-width:300px}
.cc-report-sessions-table th:first-child,.cc-report-sessions-table td:first-child{min-width:180px}
.cc-goal-badge--yes,.cc-goal-badge--true{color:var(--sp-success)}.cc-goal-badge--no,.cc-goal-badge--false{color:var(--sp-danger)}
.cc-quality-gauge--high{color:var(--sp-success)}.cc-quality-gauge--medium{color:#d97706}.cc-quality-gauge--low{color:var(--sp-danger)}
.sp-report-footer{max-width:900px;margin:2rem auto 0;padding:1rem 2rem;border-top:2px solid var(--sp-brand-wash);display:flex;justify-content:space-between;align-items:center;font-size:0.75rem;color:var(--sp-neutral)}
.sp-report-footer-brand{color:var(--sp-brand);font-weight:600}
@media print{body{padding:0}.sp-report-brand{-webkit-print-color-adjust:exact;print-color-adjust:exact}.cc-report-entity-card,.cc-report-insight,.cc-report-insight--highlight,.cc-tag-badge{-webkit-print-color-adjust:exact;print-color-adjust:exact}.cc-report-section{break-inside:avoid}.cc-report-insight{break-inside:avoid}}
</style></head><body>
<div class="sp-report-brand"><div class="sp-report-brand-inner">
<div class="sp-report-brand-logo"><div class="sp-report-brand-mark">sp</div><div class="sp-report-brand-name">systemprompt<span>.io</span></div></div>
<div class="sp-report-brand-meta"><strong>Control Center Report</strong>${date}</div>
</div></div>
<div class="sp-report-content">${clone.innerHTML}</div>
<div class="sp-report-footer"><span class="sp-report-footer-brand">systemprompt.io</span>
<span>Generated from Control Center \u2014 ${new Date().toLocaleString()}</span></div>
</body></html>`;
  }
  return '';
};

export const buildReportText = () => {
  const report = document.querySelector('#tab-report .cc-report');
  if (report) {
    const clone = report.cloneNode(true);
    clone.querySelector('.cc-report-actions')?.remove();
    clone.querySelector('.cc-report-countdown')?.remove();
    const lines = [];
    const dateEl = clone.querySelector('.cc-report-date');
    lines.push('DAILY REPORT \u2014 ' + (dateEl?.textContent?.trim() || ''));
    lines.push('Performance fact sheet \u2014 systemprompt.io');
    lines.push('');
    for (const section of clone.querySelectorAll('.cc-report-section')) {
      const title = section.querySelector('.cc-report-section-title');
      if (title) { lines.push(title.textContent.trim().toUpperCase()); lines.push('\u2500'.repeat(40)); }
      const table = section.querySelector('table');
      if (table) {
        for (const row of table.querySelectorAll('tbody tr')) {
          const cells = [...row.querySelectorAll('td')].map(td => td.textContent.trim());
          lines.push(cells.join('  |  '));
        }
        lines.push('');
      }
      for (const insight of section.querySelectorAll('.cc-report-insight')) {
        const label = insight.querySelector('.cc-report-insight-label')?.textContent?.trim();
        const text = insight.querySelector('.cc-report-insight-text')?.textContent?.trim();
        if (label && text) lines.push(`[${label}] ${text}`);
      }
    }
    return lines.join('\n');
  }
  return '';
};
