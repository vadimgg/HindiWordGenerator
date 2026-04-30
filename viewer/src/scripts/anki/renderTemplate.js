/**
 * Minimal Anki template renderer for in-browser previews and smoke checks.
 *
 * Responsible for: rendering the Mustache-style subset used by Anki card
 * templates so the web preview can share the same field objects and templates
 * as the real Anki export path.
 *
 * No dependencies on other project modules.
 */
// Responsible for: small Mustache-style renderer for Anki preview templates

/**
 * Replaces {{FieldName}} tokens in a Mustache-style Anki template string.
 * Supported block tags ({{#Foo}}...{{/Foo}}) include their content when the
 * field is non-empty and omit it otherwise.
 *
 * @param {string} template - Template string with {{Token}} placeholders.
 * @param {Record<string, string>} fields - Map of field name to HTML string.
 * @returns {string} Rendered HTML string.
 */
export function renderTemplate(template, fields) {
  let output = template.replace(/\{\{#(\w+)\}\}([\s\S]*?)\{\{\/\1\}\}/g, (_, key, inner) =>
    fields[key] ? inner : ''
  );
  output = output.replace(/\{\{(\w+)\}\}/g, (_, key) => fields[key] ?? '');
  return output;
}
