/**
 * Page interaction composer.
 *
 * Responsible for: wiring page-level interaction modules for filter panels,
 * filter chips, view mode toggles, card selection/collapse, and drag select.
 */

import { syncInitialSelection, handlePageClick, wireSelectionChangeListener } from './cardSelection.js';
import { initDragSelect } from './dragSelect.js';
import { wireFilterPanel, wireSentenceFilterChips, wireWordFilterChips } from './filterControls.js';
import { wireViewMode } from './viewMode.js';

export function initPageInteractions() {
  wireFilterPanel('pw-filter-btn', 'pw-filter-panel');
  wireFilterPanel('ps-filter-btn', 'ps-filter-panel');

  wireWordFilterChips();
  wireSentenceFilterChips();

  wireViewMode('words', 'pw');
  wireViewMode('sentences', 'ps');

  document.addEventListener('click', handlePageClick);
  initDragSelect();
  syncInitialSelection();
  wireSelectionChangeListener();
}
