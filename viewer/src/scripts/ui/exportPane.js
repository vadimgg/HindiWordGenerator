/**
 * Export pane controller — coordinates Deliver page modules.
 */

import { pollAnkiStatus, startPolling, stopPolling } from './ankiStatus.js';
import { wireDeckControls } from './deckControls.js';
import { populateDeliverPreview } from './deliverSelectionPreview.js';
import { wireExportActions } from './exportActions.js';

/**
 * Wires window-level event listeners for tab changes and selection changes.
 *
 * @returns {void}
 */
function wireWindowListeners() {
  window.addEventListener('tabchange', e => {
    if (e.detail.tab === 'deliver') { populateDeliverPreview(); startPolling(); }
    else stopPolling();
  });

  window.addEventListener('selectionchange', () => {
    const deliverPage = document.getElementById('page-deliver');
    if (!deliverPage?.classList.contains('is-active')) return;
    populateDeliverPreview();
  });
}

/**
 * Wires DOM element listeners for the deck inputs, override toggle, export button,
 * and .txt download button.
 *
 * @returns {void}
 */
function wireControlListeners() {
  wireDeckControls(pollAnkiStatus);
  wireExportActions(pollAnkiStatus);
}

/**
 * Initialises the export/deliver pane.
 *
 * @returns {void}
 */
export function initExportPane() {
  wireWindowListeners();
  wireControlListeners();
}
