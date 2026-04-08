import type {
  DesktopAppearanceSettings,
  DesktopLanguageSettings,
  DesktopVoiceSettings,
} from "./desktop-preferences";
import type { Settings } from "./wave8-client";

export function mapAccountSettingsToDesktopPreferences(input: {
  accountSettings: Settings | null;
  currentAppearance: DesktopAppearanceSettings;
  currentVoice: DesktopVoiceSettings;
  currentLanguage: DesktopLanguageSettings;
}): {
  appearance: DesktopAppearanceSettings;
  voice: DesktopVoiceSettings;
  language: DesktopLanguageSettings;
} {
  const { accountSettings, currentAppearance, currentVoice, currentLanguage } = input;
  if (!accountSettings) {
    return {
      appearance: currentAppearance,
      voice: currentVoice,
      language: currentLanguage,
    };
  }

  return {
    appearance: {
      ...currentAppearance,
      inlineToolCalls: accountSettings.viewInline,
      expandTodoLists: accountSettings.expandTodos,
      showLineNumbersInDiffs: accountSettings.showLineNumbers,
      showLineNumbersInToolViews: accountSettings.showLineNumbersInToolViews,
      wrapLinesInDiffs: accountSettings.wrapLinesInDiffs,
      alwaysShowContextSize: accountSettings.alwaysShowContextSize,
      showFlavorIcons: accountSettings.showFlavorIcons,
      compactSessionView: accountSettings.compactSessionView,
      avatarStyle:
        accountSettings.avatarStyle === "pixelated" ||
        accountSettings.avatarStyle === "gradient" ||
        accountSettings.avatarStyle === "brutalist"
          ? accountSettings.avatarStyle
          : currentAppearance.avatarStyle,
    },
    voice: {
      ...currentVoice,
      assistantLanguage: accountSettings.voiceAssistantLanguage,
      customAgentId: accountSettings.voiceCustomAgentId,
      bypassToken: accountSettings.voiceBypassToken,
    },
    language: {
      ...currentLanguage,
      appLanguage: accountSettings.preferredLanguage ?? currentLanguage.appLanguage,
    },
  };
}

export function mapDesktopPreferencePatchToAccountSettings(input: {
  appearancePatch?: Partial<DesktopAppearanceSettings>;
  voicePatch?: Partial<DesktopVoiceSettings>;
  languagePatch?: Partial<DesktopLanguageSettings>;
}): Partial<Settings> {
  const patch: Partial<Settings> = {};

  if (input.appearancePatch) {
    if ("inlineToolCalls" in input.appearancePatch) {
      patch.viewInline = !!input.appearancePatch.inlineToolCalls;
    }
    if ("expandTodoLists" in input.appearancePatch) {
      patch.expandTodos = !!input.appearancePatch.expandTodoLists;
    }
    if ("showLineNumbersInDiffs" in input.appearancePatch) {
      patch.showLineNumbers = !!input.appearancePatch.showLineNumbersInDiffs;
    }
    if ("showLineNumbersInToolViews" in input.appearancePatch) {
      patch.showLineNumbersInToolViews = !!input.appearancePatch.showLineNumbersInToolViews;
    }
    if ("wrapLinesInDiffs" in input.appearancePatch) {
      patch.wrapLinesInDiffs = !!input.appearancePatch.wrapLinesInDiffs;
    }
    if ("alwaysShowContextSize" in input.appearancePatch) {
      patch.alwaysShowContextSize = !!input.appearancePatch.alwaysShowContextSize;
    }
    if ("showFlavorIcons" in input.appearancePatch) {
      patch.showFlavorIcons = !!input.appearancePatch.showFlavorIcons;
    }
    if ("compactSessionView" in input.appearancePatch) {
      patch.compactSessionView = !!input.appearancePatch.compactSessionView;
    }
    if ("avatarStyle" in input.appearancePatch && input.appearancePatch.avatarStyle) {
      patch.avatarStyle = input.appearancePatch.avatarStyle;
    }
  }

  if (input.voicePatch) {
    if ("assistantLanguage" in input.voicePatch) {
      patch.voiceAssistantLanguage = input.voicePatch.assistantLanguage ?? null;
    }
    if ("customAgentId" in input.voicePatch) {
      patch.voiceCustomAgentId = input.voicePatch.customAgentId ?? null;
    }
    if ("bypassToken" in input.voicePatch) {
      patch.voiceBypassToken = !!input.voicePatch.bypassToken;
    }
  }

  if (input.languagePatch && "appLanguage" in input.languagePatch && input.languagePatch.appLanguage) {
    patch.preferredLanguage = input.languagePatch.appLanguage;
  }

  return patch;
}

export async function runOptimisticAccountSync<T>(input: {
  previousState: T;
  nextState: T;
  accountPatch: Partial<Settings>;
  syncAccountSettings: (patch: Partial<Settings>) => Promise<void>;
}): Promise<{
  finalState: T;
  rolledBack: boolean;
  error: string | null;
}> {
  if (Object.keys(input.accountPatch).length === 0) {
    return {
      finalState: input.nextState,
      rolledBack: false,
      error: null,
    };
  }

  try {
    await input.syncAccountSettings(input.accountPatch);
    return {
      finalState: input.nextState,
      rolledBack: false,
      error: null,
    };
  } catch (error) {
    return {
      finalState: input.previousState,
      rolledBack: true,
      error: error instanceof Error ? error.message : "Failed to sync desktop account settings",
    };
  }
}
