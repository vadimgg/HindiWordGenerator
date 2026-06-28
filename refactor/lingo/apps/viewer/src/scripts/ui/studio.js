/**
 * Generation Studio — client-side controller (live, API-driven).
 *
 * Responsible for: rendering and driving the Studio tab against the Rust viewer
 * server's `/api/studio/*` endpoints via scripts/studio/api.js. It mirrors the
 * CLI pipeline raw → import → build → check → audio and preserves the manual
 * packet loop: copy packet → paste the model reply → Apply (validated server-side
 * before anything is written).
 *
 * Data flow: `state.batches` comes from GET /status; per-batch detail (sentences,
 * cards, raw text) is loaded lazily from GET /batch and cached in `state.detail`.
 * The packet for the active import/build stage lives in `state.exchange`.
 *
 * Dependencies: studio/api.js. Wired from main.js via initStudio().
 */
// Responsible for: Studio tab rendering + live interactions over /api/studio

import { api, ApiError } from '../studio/api.js';

const STAGES = ['raw', 'import', 'build', 'check', 'audio'];
const STAGE_LABELS = { raw: 'Raw', import: 'Import', build: 'Build', check: 'Check', audio: 'Audio' };
const DOT = { done: '●', partial: '◐', next: '○', wait: '–' };

/** @type {any} */
let state;

/* ── lifecycle ───────────────────────────────────────────────────────────── */

export function initStudio() {
  const root = document.getElementById('page-studio');
  if (!root) return;

  state = {
    workspace: null,
    batches: [],
    detail: {},          // id -> { title, subtitle, raw, rawId, sentences, cards }
    checkReports: {},    // id -> check response
    selectedId: null,
    stage: 'raw',
    creating: false,
    problemsOnly: false,
    forceExchange: false,
    exchange: null,      // { kind, batch, runId, packet }
    exchangeError: null,
    config: null,
    voices: null,
    voiceTier: null,
    rawFiles: null,
    audioRunning: false,
    loadError: null,
  };

  root.addEventListener('click', onClick);
  root.addEventListener('change', onChange);
  root.addEventListener('keydown', onKeydown);
  root.addEventListener('dragover', onDragover);
  root.addEventListener('dragleave', onDragleave);
  root.addEventListener('drop', onDrop);

  renderShell('Loading workspace…');
  refresh(true).catch(reportFatal);
}

/* ── data loading ────────────────────────────────────────────────────────── */

async function refresh(selectDefault) {
  try {
    const status = await api.status();
    state.workspace = status.workspace;
    state.batches = status.batches;
    state.loadError = null;
    if (selectDefault || !getSelected()) {
      const target = state.batches.find(b => currentStage(b)) || state.batches[0];
      if (target) {
        state.selectedId = target.id;
        state.creating = false;
        await loadDetail(target.id);
        state.stage = currentStage(target) || 'audio';
      }
    }
    if (!state.config) state.config = await api.getConfig().catch(() => null);
  } catch (error) {
    state.loadError = describe(error);
  }
  renderAll();
}

async function loadDetail(id) {
  try {
    const detail = await api.batch(id);
    state.detail[id] = detail;
    const batch = state.batches.find(b => b.id === id);
    if (batch && detail.rawId) batch.rawId = detail.rawId;
  } catch {
    state.detail[id] = { sentences: [], cards: [], raw: '', title: humanize(id), subtitle: '' };
  }
}

/* ── derived state ───────────────────────────────────────────────────────── */

const getSelected = () => state.batches.find(b => b.id === state.selectedId) || null;
const detailOf = (id) => state.detail[id] || { sentences: [], cards: [], raw: '', title: humanize(id || ''), subtitle: '' };
const currentStage = (b) => STAGES.find(s => b.stages[s] !== 'done') || null;
const hasProblem = (b) => Boolean(b.problems && (b.problems.check || b.problems.audio));

function reachable(b, stage) {
  const i = STAGES.indexOf(stage);
  return STAGES.slice(0, i).every(s => b.stages[s] === 'done');
}

function summary(b) {
  const cur = currentStage(b);
  if (!cur) return { text: '✓ done', action: null };
  if (cur === 'audio') {
    const text = b.stages.audio === 'partial' ? `${b.audio.done}/${b.audio.total} audio` : 'needs audio';
    return { text, action: { label: 'Resume', stage: 'audio' } };
  }
  const map = { raw: ['needs text', 'Add'], import: ['needs import', 'Import'], build: ['needs build', 'Build'], check: ['needs check', 'Check'] };
  const [text, label] = map[cur];
  return { text, action: { label, stage: cur } };
}

/* ── top-level render ────────────────────────────────────────────────────── */

function renderShell(message) {
  const stage = document.getElementById('st-stage');
  document.getElementById('st-rail').innerHTML = '';
  document.getElementById('st-stepper').innerHTML = '';
  if (stage) stage.innerHTML = `<div class="st-empty">${esc(message)}</div>`;
}

function renderAll() {
  if (state.loadError) {
    document.getElementById('st-rail').innerHTML = '';
    document.getElementById('st-stepper').innerHTML = '';
    document.getElementById('st-stage').innerHTML =
      `<section class="st-panel"><h2>Studio unavailable</h2>
       <p class="st-sub">${esc(state.loadError)}</p>
       <p class="st-note">The Studio talks to the local <code>lingo viewer</code> server. If you opened a static build, run <code>lingo viewer</code> from inside a deck instead.</p>
       <div class="st-row"><button class="st-btn st-primary" data-action="retry">Retry</button></div></section>`;
    return;
  }
  renderRail();
  renderStepper();
  renderStage();
}

function renderRail() {
  const host = document.getElementById('st-rail');
  const ws = state.workspace || { name: '…', language: '…', batches: 0, sentences: 0 };
  const list = state.problemsOnly ? state.batches.filter(hasProblem) : state.batches;
  const cols = STAGES.length + 2;

  const rows = list.map(b => {
    const sel = b.id === state.selectedId ? ' is-selected' : '';
    const dots = STAGES.map(s => `<td><span class="st-dot ${b.stages[s]}">${DOT[b.stages[s]]}</span></td>`).join('');
    const s = summary(b);
    const quick = s.action
      ? `<button class="st-quick" data-action="quick" data-batch="${esc(b.id)}" data-stage="${s.action.stage}">▸ ${s.action.label}</button>` : '';
    return `<tr class="st-rail-row${sel}" data-action="select-batch" data-batch="${esc(b.id)}">
      <td>${esc(b.id)}</td>${dots}<td class="st-summary">${s.text}${quick}</td></tr>`;
  }).join('');

  host.innerHTML = `
    <div class="st-rail-head">
      <div class="st-rail-ws">Workspace <strong>${esc(ws.name)}</strong> · ${esc(ws.language)} · ${ws.batches} batches · ${ws.sentences} sentences</div>
      <div class="st-rail-tools">
        <button class="st-chip-btn${state.problemsOnly ? ' is-active' : ''}" data-action="toggle-problems">◇ Problems only</button>
        <button class="st-chip-btn" data-action="open-settings">⚙ Settings</button>
      </div>
    </div>
    <table class="st-rail-table">
      <thead><tr><th>Batch</th>${STAGES.map(s => `<th>${STAGE_LABELS[s]}</th>`).join('')}<th></th></tr></thead>
      <tbody>
        ${rows || `<tr><td colspan="${cols}" style="text-align:center;color:#475569;padding:1rem">${state.problemsOnly ? 'No batches with problems — all green.' : 'No batches yet. Start one below.'}</td></tr>`}
        <tr class="st-rail-new" data-action="new-batch"><td colspan="${cols}">+ New batch from raw text…</td></tr>
      </tbody>
    </table>`;
}

