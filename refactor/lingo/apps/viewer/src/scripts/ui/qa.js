/**
 * QA page interactions.
 *
 * Responsible for: wiring QA issue actions, especially jumping from an issue
 * row to the source sentence card.
 *
 * Dependencies: state/tabs.js.
 */
// Responsible for: QA issue interactions

import { switchTab } from '../state/tabs.js';

const REVIEW_STORAGE_KEY = 'hindiweb.qaReviewed.v1';

function readReviewedIssues() {
  try {
    return new Set(JSON.parse(window.localStorage?.getItem(REVIEW_STORAGE_KEY) ?? '[]'));
  } catch {
    return new Set();
  }
}

function saveReviewedIssues(reviewed) {
  try {
    window.localStorage?.setItem(REVIEW_STORAGE_KEY, JSON.stringify([...reviewed]));
  } catch {
    // Review markers are local UI state only.
  }
}

function highlightCard(card) {
  card.classList.remove('is-collapsed');
  card.querySelector('.card-header')?.setAttribute('aria-expanded', 'true');
  card.scrollIntoView({ behavior: 'smooth', block: 'center' });
  card.classList.add('qa-jump-highlight');
  window.setTimeout(() => card.classList.remove('qa-jump-highlight'), 1800);
}

function revealWordCard(index) {
  switchTab('words');
  document.getElementById('pw-mode-web')?.click();

  const card = document.getElementById(`card-${index}`);
  if (card) highlightCard(card);
}

function revealSentenceCard(index) {
  switchTab('sentences');
  document.getElementById('ps-mode-web')?.click();

  const card = document.getElementById(`sentence-card-${index}`);
  if (card) highlightCard(card);
}

function issueMatchesFilter(item, filter) {
  if (filter === 'all') return true;
  return item.dataset.qaIssueType === filter || item.dataset.qaCardType === filter;
}

function applyQaFilter(filter) {
  document.querySelectorAll('[data-qa-filter]').forEach(button => {
    button.classList.toggle('is-active', button.dataset.qaFilter === filter);
    button.setAttribute('aria-pressed', String(button.dataset.qaFilter === filter));
  });

  document.querySelectorAll('[data-qa-issue-type]').forEach(item => {
    item.hidden = !issueMatchesFilter(item, filter);
  });
}

function setIssueReviewed(key, reviewed) {
  document.querySelectorAll(`[data-qa-issue-key="${CSS.escape(key)}"]`).forEach(item => {
    item.classList.toggle('is-reviewed', reviewed);
    const button = item.querySelector('[data-qa-review]');
    if (button) {
      button.textContent = reviewed ? 'Reviewed' : 'Mark reviewed';
      button.setAttribute('aria-pressed', String(reviewed));
    }
  });
}

function restoreReviewedIssues() {
  const reviewed = readReviewedIssues();
  document.querySelectorAll('[data-qa-issue-key]').forEach(item => {
    setIssueReviewed(item.dataset.qaIssueKey, reviewed.has(item.dataset.qaIssueKey));
  });
}

function toggleReviewedIssue(key) {
  const reviewed = readReviewedIssues();
  if (reviewed.has(key)) {
    reviewed.delete(key);
    setIssueReviewed(key, false);
  } else {
    reviewed.add(key);
    setIssueReviewed(key, true);
  }
  saveReviewedIssues(reviewed);
}

export function initQA() {
  restoreReviewedIssues();

  document.querySelectorAll('[data-qa-filter]').forEach(button => {
    button.setAttribute('aria-pressed', String(button.classList.contains('is-active')));
  });

  document.addEventListener('click', event => {
    const filterButton = event.target.closest('[data-qa-filter]');
    if (filterButton) {
      applyQaFilter(filterButton.dataset.qaFilter ?? 'all');
      return;
    }

    const reviewButton = event.target.closest('[data-qa-review]');
    if (reviewButton) {
      toggleReviewedIssue(reviewButton.dataset.qaReview);
      return;
    }

    const wordButton = event.target.closest('[data-qa-jump-word]');
    if (wordButton) {
      const index = Number(wordButton.dataset.qaJumpWord);
      if (Number.isInteger(index) && index >= 0) revealWordCard(index);
      return;
    }

    const button = event.target.closest('[data-qa-jump-sentence]');
    if (!button) return;
    const index = Number(button.dataset.qaJumpSentence);
    if (!Number.isInteger(index) || index < 0) return;
    revealSentenceCard(index);
  });
}
