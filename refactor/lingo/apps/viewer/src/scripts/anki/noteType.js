/**
 * Anki note type definition — field list, card CSS, and front/back templates.
 *
 * Responsible for: exporting all constants that define the 'Hindi Vocabulary'
 * Anki note type: the ordered field list, the card stylesheet (dark theme
 * matching the website), and the front/back Mustache templates.
 *
 * These constants are consumed by export.js (to create/sync the note type)
 * and by txtFallback.js (for the #notetype and #columns headers).
 *
 * No dependencies on other project modules.
 */
// Responsible for: Anki note type definition — CSS, front/back card templates, field list

export const ANKI_NOTE_TYPE = 'Hindi Vocabulary';

export const ANKI_FIELDS = [
  'English', 'Hindi', 'Audio', 'Romanisation', 'Syllables', 'Category', 'Gender', 'Transitivity',
  'Forms', 'ExampleSentence', 'ExampleEnglish', 'UsageNotes', 'RelatedWords', 'Morphemes', 'SoundAlikes', 'DelhiNote', 'Etymology',
];

export const ANKI_CSS = [
  "@import url('https://fonts.googleapis.com/css2?family=Tiro+Devanagari+Hindi&family=Barlow+Condensed:wght@400;600;700&family=Plus+Jakarta+Sans:wght@400;500;600&family=DM+Mono&display=swap');",
  // Base — gradient matching demo
  'body{background:#0f172a;margin:0;padding:0;}',
  '.card{font-family:"Plus Jakarta Sans",sans-serif;background:linear-gradient(160deg,#1a2540 0%,#0f172a 100%);color:#e2e8f0;margin:0;padding:0;font-size:18px;line-height:1.6;}',
  '.card-wrap{max-width:680px;margin:0 auto;padding:1.5rem 1.5rem 3rem;}',
  // Front card
  '.front-face{display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:200px;text-align:center;padding:2.5rem 1.5rem;}',
  '.meaning{font-size:2.25rem;font-weight:600;color:#e2e8f0;text-shadow:0 0 60px rgba(226,232,240,0.08);margin:0 0 0.5rem 0;line-height:1.3;position:relative;display:inline-block;padding-bottom:0.5rem;}',
  '.meaning::after{content:"";position:absolute;bottom:0;left:50%;transform:translateX(-50%);width:3.5rem;height:2px;background:linear-gradient(90deg,transparent,rgba(251,191,36,0.5),transparent);border-radius:1px;}',
  '.meaning-wrap{margin-bottom:1rem;}',
  '.front-sentence{font-size:1.1rem;color:rgba(148,163,184,0.7);text-align:center;margin:.75rem 0 0;line-height:1.6;font-style:italic;}',
  '.chips{display:flex;flex-wrap:wrap;justify-content:center;gap:.5rem;}',
  // Answer header — gradient divider via ::after, no solid border
  '.answer-header{border-bottom:none;position:relative;padding-bottom:1.75rem;margin-bottom:1.75rem;text-align:center;}',
  '.answer-header::after{content:"";position:absolute;bottom:0;left:10%;right:10%;height:1px;background:linear-gradient(90deg,transparent,rgba(251,191,36,0.2),transparent);}',
  '.hindi{font-family:"Tiro Devanagari Hindi",serif;font-size:3.5rem;background:linear-gradient(135deg,#fbbf24,#f97316);-webkit-background-clip:text;-webkit-text-fill-color:transparent;background-clip:text;margin:0 0 0.5rem 0;line-height:1.2;letter-spacing:0.02em;}',
  '.roman{font-size:1.4rem;color:#5eead4;text-shadow:0 0 30px rgba(94,234,212,0.3);margin:0 0 0.5rem 0;}',
  '.meaning-reveal{font-size:1.15rem;color:#94a3b8;margin:0 0 1rem 0;line-height:1.6;}',
  '.header-chips{display:flex;flex-wrap:wrap;justify-content:center;gap:.5rem;}',
  // Section boxes
  '.field-sec{border-top:2px solid #334155;padding:1.5rem;margin-bottom:1.5rem;background:rgba(30,41,59,0.5);border-radius:14px;box-shadow:0 -1px 0 0 rgba(255,255,255,0.03);}',
  '.field-label{font-family:"Barlow Condensed",sans-serif;font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.25em;margin:0 0 1rem 0;color:#64748b;display:block;}',
  // Cross-lang / sound-alikes / urdu-punjabi item cards — vertical layout, centred
  '.clang-item{margin-bottom:.75rem;padding:1rem 1.25rem;background:rgba(15,23,42,0.7);border:1px solid rgba(51,65,85,.55);border-radius:10px;text-align:center;display:flex;flex-direction:column;align-items:center;}',
  '.clang-item:last-child{margin-bottom:0;}',
  '.meta-badge{display:inline-block;font-size:11px;font-family:"Barlow Condensed",sans-serif;font-weight:700;text-transform:uppercase;letter-spacing:.1em;padding:.25rem .65rem;border-radius:6px;margin-bottom:.5rem;}',
  '.item-note{font-size:1rem;color:#cbd5e1;line-height:1.65;margin:.5rem 0 0 0;}',
  // Syllables inline header
  '.syl-inline{font-size:1.1rem;font-weight:600;color:#5eead4;opacity:.6;font-family:"DM Mono",monospace;letter-spacing:.15em;margin:0 0 .5rem 0;}',
  // Gender / transitivity badges
  '.badge-masc{background:rgba(30,58,138,.25);color:#93c5fd;border:1px solid rgba(59,130,246,.25);font-family:"Barlow Condensed",sans-serif;font-size:.58rem;font-weight:700;text-transform:uppercase;letter-spacing:.15em;padding:.18rem .5rem;border-radius:6px;}',
  '.badge-fem{background:rgba(131,24,67,.25);color:#f9a8d4;border:1px solid rgba(236,72,153,.25);font-family:"Barlow Condensed",sans-serif;font-size:.58rem;font-weight:700;text-transform:uppercase;letter-spacing:.15em;padding:.18rem .5rem;border-radius:6px;}',
  '.badge-neutral{background:rgba(30,41,59,.6);color:#94a3b8;border:1px solid rgba(51,65,85,.4);font-family:"Barlow Condensed",sans-serif;font-size:.58rem;font-weight:700;text-transform:uppercase;letter-spacing:.15em;padding:.18rem .5rem;border-radius:6px;}',
  '.badge-neutral-anki{background:linear-gradient(135deg,rgba(30,41,59,0.7),rgba(15,23,42,0.5));color:#94a3b8;border:1px solid rgba(51,65,85,.4);font-family:"Barlow Condensed",sans-serif;font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.2em;padding:.25rem .65rem;border-radius:6px;display:inline-block;}',
  '.badge-masc-anki{background:linear-gradient(135deg,rgba(59,130,246,0.15),rgba(37,99,235,0.08));color:#93c5fd;border:1px solid rgba(147,197,253,0.2);font-family:"Barlow Condensed",sans-serif;font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.2em;padding:.25rem .65rem;border-radius:6px;display:inline-block;}',
  // Collocation rows
  '.coll-row{display:flex;justify-content:space-between;align-items:baseline;gap:1rem;padding:.75rem 0;border-bottom:1px solid rgba(51,65,85,.25);}',
  '.coll-row:last-child{border-bottom:none;}',
  '.coll-hindi{font-family:"Tiro Devanagari Hindi",serif;font-size:1.1rem;color:#fbbf24;line-height:1.3;}',
  '.coll-rom{font-family:"DM Mono",monospace;font-size:1rem;color:rgba(94,234,212,.6);}',
  '.coll-eng{font-size:1rem;color:#94a3b8;text-align:right;flex-shrink:0;}',
  // Related words / sound alikes
  '.lang-label{display:block;font-size:11px;font-family:"Barlow Condensed",sans-serif;font-weight:700;text-transform:uppercase;letter-spacing:.12em;margin-bottom:.5rem;}',
  '.word-row{display:flex;gap:.5rem;align-items:baseline;justify-content:center;flex-wrap:wrap;margin-bottom:.5rem;}',
  '.word-text{font-family:"Tiro Devanagari Hindi",serif;font-size:1.25rem;color:#fbbf24;}',
  '.word-rom{color:#5eead4;font-size:1rem;}',
  '.mean-box{background:rgba(8,13,24,0.8);border:1px solid rgba(51,65,85,.5);border-radius:8px;padding:.25rem .75rem;margin-top:.25rem;display:inline-block;}',
  '.mean-box span{color:#94a3b8;font-size:1rem;}',
  // Sound-alike vertical centred layout
  '.sound-alike-row{display:flex;align-items:baseline;justify-content:center;gap:.5rem;flex-wrap:wrap;margin-bottom:.5rem;}',
  '.sound-alike-part{font-family:"DM Mono",monospace;font-size:1.1rem;color:rgba(94,234,212,.9);text-shadow:0 0 12px rgba(94,234,212,0.25);}',
  '.sound-alike-arrow{color:#64748b;font-size:1rem;opacity:0.5;}',
  '.sound-alike-assoc{color:#e2e8f0;font-style:italic;font-size:1rem;}',
  '.sound-alike-note{font-size:1rem;color:#94a3b8;line-height:1.6;margin:.5rem 0 0 0;}',
  // Etymology chain — vertical card blocks with arrow connectors, centred
  '.etym-chain{display:flex;flex-direction:column;align-items:stretch;}',
  '.etym-stage{padding:1rem 1.25rem;background:rgba(15,23,42,0.6);border:1px solid rgba(51,65,85,.4);border-radius:10px;margin-bottom:.5rem;text-align:center;display:flex;flex-direction:column;align-items:center;}',
  '.etym-stage:last-child{margin-bottom:0;}',
  '.etym-lang{font-family:"Barlow Condensed",sans-serif;font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.15em;color:#64748b;display:block;margin-bottom:.25rem;}',
  '.etym-stage:last-child .etym-lang{color:#fbbf24;}',
  '.etym-word{font-family:"Tiro Devanagari Hindi",serif;font-size:1.25rem;color:#e2e8f0;line-height:1.3;display:block;margin-bottom:.25rem;}',
  '.etym-stage:last-child .etym-word{background:linear-gradient(135deg,#fbbf24,#f97316);-webkit-background-clip:text;-webkit-text-fill-color:transparent;background-clip:text;}',
  '.etym-sub{font-size:1rem;display:flex;align-items:baseline;justify-content:center;gap:.5rem;flex-wrap:wrap;}',
  '.etym-rom{font-family:"DM Mono",monospace;font-size:1rem;color:rgba(94,234,212,.7);}',
  '.etym-sep{color:#475569;font-size:0.75rem;}',
  '.etym-meaning{color:#64748b;font-style:italic;font-size:1rem;}',
  '.etym-arrow{text-align:center;color:rgba(251,191,36,0.3);font-size:1.2rem;line-height:1;margin:0;padding:.25rem 0;display:block;}',
  // Mobile — fonts at or above readable baseline
  '@media(max-width:520px){',
  '.card-wrap{padding:1.5rem 1rem 2rem;}',
  '.front-face{padding:1.75rem 1rem;min-height:100vh;}',
  '.hindi{font-size:3rem;}',
  '.meaning{font-size:2rem;}',
  '.roman{font-size:1.2rem;}',
  '.field-sec{padding:1.25rem 1rem 1.5rem;}',
  '}',
].join('\n');

