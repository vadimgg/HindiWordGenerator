import { setSentenceSelected, setWordSelected } from '../state/selection.js';
import { updateSelectionBadges } from './cardSelection.js';

export function initDragSelect() {
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
    lasso.style.left = x1 + 'px';
    lasso.style.top = y1 + 'px';
    lasso.style.width = Math.abs(drag.currentX - drag.startX) + 'px';
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

    drag.active = true;
    drag.startX = e.clientX;
    drag.startY = e.clientY;
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
