/**
 * Anki sentence note type definition.
 *
 * Responsible for: exporting all constants that define the 'Hindi Sentence'
 * Anki note type — field list, CSS, and front/back templates.
 *
 * Dependencies: anki/noteType.js (ANKI_CSS base).
 */
// Responsible for: Anki sentence note type definition — CSS, templates, field list

import { ANKI_CSS } from './noteType.js';

export const ANKI_SENTENCE_NOTE_TYPE = 'Hindi Sentence';

export const ANKI_SENTENCE_FIELDS = [
  'English', 'Hindi', 'Audio', 'Romanisation', 'Literal', 'Register',
  'WordBreakdown', 'Chapter', 'Tags',
];

export const ANKI_SENTENCE_CSS = [
  ANKI_CSS,
  // Register badge base class
  '.reg{font-size:11px;font-family:"Barlow Condensed",sans-serif;font-weight:700;text-transform:uppercase;letter-spacing:.2em;color:#64748b;padding:.25rem .65rem;border:1px solid rgba(100,116,139,.35);border-radius:6px;white-space:nowrap;display:inline-block;}',
  // Register badge colour variants — extend existing .reg base
  '.reg-formal{color:#a78bfa;border-color:rgba(139,92,246,.4);background:rgba(109,40,217,.2);}',
  '.reg-standard{color:#38bdf8;border-color:rgba(56,189,248,.4);background:rgba(14,165,233,.15);}',
  '.reg-casual{color:#fbbf24;border-color:rgba(251,191,36,.3);background:rgba(245,158,11,.2);}',
  '.reg-colloquial{color:#fb7185;border-color:rgba(251,113,133,.4);background:rgba(225,29,72,.2);}',
  // Literal gloss line
  '.literal-gloss{font-size:1rem;color:#64748b;font-style:italic;line-height:1.65;margin:0;}',
  // Word breakdown table
  '.breakdown-table{width:100%;border-collapse:collapse;margin-top:.25rem;}',
  '.breakdown-table th{font-family:"Barlow Condensed",sans-serif;font-size:.55rem;font-weight:700;text-transform:uppercase;letter-spacing:.16em;color:#475569;text-align:left;padding:.3rem .5rem .5rem;border-bottom:1px solid rgba(51,65,85,.6);}',
  '.breakdown-table td{padding:.6rem .5rem;vertical-align:top;border-bottom:1px solid rgba(30,41,59,.8);}',
  '.breakdown-table tr:last-child td{border-bottom:none;}',
  '.breakdown-table tr:hover td{background:rgba(30,41,59,.4);}',
  // Word token — left column of breakdown table
  '.bd-word{font-family:"Tiro Devanagari Hindi",serif;font-size:1.15rem;color:#fbbf24;display:block;line-height:1.3;}',
  '.bd-roman{font-size:.75rem;color:#5eead4;opacity:.7;font-family:monospace;display:block;margin-top:.15rem;}',
  // Meaning column
  '.bd-meaning{font-size:.95rem;color:#e2e8f0;line-height:1.5;}',
  '.bd-note{font-size:.8rem;color:#64748b;font-style:italic;line-height:1.55;margin-top:.25rem;display:block;}',
  '.bd-badges{display:flex;flex-wrap:wrap;gap:.3rem;margin-top:.3rem;}',
  // Chapter meta — recessive footer
  '.chapter-meta{font-family:"Barlow Condensed",sans-serif;font-size:.55rem;font-weight:600;text-transform:uppercase;letter-spacing:.16em;color:#334155;text-align:center;margin-top:1.5rem;padding:.4rem;background:#080d18;border-radius:8px;}',
  // Mobile
  '@media(max-width:520px){',
  '.bd-word{font-size:.9rem;}',
  '.bd-meaning{font-size:.8rem;}',
  '.literal-gloss{font-size:.8rem;}',
  '}',
].join('\n');

export const ANKI_SENTENCE_FRONT = `<div class="card-wrap front-face">
  <div class="meaning-wrap">
    <p class="meaning">{{English}}</p>
  </div>
  {{#Register}}
  <div class="chips" style="margin-top:.9rem;">
    <span class="reg reg-{{Register}}">{{Register}}</span>
  </div>
  {{/Register}}
</div>`;

export const ANKI_SENTENCE_BACK = `<div class="card-wrap">

  <div class="answer-header">
    <p class="hindi" lang="hi">{{Hindi}}</p>
    {{#Audio}}<p>{{Audio}}</p>{{/Audio}}
    <p class="roman">{{Romanisation}}</p>
    <p class="meaning-reveal">{{English}}</p>
    <div class="header-chips">
      {{#Register}}<span class="reg reg-{{Register}}">{{Register}}</span>{{/Register}}
    </div>
  </div>

  {{#Literal}}
  <div class="field-sec" style="border-top-color:#475569;">
    <p class="field-label" style="color:#475569;">Literal</p>
    <p class="literal-gloss">{{Literal}}</p>
  </div>
  {{/Literal}}

  {{#WordBreakdown}}
  <div class="field-sec" style="border-top-color:#14b8a6;">
    <p class="field-label" style="color:#14b8a6;">Word Breakdown</p>
    {{WordBreakdown}}
  </div>
  {{/WordBreakdown}}

  {{#Chapter}}
  <p class="chapter-meta">{{Chapter}}</p>
  {{/Chapter}}

</div>`;
