import {
  audioClass,
  buildWordsFromSentences,
  deriveSummary,
  filterSentences,
  groupSentences,
  hasAudio,
  normalizeText,
  sampleState,
  safeState,
  selectedOrAllReady,
  statusClass,
} from './state.mjs';
import { commandCardsForPage, commandForPage, commandLibrary } from './commands.mjs';

const root = document.querySelector('[data-app]');
const selectedIds = new Set();
const expandedIds = new Set(); // sentence ids expanded in place (replaces the slide-over)
let activePage = 'organize';
let focusedId = null;
let state = safeState(sampleState);
let serverOnline = false;
let draggedId = null;
let producer = null; // { kind, runId, prompt, section } during a prompt loop
let importPreview = null; // { packageName, path, counts, groups } after a scan
const importSelected = new Set();
const importCollapsed = new Set(); // section names collapsed in the import preview

const $ = (selector, scope = document) => scope.querySelector(selector);
const $$ = (selector, scope = document) => [...scope.querySelectorAll(selector)];

function escapeHtml(value) {
  return String(value ?? '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#039;');
}

function html(strings, ...values) {
  return strings.reduce((out, part, index) => out + part + (values[index] ?? ''), '');
}

function currentContext() {
  return {
    section: $('[data-control="section"]')?.value || state.sentences?.[0]?.section,
    limit: $('[data-control="limit"]')?.value || 20,
    importPath: $('[data-control="importPath"]')?.value || '<package-dir>',
    minCount: $('[data-control="minCount"]')?.value || 1,
    audioBackend: $('[data-control="audioBackend"]')?.value || state.config.audio.backend,
    audioVoice: $('[data-control="audioVoice"]')?.value || state.config.audio.voice,
    ankiDeck: $('[data-control="ankiDeck"]')?.value || state.config.anki.deck,
    packageDest: $('[data-control="packageDest"]')?.value || state.config.package.destination,
    packageFormat: $('[data-control="packageFormat"]')?.value || state.config.package.format,
    selectedIds,
    focusedId,
  };
}

async function loadState() {
  setServerStatus('wait', 'checking local server');
  try {
    const response = await fetch('/api/view/state', { headers: { accept: 'application/json' } });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    state = safeState(await response.json());
    serverOnline = true;
    setServerStatus('ready', 'local server connected');
  } catch (error) {
    state = safeState(sampleState);
    serverOnline = false;
    setServerStatus('draft', 'offline demo mode');
  }
  hydrateControls();
  renderAll();
}

function setServerStatus(kind, label) {
  const pill = $('[data-server-status]');
  if (!pill) return;
  pill.innerHTML = `<i class="dot ${kind}"></i>${escapeHtml(label)}`;
}

function hydrateControls() {
  $('[data-workspace-name]').textContent = state.workspace.name;
  $('[data-workspace-language]').textContent = state.workspace.language;
  $('[data-workspace-library]').textContent = state.workspace.libraryPath || 'library.db';
  const set = (selector, value) => { const node = $(selector); if (node) node.value = value ?? ''; };
  set('[data-control="audioBackend"]', state.config.audio.backend);
  set('[data-control="audioVoice"]', state.config.audio.voice);
  set('[data-control="ankiDeck"]', state.config.anki.deck);
  set('[data-control="packageDest"]', state.config.package.destination);
  set('[data-control="packageFormat"]', state.config.package.format);
  const replace = $('[data-control="ankiReplace"]');
  if (replace) replace.checked = Boolean(state.config.anki.replace);

  for (const control of $$('[data-setting]')) {
    const value = getPath(state, control.dataset.setting);
    if (control.type === 'checkbox') control.checked = Boolean(value);
    else control.value = value ?? '';
  }
}

function renderAll() {
  state.words = state.words?.length ? state.words : buildWordsFromSentences(state.sentences);
  renderSummary();
  renderPages();
  renderGenerateQueue();
  renderSentenceList();
  renderWordList();
  renderAudioList();
  renderPublishPanels();
  renderSelection();
  renderEnrichButton();
  renderCli();
}

function renderEnrichButton() {
  const btn = $('[data-action="enrich"]');
  if (!btn) return;
  btn.textContent = selectedIds.size ? `Enrich ${selectedIds.size} selected →` : 'Enrich next batch →';
}

function setText(selector, value) {
  const node = $(selector);
  if (node) node.textContent = value;
}

function renderSummary() {
  const summary = deriveSummary(state);
  setText('[data-summary-sentences]', summary.sentences);
  setText('[data-summary-draft]', summary.draft);
  setText('[data-summary-enriching]', summary.enriching);
  setText('[data-summary-enriched]', summary.enriched);
  setText('[data-summary-audio]', `${summary.audioReady}/${summary.sentences}`);
  setText('[data-summary-words]', summary.words);
  setText('[data-count-draft]', summary.draft);
  setText('[data-count-enriching]', summary.enriching);
}

