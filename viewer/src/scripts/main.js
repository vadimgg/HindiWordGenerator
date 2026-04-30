/**
 * Application bootstrapper.
 *
 * Responsible for: importing all client-side modules and calling their init
 * functions in the correct order once the DOM is ready. This is the single
 * entry point loaded by index.astro via <script>.
 *
 * Initialisation order matters:
 *   1. initSelection — seeds selection state before any UI reads it.
 *   2. initSearch    — builds DOM caches; needs selection state.
 *   3. initPageInteractions — wires filter panels, group collapse, sel-circles,
 *                             drag-to-select; needs selection state to be seeded.
 *   4. initTooltip   — annotates example cards + tooltip positioning.
 *   5. initExportPane — export pane AnkiConnect polling and send logic.
 *   6. initTabs      — fires the initial 'tabchange' event last, after all
 *                      listeners are registered.
 */
// Responsible for: bootstrapper — imports all modules and initialises them after DOMContentLoaded

import { initSelection }        from './state/selection.js';
import { initTabs }             from './state/tabs.js';
import { initSearch }           from './ui/search.js';
import { initPageInteractions } from './ui/pageInteractions.js';
import { initTooltip }          from './ui/tooltip.js';
import { initExportPane }       from './ui/exportPane.js';
import { initAnkiPreview }      from './ui/ankiPreview.js';
import { initAudio }            from './ui/audio.js';

document.addEventListener('DOMContentLoaded', () => {
  initSelection();          // seed selection state from all words
  initSearch();             // wire search inputs + build DOM caches
  initPageInteractions();   // filter panels, group collapse, sel-circles, drag-select
  initTooltip();            // annotate example cards + tooltip positioning
  initExportPane();         // export pane table + AnkiConnect polling
  initAnkiPreview();        // populate Anki flip-card preview on Words page
  initAudio();              // delegated audio playback for .wc-audio-btn elements
  initTabs();               // last: fires initial tabchange event after all modules are ready
});
