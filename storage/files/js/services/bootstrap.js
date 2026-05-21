import { initDelegation, setCloseMenus } from './events.js';
import { initDropdown, closeAllMenus } from './dropdown.js';
import { initSidebar } from './sidebar.js';
import { initTheme } from './theme.js';
import { initHeaderActions } from './header-actions.js';
import { initHeaderSearch } from './header-search.js';
import { initLogout, initUserDisplay, getUserContext } from './auth.js';

setCloseMenus(closeAllMenus);
initDelegation();
initDropdown();
initSidebar();
initTheme();
initHeaderActions();
initHeaderSearch();
initLogout();
initUserDisplay();
getUserContext();