function renderStepper() {
  const host = document.getElementById('st-stepper');
  const b = getSelected();
  if (state.creating || !b) { host.innerHTML = ''; return; }
  host.innerHTML = STAGES.map((s, i) => {
    const st = b.stages[s];
    const active = s === state.stage;
    const locked = !reachable(b, s);
    const cls = ['st-step', active ? 'is-active' : '', st === 'done' && !active ? 'is-done' : '', locked ? 'is-locked' : ''].filter(Boolean).join(' ');
    return `<button class="${cls}" role="tab" aria-selected="${active}" data-action="nav-stage" data-stage="${s}"${locked ? ' aria-disabled="true"' : ''}>
      <span class="st-step-n">${i + 1}</span>${STAGE_LABELS[s]}</button>`;
  }).join('<span class="st-step-sep">──</span>');
}

function renderStage() {
  const host = document.getElementById('st-stage');
  if (state.creating) { host.innerHTML = stageRaw(null); focusHeading(host); return; }
  const b = getSelected();
  if (!b) {
    host.innerHTML = `<section class="st-panel"><h2>Studio</h2>
      <p class="st-sub">No batch selected. Pick one above or start a new batch from raw text.</p>
      <div class="st-row"><button class="st-btn st-primary" data-action="new-batch">+ New batch</button></div></section>`;
    return;
  }
  const fn = { raw: stageRaw, import: stageImport, build: stageBuild, check: stageCheck, audio: stageAudio }[state.stage];
  host.innerHTML = fn(b);
  focusHeading(host);
}

function focusHeading(host) {
  const h = host.querySelector('h2');
  if (h) h.setAttribute('tabindex', '-1');
}

/* ── stage 0 · raw ───────────────────────────────────────────────────────── */

function stageRaw(b) {
  const d = b ? detailOf(b.id) : { raw: '', title: '', subtitle: '' };
  const id = b ? b.id : '';
  const imported = b && b.stages.import === 'done';
  const label = imported ? 'Save &amp; Re-import →' : 'Save &amp; Import →';
  return `<section class="st-panel">
    <h2>${imported ? 'Raw source · ' + esc(id) : 'New batch · raw text'}</h2>
    <p class="st-sub">Paste source material, drop a file, or pick one already under <code>raw/</code>. This convenience writes <code>raw/&lt;batch&gt;.md</code> for you — the one step the CLI can't.</p>
    ${cliHint(`lingo import ${id ? id : '<raw-file>'}`)}
    <div class="st-modes">
      <button class="st-mode is-active" data-action="raw-mode" data-mode="paste">Paste it</button>
      <button class="st-mode" data-action="raw-mode" data-mode="drop">Drop a file</button>
      <button class="st-mode" data-action="raw-mode" data-mode="pick">Pick from raw/</button>
    </div>
    <input class="st-hidden" id="st-raw-file" type="file" accept=".md,.markdown,.txt,.yaml,.yml,text/markdown,text/plain">
    <div id="st-raw-picker"></div>
    <label class="st-field"><span class="st-cap">Raw text</span>
      <textarea class="st-textarea" id="st-raw-text" style="min-height:160px" dir="auto">${esc(d.raw || '')}</textarea></label>
    <label class="st-field"><span class="st-cap">Batch id · --batch</span>
      <input class="st-input" id="st-batch-id" type="text" value="${esc(id)}" placeholder="auto from text">
      <div class="st-hint">auto from filename · editable</div></label>
    <label class="st-field"><span class="st-cap">Title · --title</span>
      <input class="st-input" id="st-title" type="text" value="${esc(d.title || '')}" placeholder="optional"></label>
    <label class="st-field"><span class="st-cap">Subtitle · --subtitle</span>
      <input class="st-input" id="st-subtitle" type="text" value="${esc(d.subtitle || '')}" placeholder="optional"></label>
    <div class="st-row">
      <span class="st-spacer"></span>
      ${b ? '' : '<button class="st-btn" data-action="cancel-new">Cancel</button>'}
      ${imported ? '<button class="st-btn" data-action="save-meta">Save title &amp; subtitle</button>' : ''}
      <button class="st-btn st-primary" data-action="save-raw">${label}</button>
    </div>
    ${imported ? '<p class="st-note">“Save title &amp; subtitle” updates this batch\'s metadata in place — no re-import or rebuild needed.</p>' : ''}
  </section>`;
}

/* ── stage 1 · import ────────────────────────────────────────────────────── */

function stageImport(b) {
  const showReview = b.stages.import === 'done' && !state.forceExchange;
  const body = showReview ? importReview(b) : packetExchange('import', b);
  return `<section class="st-panel">
    <h2>Import · ${esc(b.id)}</h2>
    <p class="st-sub">Raw text → reviewed source YAML.</p>
    ${cliHint(`lingo import --batch ${b.id}`)}
    ${body}
    <p class="st-note">Lingo validates the reply on the server; nothing is written to <code>input/</code> until it passes. Markdown fences are stripped automatically.</p>
  </section>`;
}

function importReview(b) {
  const d = detailOf(b.id);
  const cards = (d.sentences || []).map(s => `<div class="st-rcard">
    <div class="st-hi">${esc(s.target)}</div>
    ${s.romanisation ? `<div class="st-rom">${esc(s.romanisation)}</div>` : ''}
    <div class="st-en">${esc(s.english)}</div>
    <div class="st-tags">${(s.tags || []).map(t => `<span class="st-tag">${esc(t)}</span>`).join('')}</div></div>`).join('');
  return `<div class="st-review">${cards}
    <div class="st-row">
      <span class="st-ok">✓ ${(d.sentences || []).length} sentences in input/sentences/${esc(b.id)}.yaml</span>
      <span class="st-spacer"></span>
      <button class="st-btn" data-action="reopen" data-stage="import">Re-import</button>
      <button class="st-btn st-primary" data-action="nav-stage" data-stage="build">Build →</button>
    </div></div>`;
}

/* ── stage 2 · build ─────────────────────────────────────────────────────── */

