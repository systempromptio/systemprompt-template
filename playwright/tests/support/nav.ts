// The sidebar's information architecture: five sections, in render order, each
// with its items as they are labelled and where they go. Mirrors
// storage/files/admin/partials/sidebar.hbs.
//
// A page spec asserts its own nav item is the active one; this map is what lets
// it do that by label rather than by a hard-coded href, and what lets a shell
// spec prove the whole structure in one pass.
//
// Active state is carried twice and in lockstep: the handlebars helper emits
// class="is-active" aria-current="page" from the page id. SEL.navLinkActive
// uses the ARIA form because it is also what a screen reader is told.
import { PATHS, UNMOUNTED_PATHS } from './paths';

export interface NavItem {
  label: string;
  /** The key naming this route in PATHS, or in UNMOUNTED_PATHS until it lands. */
  key: string;
  path: string;
  /** Its page has a route today; false while its author has not landed it. */
  mounted: boolean;
}

export interface NavSection {
  heading: string;
  items: NavItem[];
  /** Rendered for every signed-in person, not only for the admin. */
  everyone?: boolean;
}

type RouteTable = Record<string, unknown>;

// Why look the route up rather than name the table: an item is promoted by its
// author moving the key from UNMOUNTED_PATHS to PATHS, and asking where the
// key lives makes promotion a no-op here and keeps `mounted` true by
// construction.
function item(label: string, key: string): NavItem {
  const mounted = (PATHS as RouteTable)[key];
  if (typeof mounted === 'string') return { label, key, path: mounted, mounted: true };
  const pending = (UNMOUNTED_PATHS as RouteTable)[key];
  if (typeof pending === 'string') return { label, key, path: pending, mounted: false };
  throw new Error(
    `nav item "${label}" names route "${key}", which is in neither PATHS nor ` +
      `UNMOUNTED_PATHS. Add it to one, or fix the key.`,
  );
}

export const NAV: NavSection[] = [
  {
    heading: 'People & access',
    items: [
      item('Users', 'users'),
      item('Departments', 'departments'),
      item('Access tokens', 'accessTokens'),
      item('Access control', 'accessControl'),
    ],
  },
  {
    heading: 'AI activity',
    items: [
      item('Requests', 'requests'),
      item('Sessions', 'sessions'),
      item('Traces', 'traces'),
      item('Contexts', 'contexts'),
      item('Evals', 'evals'),
    ],
  },
  {
    heading: 'Governance',
    items: [
      item('Policies', 'governance'),
      item('Decisions', 'governanceDecisions'),
      item('Hooks', 'governanceHooks'),
      item('Trace demo', 'demoTrace'),
    ],
  },
  {
    heading: 'Platform',
    items: [item('Models', 'models')],
  },
  {
    heading: 'Account',
    everyone: true,
    items: [item('Profile', 'profile'), item('Settings', 'settings')],
  },
];

/** Every nav item whose page is routed today. */
export const MOUNTED_NAV_ITEMS: NavItem[] = NAV.flatMap((s) => s.items).filter((i) => i.mounted);

/** The items a non-admin sees: the Account section only. */
export const EVERYONE_NAV_ITEMS: NavItem[] = NAV.filter((s) => s.everyone).flatMap((s) => s.items);
