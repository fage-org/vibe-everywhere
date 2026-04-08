export function normalizeTerminalPublicKeyInput(input: string): string | null {
  const trimmed = input.trim();
  if (!trimmed) {
    return null;
  }

  if (trimmed.startsWith("vibe:///terminal?")) {
    const query = trimmed.slice("vibe:///terminal?".length).trim();
    if (!query) {
      return null;
    }

    return readTerminalConnectKey(new URLSearchParams(query));
  }

  return trimmed;
}

export function readTerminalConnectKey(searchParams: URLSearchParams): string | null {
  const explicit = searchParams.get("key")?.trim();
  if (explicit) {
    return explicit;
  }

  const entries = Array.from(searchParams.entries());
  if (entries.length === 1 && entries[0][1].trim() === "") {
    return entries[0][0].trim() || null;
  }

  return null;
}
