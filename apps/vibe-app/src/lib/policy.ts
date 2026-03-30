import type {
  ConversationDetailResponse,
  ExecutionProtocol,
  ProviderKind,
  ProviderStatus,
  TaskExecutionMode
} from "@/types";

export type PolicyRuntimeContext = {
  provider: ProviderKind;
  executionProtocol: ExecutionProtocol;
};

export type SettingsPolicyRow = {
  id: string;
  provider: ProviderKind;
  protocol: ExecutionProtocol;
  mode: TaskExecutionMode;
};

export type NotificationPreference = "all" | "important";

export type NotificationPreferenceDescriptor = {
  effectivePreference: NotificationPreference;
  inherited: boolean;
};

export function preferredProjectProvider(providers: ProviderKind[] | null | undefined): ProviderKind {
  return providers?.[0] ?? "codex";
}

export function executionProtocolForProvider(provider: ProviderKind): ExecutionProtocol {
  return provider === "open_code" ? "acp" : "cli";
}

export function resolvePolicyRuntimeContext(
  detail: ConversationDetailResponse | null,
  projectProviders: ProviderKind[] | null | undefined
): PolicyRuntimeContext {
  if (detail) {
    return {
      provider: detail.conversation.provider,
      executionProtocol: detail.conversation.executionProtocol
    };
  }

  const provider = preferredProjectProvider(projectProviders);
  return {
    provider,
    executionProtocol: executionProtocolForProvider(provider)
  };
}

export function buildSettingsPolicyRows(providerStatuses: ProviderStatus[]): SettingsPolicyRow[] {
  const providerCombos = new Map<string, { provider: ProviderKind; protocol: ExecutionProtocol }>();

  for (const status of providerStatuses) {
    const key = `${status.kind}:${status.executionProtocol}`;
    if (!providerCombos.has(key)) {
      providerCombos.set(key, {
        provider: status.kind,
        protocol: status.executionProtocol
      });
    }
  }

  const rows: SettingsPolicyRow[] = [];
  for (const { provider, protocol } of providerCombos.values()) {
    for (const mode of [
      "read_only",
      "workspace_write",
      "workspace_write_and_test"
    ] as TaskExecutionMode[]) {
      rows.push({
        id: `${provider}-${protocol}-${mode}`,
        provider,
        protocol,
        mode
      });
    }
  }

  return rows;
}

export function describeNotificationPreference(
  defaultPreference: NotificationPreference,
  projectOverride: NotificationPreference | null
): NotificationPreferenceDescriptor {
  return {
    effectivePreference: projectOverride ?? defaultPreference,
    inherited: projectOverride === null
  };
}
