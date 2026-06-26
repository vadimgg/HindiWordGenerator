export type DisplayLead = "romanisation" | "target";

export interface ViewerCard {
  readonly id: string;
  readonly lead: string;
  readonly secondary: string | null;
  readonly english: string;
  readonly literal: string;
  readonly register: string;
  readonly audio_url: string | null;
}

export interface ViewerSession {
  readonly lead: DisplayLead;
  readonly cards: readonly ViewerCard[];
}

export async function loadSession(signal?: AbortSignal): Promise<ViewerSession> {
  const response = await fetch("/api/session", {
    method: "GET",
    headers: { Accept: "application/json" },
    signal,
  });
  if (!response.ok) {
    throw new Error(`viewer API failed with HTTP ${response.status}`);
  }
  return parseViewerSession(await response.json());
}

export function parseViewerSession(value: unknown): ViewerSession {
  const object = expectRecord(value, "session");
  const lead = expectDisplayLead(object.lead);
  const cards = expectArray(object.cards, "session.cards").map((card, index) =>
    parseViewerCard(card, `session.cards[${index}]`),
  );
  return { lead, cards };
}

function parseViewerCard(value: unknown, path: string): ViewerCard {
  const object = expectRecord(value, path);
  return {
    id: expectString(object.id, `${path}.id`),
    lead: expectString(object.lead, `${path}.lead`),
    secondary: expectOptionalString(object.secondary, `${path}.secondary`),
    english: expectString(object.english, `${path}.english`),
    literal: expectString(object.literal, `${path}.literal`),
    register: expectString(object.register, `${path}.register`),
    audio_url: expectOptionalString(object.audio_url, `${path}.audio_url`),
  };
}

function expectRecord(value: unknown, path: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${path} must be an object`);
  }
  return value as Record<string, unknown>;
}

function expectArray(value: unknown, path: string): readonly unknown[] {
  if (!Array.isArray(value)) {
    throw new Error(`${path} must be an array`);
  }
  return value;
}

function expectString(value: unknown, path: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`${path} must be a non-empty string`);
  }
  return value;
}

function expectOptionalString(value: unknown, path: string): string | null {
  if (value === null || value === undefined) {
    return null;
  }
  return expectString(value, path);
}

function expectDisplayLead(value: unknown): DisplayLead {
  if (value === "romanisation" || value === "target") {
    return value;
  }
  throw new Error("session.lead must be romanisation or target");
}
