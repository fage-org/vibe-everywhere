export type OutgoingSessionMessageOptions = {
  sentFrom: string;
  permissionMode?: string | null;
  model?: string | null;
  displayText?: string | null;
};

export function buildOutgoingUserMessageRecord(
  text: string,
  options: OutgoingSessionMessageOptions,
): Record<string, unknown> {
  return {
    role: "user",
    content: {
      type: "text",
      text,
    },
    meta: {
      sentFrom: options.sentFrom,
      permissionMode: options.permissionMode ?? undefined,
      model: options.model ?? null,
      ...(options.displayText ? { displayText: options.displayText } : {}),
    },
  };
}
