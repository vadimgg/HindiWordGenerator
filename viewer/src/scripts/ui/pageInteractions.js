/**
 * Page interaction handler — group collapse, selection circles, group checkboxes, drag-to-select.
 *
 * Responsible for: all user interactions with the Words and Sentences pages:
 *   - Filter panel toggle (open/close) for both words and sentences pages.
 *   - Card group collapse: clicking a group header toggles .is-collapsed on its wrapper,
 *     which CSS uses to hide the sibling .card-list.
 *   - Selection circles: clicking .sel-circle toggles .is-selected on the circle and the
 *     parent article; syncs with selection state.
 *   - Group checkbox: clicking .card-group-checkbox selects/deselects all cards in the group.
 *   - View mode toggle: Web / Anki segmented buttons on words and sentences pages.
 *   - Drag-to-select lasso: mousedown/mousemove/mouseup on the page draws a lasso rectangle
 *     and selects cards that intersect it on mouseup.
 *
 * Filter re-application after selection changes is handled by the 'selectionchange'
 * CustomEvent that selection.js dispatches — no direct call to search.js needed.
 *
 * Dependencies: state/selection.js.
 */
// Responsible for: page-level interactions — filter panel, group collapse, sel-circle, drag-select

import { setWordSelected, setSentenceSelected, hasWord, hasSentence } from '../state/selection.js';

// ── Filter panel toggle ───────────────────────────────────────────────────────

/**
 * Wires the open/close toggle for a filter panel button + panel pair.
 *
 * @param {string} btnId   - Element id of the toggle button.
 * @param {string} panelId - Element id of the collapsible panel.
 * @returns {void}
 */
function wireFilterPanel(btnId, panelId) {
  const btn   = document.getElementById(btnId);
  const panel = document.getElementById(panelId);
  if (!btn || !panel) return;

  let open = false;

  btn.addEventListener('click', () => {
    open = !open;
    panel.classList.toggle('is-hidden', !open);
    btn.classList.toggle('is-active', open);
    btn.setAttribute('aria-expanded', String(open));
  });
}

// ── View mode toggle (Web / Anki) ─────────────────────────────────────────────

/**
 * Sets the view mode on a page (web or anki).
 * Toggles .mode-web / .mode-anki on the page root and updates segmented button states.
 *
 * @param {string} page - 'words' | 'sentences'
 * @param {string} mode - 'web' | 'anki'
 * @returns {void}
 */
function setViewMode(page, mode) {
  const pageEl = document.getElementById(`page-${page}`);
  if (!pageEl) return;
  pageEl.classList.toggle('mode-web',  mode === 'web');
  pageEl.classList.toggle('mode-anki', mode === 'anki');

  const prefix = page === 'words' ? 'pw' : 'ps';
  const webBtn  = document.getElementById(`${prefix}-mode-web`);
  const ankiBtn = document.getElementById(`${prefix}-mode-anki`);
  if (webBtn)  { webBtn.classList.toggle('is-active',  mode === 'web');  webBtn.setAttribute('aria-pressed', String(mode === 'web')); }
  if (ankiBtn) { ankiBtn.classList.toggle('is-active', mode === 'anki'); ankiBtn.setAttribute('aria-pressed', String(mode === 'anki')); }
}

/**
 * Wires the Web/Anki view mode buttons for a given page prefix.
 *
 * @param {string} page   - 'words' | 'sentences'
 * @param {string} prefix - 'pw' | 'ps'
 * @returns {void}
 */
function wireViewMode(page, prefix) {
  document.getElementById(`${prefix}-mode-web`)?.addEventListener('click', () => setViewMode(page, 'web'));
  document.getElementById(`${prefix}-mode-anki`)?.addEventListener('click', () => setViewMode(page, 'anki'));
}

// ── Card group collapse ───────────────────────────────────────────────────────

/**
 * Toggles the collapsed state of a card group.
 * Clicking anywhere in the group header except the checkbox triggers this.
 *
 * @param {Element} headerEl - The .card-group-header element.
 * @returns {void}
 */
function toggleGroupCollapse(headerEl) {
  const wrapperId = headerEl.dataset.groupToggle;
  if (!wrapperId) return;
  document.getElementById(wrapperId)?.classList.toggle('is-collapsed');
}

// ── Selection circle ──────────────────────────────────────────────────────────

/**
 * Toggles selection state on a word or sentence card.
 * Updates the .sel-circle button and the parent article, and syncs with state/selection.js.
 *
 * @param {Element} circle - The .sel-circle button element.
 * @returns {void}
 */