function stageBuild(b) {
  const showReview = b.stages.build === 'done' && !state.forceExchange;
  const body = showReview ? buildReview(b) : packetExchange('build', b);
  return `<section class="st-panel">
    <h2>Build · ${esc(b.id)}</h2>
    <p class="st-sub">Reviewed sentences → enriched card JSON.</p>
    ${cliHint(`lingo build --batch ${b.id}`)}
    ${body}
    <p class="st-note">Build replies also pass deterministic <code>check_card_batch</code> on the server before they are accepted.</p>
  </section>`;
}

function buildReview(b) {
  const d = detailOf(b.id);
  const cards = (d.cards || []).map(reviewCard).join('');
  return `<div class="st-review">${cards}
    <div class="st-row">
      <span class="st-ok">✓ ${(d.cards || []).length} cards in output/sentences/${esc(b.id)}.json</span>
      <span class="st-spacer"></span>
      <button class="st-btn" data-action="reopen" data-stage="build">Re-build</button>
      <button class="st-btn st-primary" data-action="nav-stage" data-stage="check">Check →</button>
    </div></div>`;
}

/** A single enriched card, styled to echo the Sentences tab cards. */
function reviewCard(c) {
  const words = Array.isArray(c.words) ? c.words : [];
  return `<article class="st-card">
    <div class="st-card-head">
      <div class="st-card-lead">
        <h3 class="st-card-deva" lang="hi">${esc(c.target)}</h3>
        ${c.romanisation ? `<p class="st-card-rom">${esc(c.romanisation)}</p>` : ''}
        <div class="st-card-meta">
          <span class="st-card-en">${esc(c.english)}</span>
          ${c.register ? `<span class="st-reg">${esc(c.register)}</span>` : ''}
          ${words.length ? `<span class="st-wcount">${words.length} ${words.length === 1 ? 'word' : 'words'}</span>` : ''}
        </div>
      </div>
      ${c.audioUrl ? `<button class="st-play" data-action="play" data-audio="${esc(c.audioUrl)}">▶ Play</button>` : ''}
    </div>
    ${c.literal ? `<div class="st-card-literal"><span class="st-card-lbl">Literal</span>${esc(c.literal)}</div>` : ''}
    ${words.length ? `<div class="st-card-words">
      <div class="st-card-lbl st-card-wlbl">Word by word</div>
      <div class="st-wlist">${words.map(reviewWord).join('')}</div>
    </div>` : ''}
  </article>`;
}

function reviewWord(w) {
  const { gloss, detail } = splitMeaning(w.meaning || '');
  return `<div class="st-word">
    <div class="st-word-lex">
      <span class="st-word-deva" lang="hi">${esc(w.target)}</span>
      ${w.romanisation ? `<span class="st-word-rom">${esc(w.romanisation)}</span>` : ''}
    </div>
    <div class="st-word-body">
      <div class="st-word-head">
        <span class="st-word-gloss">${esc(gloss || w.meaning)}</span>
        ${w.kind ? `<span class="st-word-kind">${esc(w.kind)}</span>` : ''}
      </div>
      ${detail ? `<p class="st-word-detail">${esc(detail)}</p>` : ''}
    </div>
  </div>`;
}

// Split "gloss — explanation" into two parts on the first em/en dash (or a
// spaced hyphen), mirroring the Sentences renderer so wording stays consistent.
function splitMeaning(meaning) {
  const text = String(meaning ?? '').trim();
  const match = text.match(/\s*[—–]\s*|\s+-\s+/);
  if (!match) return { gloss: text, detail: '' };
  const at = match.index ?? -1;
  return { gloss: text.slice(0, at).trim(), detail: text.slice(at + match[0].length).trim() };
}

/* ── packet exchange ─────────────────────────────────────────────────────── */

function packetExchange(kind, b) {
  if (state.exchangeError) {
    return `<div class="st-problems"><h4>Could not prepare the packet</h4><ul><li>${esc(state.exchangeError)}</li></ul></div>
      <div class="st-row"><button class="st-btn st-primary" data-action="reopen" data-stage="${kind}">Try again</button></div>`;
  }
  const ready = state.exchange && state.exchange.kind === kind && state.exchange.batch === b.id;
  if (!ready) {
    // Kick off preparation, then re-render when it resolves.
    ensureExchange(kind).then(() => { if (state.stage === kind) renderStage(); });
    return `<div class="st-empty">Preparing the ${kind} packet…</div>`;
  }
  const fmt = kind === 'import' ? 'YAML' : 'JSON';
  return `<div class="st-pe-steps">
      <span><b>①</b> Copy the packet</span>·
      <span><b>②</b> Paste into ChatGPT / Claude, copy the reply</span>·
      <span><b>③</b> Paste the reply &amp; Apply</span></div>
    <div class="st-panes">
      <div class="st-pane">
        <div class="st-pane-head"><span>Packet (read-only) · ${fmt} expected</span><span>${kind} prompt</span></div>
        <pre class="st-packet" id="st-packet-${kind}" tabindex="0">${esc(state.exchange.packet)}</pre>
        <div class="st-row" style="margin-top:.6rem">
          <button class="st-btn st-primary" data-action="copy" data-target="st-packet-${kind}">⧉ Copy packet</button>
          <button class="st-btn" data-action="download" data-kind="${kind}">Download</button>
        </div>
      </div>
      <div class="st-pane">
        <div class="st-pane-head"><span>Reply (paste here)</span></div>
        <textarea class="st-textarea" id="st-reply-${kind}" dir="auto"
          placeholder="⌁ paste the model's ${fmt} reply, then Apply"></textarea>
        <div class="st-row" style="margin-top:.6rem">
          <span class="st-spacer"></span>
          <button class="st-btn st-primary" data-action="apply" data-kind="${kind}">Apply →</button>
        </div>
      </div>
    </div>
    <div id="st-problems-${kind}"></div>`;
}

/* ── stage 3 · check ─────────────────────────────────────────────────────── */

function stageCheck(b) {
  const report = state.checkReports[b.id];
  const done = b.stages.check === 'done';
  let body;
  if (report) {
    const batch = report.batches.find(x => x.batch === b.id) || report.batches[0];
    body = renderCheckReport(batch);
  } else if (done) {
    body = `<div class="st-report"><div class="st-ok">✓ this batch passed validation</div>
      ${b.stages.audio !== 'done' ? `<div class="st-warn">⚠ ${b.audio.total - b.audio.done} cards missing audio — resolved in Stage 5</div>` : ''}</div>`;
  } else {
    body = `<div class="st-empty">Run the deterministic checks for this batch.</div>`;
  }
  const actions = done || report
    ? `<button class="st-btn" data-action="run-check">Re-run check</button>
       <span class="st-spacer"></span>
       <button class="st-btn st-primary" data-action="nav-stage" data-stage="audio">Audio →</button>`
    : `<button class="st-btn st-primary" data-action="run-check">Run check ✓</button>`;
  return `<section class="st-panel">
    <h2>Check · ${esc(b.id)}</h2>
    <p class="st-sub">Deterministic validation — no model involved.</p>
    ${cliHint(`lingo check --batch ${b.id}`)}
    ${body}
    <div class="st-row">${actions}</div>
  </section>`;
}

