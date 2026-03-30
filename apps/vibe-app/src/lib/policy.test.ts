import { describe, expect, it } from "vitest";
import {
  buildSettingsPolicyRows,
  describeNotificationPreference,
  resolvePolicyRuntimeContext
} from "@/lib/policy";
import type { ConversationDetailResponse, ProviderStatus } from "@/types";

describe("resolvePolicyRuntimeContext", () => {
  it("uses conversation runtime when detail exists", () => {
    const detail = {
      conversation: {
        provider: "claude_code",
        executionProtocol: "cli"
      }
    } as ConversationDetailResponse;

    expect(resolvePolicyRuntimeContext(detail, ["open_code"])).toEqual({
      provider: "claude_code",
      executionProtocol: "cli"
    });
  });

  it("uses preferred project provider when detail is missing", () => {
    expect(resolvePolicyRuntimeContext(null, ["open_code", "codex"])).toEqual({
      provider: "open_code",
      executionProtocol: "acp"
    });
  });
});

describe("buildSettingsPolicyRows", () => {
  it("builds rows from real provider and protocol combinations", () => {
    const providers = [
      {
        kind: "codex",
        executionProtocol: "cli"
      },
      {
        kind: "open_code",
        executionProtocol: "acp"
      }
    ] as ProviderStatus[];

    const rows = buildSettingsPolicyRows(providers);

    expect(rows.some((row) => row.provider === "open_code" && row.protocol === "acp")).toBe(true);
    expect(rows.some((row) => row.provider === "codex" && row.protocol === "acp")).toBe(false);
  });
});

describe("describeNotificationPreference", () => {
  it("marks project override separately from inherited default", () => {
    expect(describeNotificationPreference("important", null)).toEqual({
      effectivePreference: "important",
      inherited: true
    });
    expect(describeNotificationPreference("important", "all")).toEqual({
      effectivePreference: "all",
      inherited: false
    });
  });
});