function toggleSelCircle(circle) {
  const cardIndex = parseInt(circle.dataset.selCircle ?? '-1');
  if (isNaN(cardIndex) || cardIndex < 0) return;

  // Determine if this is a sentence card or word card
  const article = circle.closest('article');
  if (!article) return;
  const isSentence = article.hasAttribute('data-sentence-card') ||
                     article.closest('[id="page-sentences"]') !== null;

  const nowSelected = !circle.classList.contains('is-selected');
  circle.classList.toggle('is-selected', nowSelected);
  article.classList.toggle('is-selected', nowSelected);

  if (isSentence) {
    setSentenceSelected(cardIndex, nowSelected);
  } else {
    setWordSelected(cardIndex, nowSelected);
  }
}

// ── Group checkbox ────────────────────────────────────────────────────────────

/**
 * Toggles all selection circles within a group's .card-list.
 * If any card is deselected → select all. If all selected → deselect all.
 *
 * @param {Element} checkboxEl - The .card-group-checkbox element.
 * @returns {void}
 */
function toggleGroupCheckbox(checkboxEl) {
  const wrapperId = checkboxEl.dataset.groupCheckbox;
  if (!wrapperId) return;

  // Find the sibling .card-list for this group
  const wrapper  = document.getElementById(wrapperId);
  const cardList = wrapper?.nextElementSibling;
  if (!cardList) return;

  const circles = [...cardList.querySelectorAll('.sel-circle')];
  const allOn   = circles.every(c => c.classList.contains('is-selected'));
  const goOn    = !allOn;

  checkboxEl.classList.toggle('is-checked', goOn);
  checkboxEl.classList.remove('is-indeterminate');
  checkboxEl.setAttribute('aria-checked', String(goOn));

  circles.forEach(circle => {
    const cardIndex = parseInt(circle.dataset.selCircle ?? '-1');
    if (isNaN(cardIndex) || cardIndex < 0) return;
    circle.classList.toggle('is-selected', goOn);
    circle.closest('article')?.classList.toggle('is-selected', goOn);

    // Determine sentence vs word by checking which page we're in
    const isInSentences = circle.closest('#page-sentences') !== null;
    if (isInSentences) {
      setSentenceSelected(cardIndex, goOn);
    } else {
      setWordSelected(cardIndex, goOn);
    }
  });
}

// ── Selection count badge ─────────────────────────────────────────────────────

/**
 * Updates the selection badge text and visibility.
 * Reads circles in the page to count selected items.
 *
 * @returns {void}
 */
function updateSelectionBadges() {
  const wordBadge = document.getElementById('pw-selection-badge');
  if (wordBadge) {
    const count = document.querySelectorAll('#page-words .sel-circle.is-selected').length;
    wordBadge.textContent = `${count} selected`;
    wordBadge.classList.toggle('is-hidden', count === 0);
  }
  const sentBadge = document.getElementById('ps-selection-badge');
  if (sentBadge) {
    const count = document.querySelectorAll('#page-sentences .sel-circle.is-selected').length;
    sentBadge.textContent = `${count} selected`;
    sentBadge.classList.toggle('is-hidden', count === 0);
  }
}

// ── Card collapse/expand (delegated from demo's card-header click) ────────────

/**
 * Handles clicks on a card header to toggle collapse state.
 * Does NOT fire when clicking selection or audio controls inside the header.
 *
 * @param {MouseEvent} e - The click event.
 * @returns {void}
 */
function handleCardHeaderClick(e) {
  const header = e.target.closest('.card-header');
  if (!header) return;
  if (e.target.closest('.sel-circle, .wc-audio-btn')) return;

  const card = header.closest('article');
  if (!card) return;
  const expanded = !card.classList.contains('is-collapsed');
  card.classList.toggle('is-collapsed', expanded);
  header.setAttribute('aria-expanded', String(!expanded));
}

// ── Drag-to-select lasso ──────────────────────────────────────────────────────

/**
 * Sets up the drag-to-select lasso on the document.
 * Only active when the Words or Sentences page is showing.
 *
 * @returns {void}
 */
