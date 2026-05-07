/**
 * Anki preview renderer for the Words and Sentences Anki mode areas.
 *
 * Responsible for: rendering flip-card previews for selected words and
 * sentences. Uses the same field builders and templates as the Anki export
 * path, plus a small local renderer that simulates Anki's template tags.
 *
 * Dependencies: anki/fields, anki/noteType.js, anki/sentenceNoteType.js,
 *               anki/renderTemplate.js, state/selection.js, data.js.
 */
// Responsible for: rendering Anki flip-card previews for selected words and sentences

import { wordToAnkiFields } from '../anki/fields/index.js';
import { ANKI_FRONT, ANKI_BACK } from '../anki/noteType.js';
import { ANKI_SENTENCE_FRONT, ANKI_SENTENCE_BACK } from '../anki/sentenceNoteType.js';
import { sentenceToAnkiFields } from '../anki/fields/sentence.js';
import { renderTemplate } from '../anki/renderTemplate.js';
import { getSelectedWordObjects, getSelectedSentenceIndices } from '../state/selection.js';
import { getAllSentences } from '../data.js';

/**
 * Renders the HTML string for a single Anki flip card.
 *
 * @param {Record<string, string>} fields - Anki field values for the card.
 * @param {string} frontTemplate - Front template HTML.
 * @param {string} backTemplate - Back template HTML.
 * @param {number} index - Zero-based index used for unique element IDs.
 * @returns {string} HTML string for a .anki-flip-card element.
 */
function renderFlipCard(fields, frontTemplate, backTemplate, index) {
  const frontHtml = renderTemplate(frontTemplate, fields);
  const backHtml  = renderTemplate(backTemplate, fields);

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
  container.innerHTML  = selectedWords
    .map((w, i) => renderFlipCard(wordToAnkiFields(w), ANKI_FRONT, ANKI_BACK, i))
    .join('');
  container.querySelectorAll('.anki-flip-card').forEach(initFlipCard);
}

/**
 * Populates #ps-anki-cards with flip cards for currently selected sentences.
 *
 * @returns {void}
 */
function populateSentenceAnkiPreview() {
  const notice    = document.getElementById('ps-anki-notice');
  const container = document.getElementById('ps-anki-cards');
  if (!notice || !container) return;

  const allSentences = getAllSentences();
  const selectedSentences = getSelectedSentenceIndices()
    .map(i => allSentences[i])
    .filter(Boolean);

  if (selectedSentences.length === 0) {
    notice.style.display = '';
    container.innerHTML = '';
    return;
  }

  notice.style.display = 'none';
  container.innerHTML = selectedSentences
    .map((s, i) => renderFlipCard(
      sentenceToAnkiFields(s, s.groupLabel ?? ''),
      ANKI_SENTENCE_FRONT,
      ANKI_SENTENCE_BACK,
      i
    ))
    .join('');
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
  populateSentenceAnkiPreview();

  window.addEventListener('selectionchange', () => {
    populateAnkiPreview();
    populateSentenceAnkiPreview();
  });

  const ankiModeBtn = document.getElementById('pw-mode-anki');
  if (ankiModeBtn) {
    ankiModeBtn.addEventListener('click', () => populateAnkiPreview());
  }

  const sentenceAnkiModeBtn = document.getElementById('ps-mode-anki');
  if (sentenceAnkiModeBtn) {
    sentenceAnkiModeBtn.addEventListener('click', () => populateSentenceAnkiPreview());
  }
}
