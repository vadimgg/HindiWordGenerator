export function wireFilterPanel(btnId, panelId) {
  const btn = document.getElementById(btnId);
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

export function wireWordFilterChips() {
  const container = document.getElementById('pw-filter-chips');
  if (!container) return;

  container.addEventListener('click', e => {
    const chip = e.target.closest('.pw-filter-chip');
    if (!chip) return;

    container.querySelectorAll('.pw-filter-chip').forEach(c => c.classList.remove('is-active'));
    chip.classList.add('is-active');

    const prefix = chip.dataset.datePrefix ?? '';
    const isAll = chip.dataset.dateChip === 'all';

    document.querySelectorAll('#page-words .card-group-wrapper').forEach(wrapper => {
      const cardList = wrapper.nextElementSibling;
      const show = isAll || (cardList && [...cardList.querySelectorAll('[data-word-card]')].some(
        card => (card.dataset.wordDate ?? '').startsWith(prefix)
      ));
      wrapper.style.display = show ? '' : 'none';
      if (cardList) cardList.style.display = show ? '' : 'none';
    });

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

export function wireSentenceFilterChips() {
  const container = document.getElementById('ps-filter-chips');
  if (!container) return;

  container.addEventListener('click', e => {
    const chip = e.target.closest('.pw-filter-chip');
    if (!chip) return;

    container.querySelectorAll('.pw-filter-chip').forEach(c => c.classList.remove('is-active'));
    chip.classList.add('is-active');

    const value = chip.dataset.groupChip ?? 'all';

    document.querySelectorAll('#page-sentences .card-group-wrapper').forEach(wrapper => {
      const label = wrapper.querySelector('.card-group-label')?.textContent?.trim() ?? '';
      const show = value === 'all' || label === value;
      const cardList = wrapper.nextElementSibling;
      wrapper.style.display = show ? '' : 'none';
      if (cardList) cardList.style.display = show ? '' : 'none';
    });

    const countEl = document.getElementById('ps-filter-count');
    if (countEl) {
      const visibleCount = document.querySelectorAll('#page-sentences [data-sentence-card]:not(.hidden)').length;
      const total = document.querySelectorAll('#page-sentences [data-sentence-card]').length;
      countEl.innerHTML = value === 'all'
        ? `Showing <strong>${total}</strong> sentences`
        : `Showing <strong>${visibleCount}</strong> of ${total} sentences`;
    }
  });
}