function renderCheckReport(batch) {
  if (!batch) return `<div class="st-empty">Nothing to check yet.</div>`;
  const lines = [];
  if (batch.clean && batch.warnings === 0) lines.push(`<div class="st-ok">✓ ${batch.errors === 0 ? 'all cards valid' : ''} — no problems</div>`);
  if (batch.errors > 0) lines.push(`<div class="st-err">✘ ${batch.errors} error(s)</div>`);
  (batch.diagnostics || []).forEach(d => {
    const cls = d.severity === 'error' ? 'st-err' : 'st-warn';
    const glyph = d.severity === 'error' ? '✘' : '⚠';
    lines.push(`<div class="${cls}">${glyph} ${esc(d.message)}</div>`);
  });
  if (lines.length === 0) lines.push(`<div class="st-ok">✓ no problems</div>`);
  return `<div class="st-report">${lines.join('')}</div>`;
}

/* ── stage 4 · audio ─────────────────────────────────────────────────────── */

function stageAudio(b) {
  const isDone = b.stages.audio === 'done';
  const pct = b.audio.total ? Math.round(b.audio.done / b.audio.total * 100) : 0;
  const missing = b.audio.total - b.audio.done;
  const backend = (state.config && state.config.audio && state.config.audio.backend) || 'gtts';
  const el = backend === 'elevenlabs';
  const keyPresent = state.config && state.config.audio && state.config.audio.keyPresent;
  const keyEnv = (state.config && state.config.audio && state.config.audio.keyEnv) || 'ELEVENLABS_API_KEY';
  const cards = detailOf(b.id).cards || [];
  const missingCards = cards.filter(c => !c.audioUrl).length;
  const done = isDone ? `<div class="st-done">
      <div class="st-done-title">${esc(b.id)} is ready</div>
      <div class="st-en">${b.audio.total} clips · checks green</div>
      <div class="st-row">
        <button class="st-btn" data-action="goto-sentences">View in Sentences</button>
        <button class="st-btn" data-action="export">Export to Anki…</button>
      </div></div>` : '';
  return `<section class="st-panel">
    <h2>Audio · ${esc(b.id)}</h2>
    <p class="st-sub">Generate missing clips, or pick sentences to re-record with your chosen voice.</p>
    ${cliHint(audioCommand({ batch: b.id, backend, voice: undefined, force: false }), 'st-audio-cli')}
    <div class="st-drawer-gl" style="color:#94a3b8;font-size:11px">Backend · --backend</div>
    <label class="st-radio"><input type="radio" name="st-backend" value="gtts"${el ? '' : ' checked'}> gTTS <span class="st-hint">(default · free)</span></label>
    <label class="st-radio"><input type="radio" name="st-backend" value="elevenlabs"${el ? ' checked' : ''}> ElevenLabs</label>
    <div class="st-voicewrap${el ? '' : ' st-hidden'}" id="st-voicewrap">${voiceWrapInner()}</div>
    ${elevenlabsHelp(el, keyEnv, keyPresent)}
    ${audioCardPicker(cards)}
    <p class="st-sub" id="st-audio-status" style="margin:.9rem 0 .2rem">${isDone ? `${b.audio.total} of ${b.audio.total} clips synthesized · check green` : `${missing} of ${b.audio.total} clips missing`}</p>
    <div class="st-progress"><i id="st-audio-bar" style="width:${pct}%"></i></div>
    <div class="st-row">
      <button class="st-btn" id="st-gen-missing" data-action="gen-missing"${missingCards && !state.audioRunning ? '' : ' disabled'}>Generate missing${missingCards ? ` · ${missingCards}` : ''}</button>
      <span class="st-spacer"></span>
      <button class="st-btn st-primary" id="st-regen-sel" data-action="regen-selected" disabled>Regenerate selected</button>
    </div>
    ${done}
  </section>`;
}

/** Per-sentence selection list for targeted audio (re)generation. */
function audioCardPicker(cards) {
  if (!cards.length) return '';
  const rows = cards.map(c => `<div class="st-audio-pick-row">
    <label class="st-audio-pick">
      <input type="checkbox" class="st-audio-card" data-id="${esc(c.id)}">
      <span class="st-audio-pick-deva" lang="hi">${esc(c.target)}</span>
      <span class="st-audio-pick-rom">${esc(c.romanisation || '')}</span>
      <span class="st-spacer"></span>
      <span class="st-audio-pick-state ${c.audioUrl ? 'is-ok' : 'is-missing'}">${c.audioUrl ? '♪ audio' : 'no audio'}</span>
    </label>
    ${c.audioUrl ? `<button type="button" class="st-play" data-action="play" data-audio="${esc(c.audioUrl)}">▶</button>` : ''}
  </div>`).join('');
  return `<div class="st-audio-picker">
    <div class="st-audio-pick-head">
      <label class="st-check"><input type="checkbox" id="st-audio-all"> Select all</label>
      <span class="st-spacer"></span>
      <span class="st-hint" id="st-audio-selcount">0 selected</span>
    </div>
    <div class="st-audio-picklist">${rows}</div>
  </div>`;
}

/** Compose the `lingo audio …` command that matches the current control state. */
function audioCommand({ batch, backend, voice, force }) {
  let cmd = `lingo audio --batch ${batch}`;
  if (backend === 'elevenlabs') cmd += ' --backend elevenlabs';
  if (voice) cmd += ` --voice ${voice}`;
  if (force) cmd += ' --force';
  return cmd;
}

/** Re-render the Audio CLI chip from the live control state. */
function updateAudioCli() {
  const b = getSelected();
  const code = document.querySelector('#st-audio-cli code');
  if (!b || !code) return;
  const backend = (document.querySelector('input[name="st-backend"]:checked') || {}).value;
  const forRun = document.getElementById('st-voice-forrun')?.checked;
  const voice = backend === 'elevenlabs' && forRun ? document.getElementById('st-voice-select')?.value : undefined;
  code.textContent = audioCommand({ batch: b.id, backend, voice, force: false });
}

