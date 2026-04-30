/**
 * Delegated audio playback handler for .wc-audio-btn elements.
 *
 * Responsible for: listening for clicks on .wc-audio-btn elements anywhere in
 * the document, managing a single active Audio instance, toggling is-playing
 * state, and cleaning up on playback end or error.
 *
 * Dependencies: none (pure DOM).
 */
// Responsible for: delegated audio playback handler for .wc-audio-btn elements

/**
 * Initialises the delegated audio click handler on the document.
 * Safe to call once — attaches a single event listener.
 *
 * @returns {void}
 */
export function initAudio() {
  let current = null; // currently playing Audio instance

  document.addEventListener('click', function (e) {
    const btn = e.target.closest('.wc-audio-btn');
    if (!btn) return;
    const src = btn.dataset.src;
    if (!src) return;

    // Stop any currently playing audio
    if (current) {
      current.pause();
      current = null;
    }

    // Reset all buttons to default state
    document.querySelectorAll('.wc-audio-btn.is-playing').forEach(function (b) {
      b.classList.remove('is-playing');
      b.textContent = '🔊 ' + b.dataset.label;
    });

    current = new Audio(src);
    btn.classList.add('is-playing');
    btn.textContent = '■ ' + (btn.dataset.label || '');

    current.onended = function () {
      btn.classList.remove('is-playing');
      btn.textContent = '🔊 ' + (btn.dataset.label || '');
      current = null;
    };

    current.play().catch(function () {
      btn.classList.remove('is-playing');
      btn.textContent = '🔊 ' + (btn.dataset.label || '');
      current = null;
    });
  });
}
