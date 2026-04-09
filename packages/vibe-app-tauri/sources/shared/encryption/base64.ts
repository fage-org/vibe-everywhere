export function decodeBase64(
  base64: string,
  encoding: "base64" | "base64url" = "base64",
): Uint8Array {
  let normalizedBase64 = base64;

  if (encoding === "base64url") {
    normalizedBase64 = base64.replace(/-/g, "+").replace(/_/g, "/");

    const padding = normalizedBase64.length % 4;
    if (padding) {
      normalizedBase64 += "=".repeat(4 - padding);
    }
  }

  const binaryString = atob(normalizedBase64);
  const bytes = new Uint8Array(binaryString.length);

  for (let index = 0; index < binaryString.length; index += 1) {
    bytes[index] = binaryString.charCodeAt(index);
  }

  return bytes;
}

export function encodeBase64(
  buffer: Uint8Array,
  encoding: "base64" | "base64url" = "base64",
): string {
  const binaryString = String.fromCharCode(...buffer);
  const base64 = btoa(binaryString);

  if (encoding === "base64url") {
    return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
  }

  return base64;
}