export const ANKI_FRONT = `<div class="card-wrap front-face">
  <div class="meaning-wrap">
    <p class="meaning">{{English}}</p>
  </div>
  {{#ExampleEnglish}}
  <p class="front-sentence">{{ExampleEnglish}}</p>
  {{/ExampleEnglish}}
  {{#Category}}<div class="chips" style="margin-top:.75rem;">{{Category}}</div>{{/Category}}
</div>`;

export const ANKI_BACK = `<div class="card-wrap">
  <div class="answer-header">
    <p class="hindi" lang="hi">{{Hindi}}</p>
    {{#Audio}}<p>{{Audio}}</p>{{/Audio}}
    <p class="roman">{{Romanisation}}</p>
    {{#Syllables}}<p class="syl-inline">{{Syllables}}</p>{{/Syllables}}
    <p class="meaning-reveal">{{English}}</p>
    <div class="header-chips">
      {{#Category}}{{Category}}{{/Category}}
      {{#Gender}}{{Gender}}{{/Gender}}
      {{#Transitivity}}{{Transitivity}}{{/Transitivity}}
    </div>
    {{#Forms}}<div style="margin-top:.9rem;">{{Forms}}</div>{{/Forms}}
  </div>
  {{#ExampleSentence}}<div>{{ExampleSentence}}</div>{{/ExampleSentence}}
  {{#UsageNotes}}<div>{{UsageNotes}}</div>{{/UsageNotes}}
  {{#RelatedWords}}<div>{{RelatedWords}}</div>{{/RelatedWords}}
  {{#Morphemes}}<div>{{Morphemes}}</div>{{/Morphemes}}
  {{#SoundAlikes}}<div>{{SoundAlikes}}</div>{{/SoundAlikes}}
  {{#DelhiNote}}<div>{{DelhiNote}}</div>{{/DelhiNote}}
  {{#Etymology}}<div>{{Etymology}}</div>{{/Etymology}}
</div>`;
