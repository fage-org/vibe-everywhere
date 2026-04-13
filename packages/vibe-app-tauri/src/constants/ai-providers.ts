/**
 * AI Provider Configuration
 *
 * Defines the supported AI providers with their token formats and help URLs.
 * Each provider has different authentication requirements.
 */

export interface AIProvider {
  id: string;
  name: string;
  description: string;
  tokenPrefix: string;
  tokenPlaceholder: string;
  tokenHelpUrl: string;
  validationHint: string;
}

/**
 * Supported AI providers
 *
 * IMPORTANT: Each provider uses different token formats:
 * - OpenAI: sk-... prefix, Bearer token auth (used by Codex)
 * - Anthropic: sk-ant-... prefix, x-api-key header (NOT OpenAI compatible)
 * - Gemini: AI... prefix, query parameter auth
 */
export const AI_PROVIDERS: AIProvider[] = [
  {
    id: "openai",
    name: "OpenAI",
    description: "GPT-4, GPT-3.5, Codex compatible",
    tokenPrefix: "sk-",
    tokenPlaceholder: "sk-proj-...",
    tokenHelpUrl: "https://platform.openai.com/api-keys",
    validationHint: "Requires OpenAI API key (sk-...)",
  },
  {
    id: "anthropic",
    name: "Anthropic",
    description: "Claude series models",
    tokenPrefix: "sk-ant-",
    tokenPlaceholder: "sk-ant-...",
    tokenHelpUrl: "https://console.anthropic.com/settings/keys",
    validationHint:
      "Requires Anthropic API key (not OpenAI format)",
  },
  {
    id: "gemini",
    name: "Google AI",
    description: "Gemini models",
    tokenPrefix: "AI",
    tokenPlaceholder: "AI...",
    tokenHelpUrl: "https://aistudio.google.com/app/apikey",
    validationHint: "Requires Google AI API key",
  },
];

/**
 * Get a provider by ID
 */
export function getProviderById(id: string): AIProvider | undefined {
  return AI_PROVIDERS.find((p) => p.id === id);
}

/**
 * Check if a token matches a provider's expected format
 */
export function tokenMatchesFormat(provider: AIProvider, token: string): boolean {
  return token.startsWith(provider.tokenPrefix);
}

export default AI_PROVIDERS;
