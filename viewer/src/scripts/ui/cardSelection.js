import { hasSentence, hasWord, setSentenceSelected, setWordSelected } from '../state/selection.js';

export function updateSelectionBadges() {
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

function toggleGroupCollapse(headerEl) {
  const wrapperId = headerEl.dataset.groupToggle;
  if (!wrapperId) return;
  document.getElementById(wrapperId)?.classList.toggle('is-collapsed');
}

function toggleSelCircle(circle) {
  const cardIndex = parseInt(circle.dataset.selCircle ?? '-1');
  if (isNaN(cardIndex) || cardIndex < 0) return;

  const article = circle.closest('article');
  if (!article) return;
  const isSentence = article.hasAttribute('data-sentence-card') ||
                     article.closest('[id="page-sentences"]') !== null;

  const nowSelected = !circle.classList.contains('is-selected');
  circle.classList.toggle('is-selected', nowSelected);
  article.classList.toggle('is-selected', nowSelected);

  if (isSentence) setSentenceSelected(cardIndex, nowSelected);
  else setWordSelected(cardIndex, nowSelected);
}

function toggleGroupCheckbox(checkboxEl) {
  const wrapperId = checkboxEl.dataset.groupCheckbox;
  if (!wrapperId) return;

  const wrapper = document.getElementById(wrapperId);
  const cardList = wrapper?.nextElementSibling;
  if (!cardList) return;

  const circles = [...cardList.querySelectorAll('.sel-circle')];
  const allOn = circles.every(c => c.classList.contains('is-selected'));
  const goOn = !allOn;

  checkboxEl.classList.toggle('is-checked', goOn);
  checkboxEl.classList.remove('is-indeterminate');
  checkboxEl.setAttribute('aria-checked', String(goOn));

  circles.forEach(circle => {
    const cardIndex = parseInt(circle.dataset.selCircle ?? '-1');
    if (isNaN(cardIndex) || cardIndex < 0) return;
    circle.classList.toggle('is-selected', goOn);
    circle.closest('article')?.classList.toggle('is-selected', goOn);

    const isInSentences = circle.closest('#page-sentences') !== null;
    if (isInSentences) setSentenceSelected(cardIndex, goOn);
    else setWordSelected(cardIndex, goOn);
  });
}

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

export function handlePageClick(e) {
  const circle = e.target.closest('.sel-circle');
  if (circle) {
    e.stopPropagation();
    toggleSelCircle(circle);
    updateSelectionBadges();
    return;
  }

  const groupCheckbox = e.target.closest('.card-group-checkbox');
  if (groupCheckbox) {
    e.stopPropagation();
    toggleGroupCheckbox(groupCheckbox);
    updateSelectionBadges();
    return;
  }

  const groupHeader = e.target.closest('.card-group-header');
  if (groupHeader && !e.target.closest('.card-group-checkbox, .quick-export-btn')) {
    toggleGroupCollapse(groupHeader);
    return;
  }

  handleCardHeaderClick(e);
}

export function syncInitialSelection() {
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

export function wireSelectionChangeListener() {
  window.addEventListener('selectionchange', () => {
    syncInitialSelection();
  });
}
