function textSpan(className, text, attrs = {}) {
  const span = document.createElement('span');
  span.className = className;
  span.textContent = text ?? '';
  for (const [key, value] of Object.entries(attrs)) {
    span.setAttribute(key, value);
  }
  return span;
}

function appendSeparator(row) {
  row.append(textSpan('deliver-row-sep', '·'));
}

function issueChip(text) {
  return textSpan('deliver-row-chip is-warning', text);
}

export function renderWordRow(word, hasAudio = false) {
  const row = document.createElement('div');
  row.className = 'deliver-row';
  row.append(textSpan('deliver-row-hindi', word.hindi || '', { lang: 'hi' }));
  appendSeparator(row);
  row.append(textSpan('deliver-row-roman', word.romanisation || ''));
  appendSeparator(row);
  row.append(textSpan('deliver-row-english', (word.english || '').split(',')[0].trim()));
  row.append(textSpan(
    'deliver-row-meta',
    [word.pos, word.gender ? (word.gender === 'masculine' ? 'masc.' : 'fem.') : '']
      .filter(Boolean)
      .join(' · ')
  ));
  row.append(textSpan(hasAudio ? 'deliver-row-chip' : 'deliver-row-chip is-warning', hasAudio ? 'audio' : 'no audio'));
  return row;
}

export function renderSentenceRow(sentence, hasAudio = false) {
  const row = document.createElement('div');
  row.className = 'deliver-row';
  row.append(textSpan('deliver-row-hindi', sentence.hindi || '', { lang: 'hi' }));
  appendSeparator(row);
  row.append(textSpan('deliver-row-english', sentence.english || ''));

  if (sentence.register) {
    const badge = textSpan('deliver-row-badge', '');
    const register = textSpan('reg', sentence.register);
    register.classList.add(`reg-${sentence.register}`);
    register.style.fontSize = '11px';
    register.style.padding = '0.1rem 0.45rem';
    badge.append(register);
    row.append(badge);
  }

  const wordTokenCount = (sentence.tokens ?? []).filter(token => token.kind === 'word').length;
  row.append(textSpan(
    wordTokenCount > 0 ? 'deliver-row-chip' : 'deliver-row-chip is-muted',
    wordTokenCount > 0 ? `${wordTokenCount} words` : 'token gap'
  ));

  if (hasAudio) {
    row.append(textSpan('deliver-row-chip', 'audio'));
  } else {
    row.append(issueChip('no audio'));
  }

  return row;
}