function renderPages() {
  for (const page of $$('[data-page]')) page.hidden = page.dataset.page !== activePage;
  for (const link of $$('[data-page-link]')) link.setAttribute('aria-current', link.dataset.pageLink === activePage ? 'page' : 'false');
}

function renderGenerateQueue() {
  const queue = $('[data-generate-queue]');
  if (!queue) return;
  const rows = state.sentences.filter((s) => s.status === 'draft' || s.status === 'enriching');
  queue.innerHTML = rows.length
    ? rows.map((sentence) => sentenceRow(sentence, 'generate')).join('')
    : '<div class="empty-state"><strong>No drafts pending</strong><span>Extract raw text or reset abandoned claims.</span></div>';
}

function sentenceMatchesControls() {
  return filterSentences(state.sentences, {
    search: $('[data-control="search"]')?.value || '',
    status: $('[data-control="statusFilter"]')?.value || 'all',
  });
}

function renderSentenceList() {
  const list = $('[data-sentence-list]');
  if (!list) return;
  const groups = groupSentences(sentenceMatchesControls());
  if (!groups.length) {
    list.innerHTML = '<div class="empty-state"><strong>No matching sentences</strong><span>Try a different status or search term.</span></div>';
    return;
  }
  list.innerHTML = groups.map(({ section, rows }) => html`
    <section class="sentence-section">
      <div class="section-title">
        <span>${escapeHtml(section)}</span><span>${rows.length}</span>
        <button class="mini-btn section-rename" type="button" data-action="rename-section" data-section="${escapeHtml(section)}" title="Rename this subtitle">✎</button>
      </div>
      ${rows.map((sentence) => sentenceRow(sentence, 'organize')).join('')}
    </section>
  `).join('');
}

// One reusable sentence row, shared by Organize, Audio, and the Generate queue.
// Collapsed it shows target + roman·english + per-mode controls; tapping the
// header expands it in place (no slide-over) to a word-by-word breakdown.
function sentenceRow(sentence, mode = 'organize') {
  const selected = selectedIds.has(sentence.id);
  const expanded = expandedIds.has(sentence.id);
  const hasRoman = Boolean(sentence.romanisation);
  const hasEng = Boolean(sentence.english);
  const selectable = true; // every page can select rows for batch actions (enrich, audio, section)
  return html`
    <article class="row sentence-row ${selected ? 'is-selected' : ''} ${expanded ? 'is-expanded' : ''}" data-sentence-id="${escapeHtml(sentence.id)}" data-expand-for="${escapeHtml(sentence.id)}" ${mode === 'organize' ? 'draggable="true"' : ''}>
      <div class="row-head" data-expand-toggle="${escapeHtml(sentence.id)}">
        ${selectable ? html`<label class="select-box" title="Select"><input type="checkbox" data-select-sentence="${escapeHtml(sentence.id)}" ${selected ? 'checked' : ''} /></label>` : ''}
        <div class="row-main">
          <div class="target" lang="hi">${escapeHtml(sentence.target)}</div>
          <div class="sub-line"><span class="roman">${escapeHtml(sentence.romanisation || '')}</span>${hasRoman && hasEng ? '<span class="sep">·</span>' : ''}<span class="english">${escapeHtml(sentence.english || '')}</span></div>
        </div>
        <div class="row-meta">${sentenceRowControls(sentence, mode)}</div>
      </div>
      ${expanded ? sentenceExpand(sentence) : ''}
    </article>
  `;
}

// Trailing controls differ per page; the chevron is shared.
function sentenceRowControls(sentence, mode) {
  const chevron = '<span class="row-chevron">›</span>';
  if (mode === 'audio') {
    return html`
      <span class="chip ${audioClass(sentence)}">${hasAudio(sentence) ? sentence.audio.backend || 'audio' : 'missing'}</span>
      <button class="mini-btn" type="button" data-action="play-audio" data-audio-path="${escapeHtml(sentence.audio?.path || '')}" ${hasAudio(sentence) ? '' : 'disabled'}>Play</button>
      <button class="mini-btn" type="button" data-action="regen-one" data-sentence-id="${escapeHtml(sentence.id)}" title="Regenerate">↻</button>
      ${chevron}
    `;
  }
  if (mode === 'generate') {
    return html`<span class="chip ${statusClass(sentence.status)}">${escapeHtml(sentence.status)}</span>${chevron}`;
  }
  const human = Object.values(sentence.authority || {}).includes('human');
  return html`
    ${!hasAudio(sentence) ? '<span class="row-flag warn" title="No audio">no audio</span>' : ''}
    ${human ? '<span class="row-flag" title="Human-authored fields">🔒</span>' : ''}
    <i class="dot ${statusClass(sentence.status)}" title="${escapeHtml(sentence.status)}"></i>
    <button class="mini-btn handle" type="button" title="Drag to reorder">⠿</button>
    ${chevron}
  `;
}