function initDragSelect() {
  const lasso = document.createElement('div');
  lasso.id = 'drag-lasso';
  lasso.style.cssText = [
    'position:fixed',
    'z-index:100',
    'pointer-events:none',
    'display:none',
    'background:rgba(251,191,36,0.06)',
    'border:1.5px solid rgba(251,191,36,0.35)',
    'border-radius:4px',
  ].join(';');
  document.body.appendChild(lasso);

  const drag = { active: false, startX: 0, startY: 0, currentX: 0, currentY: 0 };

  function isDragEligiblePage() {
    return document.getElementById('page-words')?.classList.contains('is-active') ||
           document.getElementById('page-sentences')?.classList.contains('is-active');
  }

  function updateLasso() {
    const x1 = Math.min(drag.startX, drag.currentX);
    const y1 = Math.min(drag.startY, drag.currentY);
    lasso.style.left   = x1 + 'px';
    lasso.style.top    = y1 + 'px';
    lasso.style.width  = Math.abs(drag.currentX - drag.startX) + 'px';
    lasso.style.height = Math.abs(drag.currentY - drag.startY) + 'px';
  }

  function rectsIntersect(a, b) {
    return !(a.right < b.left || a.left > b.right || a.bottom < b.top || a.top > b.bottom);
  }

  function highlightLassoCards() {
    const lassoRect = lasso.getBoundingClientRect();
    document.querySelectorAll('.wc-card, .sc-card').forEach(card => {
      card.classList.toggle('drag-hover', rectsIntersect(lassoRect, card.getBoundingClientRect()));
    });
  }

  document.addEventListener('mousedown', e => {
    if (!isDragEligiblePage() || e.button !== 0) return;
    if (e.target.closest('.card-header, .sel-circle, .card-group-header, .pw-filter-panel, .ps-filter-panel, .page-filter-btn, .view-mode-toggle, .nav-bar')) return;

    drag.active   = true;
    drag.startX   = e.clientX;
    drag.startY   = e.clientY;
    drag.currentX = e.clientX;
    drag.currentY = e.clientY;
    lasso.style.display = 'block';
    updateLasso();
    e.preventDefault();
  });

  document.addEventListener('mousemove', e => {
    if (!drag.active) return;
    drag.currentX = e.clientX;
    drag.currentY = e.clientY;
    updateLasso();
    highlightLassoCards();
  });

  document.addEventListener('mouseup', () => {
    if (!drag.active) return;
    drag.active = false;

    document.querySelectorAll('.wc-card.drag-hover, .sc-card.drag-hover').forEach(card => {
      card.classList.remove('drag-hover');
      card.classList.add('is-selected');
      const circle = card.querySelector('.sel-circle');
      if (!circle) return;
      circle.classList.add('is-selected');
      const cardIndex = parseInt(circle.dataset.selCircle ?? '-1');
      if (isNaN(cardIndex) || cardIndex < 0) return;
      const isInSentences = card.closest('#page-sentences') !== null;
      if (isInSentences) setSentenceSelected(cardIndex, true);
      else setWordSelected(cardIndex, true);
    });

    lasso.style.display = 'none';
    updateSelectionBadges();
  });
}

// ── Delegated page event handler ──────────────────────────────────────────────

/**
 * Single delegated click handler for the entire document covering both pages.
 *
 * Dispatches to: sel-circle toggle, group-checkbox toggle, group-header collapse,
 * card-header collapse/expand.
 *
 * @param {MouseEvent} e - The click event.
 * @returns {void}
 */
function handlePageClick(e) {
  // Selection circle
  const circle = e.target.closest('.sel-circle');
  if (circle) {
    e.stopPropagation();
    toggleSelCircle(circle);
    updateSelectionBadges();
    return;
  }

  // Group checkbox (must come before group header)
  const groupCheckbox = e.target.closest('.card-group-checkbox');
  if (groupCheckbox) {
    e.stopPropagation();
    toggleGroupCheckbox(groupCheckbox);
    updateSelectionBadges();
    return;
  }

  // Group header (collapse/expand) — but not on the checkbox itself
  const groupHeader = e.target.closest('.card-group-header');
  if (groupHeader && !e.target.closest('.card-group-checkbox')) {
    toggleGroupCollapse(groupHeader);
    return;
  }

  // Card header (collapse/expand)
  handleCardHeaderClick(e);
}

// ── Filter chip filtering ─────────────────────────────────────────────────────

/**
 * Wires date-chip buttons in `#pw-filter-chips` to filter word card groups.
 *
 * Each chip carries `data-date-chip` (month-year label, e.g. "Jan 2026") and
 * `data-date-prefix` (YYYY-MM, e.g. "2026-01"). "all" shows every group; any
 * other chip shows only groups that contain at least one card whose
 * `data-word-date` attribute starts with the chip's YYYY-MM prefix.
 *
 * @returns {void}
 */
function wireWordFilterChips() {
  const container = document.getElementById('pw-filter-chips');
  if (!container) return;

  container.addEventListener('click', e => {
    const chip = e.target.closest('.pw-filter-chip');
    if (!chip) return;

    // Update active chip
    container.querySelectorAll('.pw-filter-chip').forEach(c => c.classList.remove('is-active'));
    chip.classList.add('is-active');

    const prefix = chip.dataset.datePrefix ?? '';  // YYYY-MM or '' for "all"
    const isAll  = chip.dataset.dateChip === 'all';

    // Show/hide card-group-wrappers + their card-list siblings
    document.querySelectorAll('#page-words .card-group-wrapper').forEach(wrapper => {
      const cardList = wrapper.nextElementSibling;
      const show = isAll || (cardList && [...cardList.querySelectorAll('[data-word-card]')].some(
        card => (card.dataset.wordDate ?? '').startsWith(prefix)
      ));
      if (wrapper)  wrapper.style.display  = show ? '' : 'none';
      if (cardList) cardList.style.display = show ? '' : 'none';
    });

    // Update count label
    const countEl = document.getElementById('pw-filter-count');
    if (countEl) {
      const visibleCards = [...document.querySelectorAll('#page-words [data-word-card]')]
        .filter(card => !card.classList.contains('hidden') && card.closest('.card-list')?.style.display !== 'none');
      const total = document.querySelectorAll('#page-words [data-word-card]').length;
      countEl.innerHTML = isAll
        ? `Showing <strong>${total}</strong> words`
        : `Showing <strong>${visibleCards.length}</strong> of ${total} words`;
    }
  });
}