/** Collapsible "set up ElevenLabs" panel — paste a key, stored locally. */
function elevenlabsHelp(open, keyEnv, keyPresent) {
  const env = esc(keyEnv);
  return `<details class="st-help${open ? '' : ' st-hidden'}" id="st-el-help"${open ? ' open' : ''}>
    <summary class="st-help-sum">
      <span class="st-help-q">?</span>
      <span class="st-help-title">ElevenLabs setup</span>
      <span class="st-help-sub">Paste your API key — stored locally, no restart needed</span>
      <span class="st-help-chev">▾</span>
    </summary>
    <div class="st-help-body">
      <label class="st-cap" for="st-el-key">API key</label>
      <div class="st-el-keyrow">
        <input class="st-input" id="st-el-key" type="password" autocomplete="off" spellcheck="false"
          placeholder="${keyPresent ? '•••••••••••• saved — paste a new key to replace' : 'sk_…'}">
        <button class="st-btn st-primary" data-action="save-el-key">Save</button>
      </div>
      <div id="st-el-keystatus" class="st-el-keystatus ${keyPresent ? 'st-keyok' : 'st-keyno'}">
        ${keyPresent ? '● Key configured — voices will load above' : '○ No key yet — Lingo falls back to gTTS'}
      </div>
      <ol class="st-help-steps">
        <li>Sign in at <strong>elevenlabs.io</strong> → profile menu → <strong>API Keys</strong>, then generate a key.</li>
        <li>Paste it above and press <strong>Save</strong>. It is written to <code>.lingo.secrets.toml</code> (gitignored), never to <code>config.toml</code>, and takes effect right away.</li>
      </ol>
      <p class="st-hint">Prefer environment variables? Export <code>$${env}</code> before launching <code>lingo viewer</code> instead — that always takes precedence.</p>
    </div>
  </details>`;
}

/* ── event handling ──────────────────────────────────────────────────────── */

function onClick(e) {
  const t = e.target.closest('[data-action]');
  if (!t) return;
  const a = t.dataset.action;
  switch (a) {
    case 'retry': refresh(true).catch(reportFatal); break;
    case 'select-batch': selectBatch(t.dataset.batch); break;
    case 'quick': selectBatch(t.dataset.batch, t.dataset.stage); break;
    case 'new-batch': startNew(); break;
    case 'cancel-new': cancelNew(); break;
    case 'toggle-problems': state.problemsOnly = !state.problemsOnly; renderRail(); break;
    case 'nav-stage': goStage(t.dataset.stage); break;
    case 'raw-mode': pickRawMode(t); break;
    case 'save-raw': saveRaw(); break;
    case 'save-meta': saveMeta(); break;
    case 'copy': copyPacket(t.dataset.target, t); break;
    case 'download': downloadPacket(t.dataset.kind); break;
    case 'apply': applyReply(t.dataset.kind); break;
    case 'reopen': state.forceExchange = true; state.exchange = null; goStage(t.dataset.stage, true); break;
    case 'run-check': runCheck(); break;
    case 'gen-missing': generateAudio({ cards: [], force: false }); break;
    case 'regen-selected': regenSelected(); break;
    case 'save-el-key': saveElevenLabsKey(); break;
    case 'play': playAudio(t.dataset.audio); break;
    case 'goto-sentences': document.getElementById('nav-sentences')?.click(); break;
    case 'export': toast('Hand-off → Deliver: lingo export'); break;
    case 'open-settings': openSettings(); break;
    case 'close-settings': closeSettings(); break;
    case 'save-settings': saveSettings(); break;
    case 'seg': segSelect(t); break;
    case 'select-raw': selectRawFile(t.dataset.raw); break;
    default: break;
  }
}

function onChange(e) {
  const t = e.target;
  if (t.name === 'st-backend') {
    const elevenlabs = t.value === 'elevenlabs';
    document.getElementById('st-voicewrap')?.classList.toggle('st-hidden', !elevenlabs);
    const help = document.getElementById('st-el-help');
    if (help) { help.classList.toggle('st-hidden', !elevenlabs); help.open = elevenlabs; }
    if (elevenlabs && !state.voices) loadVoices();
  }
  if (t.name === 'st-backend' || t.id === 'st-voice-forrun' || t.id === 'st-voice-select') {
    updateAudioCli();
  }
  if (t.id === 'st-audio-all') {
    document.querySelectorAll('.st-audio-card').forEach(cb => { cb.checked = t.checked; });
    updateAudioSelection();
  }
  if (t.classList && t.classList.contains('st-audio-card')) {
    updateAudioSelection();
  }
  if (t.id === 'st-raw-file' && t.files?.[0]) {
    readFileIntoRaw(t.files[0]);
    t.value = '';
  }
}

function onKeydown(e) {
  if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
    if (document.getElementById('st-reply-import')) { e.preventDefault(); applyReply('import'); }
    else if (document.getElementById('st-reply-build')) { e.preventDefault(); applyReply('build'); }
  }
}

function onDragover(e) {
  if (!document.getElementById('st-raw-text')) return;
  e.preventDefault();
  document.getElementById('st-raw-text')?.classList.add('is-drop');
}

function onDragleave(e) {
  if (!document.getElementById('st-raw-text')) return;
  if (!e.currentTarget.contains(e.relatedTarget)) {
    document.getElementById('st-raw-text')?.classList.remove('is-drop');
  }
}

function onDrop(e) {
  const textarea = document.getElementById('st-raw-text');
  if (!textarea) return;
  e.preventDefault();
  textarea.classList.remove('is-drop');
  const file = e.dataTransfer?.files?.[0];
  if (file) readFileIntoRaw(file);
}

/* ── navigation ──────────────────────────────────────────────────────────── */

async function selectBatch(id, stage) {
  state.selectedId = id;
  state.creating = false;
  state.forceExchange = false;
  state.exchange = null;
  if (!state.detail[id]) await loadDetail(id);
  const b = getSelected();
  state.stage = stage && reachable(b, stage) ? stage : (currentStage(b) || 'audio');
  renderAll();
}

async function goStage(stage, keepForce) {
  const b = getSelected();
  if (!b) return;
  if (!reachable(b, stage)) { toast('Finish the earlier stages first'); return; }
  if (!keepForce) { state.forceExchange = false; }
  state.stage = stage;
  if (!state.detail[b.id]) await loadDetail(b.id);
  renderAll();
  document.getElementById('st-stage')?.scrollIntoView({ behavior: 'smooth', block: 'start' });
}

function startNew() {
  state.creating = true;
  state.selectedId = null;
  state.stage = 'raw';
  renderAll();
  toast('Paste your raw text, then Save & Import');
}

function cancelNew() {
  state.creating = false;
  refresh(true).catch(reportFatal);
}

/* ── raw stage actions ───────────────────────────────────────────────────── */

function pickRawMode(btn) {
  btn.parentElement.querySelectorAll('.st-mode').forEach(m => m.classList.toggle('is-active', m === btn));
  if (btn.dataset.mode === 'drop') {
    const picker = document.getElementById('st-raw-picker');
    if (picker) picker.innerHTML = '';
    document.getElementById('st-raw-file')?.click();
  }
  if (btn.dataset.mode === 'pick') showRawPicker();
}

