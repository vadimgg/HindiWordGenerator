/**
 * Anki preview renderer for the Words page Anki mode area.
 *
 * Responsible for: rendering flip-card previews for selected words inside
 * #pw-anki-area. Listens for selectionchange events and Anki mode button
 * clicks to re-populate the preview with up-to-date selected words.
 *
 * Dependencies: anki/fields/index.js, anki/noteType.js, state/selection.js
 */
// Responsible for: rendering Anki flip-card previews for selected words in the Words page Anki mode area

import { wordToAnkiFields }     from '../anki/fields/index.js';
import { ANKI_FRONT, ANKI_BACK } from '../anki/noteType.js';
import { getSelectedWordObjects } from '../state/selection.js';

/**
 * Replaces {{FieldName}} tokens in a Mustache-style template string.
 * Unsupported block tags ({{#Foo}}…{{/Foo}}) are resolved naively:
 * the block content is included when the field is non-empty, omitted otherwise.
 *
 * @param {string} template - Template string with {{Token}} placeholders.
 * @param {Record<string, string>} fields - Map of field name → HTML string.
 * @returns {string} Rendered HTML string.
 */
function renderTemplate(template, fields) {
  // Strip block tags: {{#Foo}}...{{/Foo}} — keep inner content when field is truthy
  let out = template.replace(/\{\{#(\w+)\}\}([\s\S]*?)\{\{\/\1\}\}/g, (_, key, inner) =>
    fields[key] ? inner : ''
  );
  // Replace simple {{Token}} with field value (or empty string)
  out = out.replace(/\{\{(\w+)\}\}/g, (_, key) => fields[key] ?? '');
  return out;
}

/**
 * Renders the HTML string for a single Anki flip card.
 *
 * @param {object} word  - Vocabulary word object from the vocab JSON files.
 * @param {number} index - Zero-based index used for unique element IDs.
 * @returns {string} HTML string for a .anki-flip-card element.
 */
function renderFlipCard(word, index) {
  const fields   = wordToAnkiFields(word);
  const frontHtml = renderTemplate(ANKI_FRONT, fields);
  const backHtml  = renderTemplate(ANKI_BACK, fields);

  return `
    <div class="anki-flip-card" id="preview-card-${index}">
      <div class="anki-flip-inner">
        <div class="anki-flip-front">
          <div class="anki-shell">
            <div class="card">
              ${frontHtml}
            </div>
          </div>
        </div>
        <div class="anki-flip-back">
          <div class="anki-shell" style="overflow-y:auto;max-height:600px;">
            <div class="card">
              ${backHtml}
            </div>
          </div>
        </div>
      </div>
      <p class="anki-flip-hint">Click to flip</p>
    </div>
  `.trim();
}

/**
 * Wires click-to-flip behaviour on a single flip card element.
 * Sets initial wrapper height to the front face's scrollHeight,
 * and updates it on each click to match the incoming face.
 *
 * @param {HTMLElement} cardEl - A .anki-flip-card DOM element.
 * @returns {void}
 */
function initFlipCard(cardEl) {
  const front = cardEl.querySelector('.anki-flip-front');
  if (front) cardEl.style.height = front.scrollHeight + 'px';

  cardEl.addEventListener('click', () => {
    cardEl.classList.toggle('is-flipped');
    const activeFace = cardEl.classList.contains('is-flipped')
      ? cardEl.querySelector('.anki-flip-back')
      : cardEl.querySelector('.anki-flip-front');
    if (activeFace) cardEl.style.height = activeFace.scrollHeight + 'px';
  });
}

/**
 * Populates #pw-anki-cards with flip cards for currently selected words.
 * Shows #pw-anki-notice when nothing is selected; hides it otherwise.
 *
 * @returns {void}
 */
function populateAnkiPreview() {
  const notice    = document.getElementById('pw-anki-notice');
  const container = document.getElementById('pw-anki-cards');
  if (!notice || !container) return;

  const selectedWords = getSelectedWordObjects();
  if (selectedWords.length === 0) {
    notice.style.display    = '';
    container.innerHTML     = '';
    return;
  }

  notice.style.display = 'none';
  container.innerHTML  = selectedWords.map((w, i) => renderFlipCard(w, i)).join('');
  container.querySelectorAll('.anki-flip-card').forEach(initFlipCard);
}

/**
 * Initialises the Anki preview module.
 * Runs an initial population pass and subscribes to selectionchange events
 * and clicks on the Anki mode button.
 *
 * @returns {void}
 */
export function initAnkiPreview() {
  populateAnkiPreview();

  window.addEventListener('selectionchange', () => populateAnkiPreview());

  const ankiModeBtn = document.getElementById('pw-mode-anki');
  if (ankiModeBtn) {
    ankiModeBtn.addEventListener('click', () => populateAnkiPreview());
  }
}