// In-place expansion: aligned token stack + scannable table + meta. Each token
// carries data-token so hovering one lights up its Devanagari, roman, and gloss
// across the stack and the table (Grasp-style cross-highlight).
function sentenceExpand(sentence) {
  const breakdown = sentence.breakdown || [];
  const body = breakdown.length ? html`
    <div class="sentence-stack" aria-label="Word by word">
      ${breakdown.map((item, i) => html`
        <div class="stack-col" data-token="${i}">
          <span class="stack-deva" lang="hi">${escapeHtml(item.surface || '')}</span>
          <span class="stack-roman">${escapeHtml(item.roman || '')}</span>
          <span class="stack-gloss">${escapeHtml(item.gloss || '')}</span>
        </div>
      `).join('')}
    </div>
    <div class="breakdown-table" role="table">
      ${breakdown.map((item, i) => html`
        <div class="bt-row" data-token="${i}" role="row">
          <span class="bt-deva" lang="hi">${escapeHtml(item.surface || '')}</span>
          <span class="bt-roman">${escapeHtml(item.roman || '')}</span>
          <span class="bt-gloss">${escapeHtml(item.gloss || '')}</span>
          <span class="bt-kind">${escapeHtml(item.kind || '')}</span>
        </div>
      `).join('')}
    </div>
  ` : '<p class="subtle expand-empty">No word-by-word breakdown yet — enrich this sentence to see it.</p>';

  const fields = ['target', 'romanisation', 'english', 'literal', 'breakdown'];
  return html`
    <div class="sentence-expand">
      ${body}
      <div class="expand-meta">
        ${sentence.literal ? `<span class="meta-literal">${escapeHtml(sentence.literal)}</span>` : ''}
        ${sentence.register ? `<span class="meta-tag">${escapeHtml(sentence.register)}</span>` : ''}
        ${(sentence.tags || []).map((tag) => `<span class="meta-tag">#${escapeHtml(tag)}</span>`).join('')}
        <span class="spacer"></span>
        ${hasAudio(sentence) ? `<button class="mini-btn" type="button" data-action="play-audio" data-audio-path="${escapeHtml(sentence.audio?.path || '')}">▶ Play</button>` : ''}
        ${sentence.status === 'enriched' ? `<button class="mini-btn confirm" type="button" data-action="confirm-active" data-sentence-id="${escapeHtml(sentence.id)}">✓ Confirm active</button>` : ''}
        ${sentence.status === 'active' ? `<button class="mini-btn" type="button" data-action="revert-enriched" data-sentence-id="${escapeHtml(sentence.id)}">↩ Unpublish</button>` : ''}
        <button class="mini-btn" type="button" data-action="copy-cli" data-command-key="organize.edit" data-sentence-id="${escapeHtml(sentence.id)}">⧉ Edit CLI</button>
      </div>
      <div class="authority-chips">
        ${fields.map((field) => {
          const owner = sentence.authority?.[field] || 'empty';
          return `<span class="auth-chip ${owner}">${escapeHtml(field)}<b>${owner === 'human' ? '🔒 human' : owner}</b></span>`;
        }).join('')}
      </div>
    </div>
  `;
}

function renderWordList() {
  const list = $('[data-word-list]');
  if (!list) return;
  const min = Number($('[data-control="minCount"]')?.value || 1);
  const needle = normalizeText($('[data-control="wordSearch"]')?.value || '');
  const words = (state.words ?? []).filter((word) => {
    if ((word.count ?? 0) < min) return false;
    if (!needle) return true;
    return normalizeText([word.form, word.roman, word.kind, ...(word.meanings ?? [])].join(' ')).includes(needle);
  });
  list.innerHTML = words.length ? words.map((word) => html`
    <button class="row word-row" type="button" data-word-key="${escapeHtml(word.key)}">
      <span>
        <span class="word-form" lang="hi">${escapeHtml(word.form)}</span>
        <span class="roman">${escapeHtml(word.roman || '')}</span>
        <span class="word-meaning">${escapeHtml((word.meanings || []).join(', '))}</span>
      </span>
      <span class="chip ready">${word.count}×</span>
    </button>
  `).join('') : '<div class="empty-state"><strong>No matching words</strong><span>The lexicon is projected from enriched sentence breakdowns.</span></div>';
}

function renderWordDetail(key) {
  const detail = $('[data-word-detail]');
  if (!detail) return;
  const word = state.words.find((row) => row.key === key);
  if (!word) {
    detail.innerHTML = '<div class="empty-state"><strong>Select a word</strong><span>Its meanings and sentences appear here.</span></div>';
    return;
  }
  const sentences = state.sentences.filter((sentence) => word.sentenceIds?.includes(sentence.id));
  detail.innerHTML = html`
    <div class="panel-head"><h2><span class="word-form" lang="hi">${escapeHtml(word.form)}</span> ${escapeHtml(word.roman || '')}</h2><span class="chip ready">${word.count} sentences</span></div>
    <p class="subtle">${escapeHtml((word.meanings || []).join(', ') || 'No meanings recorded')}</p>
    <div class="breakdown-list">
      ${sentences.map((sentence) => html`
        <div class="breakdown-row">
          <span class="target" lang="hi">${escapeHtml(sentence.target)}</span>
          <span class="english">${escapeHtml(sentence.english || '')}</span>
        </div>
      `).join('')}
    </div>
  `;
}

function renderAudioList() {
  const list = $('[data-audio-list]');
  if (!list) return;
  const ready = state.sentences.filter(hasAudio).length;
  setText('[data-audio-ready]', ready);
  setText('[data-audio-missing]', state.sentences.length - ready);
  list.innerHTML = state.sentences.map((sentence) => sentenceRow(sentence, 'audio')).join('');
}

function renderPublishPanels() {
  const selected = selectedOrAllReady(state, selectedIds);
  const selectedReady = selected.filter((sentence) => sentence.status === 'enriched' && hasAudio(sentence));
  const selectedMissingAudio = selected.filter((sentence) => !hasAudio(sentence));
  const selectedDraft = selected.filter((sentence) => sentence.status !== 'enriched');
  const summary = [
    ['Selected', selected.length],
    ['Ready', selectedReady.length],
    ['No audio', selectedMissingAudio.length],
    ['Needs enrich', selectedDraft.length],
  ];
  for (const target of ['[data-anki-summary]', '[data-package-summary]']) {
    const node = $(target);
    if (!node) continue;
    node.innerHTML = summary.map(([label, count]) => `<div class="summary-box"><strong>${count}</strong><span>${label}</span></div>`).join('');
  }
}

function renderSelection() {
  const bar = $('[data-selectionbar]');
  if (!bar) return;
  const count = $('[data-selection-count]');
  if (selectedIds.size) {
    bar.hidden = false;
    if (count) count.textContent = selectedIds.size;
  } else {
    bar.hidden = true;
  }
}


function renderCli() {
  const ctx = currentContext();
  const active = commandForPage(activePage, state, ctx);
  $('[data-active-cli]').textContent = active;
  const list = $('[data-cli-list]');
  if (!list) return;
  list.innerHTML = commandCardsForPage(activePage, state, ctx).map((card) => html`
    <article class="command-card">
      <h3>${escapeHtml(card.title)}</h3>
      <p>${escapeHtml(card.description)}</p>
      <pre><code>${escapeHtml(card.command)}</code></pre>
      <button class="btn secondary" type="button" data-copy-command="${escapeHtml(card.command)}">Copy</button>
    </article>
  `).join('');
}

async function runAction(kind, payload = {}, fallbackCommandKey = null) {
  if (!serverOnline) {
    const command = fallbackCommandKey ? commandLibrary(state, currentContext())[fallbackCommandKey]?.command : commandForPage(activePage, state, currentContext());
    toast(`Offline demo mode. Run from the CLI instead: ${command}`, true);
    return;
  }
  try {
    const response = await fetch('/api/view/action', {
      method: 'POST',
      headers: { 'content-type': 'application/json', accept: 'application/json' },
      body: JSON.stringify({ kind, payload }),
    });
    const body = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(body.message || `HTTP ${response.status}`);
    if (body.state) state = safeState(body.state);
    toast(body.message || 'Action complete');
    renderAll();
  } catch (error) {
    toast(`Action failed: ${error.message}`, true);
  }
}

// ── producer prompt loop (extract / enrich) ───────────────────────────────
async function startProducer(kind, payload) {
  if (!serverOnline) {
    toast(`Offline demo mode. Run from the CLI instead: ${commandForPage(activePage, state, currentContext())}`, true);
    return;
  }
  try {
    const response = await fetch('/api/view/action', {
      method: 'POST',
      headers: { 'content-type': 'application/json', accept: 'application/json' },
      body: JSON.stringify({ kind: `${kind}.prepare`, payload }),
    });
    const body = await response.json().catch(() => ({}));
    if (body.prompt) {
      producer = { kind, runId: body.runId, prompt: body.prompt, section: payload.section };
      renderProducer();
      $('[data-producer-panel]')?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    } else {
      if (body.state) state = safeState(body.state);
      toast(body.message || 'Nothing to do.');
      renderAll();
    }
  } catch (error) {
    toast(`Failed: ${error.message}`, true);
  }
}

async function applyProducer() {
  if (!producer) return;
  const producerKind = producer.kind;
  const reply = $('[data-reply]')?.value || '';
  if (!reply.trim()) { toast('Paste the model reply first.', true); return; }
  try {
    const response = await fetch('/api/view/action', {
      method: 'POST',
      headers: { 'content-type': 'application/json', accept: 'application/json' },
      body: JSON.stringify({ kind: `${producer.kind}.apply`, payload: { runId: producer.runId, reply, section: producer.section } }),
    });
    const body = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(body.message || `HTTP ${response.status}`);
    if (body.state) state = safeState(body.state);
    toast(body.message || 'Applied.');
    producer = null;
    if (producerKind === 'enrich') {
      selectedIds.clear(); // the batch is enriched; drop its selection
      activePage = 'organize'; // land on Review so the just-enriched rows are visible to confirm
      const filter = $('[data-control="statusFilter"]');
      if (filter) filter.value = 'enriched';
    }
    renderAll();
    renderProducer();
  } catch (error) {
    toast(`Apply failed: ${error.message}`, true);
  }
}

function renderProducer() {
  const panel = $('[data-producer-panel]');
  if (!panel) return;
  if (!producer) { panel.innerHTML = ''; return; }
  const step = producer.kind === 'extract' ? 'extract' : 'enrich';
  panel.innerHTML = html`
    <div class="prompt-step">
      <div class="detail-label">Prompt — paste into ChatGPT or Claude</div>
      <pre class="cli-snippet prompt-box">${escapeHtml(producer.prompt)}</pre>
      <div class="head-actions">
        <button class="btn" type="button" data-action="copy-prompt">Copy prompt</button>
        <button class="btn secondary" type="button" data-action="cancel-producer">Cancel</button>
      </div>
    </div>
    <div class="reply-step">
      <div class="detail-label">Reply — paste the model's ${step === 'extract' ? 'YAML' : 'JSON'} answer</div>
      <textarea class="raw-box" data-reply placeholder="Paste the reply here…"></textarea>
      <div class="head-actions" style="margin-top:0.6rem">
        <button class="btn primary" type="button" data-action="apply-reply">Apply →</button>
      </div>
    </div>
  `;
}

// ── import: scan → preview → selective commit ─────────────────────────────
async function previewImport() {
  const path = $('[data-control="importPath"]')?.value?.trim();
  if (!path) { toast('Enter the package path first.', true); return; }
  if (!serverOnline) { toast('Offline demo mode — run the CLI import instead.', true); return; }
  const panel = $('[data-import-preview]');
  if (panel) panel.innerHTML = '<div class="empty-state"><strong>Scanning package…</strong></div>';
  try {
    const response = await fetch('/api/view/action', {
      method: 'POST',
      headers: { 'content-type': 'application/json', accept: 'application/json' },
      body: JSON.stringify({ kind: 'import.preview', payload: { path } }),
    });
    const body = await response.json().catch(() => ({}));
    if (!response.ok || !body.preview) throw new Error(body.message || `HTTP ${response.status}`);
    importPreview = body.preview;
    importCollapsed.clear();
    setImportSelection('new'); // default: everything not already in the library
  } catch (error) {
    importPreview = null;
    importSelected.clear();
    renderImportPreview();
    toast(`Preview failed: ${error.message}`, true);
  }
}

function setImportSelection(mode) {
  if (!importPreview) return;
  importSelected.clear();
  if (mode !== 'none') {
    for (const group of importPreview.groups) {
      for (const row of group.rows) {
        if (mode === 'all' || (mode === 'new' && !row.duplicate)) importSelected.add(row.id);
      }
    }
  }
  renderImportPreview();
}

async function commitImport() {
  if (!importPreview || !importSelected.size) return;
  const path = importPreview.path || $('[data-control="importPath"]')?.value?.trim();
  await runAction('import.commit', { path, ids: [...importSelected] }, 'import');
  importPreview = null;
  importSelected.clear();
  renderImportPreview();
  activePage = 'organize';
  renderAll();
}

function renderImportPreview() {
  const panel = $('[data-import-preview]');
  if (!panel) return;
  if (!importPreview) { panel.innerHTML = ''; return; }
  const { packageName, counts, groups } = importPreview;
  const n = importSelected.size;
  const allCollapsed = groups.length > 0 && groups.every((g) => importCollapsed.has(g.section));
  panel.innerHTML = html`
    <div class="import-head">
      <div class="import-meta">
        <strong>${escapeHtml(packageName)}</strong>
        <span class="subtle">${counts.total} sentences · <span class="ok">${counts.new} new</span> · <span class="dup">${counts.duplicate} already in library</span></span>
      </div>
      <div class="head-actions">
        <button class="btn secondary" type="button" data-action="${allCollapsed ? 'import-expand-all' : 'import-collapse-all'}">${allCollapsed ? 'Expand all' : 'Collapse all'}</button>
        <button class="btn secondary" type="button" data-action="import-select-new">Select new</button>
        <button class="btn secondary" type="button" data-action="import-select-all">All</button>
        <button class="btn secondary" type="button" data-action="import-select-none">None</button>
      </div>
    </div>
    ${groups.map(importGroup).join('')}
    <div class="import-commit-bar">
      <button class="btn primary" type="button" data-action="import-commit" ${n ? '' : 'disabled'}>Import ${n} selected →</button>
      <button class="btn secondary" type="button" data-action="import-clear-preview">Cancel</button>
    </div>
  `;

  // tri-state chapter checkboxes (indeterminate when partially selected)
  for (const box of $$('[data-import-group-select]')) {
    const group = importPreview.groups.find((g) => g.section === box.dataset.importGroupSelect);
    if (!group) continue;
    const selected = group.rows.filter((row) => importSelected.has(row.id)).length;
    box.indeterminate = selected > 0 && selected < group.count;
  }
}

function importGroup(group) {
  const collapsed = importCollapsed.has(group.section);
  const selected = group.rows.filter((row) => importSelected.has(row.id)).length;
  return html`
    <section class="sentence-section import-group ${collapsed ? 'is-collapsed' : ''}">
      <div class="import-group-head">
        <label class="group-check" title="Select whole chapter">
          <input type="checkbox" data-import-group-select="${escapeHtml(group.section)}" ${selected === group.count ? 'checked' : ''} />
        </label>
        <button class="group-toggle" type="button" data-action="import-toggle" data-section="${escapeHtml(group.section)}">
          <span class="chevron">▾</span>
          <span class="group-name">${escapeHtml(group.section)}</span>
          <span class="group-count">${selected}/${group.count}</span>
        </button>
      </div>
      <div class="import-group-rows">
        ${group.rows.map(importRow).join('')}
      </div>
    </section>
  `;
}

function importRow(row) {
  const selected = importSelected.has(row.id);
  return html`
    <article class="row import-row ${row.duplicate ? 'is-dup' : ''} ${selected ? 'is-selected' : ''}">
      <label class="select-box always" title="Select"><input type="checkbox" data-import-select="${escapeHtml(row.id)}" ${selected ? 'checked' : ''} /></label>
      <div class="row-main">
        <div class="target" lang="hi">${escapeHtml(row.target)}</div>
        <div class="sub-line"><span class="roman">${escapeHtml(row.romanisation || '')}</span>${row.romanisation && row.english ? '<span class="sep">·</span>' : ''}<span class="english">${escapeHtml(row.english || '')}</span></div>
      </div>
      <div class="row-meta">
        ${row.hasAudio ? '<span class="row-flag" title="Has audio">♪</span>' : ''}
        ${row.duplicate ? '<span class="chip wait">in library</span>' : '<span class="chip ready">new</span>'}
      </div>
    </article>
  `;
}

function bindEvents() {
  document.addEventListener('click', async (event) => {
    const pageLink = event.target.closest('[data-page-link]');
    if (pageLink) {
      activePage = pageLink.dataset.pageLink;
      renderAll();
      return;
    }
    const drawerOpen = event.target.closest('[data-drawer-open]');
    if (drawerOpen) {
      openDrawer(drawerOpen.dataset.drawerOpen);
      return;
    }
    if (event.target.closest('[data-drawer-close]') || event.target.matches('[data-drawer-backdrop]')) {
      closeDrawers();
      return;
    }
    const toggle = event.target.closest('[data-expand-toggle]');
    if (toggle && !event.target.closest('[data-action], [data-select-sentence], .handle, button, input, label')) {
      const id = toggle.dataset.expandToggle;
      expandedIds.has(id) ? expandedIds.delete(id) : expandedIds.add(id);
      renderAll();
      return;
    }
    const word = event.target.closest('[data-word-key]');
    if (word) {
      renderWordDetail(word.dataset.wordKey);
      return;
    }
    const copyCommand = event.target.closest('[data-copy-command]');
    if (copyCommand) {
      await copyText(copyCommand.dataset.copyCommand);
      return;
    }
    const action = event.target.closest('[data-action]');
    if (!action) return;
    await handleAction(action);
  });

  document.addEventListener('change', (event) => {
    const groupBox = event.target.closest('[data-import-group-select]');
    if (groupBox) {
      const group = importPreview?.groups.find((g) => g.section === groupBox.dataset.importGroupSelect);
      if (group) {
        const all = group.rows.every((row) => importSelected.has(row.id));
        for (const row of group.rows) all ? importSelected.delete(row.id) : importSelected.add(row.id);
      }
      renderImportPreview();
      return;
    }
    const importBox = event.target.closest('[data-import-select]');
    if (importBox) {
      importBox.checked ? importSelected.add(importBox.dataset.importSelect) : importSelected.delete(importBox.dataset.importSelect);
      renderImportPreview();
      return;
    }
    const checkbox = event.target.closest('[data-select-sentence]');
    if (checkbox) {
      checkbox.checked ? selectedIds.add(checkbox.dataset.selectSentence) : selectedIds.delete(checkbox.dataset.selectSentence);
      renderAll();
      return;
    }
    if (event.target.matches('[data-control]')) renderAll();
  });

  document.addEventListener('input', (event) => {
    if (event.target.matches('[data-control]')) renderAll();
  });

  // Grasp-style cross-highlight: hovering a token lights up its Devanagari,
  // romanisation, and gloss across the aligned stack and the breakdown table.
  const hoverToken = (event, on) => {
    const el = event.target.closest('[data-token]');
    if (!el) return;
    const scope = el.closest('[data-expand-for]');
    if (!scope) return;
    for (const node of scope.querySelectorAll(`[data-token="${el.dataset.token}"]`)) {
      node.classList.toggle('tok-hi', on);
    }
  };
  document.addEventListener('mouseover', (event) => hoverToken(event, true));
  document.addEventListener('mouseout', (event) => hoverToken(event, false));

  document.addEventListener('dragstart', (event) => {
    const row = event.target.closest('[data-sentence-id]');
    if (!row) return;
    draggedId = row.dataset.sentenceId;
    row.classList.add('is-dragging');
    event.dataTransfer.effectAllowed = 'move';
  });
  document.addEventListener('dragover', (event) => {
    if (event.target.closest('[data-sentence-id]')) event.preventDefault();
  });
  document.addEventListener('drop', (event) => {
    const row = event.target.closest('[data-sentence-id]');
    if (!row || !draggedId) return;
    event.preventDefault();
    reorderInMemory(draggedId, row.dataset.sentenceId);
    runAction('organize.reorder', { orderedIds: state.sentences.map((s) => s.id) }, 'organize.move');
    draggedId = null;
    renderAll();
  });
  document.addEventListener('dragend', () => {
    $$('.is-dragging').forEach((node) => node.classList.remove('is-dragging'));
    draggedId = null;
  });

  $('[data-settings-form]')?.addEventListener('submit', async (event) => {
    event.preventDefault();
    for (const control of $$('[data-setting]')) {
      setPath(state, control.dataset.setting, control.type === 'checkbox' ? control.checked : control.value);
    }
    syncSettingsControls();
    await runAction('config.save', { config: state.config, workspace: state.workspace }, 'settings');
    hydrateControls(); // reflect the saved title/language in the top bar
    closeDrawers();
    renderAll();
  });
}

async function handleAction(node) {
  const action = node.dataset.action;
  if (action === 'copy-active-cli') return copyText($('[data-active-cli]').textContent);
  if (action === 'copy-cli') {
    if (node.dataset.sentenceId) focusedId = node.dataset.sentenceId;
    const key = node.dataset.commandKey;
    const command = commandLibrary(state, currentContext())[key]?.command || commandForPage(activePage, state, currentContext());
    return copyText(command);
  }
  if (action === 'clear-selection') { selectedIds.clear(); renderAll(); return; }
  if (action === 'extract') return startProducer('extract', { rawText: $('[data-control="rawText"]')?.value, section: $('[data-control="section"]')?.value });
  if (action === 'enrich') return startProducer('enrich', { limit: Number($('[data-control="limit"]')?.value || 20), ids: [...selectedIds] });
  if (action === 'set-section') {
    if (!selectedIds.size) { toast('Select sentences first.', true); return; }
    const section = $('[data-bulk-section]')?.value?.trim() ?? '';
    await runAction('organize.set-section', { ids: [...selectedIds], section }, 'organize');
    selectedIds.clear();
    const input = $('[data-bulk-section]'); if (input) input.value = '';
    renderAll();
    return;
  }
  if (action === 'confirm-active') {
    const ids = node.dataset.sentenceId ? [node.dataset.sentenceId] : [...selectedIds];
    if (!ids.length) { toast('Select sentences to confirm.', true); return; }
    await runAction('organize.set-status', { ids, status: 'active' }, 'organize');
    selectedIds.clear();
    renderAll();
    return;
  }
  if (action === 'revert-enriched') {
    if (!node.dataset.sentenceId) return;
    return runAction('organize.set-status', { ids: [node.dataset.sentenceId], status: 'enriched' }, 'organize');
  }
  if (action === 'rename-section') {
    const current = node.dataset.section;
    const next = window.prompt('Rename subtitle / section', current === 'Unsectioned' ? '' : current);
    if (next == null) return;
    const trimmed = next.trim();
    if (trimmed === current) return;
    const inGroup = current === 'Unsectioned' ? (s) => !s.section : (s) => s.section === current;
    const ids = state.sentences.filter(inGroup).map((s) => s.id);
    return runAction('organize.set-section', { ids, section: trimmed }, 'organize');
  }
  if (action === 'copy-prompt') return copyText(producer?.prompt || '');
  if (action === 'apply-reply') return applyProducer();
  if (action === 'cancel-producer') { producer = null; renderProducer(); return; }
  if (action === 'reset-enrich') return runAction('enrich.reset', {}, 'generate.reset');
  if (action === 'import') return previewImport();
  if (action === 'import-commit') return commitImport();
  if (action === 'import-select-all') return setImportSelection('all');
  if (action === 'import-select-new') return setImportSelection('new');
  if (action === 'import-select-none') return setImportSelection('none');
  if (action === 'import-toggle') {
    const section = node.dataset.section;
    importCollapsed.has(section) ? importCollapsed.delete(section) : importCollapsed.add(section);
    renderImportPreview();
    return;
  }
  if (action === 'import-collapse-all') { for (const g of importPreview?.groups || []) importCollapsed.add(g.section); renderImportPreview(); return; }
  if (action === 'import-expand-all') { importCollapsed.clear(); renderImportPreview(); return; }
  if (action === 'import-clear-preview') { importPreview = null; importSelected.clear(); importCollapsed.clear(); renderImportPreview(); return; }
  if (action === 'audio-missing') return runAction('audio.missing', audioPayload(false), 'audio');
  if (action === 'audio-regenerate') return runAction('audio.force', audioPayload(true), 'audio.force');
  if (action === 'regen-one') return runAction('audio.force', { ...audioPayload(true), ids: [node.dataset.sentenceId] }, 'audio.force');
  if (action === 'anki-export') return runAction('anki.export', publishPayload(), 'anki');
  if (action === 'package-export') return runAction('package.export', publishPayload(), 'package');
  if (action === 'play-audio') return playAudio(node.dataset.audioPath);
}

function audioPayload(force) {
  return {
    ids: [...selectedIds],
    force,
    backend: $('[data-control="audioBackend"]')?.value || state.config.audio.backend,
    voice: $('[data-control="audioVoice"]')?.value || state.config.audio.voice,
  };
}

function publishPayload() {
  return {
    ids: [...selectedIds],
    deck: $('[data-control="ankiDeck"]')?.value,
    replace: $('[data-control="ankiReplace"]')?.checked,
    destination: $('[data-control="packageDest"]')?.value,
    format: $('[data-control="packageFormat"]')?.value,
  };
}

function syncSettingsControls() {
  $('[data-control="audioBackend"]').value = state.config.audio.backend;
  $('[data-control="audioVoice"]').value = state.config.audio.voice || '';
  $('[data-control="ankiDeck"]').value = state.config.anki.deck || '';
  $('[data-control="packageDest"]').value = state.config.package.destination || '';
  $('[data-control="packageFormat"]').value = state.config.package.format || 'json';
}

function reorderInMemory(fromId, toId) {
  if (fromId === toId) return;
  const rows = [...state.sentences].sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
  const fromIndex = rows.findIndex((row) => row.id === fromId);
  const toIndex = rows.findIndex((row) => row.id === toId);
  if (fromIndex < 0 || toIndex < 0) return;
  const [moved] = rows.splice(fromIndex, 1);
  rows.splice(toIndex, 0, moved);
  rows.forEach((row, index) => { row.order = index + 1; });
  state.sentences = rows;
}

function openDrawer(name) {
  renderCli();
  $('[data-drawer-backdrop]').hidden = false;
  for (const drawer of $$('[data-drawer]')) drawer.hidden = drawer.dataset.drawer !== name;
}

function closeDrawers() {
  $('[data-drawer-backdrop]').hidden = true;
  for (const drawer of $$('[data-drawer]')) drawer.hidden = true;
}

async function copyText(value) {
  try {
    await navigator.clipboard.writeText(value);
    toast('Copied to clipboard');
  } catch {
    toast(value, false);
  }
}

function toast(message, error = false) {
  const node = $('[data-toast]');
  if (!node) return;
  node.textContent = message;
  node.classList.toggle('error', error);
  node.hidden = false;
  clearTimeout(toast.timer);
  toast.timer = setTimeout(() => { node.hidden = true; }, 4200);
}

function playAudio(path) {
  if (!path) return toast('No audio file is attached to this sentence.', true);
  const audio = new Audio(path);
  audio.play().catch((error) => toast(`Could not play audio: ${error.message}`, true));
}

function getPath(object, path) {
  return String(path).split('.').reduce((value, key) => value?.[key], object);
}

function setPath(object, path, value) {
  const parts = String(path).split('.');
  const last = parts.pop();
  let target = object;
  for (const part of parts) target = target[part] ||= {};
  target[last] = value;
}

bindEvents();
loadState();

export const privateForTests = { escapeHtml, reorderInMemory };
