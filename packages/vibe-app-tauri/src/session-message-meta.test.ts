import { describe, expect, it } from "vitest";
import { buildOutgoingUserMessageRecord } from "./session-message-meta";

describe("session message meta", () => {
  it("builds an outgoing user record with message meta", () => {
    expect(
      buildOutgoingUserMessageRecord("Ship it", {
        sentFrom: "web",
        permissionMode: "plan",
        model: "gpt-5.4",
        displayText: "Ship it",
      }),
    ).toEqual({
      role: "user",
      content: {
        type: "text",
        text: "Ship it",
      },
      meta: {
        sentFrom: "web",
        permissionMode: "plan",
        model: "gpt-5.4",
        displayText: "Ship it",
      },
    });
  });

  it("keeps model reset explicit when no override is provided", () => {
    expect(
      buildOutgoingUserMessageRecord("Ship it", {
        sentFrom: "android",
      }),
    ).toEqual({
      role: "user",
      content: {
        type: "text",
        text: "Ship it",
      },
      meta: {
        sentFrom: "android",
        permissionMode: undefined,
        model: null,
      },
    });
  });
});
