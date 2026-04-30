/**
 * Tab switching state and DOM coordination.
 *
 * Responsible for: tracking which tab is active, showing/hiding the correct
 * page panels, updating the top nav bar button styling (is-active class),
 * and dispatching a 'tabchange' CustomEvent so other modules can react.
 *
 * Page panels are: #page-words, #page-sentences, #page-deliver.
 * Each panel uses `.page` / `.page.is-active` (display:none → display:block).
 *
 * Nav buttons are `[data-tab]` buttons in NavBar.astro, identified by id:
 *   #nav-words, #nav-sentences, #nav-deliver.
 *
 * No dependencies on other project modules.
 */
// Responsible for: active tab state, DOM panel switching, and tabchange event dispatch

/** @type {'words'|'sentences'|'deliver'} Currently active tab name. */
let activeTab = 'words';

/**
 * Returns the currently active tab name.
 *
 * @returns {'words'|'sentences'|'deliver'}
 */
export function getActiveTab() { return activeTab; }

/**
 * Wires click handlers on all [data-tab] buttons and activates the Words tab.
 *
 * Must be called last during init so the initial 'tabchange' event fires after
 * all other module listeners are already registered.
 *
 * @returns {void}
 */
export function initTabs() {
  document.querySelectorAll('[data-tab]').forEach(btn => {
    btn.addEventListener('click', () => switchTab(btn.dataset.tab));
  });
  switchTab('words');
}

/**
 * Updates nav bar [data-tab] button styling to reflect which tab is now active.
 * Adds `is-active` class to the active button and removes it from others.
 * Also sets aria-current attribute for accessibility.
 *
 * @param {'words'|'sentences'|'deliver'} name - The newly active tab name.
 * @returns {void}
 */
function updateNavButtons(name) {
  document.querySelectorAll('[data-tab]').forEach(btn => {
    const isActive = btn.dataset.tab === name;
    btn.classList.toggle('is-active', isActive);
    btn.setAttribute('aria-current', isActive ? 'page' : 'false');
  });
}

/**
 * Activates a tab by name: shows its page panel, hides the others,
 * updates the nav bar button styling, and dispatches a 'tabchange' event.
 *
 * Page panel IDs: #page-words, #page-sentences, #page-deliver.
 *
 * @param {'words'|'sentences'|'deliver'} name - The tab to activate.
 * @returns {void}
 */
export function switchTab(name) {
  activeTab = name;

  const PAGES = ['words', 'sentences', 'deliver'];
  PAGES.forEach(p => {
    const pageEl = document.getElementById(`page-${p}`);
    if (pageEl) pageEl.classList.toggle('is-active', p === name);
  });

  updateNavButtons(name);

  // Sync search input placeholder to current page context
  const placeholder = name === 'sentences' ? 'Search sentences…' : 'Search मैं, maĩ, I…';
  document.querySelectorAll('[data-search-input]').forEach(inp => {
    inp.placeholder = placeholder;
  });

  window.dispatchEvent(new CustomEvent('tabchange', { detail: { tab: name } }));
}
