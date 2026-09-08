// The single place the suite names a class from the admin design system.
//
// Page objects and specs never hard-code a class: they go through this map, so
// a rename in the design system is one edit here rather than a sweep through
// every spec. These are agent A3's shipped names, not guesses — a template
// that uses a class the CSS does not define now fails a gate, so a wrong name
// here shows up as a selector matching nothing.
//
// Where the design system expresses state through ARIA rather than a class,
// this map follows it: sorting is th[aria-sort], the current breadcrumb and the
// active nav item are [aria-current="page"], the active tab is
// aria-selected="true". Those are the accessible truth and they cannot drift
// from what a screen reader is told.
export const SEL = {
  // The sidebar's nav element carries data-nav; the active link is marked with
  // aria-current, emitted by the navActive helper alongside .is-active.
  sidebar: '.sp-admin-sidebar',
  nav: 'nav[data-nav]',
  navLink: 'nav[data-nav] a',
  navLinkActive: 'nav[data-nav] a[aria-current="page"]',
  // The class form of the same state. The helper emits both and sidebar.js
  // sets and removes both together, so they cannot disagree; the ARIA form is
  // preferred above because it is what a screen reader is told.
  navLinkActiveClass: '.sp-admin-sidebar nav a.is-active',
  navSectionLabel: '.sp-nav-label',

  pageHeader: '.sp-page-header',
  heading: '.sp-page-header__title',
  subtitle: '.sp-page-header__subtitle',
  headerActions: '.sp-page-header__actions',
  breadcrumb: '.sp-breadcrumbs',
  breadcrumbCurrent: '.sp-breadcrumbs [aria-current="page"]',

  toolbar: '.sp-toolbar',
  toolbarSearch: '.sp-toolbar__search',
  toolbarSearchInput: '.sp-toolbar__search input',
  toolbarFilter: '.sp-toolbar__filter',
  toolbarCount: '.sp-toolbar__count',
  toolbarActions: '.sp-toolbar__actions',

  // .sp-table is the card. The element is .sp-table__el inside it, so a row
  // selector that starts at the card must name the element or it will also
  // match rows of a nested table.
  table: '.sp-table',
  // The scroll region paints its edge fades only while the table actually
  // overflows, so asserting on its background is a scroll-state assertion and
  // not a static one.
  tableScroll: '.sp-table__scroll',
  tableEl: '.sp-table__el',
  tableRow: '.sp-table__el tbody tr',
  tableEmptyRow: 'tr.sp-table__empty',
  tableHeaderCell: '.sp-table__el thead th',
  tableSorted: 'th[aria-sort]',
  // Timestamp cells are nowrap and tabular, so a date column is addressable by
  // class instead of by position.
  tableCellDate: '.sp-table__cell--date',
  tableCellNum: '.sp-table__cell--num',
  tableCellMono: '.sp-table__cell--mono',
  tableCellActions: '.sp-table__cell--actions',

  pagination: '.sp-pagination',
  paginationInfo: '.sp-pagination__info',
  paginationControls: '.sp-pagination__controls',
  paginationCurrent: '.sp-pagination__current',
  paginationSize: 'select.sp-pagination__size',

  kpi: '.sp-kpi',
  kpiGrid: '.sp-kpi-grid',
  kpiLabel: '.sp-kpi__label',
  kpiValue: '.sp-kpi__value',
  kpiUnit: '.sp-kpi__unit',
  kpiSub: '.sp-kpi__sub',

  badge: '.sp-badge',
  empty: '.sp-empty',
  emptyTitle: '.sp-empty__title',
  notice: '.sp-notice',
  skeleton: '.sp-skeleton',

  tabs: '.sp-tabs',
  tab: '.sp-tab',
  tabActive: '.sp-tab--active',

  // Toast and confirm are web components. The HOST element is what the page
  // writes, but it is zero-size — the overlay lives in its shadow root — so a
  // visibility assertion on the host reads hidden even while the dialog is on
  // screen. Assert and click through the shadow content instead; Playwright
  // pierces shadow roots for both CSS and roles.
  toast: 'sp-toast',
  dialogHost: 'sp-confirm-dialog',
  dialog: 'sp-confirm-dialog [role="dialog"]',
  dialogConfirm: 'sp-confirm-dialog [data-role="confirm"]',
  dialogCancel: 'sp-confirm-dialog [data-role="cancel"]',

  scopeForm: '[data-scope-selector]',
  rangeLink: '[data-scope-range]',
} as const;

/** The one custom-property namespace. There is no second one. */
export const TOKEN_PREFIX = '--sp-';

// Tokens declared on :root and stable enough to assert by name. --sp-density-row
// is the compact-by-default row height: read it rather than measuring a row, so
// a page agent restyling a row cannot fail a density guard for the wrong reason.
export const TOKENS = {
  accent: '--sp-accent',
  surface: '--sp-bg-surface',
  textPrimary: '--sp-text-primary',
  densityRow: '--sp-density-row',
} as const;
