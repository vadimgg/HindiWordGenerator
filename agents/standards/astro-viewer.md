# Astro Viewer Standard

Astro, TypeScript, CSS, and browser-interaction standards for the integrated
`viewer/` app.

## Core Principles

- The viewer is a local study/debug UI for generated data, not the source of
  truth for cards.
- The viewer is a **product UI**: design serves the task of inspecting,
  searching, reviewing, and playing generated cards. Familiarity, density, and
  consistency matter more than spectacle.
- Read live data from project-root `output/words/` and `output/sentences/`.
- Serve live audio from project-root `audio/` through `viewer/public/audio`.
- Do not reintroduce copied `vocab/` or `audio_output/` data flows.
- A browser refresh should be enough to pick up newly generated JSON and audio
  paths during local development.
- Missing or older fields should degrade gracefully; do not break the page
  because one batch predates the current schema.

## Product UI Design Register

Use product-register judgment for viewer work.

- The interface should feel trustworthy to someone using it repeatedly while
  generating and reviewing study data.
- Familiar patterns are good: tabs, search, filters, card lists, selected states,
  and playback controls should behave predictably.
- Density is allowed when it helps scanning. Do not turn review surfaces into
  marketing-style hero pages.
- Decorative surprises should be rare. Delight belongs in small interaction
  moments, not in every surface.
- If a visual choice would slow down card review, search, audio playback, or QA,
  it is probably wrong for this app.

Before substantial visual changes, write a one-sentence scene for the design
decision: who is using the viewer, where, in what light, and for what task. Let
that scene choose density, contrast, theme, and motion.

## Ownership Boundaries

- `viewer/src/pages/index.astro` owns server-side data loading from generated
  output files and the `window.__APP_DATA__` payload.
- `viewer/src/components/` owns rendering and layout only.
- `viewer/src/components/cards/sections/` owns repeated card subsections.
- `viewer/src/scripts/state/` owns client-side state such as tabs and selection.
- `viewer/src/scripts/ui/` owns browser interactions and DOM behavior.
- `viewer/src/scripts/anki/` owns Anki export behavior only.
- `viewer/src/utils/` owns pure helpers and shared TypeScript types.
- `viewer/scripts/` owns local maintenance scripts such as audio symlink setup.
- `viewer/src/styles/global.css` owns current styling until a future CSS split is
  planned.

## Architecture Boundaries

- Astro components may prepare display values and render markup, but should not
  own complex interaction state.
- Client scripts should not parse or load filesystem data; they should consume
  `window.__APP_DATA__`.
- Utilities should be pure when practical. Avoid DOM access in `viewer/src/utils/`.
- Keep Anki-specific formatting in `viewer/src/scripts/anki/`, not in card
  components.
- Keep audio source resolution in `viewer/src/utils/audioHelpers.ts`.
- Keep output-file discovery in one loader path rather than duplicating it in
  components or client scripts.

## File And Function Size

Size is a review trigger, not an automatic failure.

- Target Astro components: about 150 lines when practical.
- Target client script modules: about 200 lines when practical.
- Target functions: one comfortable screen, about 40 lines.
- Files around 300 lines need either a split plan or a short reason to stay
  together.
- Files over 500 lines are refactor candidates unless they are generated,
  legacy-carried, or one cohesive interaction controller with a documented split
  plan.
- Keep nesting to two levels when practical.

Current known large-file triggers:

- `viewer/src/styles/global.css`: inherited single stylesheet, future split by
  surface or component family.
- `viewer/src/scripts/ui/pageInteractions.js`: inherited interaction controller,
  future split by filter, lasso selection, group toggles, and card expansion.
- `viewer/src/scripts/ui/exportPane.js`: inherited export controller, future
  split if Anki export remains part of this project.

Do not churn inherited large files just to satisfy the threshold. Split them
when the task naturally touches a clear ownership boundary.

## Component Standards

- Components should receive explicit props and render predictable markup.
- Avoid UI cards nested inside other UI cards unless the component is a real
  repeated item or modal.
- Keep compact tool surfaces compact; do not use oversized hero-style text in
  card lists or controls.
- Use stable `data-*` attributes for client scripts instead of brittle DOM
  hierarchy assumptions.
- Interactive controls inside card headers must not accidentally trigger card
  expand/collapse.
- Missing optional data should hide the section, not render empty chrome.
- Sentence rendering should support both current `tokens` and older `words`-only
  batches until old outputs are migrated.
- Every interactive component should account for default, hover, focus, active,
  selected, disabled, and missing-data states where applicable.
- Empty states should teach the operator what data is missing and where it comes
  from, such as `output/words/`, `output/sentences/`, or `audio/`.