async function showRawPicker() {
  try {
    const { files } = await api.listRaw();
    state.rawFiles = files;
    const host = document.getElementById('st-raw-picker');
    if (!host) return;
    if (!files.length) {
      host.innerHTML = `<div class="st-raw-pick"><div class="st-hint">No files under raw/ yet.</div></div>`;
      return;
    }
    host.innerHTML = `<div class="st-raw-pick" role="listbox" aria-label="Raw files">
      ${files.map(file => {
        const batch = file.batch || file.id;
        return `<button type="button" class="st-raw-item" data-action="select-raw" data-raw="${esc(file.id)}">
          <strong>${esc(file.id)}</strong>
          <span>${esc(file.path)} · batch ${esc(batch)}</span>
        </button>`;
      }).join('')}
    </div>`;
  } catch (error) { toast(describe(error), true); }
}

async function selectRawFile(id) {
  if (!id) return;
  try {
    const raw = await api.rawDetail(id);
    const text = document.getElementById('st-raw-text');
    const batch = document.getElementById('st-batch-id');
    const title = document.getElementById('st-title');
    const subtitle = document.getElementById('st-subtitle');
    if (text) text.value = raw.text || '';
    if (batch) batch.value = raw.batch || raw.id || '';
    if (title && !title.value.trim()) title.value = raw.title || humanize(raw.batch || raw.id || '');
    if (subtitle && !subtitle.value.trim()) subtitle.value = raw.subtitle || '';
    document.getElementById('st-raw-picker').innerHTML = '';
    toast(`Loaded raw/${raw.id}.md`);
  } catch (error) { toast(describe(error), true); }
}

function readFileIntoRaw(file) {
  const reader = new FileReader();
  reader.onload = () => {
    const text = document.getElementById('st-raw-text');
    const batch = document.getElementById('st-batch-id');
    const title = document.getElementById('st-title');
    if (text) text.value = String(reader.result || '');
    const stem = file.name.replace(/\.[^.]+$/, '');
    if (batch && !batch.value.trim()) batch.value = slugify(stem);
    if (title && !title.value.trim()) title.value = humanize(stem);
    toast(`Loaded ${file.name}`);
  };
  reader.onerror = () => toast(`Could not read ${file.name}`, true);
  reader.readAsText(file);
}

async function saveRaw() {
  const text = val('st-raw-text');
  const batch = val('st-batch-id').trim();
  const title = val('st-title').trim();
  const subtitle = val('st-subtitle').trim();
  const overwrite = !state.creating && Boolean(state.selectedId);
  if (!text.trim()) { toast('Add some raw text first', true); return; }
  try {
    const saved = await api.saveRaw({
      text,
      batch: batch || undefined,
      title: title || undefined,
      subtitle: subtitle || undefined,
      overwrite,
    });
    toast(`Saved raw/${saved.batch}.md`);
    const prepared = await api.prepareImport({
      rawId: saved.rawId,
      batch: saved.batch,
      title: title || saved.title || undefined,
      subtitle: subtitle || saved.subtitle || undefined,
    });
    state.exchange = { kind: 'import', batch: prepared.batch, runId: prepared.runId, packet: prepared.packet };
    state.exchangeError = null;
    state.creating = false;
    state.selectedId = prepared.batch;
    state.forceExchange = true;
    state.stage = 'import';
    await refresh(false);     // pull the new batch into the rail
    state.selectedId = prepared.batch;
    state.stage = 'import';
    renderAll();
  } catch (error) { toast(describe(error), true); }
}

/** Update title/subtitle of an already-imported batch in place (no re-import). */
async function saveMeta() {
  const b = getSelected();
  if (!b) return;
  const title = val('st-title').trim();
  const subtitle = val('st-subtitle').trim();
  if (!title) { toast('Title cannot be empty', true); return; }
  const btn = document.querySelector('[data-action="save-meta"]');
  if (btn) btn.disabled = true;
  try {
    const saved = await api.updateMeta({ batch: b.id, title, subtitle: subtitle || undefined });
    if (state.detail[b.id]) {
      state.detail[b.id].title = saved.title || title;
      state.detail[b.id].subtitle = saved.subtitle || '';
    }
    // Let the Sentences tab pick up the new metadata on its next refresh.
    window.dispatchEvent(new Event('sentencesrefresh'));
    toast('Title & subtitle saved');
  } catch (error) {
    toast(describe(error), true);
  } finally {
    if (btn) btn.disabled = false;
  }
}

/* ── packet exchange actions ─────────────────────────────────────────────── */

async function ensureExchange(kind) {
  const b = getSelected();
  if (!b) return false;
  if (state.exchange && state.exchange.kind === kind && state.exchange.batch === b.id && !state.forceExchange) return true;
  try {
    const d = detailOf(b.id);
    const payload = kind === 'import'
      ? { rawId: b.rawId, batch: b.id, title: d.title || undefined, subtitle: d.subtitle || undefined }
      : { batch: b.id };
    const res = kind === 'import' ? await api.prepareImport(payload) : await api.prepareBuild(payload);
    state.exchange = { kind, batch: res.batch || b.id, runId: res.runId, packet: res.packet };
    state.forceExchange = false;
    state.exchangeError = null;
    return true;
  } catch (error) {
    state.exchange = null;
    state.exchangeError = describe(error);
    return false;
  }
}

function copyPacket(targetId, btn) {
  const pre = document.getElementById(targetId);
  if (!pre) return;
  const text = pre.textContent;
  const done = () => { const o = btn.textContent; btn.textContent = '✓ copied'; toast('Packet copied to clipboard'); setTimeout(() => { btn.textContent = o; }, 1400); };
  if (navigator.clipboard?.writeText) navigator.clipboard.writeText(text).then(done).catch(() => { selectText(pre); toast('Select all & copy manually', true); });
  else { selectText(pre); done(); }
}