/**
 * Wires chapter-chip buttons in `#ps-filter-chips` to filter sentence card groups.
 *
 * Each chip has a `data-chapter-chip` attribute. "all" shows every chapter group;
 * any other value shows only the group whose chapter name matches that chip value.
 *
 * @returns {void}
 */
function wireSentenceFilterChips() {
  const container = document.getElementById('ps-filter-chips');
  if (!container) return;

  container.addEventListener('click', e => {
    const chip = e.target.closest('.pw-filter-chip');
    if (!chip) return;

    // Update active chip
    container.querySelectorAll('.pw-filter-chip').forEach(c => c.classList.remove('is-active'));
    chip.classList.add('is-active');

    const value = chip.dataset.chapterChip ?? 'all';

    // Show/hide chapter group-wrappers and their card-list siblings
    document.querySelectorAll('#page-sentences .card-group-wrapper').forEach(wrapper => {
      const label = wrapper.querySelector('.card-group-label')?.textContent?.trim() ?? '';
      const show = value === 'all' || label === value;
      const cardList = wrapper.nextElementSibling;
      if (wrapper)  wrapper.style.display  = show ? '' : 'none';
      if (cardList) cardList.style.display = show ? '' : 'none';
    });

    // Update count label
    const countEl = document.getElementById('ps-filter-count');
    if (countEl) {
      const visibleCount = document.querySelectorAll('#page-sentences [data-sentence-card]:not(.hidden)').length;
      const total        = document.querySelectorAll('#page-sentences [data-sentence-card]').length;
      countEl.innerHTML  = value === 'all'
        ? `Showing <strong>${total}</strong> sentences`
        : `Showing <strong>${visibleCount}</strong> of ${total} sentences`;
    }
  });
}

// ── Initialise pre-selected state ─────────────────────────────────────────────

/**
 * Syncs the sel-circle and article.is-selected state to match
 * the current selection state from selection.js.
 *
 * Called after initSelection() has seeded all words as selected.
 *
 * @returns {void}
 */
function syncInitialSelection() {
  document.querySelectorAll('#page-words [data-sel-circle]').forEach(circle => {
    const idx = parseInt(circle.dataset.selCircle ?? '-1');
    if (isNaN(idx) || idx < 0) return;
    const on = hasWord(idx);
    circle.classList.toggle('is-selected', on);
    circle.closest('article')?.classList.toggle('is-selected', on);
  });
  document.querySelectorAll('#page-sentences [data-sel-circle]').forEach(circle => {
    const idx = parseInt(circle.dataset.selCircle ?? '-1');
    if (isNaN(idx) || idx < 0) return;
    const on = hasSentence(idx);
    circle.classList.toggle('is-selected', on);
    circle.closest('article')?.classList.toggle('is-selected', on);
  });
  updateSelectionBadges();
}

// ── Listen for external selection changes ─────────────────────────────────────

/**
 * Keeps sel-circles and badges in sync when selection changes externally
 * (e.g. from drag-select or other modules via 'selectionchange' event).
 *
 * @returns {void}
 */
function wireSelectionChangeListener() {
  window.addEventListener('selectionchange', () => {
    syncInitialSelection();
  });
}

// ── Public init ───────────────────────────────────────────────────────────────

/**
 * Initialises all page-level interactions.
 *
 * Must be called after DOMContentLoaded and after initSelection() has run.
 *
 * @returns {void}
 */
export function initPageInteractions() {
  // Filter panels
  wireFilterPanel('pw-filter-btn', 'pw-filter-panel');
  wireFilterPanel('ps-filter-btn', 'ps-filter-panel');

  // Filter chips
  wireWordFilterChips();
  wireSentenceFilterChips();

  // View mode toggles
  wireViewMode('words',     'pw');
  wireViewMode('sentences', 'ps');

  // Global delegated click handler
  document.addEventListener('click', handlePageClick);

  // Drag-to-select
  initDragSelect();

  // Sync initial selection circles to state
  syncInitialSelection();

  // Keep circles in sync on external changes
  wireSelectionChangeListener();
}
