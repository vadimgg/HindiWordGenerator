# Backlog

Improvement items to keep visible while the generator and Astro viewer are being
integrated.

Status values: `open`, `in-progress`, `done`.

## Viewer And Anki

| Item | Priority | Status | Notes |
|---|---|---|---|
| Extract shared Anki preview template rendering into `viewer/src/scripts/anki/renderTemplate.js` and add a smoke check that preview fields/templates stay aligned with export fields/templates | P2 | done | Preview and export share `wordToAnkiFields`, `sentenceToAnkiFields`, note templates, and the preview renderer. `npm run check:anki` verifies representative word and sentence previews. |
