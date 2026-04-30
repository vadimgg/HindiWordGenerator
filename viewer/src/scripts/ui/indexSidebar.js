/**
 * Sidebar interaction handler — drag-select, group controls, scroll-to-card.
 *
 * Responsible for: all user interactions with the words index sidebar:
 *   - Drag-select: hold and drag across checkboxes to select/deselect a range.
 *   - Group checkbox: toggle all words in a group on/off in one click.
 *   - Group collapse: hide/show the word list inside a group.
 *   - Row click: smoothly scroll the main content to the corresponding card.
 *
 * Filter re-application after selection changes is handled by the 'selectionchange'
 * CustomEvent that selection.js dispatches — no direct call to search.js needed.
 *
 * Dependencies: state/selection.js.
 */
// Responsible for: sidebar interactions — drag-select checkboxes, group collapse, scroll-to-card

import { setWordSelected, hasWord, syncGroupCheckbox } from '../state/selection.js';

/**
 * Whether a drag-select gesture is currently in progress.
 * Set to true on mousedown on a checkbox; reset to false on mouseup.
 * @type {boolean}
 */
let isDragging = false;

/**
 * The selection value being applied during the current drag (true = select, false = deselect).
 * Determined by the state of the first checkbox touched at drag start.
 * @type {boolean}
 */
let dragSelectMode = true;

/**
 * Handles a delegated click inside the words index sidebar.
 * Dispatches to: group-checkbox toggle, collapse button, group-header toggle, scroll-to-card.
 * Word-checkbox clicks are handled by the mousedown handler instead.
 *
 * @param {MouseEvent} e - The click event from the delegated listener.
 * @returns {void}
 */
function handleSidebarClick(e) {
  if (e.target.closest('.word-checkbox')) return;  // handled by mousedown

  const groupCheckbox = e.target.closest('.group-checkbox');
  if (groupCheckbox) {
    const groupDate = groupCheckbox.dataset.groupDate;
    const groupEl   = document.querySelector(`#idx-words .date-group[data-group="${groupDate}"]`);
    if (!groupEl) return;
    const rows  = [...groupEl.querySelectorAll('.idx-row[data-index]')];
    const allOn = rows.every(r => hasWord(parseInt(r.dataset.index)));
    rows.forEach(r => setWordSelected(parseInt(r.dataset.index), !allOn));
    syncGroupCheckbox(groupDate);
    // applyFilter() fires automatically via 'selectionchange' event.
    return;
  }

  const collapseBtn = e.target.closest('.collapse-btn');
  if (collapseBtn) {
    collapseBtn.closest('.date-group')?.querySelector('.date-group-words')?.classList.toggle('collapsed');
    collapseBtn.classList.toggle('is-collapsed');
    return;
  }

  const groupToggle = e.target.closest('.date-group-toggle');
  if (groupToggle) {
    const group = groupToggle.closest('.date-group');
    group?.querySelector('.date-group-words')?.classList.toggle('collapsed');
    group?.querySelector('.collapse-btn')?.classList.toggle('is-collapsed');
    return;
  }

  const scrollTarget = e.target.closest('[data-scroll-to]');
  if (scrollTarget) {
    document.getElementById(`card-${scrollTarget.dataset.scrollTo}`)
      ?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }
}

/**
 * Initialises all sidebar event listeners on the #idx-words element.
 *
 * Uses a single delegated handler per event type so we don't attach dozens of
 * listeners to individual checkboxes.
 *
 * Must be called after DOMContentLoaded.
 *
 * @returns {void}
 */
export function initSidebar() {
  const idxWords = document.getElementById('idx-words');

  // Drag-select: begin on mousedown on a checkbox
  idxWords?.addEventListener('mousedown', e => {
    const checkbox = e.target.closest('.word-checkbox');
    if (!checkbox) return;
    e.preventDefault();  // prevent text selection during drag
    const wordIndex = parseInt(checkbox.dataset.wordIndex);
    dragSelectMode  = !hasWord(wordIndex);  // toggle direction from current state
    setWordSelected(wordIndex, dragSelectMode);
    isDragging = true;
    syncGroupCheckbox(checkbox.closest('.date-group')?.dataset.group ?? '');
    // applyFilter() fires automatically via 'selectionchange' event.
  });

  // Drag-select: extend to hovered checkboxes while dragging
  idxWords?.addEventListener('mouseover', e => {
    if (!isDragging) return;
    const checkbox = e.target.closest('.word-checkbox');
    if (!checkbox) return;
    const wordIndex = parseInt(checkbox.dataset.wordIndex);
    if (hasWord(wordIndex) === dragSelectMode) return;  // already in target state
    setWordSelected(wordIndex, dragSelectMode);
    syncGroupCheckbox(checkbox.closest('.date-group')?.dataset.group ?? '');
    // applyFilter() fires automatically via 'selectionchange' event.
  });

  document.addEventListener('mouseup', () => { isDragging = false; });
  idxWords?.addEventListener('click', handleSidebarClick);
}
