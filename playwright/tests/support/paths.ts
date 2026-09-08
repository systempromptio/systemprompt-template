// The admin route map, in one place.
//
// Specs and page objects never write a literal admin URL: they name a route
// here, so a move is one edit rather than a sweep. The table mirrors
// extensions/web/admin/src/routes/ssr.rs — flat `/admin/<x>[/{id}]` for every
// entity, with the user detail page still on its query form.
export const PATHS = {
  // /admin serves the overview dashboard.
  root: '/admin',

  users: '/admin/users',
  user: (id: string) => `/admin/users/${encodeURIComponent(id)}`,
  departments: '/admin/departments',
  department: (id: string) => `/admin/departments/${id}`,
  accessTokens: '/admin/access-tokens',
  accessControl: '/admin/access-control',

  requests: '/admin/requests',
  request: (id: string) => `/admin/requests/${id}`,
  sessions: '/admin/sessions',
  session: (id: string) => `/admin/sessions/${id}`,
  traces: '/admin/traces',
  trace: (id: string) => `/admin/traces/${id}`,
  contexts: '/admin/contexts',
  context: (id: string) => `/admin/contexts/${id}`,
  evals: '/admin/evals',
  evalRun: (id: string) => `/admin/evals/runs/${id}`,

  governance: '/admin/governance',
  governancePolicy: (id: string) => `/admin/governance/policies/${id}`,
  governanceDecisions: '/admin/governance/decisions',
  governanceHooks: '/admin/governance/hooks',
  demoTrace: '/admin/demo/trace',

  models: '/admin/models',

  settings: '/admin/settings',
  setup: '/admin/setup',
  profile: '/admin/profile',
  login: '/admin/login',
} as const;

// In the sidebar but not mounted yet. Empty on this instance: every sidebar
// item has a route. Kept so nav.ts can promote an item without an edit here.
export const UNMOUNTED_PATHS = {} as const;