function downloadPacket(kind) {
  if (!state.exchange || state.exchange.kind !== kind) return;
  const blob = new Blob([state.exchange.packet], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url; a.download = `${kind}-packet-${state.exchange.batch}.md`;
  a.click();
  URL.revokeObjectURL(url);
  toast('Packet downloaded');
}

async function applyReply(kind) {
  const b = getSelected();
  const ta = document.getElementById(`st-reply-${kind}`);
  const probs = document.getElementById(`st-problems-${kind}`);
  if (!ta || !b) return;
  if (probs) probs.innerHTML = '';

  // Strip stray markdown fences, like the CLI does.
  let raw = ta.value;
  const stripped = raw.replace(/```[a-z]*\n?|```/gi, '');
  if (stripped !== raw) { raw = stripped; toast('Removed markdown fences from the reply'); }
  const reply = raw.trim();
  if (!reply) { toast(`Empty reply — canonical ${kind === 'import' ? 'source' : 'card'} data unchanged`); return; }

  const applyBtn = document.querySelector(`[data-action="apply"][data-kind="${kind}"]`);
  if (applyBtn) applyBtn.disabled = true;
  try {
    const payload = { runId: state.exchange.runId, reply };
    if (kind === 'import') await api.applyImport(payload);
    else await api.applyBuild(payload);

    state.exchange = null;
    state.forceExchange = false;
    state.checkReports[b.id] = null;
    await refresh(false);
    await loadDetail(b.id);
    state.selectedId = b.id;
    state.stage = kind;       // stay on the stage; it now shows the review
    renderAll();
    toast(kind === 'import' ? 'Sentences imported' : 'Cards built');
  } catch (error) {
    if (applyBtn) applyBtn.disabled = false;
    if (error instanceof ApiError && error.status === 422 && probs) {
      const items = (error.problems.length ? error.problems : [error.message]).map(p => `<li>${esc(p)}</li>`).join('');
      probs.innerHTML = `<div class="st-problems"><h4>${esc(error.message)}</h4><ul>${items}</ul></div>`;
      toast('Validation failed — nothing was saved', true);
    } else {
      toast(describe(error), true);
    }
  }
}

/* ── check / audio ───────────────────────────────────────────────────────── */

async function runCheck() {
  const b = getSelected();
  if (!b) return;
  try {
    const report = await api.check({ batch: b.id });
    state.checkReports[b.id] = report;
    await refresh(false);
    state.selectedId = b.id;
    state.stage = 'check';
    renderAll();
    toast(report.clean ? 'Checks passed' : 'Checks found problems');
  } catch (error) { toast(describe(error), true); }
}

/** Update the selection summary + Regenerate button as cards are checked. */
function updateAudioSelection() {
  const boxes = [...document.querySelectorAll('.st-audio-card')];
  const count = boxes.filter(box => box.checked).length;
  const countEl = document.getElementById('st-audio-selcount');
  if (countEl) countEl.textContent = `${count} selected`;
  const all = document.getElementById('st-audio-all');
  if (all) {
    all.checked = count > 0 && count === boxes.length;
    all.indeterminate = count > 0 && count < boxes.length;
  }
  const regen = document.getElementById('st-regen-sel');
  if (regen) {
    regen.disabled = count === 0 || state.audioRunning;
    regen.textContent = count ? `Regenerate selected · ${count}` : 'Regenerate selected';
  }
}

function regenSelected() {
  const ids = [...document.querySelectorAll('.st-audio-card:checked')].map(cb => cb.dataset.id);
  if (!ids.length) { toast('Select at least one sentence first', true); return; }
  generateAudio({ cards: ids, force: true });
}

async function generateAudio(opts = {}) {
  const b = getSelected();
  if (!b || state.audioRunning) return;
  const cards = opts.cards || [];
  state.audioRunning = true;
  const backend = (document.querySelector('input[name="st-backend"]:checked') || {}).value;
  const forRun = document.getElementById('st-voice-forrun')?.checked;
  const voice = backend === 'elevenlabs' && forRun ? document.getElementById('st-voice-select')?.value : undefined;
  const force = opts.force || false;

  ['st-gen-missing', 'st-regen-sel'].forEach(id => { const el = document.getElementById(id); if (el) el.disabled = true; });
  const bar = document.getElementById('st-audio-bar');
  const status = document.getElementById('st-audio-status');
  if (status) status.textContent = cards.length ? `re-recording ${cards.length} clip${cards.length === 1 ? '' : 's'}…` : 'synthesizing… (this can take a while)';
  // Indeterminate creep while the blocking call runs; jumps to 100% on success.
  let creep = b.audio.total ? Math.round(b.audio.done / b.audio.total * 100) : 5;
  const iv = setInterval(() => { creep = Math.min(creep + 4, 92); if (bar) bar.style.width = `${creep}%`; }, 350);

  try {
    await api.audio({ batch: b.id, backend, voice, force, cards });
    clearInterval(iv);
    if (bar) bar.style.width = '100%';
    state.audioRunning = false;
    state.checkReports[b.id] = null;
    await refresh(false);
    await loadDetail(b.id);
    state.selectedId = b.id;
    state.stage = 'audio';
    renderAll();
    toast(cards.length ? `Re-recorded ${cards.length} clip${cards.length === 1 ? '' : 's'} · ${b.id}` : `Audio complete · ${b.id}`);
  } catch (error) {
    clearInterval(iv);
    state.audioRunning = false;
    if (status) status.textContent = 'synthesis failed';
    renderStage();   // restore the button states
    toast(describe(error), true);
  }
}

async function loadVoices() {
  try {
    const { voices, tier } = await api.voices();
    state.voices = voices;
    state.voiceTier = tier || null;
    // Update only the voice picker in place — a full re-render would reset the
    // transient backend radio selection back to the saved config.
    if (state.stage === 'audio') refreshVoiceSelect();
  } catch (error) {
    state.voices = [];
    toast(describe(error), true);
  }
}

/** On the free plan the API only accepts your own cloned voices via the API;
 *  the default/library voices return HTTP 402. */
const voiceUsable = (v) => state.voiceTier !== 'free' || v.category === 'cloned';

/** <option>s for the available ElevenLabs voices, marking unusable ones. */
function voiceOptionsHtml() {
  const voices = state.voices || [];
  if (!voices.length) return '';
  let firstUsableMarked = false;
  return voices.map(v => {
    const ok = voiceUsable(v);
    const selected = ok && !firstUsableMarked;
    if (selected) firstUsableMarked = true;
    const label = `${esc(v.name)}${v.category ? ` · ${esc(v.category)}` : ''}${ok ? '' : ' · needs paid plan'}`;
    return `<option value="${esc(v.id)}"${selected ? ' selected' : ''}${ok ? '' : ' disabled'}>${label}</option>`;
  }).join('');
}

/** Inner markup of the ElevenLabs voice picker (shared by stage + refresh). */
function voiceWrapInner() {
  const keyPresent = state.config && state.config.audio && state.config.audio.keyPresent;
  const keyEnv = (state.config && state.config.audio && state.config.audio.keyEnv) || 'ELEVENLABS_API_KEY';
  const opts = voiceOptionsHtml();
  const voices = state.voices || [];
  const freePlan = state.voiceTier === 'free';
  const usableCount = voices.filter(voiceUsable).length;
  const noneUsable = freePlan && usableCount === 0;
  const dis = noneUsable ? ' disabled' : '';
  let banner = '';
  if (freePlan && usableCount === 0) {
    banner = `<div class="st-voice-warn"><b>None of these voices are free.</b> On the ElevenLabs Free plan the API rejects the default/library voices (HTTP 402), even though they preview in the ElevenLabs app. Clone your own voice in ElevenLabs (the free plan includes a few) to use it here — or use <b>gTTS</b> (free, above), or upgrade your plan.</div>`;
  } else if (freePlan) {
    banner = `<div class="st-voice-warn">On the ElevenLabs Free plan only your <b>own cloned voices</b> work via the API. The default/library voices are disabled below (they’d return HTTP 402). Use a cloned voice, <b>gTTS</b>, or upgrade your plan.</div>`;
  }
  return `
    ${banner}
    ${opts ? `<select class="st-select" id="st-voice-select"${dis}>${opts}</select>` : `<span class="st-hint">${keyPresent ? 'Loading voices…' : `Set $${esc(keyEnv)} to list voices`}</span>`}
    <label class="st-check"><input type="checkbox" id="st-voice-forrun"${dis}> Use for this run only <span class="st-hint">--for-run / --voice</span></label>
    <div class="${keyPresent ? 'st-keyok' : 'st-keyno'}">${keyPresent ? `● $${esc(keyEnv)} detected${state.voiceTier ? ` · plan: ${esc(state.voiceTier)}` : ''}` : `○ $${esc(keyEnv)} not set — gTTS fallback`}</div>`;
}

/** Rebuild the voice picker block without disturbing the backend radios. */
function refreshVoiceSelect() {
  const wrap = document.getElementById('st-voicewrap');
  if (wrap) wrap.innerHTML = voiceWrapInner();
}

async function saveElevenLabsKey() {
  const input = document.getElementById('st-el-key');
  const key = (input?.value || '').trim();
  if (!key) { toast('Paste your ElevenLabs API key first', true); return; }
  const btn = document.querySelector('[data-action="save-el-key"]');
  if (btn) btn.disabled = true;
  try {
    state.config = await api.setConfig({ elevenlabsApiKey: key });
    if (input) { input.value = ''; input.placeholder = '•••••••••••• saved — paste a new key to replace'; }
    const status = document.getElementById('st-el-keystatus');
    if (status) {
      status.className = 'st-el-keystatus st-keyok';
      status.textContent = '● Key configured — voices will load above';
    }
    state.voices = null;
    refreshVoiceSelect();   // reflect keyPresent immediately
    loadVoices();           // fetch the voice list with the new key
    toast('API key saved');
  } catch (error) {
    toast(describe(error), true);
  } finally {
    if (btn) btn.disabled = false;
  }
}

function playAudio(url) {
  if (!url) return;
  try { new Audio(url).play().catch(() => {}); } catch { /* ignore */ }
}

/* ── settings drawer ─────────────────────────────────────────────────────── */

async function openSettings() {
  if (document.getElementById('st-drawer-host')) return;
  if (!state.config) state.config = await api.getConfig().catch(() => null);
  const c = state.config || { display: { lead: 'romanisation', showSecondary: true }, audio: {} };
  const seg = (group, value, label, on) =>
    `<button class="${on ? 'is-on' : ''}" data-action="seg" data-group="${group}" data-value="${value}">${label}</button>`;
  const a = c.audio || {};
  const host = document.createElement('div');
  host.id = 'st-drawer-host';
  host.innerHTML = `
    <div class="st-drawer-bg" data-action="close-settings"></div>
    <aside class="st-drawer" id="st-drawer" role="dialog" aria-label="Deck settings">
      <h3>Deck settings</h3>
      <p class="st-hint">Writes back to <code>config.toml</code>.</p>
      <div class="st-drawer-grp">
        <div class="st-drawer-gl">Display · [display]</div>
        <div class="st-check">Lead &nbsp;<span class="st-seg">
          ${seg('lead', 'romanisation', 'Romanisation', c.display.lead === 'romanisation')}
          ${seg('lead', 'target', 'Target', c.display.lead === 'target')}</span></div>
        <label class="st-check"><input type="checkbox" id="st-set-secondary"${c.display.showSecondary ? ' checked' : ''}> Show secondary line</label>
      </div>
      <div class="st-drawer-grp">
        <div class="st-drawer-gl">Audio · [audio]</div>
        <div class="st-check">Backend &nbsp;<span class="st-seg">
          ${seg('backend', 'gtts', 'gTTS', a.backend === 'gtts')}
          ${seg('backend', 'elevenlabs', 'ElevenLabs', a.backend === 'elevenlabs')}</span></div>
        <label class="st-field" style="margin-top:.5rem"><span class="st-cap">ElevenLabs voice · audio voice set</span>
          <input class="st-input" id="st-set-voice" type="text" value="${esc(a.voice || '')}" placeholder="voice id"></label>
        <div class="${a.keyPresent ? 'st-keyok' : 'st-keyno'}">${a.keyPresent ? '● ' + esc(a.keyEnv || 'ELEVENLABS_API_KEY') + ' set (read from env)' : '○ API key env not set'}</div>
      </div>
      <div class="st-row"><span class="st-spacer"></span>
        <button class="st-btn" data-action="close-settings">Close</button>
        <button class="st-btn st-primary" data-action="save-settings">Save</button></div>
    </aside>`;
  document.getElementById('page-studio').appendChild(host);
  // Track pending edits on the drawer's segmented controls.
  state.settingsDraft = { lead: c.display.lead, backend: a.backend };
}

function segSelect(btn) {
  const { group, value } = btn.dataset;
  if (state.settingsDraft) state.settingsDraft[group] = value;
  btn.parentElement.querySelectorAll('button').forEach(x => x.classList.toggle('is-on', x === btn));
}

async function saveSettings() {
  const draft = state.settingsDraft || {};
  const payload = {
    lead: draft.lead,
    backend: draft.backend,
    showSecondary: document.getElementById('st-set-secondary')?.checked,
    voice: (document.getElementById('st-set-voice')?.value || '').trim() || undefined,
  };
  try {
    state.config = await api.setConfig(payload);
    closeSettings();
    toast('Saved to config.toml');
    if (state.stage === 'audio') renderStage();
  } catch (error) { toast(describe(error), true); }
}

function closeSettings() {
  document.getElementById('st-drawer-host')?.remove();
  state.settingsDraft = null;
}

/* ── utilities ───────────────────────────────────────────────────────────── */

/** A small "this is the equivalent CLI command" chip shown in each stage. */
function cliHint(cmd, id) {
  return `<div class="st-cli"${id ? ` id="${id}"` : ''}><span class="st-cli-tag">CLI</span><code>${esc(cmd)}</code></div>`;
}

function reportFatal(error) {
  state.loadError = describe(error);
  renderAll();
}

function describe(error) {
  if (error instanceof ApiError) return error.message;
  return error && error.message ? error.message : String(error);
}

function toast(msg, isError) {
  let t = document.getElementById('st-toast');
  if (!t) { t = document.createElement('div'); t.id = 'st-toast'; t.className = 'st-toast'; document.body.appendChild(t); }
  t.textContent = msg;
  t.classList.toggle('is-error', !!isError);
  t.classList.add('is-show');
  clearTimeout(t._timer);
  t._timer = setTimeout(() => t.classList.remove('is-show'), 2200);
}

const val = (id) => (document.getElementById(id)?.value ?? '');

function esc(s) {
  return String(s ?? '').replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
}

function selectText(el) {
  const range = document.createRange();
  range.selectNodeContents(el);
  const sel = window.getSelection();
  sel.removeAllRanges();
  sel.addRange(range);
}

function humanize(id) {
  return String(id).split(/[-_]/).filter(Boolean).map(p => p[0].toUpperCase() + p.slice(1)).join(' ');
}

function slugify(value) {
  return String(value)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}