## Client Script Standards

- Prefer event delegation for repeated cards and controls.
- Scope DOM queries to the owning page or container when practical.
- Avoid broad body text extraction or global selectors when a stable `data-*`
  selector exists.
- Keep one module responsible for one interaction family.
- Client scripts should tolerate empty words, empty sentences, and missing audio.
- Side-effect functions should name the effect, such as `initAudio`,
  `syncSelectionBadges`, or `renderExportPane`.

## Data And Audio Standards

- Use explicit `audio` fields from JSON when present.
- Do not infer old audio paths unless the task explicitly restores legacy data.
- Normalize relative audio paths to browser-root paths in one helper.
- Invalid JSON files should be skipped with a warning during local dev, not crash
  the whole viewer unless strict validation is specifically requested.
- Do not mutate generated JSON from the viewer.

## Styling Standards

- Reuse existing classes and visual language before adding new styles.
- Keep layout dimensions stable for cards, buttons, tab bars, and counters.
- Text inside buttons and compact controls must fit at desktop and mobile sizes.
- Avoid one-off decorative palettes; new colors should fit the existing dark
  viewer theme and be reused through classes.
- Prefer CSS classes over inline styles for new work. Existing inline styles can
  be cleaned up when touching the surrounding component.
- Keep body/prose line length readable, roughly 65-75ch where text is paragraph
  content.
- Use restrained accent color for selection, focus, and active state. Do not use
  saturated color as decoration on inactive UI.
- Avoid pure black and pure white in new theme colors; use tinted dark/light
  neutrals that fit the current viewer palette.
- Do not rely on hover for functionality; touch users must be able to perform
  the same action.
- Touch targets should be at least 44x44px on coarse pointers when practical.

## Design Anti-Patterns

Avoid these unless a future design standard explicitly reverses the rule:

- gradient text
- decorative glassmorphism
- colored side-stripe card accents
- nested decorative cards
- identical marketing-style card grids
- page-load choreography
- bounce or elastic easing
- reinvented standard controls for flavor
- gray text on colored backgrounds when contrast suffers
- redundant copy that restates a nearby heading

The viewer should pass the product UI test: a user should trust it immediately
and not pause at subtly strange controls.

## Responsive Standards

- Write mobile/base behavior first, then layer desktop complexity with
  `min-width` breakpoints.
- Use content-driven breakpoints; add a breakpoint when the card, toolbar, or
  tab layout actually breaks.
- Use pointer and hover media queries for interaction differences:
  - `@media (pointer: coarse)` for larger touch targets
  - `@media (hover: hover)` for hover-only enhancements
- Include `viewport-fit=cover` only when the layout actually handles safe areas.
- Test narrow, tablet-ish, and desktop widths for every card or toolbar change.
- No horizontal scroll from cards, buttons, long Hindi text, romanisation, or
  audio controls.

## Audit And Polish Workflow

For UI review, score the viewer surface across these dimensions:

- Accessibility: semantics, keyboard flow, focus, labels, contrast.
- Performance: bundle size, layout thrash, expensive effects, smooth scrolling.
- Responsive design: narrow viewports, touch targets, overflow, reflow.
- Theming: consistent colors, state vocabulary, no hard-coded drift in new code.
- Anti-patterns: check the design anti-pattern list above.

Findings should be grouped by severity:

- P0: blocks the task or breaks the page
- P1: major usability, accessibility, or responsive issue
- P2: minor friction with a workaround
- P3: polish

Before marking visual work done, use the surface yourself:

- search words and sentences
- switch tabs
- expand/collapse cards
- play audio
- inspect an empty or missing-audio state when available
- refresh after output changes if live data loading was touched

## Validation

For viewer behavior changes, run:

```bash
cd viewer
npm run build
```

For browser-visible UI changes, also smoke test:

- viewer loads at `http://127.0.0.1:4321/`
- word cards load from `output/words/`
- sentence cards load from `output/sentences/`
- audio buttons appear only when `audio` exists
- refreshing after generated output changes picks up the new files
- no console errors appear for the changed path
- viewport checks cover mobile-width and desktop-width layouts

Report npm audit findings, but do not run broad audit fixes unless the task is
dependency maintenance.

## Comments

Use comments to explain non-obvious interaction contracts, data compatibility,
or inherited legacy decisions.

Good comment topics:

- why live output loading is server-side in Astro
- why old sentence batches without `tokens` are still supported
- why a click handler ignores audio buttons or selection controls
- why Anki export remains isolated from card rendering

Do not add comments that merely restate markup or obvious DOM calls.
