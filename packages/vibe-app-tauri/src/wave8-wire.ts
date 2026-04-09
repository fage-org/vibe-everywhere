import { z } from "zod";
import {
  ApiUpdateAccountSchema,
  ApiDeleteSessionSchema,
  ApiUpdateContainerSchema,
  ApiUpdateMachineStateSchema,
  ApiUpdateNewSessionSchema,
  ApiMessageSchema,
} from "../sources/shared/sync/types";
import {
  AgentStateSchema,
  MachineMetadataSchema,
  MetadataSchema,
} from "../sources/shared/sync/types";
import {
  UserProfileSchema,
  UserResponseSchema,
} from "../sources/shared/sync/types";

export const StoredCredentialsSchema = z.object({
  token: z.string(),
  secret: z.string(),
});
export type StoredCredentials = z.infer<typeof StoredCredentialsSchema>;

export const AccountProfileSchema = z.object({
  id: z.string(),
  firstName: z.string().nullable().optional(),
  lastName: z.string().nullable().optional(),
  username: z.string().nullable().optional(),
  connectedServices: z.array(z.string()).default([]),
});
export type AccountProfile = z.infer<typeof AccountProfileSchema>;
export type MachineMetadata = z.infer<typeof MachineMetadataSchema>;
export type UserProfile = z.infer<typeof UserProfileSchema>;

export { MachineMetadataSchema, UserResponseSchema, UserProfileSchema };

export const RemoteSessionRecordSchema = z.object({
  id: z.string(),
  seq: z.number(),
  createdAt: z.number(),
  updatedAt: z.number(),
  active: z.boolean(),
  activeAt: z.number(),
  metadata: z.string(),
  metadataVersion: z.number(),
  agentState: z.string().nullable(),
  agentStateVersion: z.number(),
  dataEncryptionKey: z.string().nullable(),
});
export type RemoteSessionRecord = z.infer<typeof RemoteSessionRecordSchema>;

export const RemoteMessageRecordSchema = ApiMessageSchema;
export type RemoteMessageRecord = z.infer<typeof RemoteMessageRecordSchema>;

export const SessionMetadataSchema = MetadataSchema.nullable();
export type SessionMetadata = z.infer<typeof SessionMetadataSchema>;

export const SessionAgentStateSchema = AgentStateSchema.nullable();
export type SessionAgentState = z.infer<typeof SessionAgentStateSchema>;

export const SessionMetadataUpdateSchema = z.object({
  value: z.string(),
  version: z.number(),
});
export type SessionMetadataUpdate = z.infer<typeof SessionMetadataUpdateSchema>;

export const SessionAgentStateUpdateSchema = z.object({
  value: z.string().nullable(),
  version: z.number(),
});
export type SessionAgentStateUpdate = z.infer<typeof SessionAgentStateUpdateSchema>;

export const ArtifactHeaderSchema = z.object({
  title: z.string().nullable(),
  sessions: z.array(z.string()).optional(),
  draft: z.boolean().optional(),
});
export type ArtifactHeader = z.infer<typeof ArtifactHeaderSchema>;

export const ArtifactBodySchema = z.object({
  body: z.string().nullable(),
});
export type ArtifactBody = z.infer<typeof ArtifactBodySchema>;

export const RemoteArtifactRecordSchema = z.object({
  id: z.string(),
  header: z.string(),
  headerVersion: z.number(),
  body: z.string().optional(),
  bodyVersion: z.number().optional(),
  dataEncryptionKey: z.string(),
  seq: z.number(),
  createdAt: z.number(),
  updatedAt: z.number(),
});
export type RemoteArtifactRecord = z.infer<typeof RemoteArtifactRecordSchema>;

export const ArtifactUpdateResponseSchema = z.discriminatedUnion("success", [
  z.object({
    success: z.literal(true),
    headerVersion: z.number().optional(),
    bodyVersion: z.number().optional(),
  }),
  z.object({
    success: z.literal(false),
    error: z.literal("version-mismatch"),
    currentHeaderVersion: z.number().optional(),
    currentBodyVersion: z.number().optional(),
    currentHeader: z.string().optional(),
    currentBody: z.string().optional(),
  }),
]);
export type ArtifactUpdateResponse = z.infer<typeof ArtifactUpdateResponseSchema>;

export const MachineDaemonStateSchema = z.unknown().nullable();

export const RemoteMachineRecordSchema = z.object({
  id: z.string(),
  seq: z.number(),
  createdAt: z.number(),
  updatedAt: z.number(),
  active: z.boolean(),
  activeAt: z.number(),
  metadata: z.string(),
  metadataVersion: z.number(),
  daemonState: z.string().nullable().optional(),
  daemonStateVersion: z.number(),
  dataEncryptionKey: z.string().nullable().optional(),
});
export type RemoteMachineRecord = z.infer<typeof RemoteMachineRecordSchema>;

export const MachineDetailResponseSchema = z.object({
  machine: RemoteMachineRecordSchema,
});

export const MachinesResponseSchema = z.array(RemoteMachineRecordSchema);

export const SessionRpcAckSchema = z.discriminatedUnion("ok", [
  z.object({
    ok: z.literal(true),
    result: z.string(),
  }),
  z.object({
    ok: z.literal(false),
    error: z.string().optional(),
  }),
]);
export type SessionRpcAck = z.infer<typeof SessionRpcAckSchema>;

export const SessionBashRequestSchema = z.object({
  command: z.string(),
  cwd: z.string().optional(),
  timeout: z.number().optional(),
});
export type SessionBashRequest = z.infer<typeof SessionBashRequestSchema>;

export const SessionBashResponseSchema = z.object({
  success: z.boolean(),
  stdout: z.string(),
  stderr: z.string(),
  exitCode: z.number(),
  error: z.string().optional(),
});
export type SessionBashResponse = z.infer<typeof SessionBashResponseSchema>;

export const SessionReadFileRequestSchema = z.object({
  path: z.string(),
});
export type SessionReadFileRequest = z.infer<typeof SessionReadFileRequestSchema>;

export const SessionReadFileResponseSchema = z.object({
  success: z.boolean(),
  content: z.string().optional(),
  error: z.string().optional(),
});
export type SessionReadFileResponse = z.infer<typeof SessionReadFileResponseSchema>;

export const SessionsResponseSchema = z.object({
  sessions: z.array(RemoteSessionRecordSchema),
});

export const SessionResponseSchema = z.object({
  session: RemoteSessionRecordSchema,
});

export const SessionMessagesResponseSchema = z.object({
  messages: z.array(RemoteMessageRecordSchema),
  hasMore: z.boolean(),
});

export const ArtifactsResponseSchema = z.array(RemoteArtifactRecordSchema);

export const AccountTokenResponseSchema = z.object({
  success: z.boolean(),
  token: z.string(),
});

export const UsageBucketSchema = z.object({
  timestamp: z.number(),
  tokens: z.record(z.string(), z.number()).default({}),
  cost: z.record(z.string(), z.number()).default({}),
  reportCount: z.number(),
});
export type UsageBucket = z.infer<typeof UsageBucketSchema>;

export const UsageQueryResponseSchema = z.object({
  usage: z.array(UsageBucketSchema),
  groupBy: z.enum(["hour", "day"]),
  totalReports: z.number(),
});
export type UsageQueryResponse = z.infer<typeof UsageQueryResponseSchema>;

export const AccountLinkRequestResponseSchema = z.discriminatedUnion("state", [
  z.object({
    state: z.literal("requested"),
  }),
  z.object({
    state: z.literal("authorized"),
    token: z.string(),
    response: z.string(),
  }),
]);

export const LoopbackAttemptSchema = z.object({
  attemptId: z.string(),
  browserUrl: z.string(),
});
export type LoopbackAttempt = z.infer<typeof LoopbackAttemptSchema>;

export const LoopbackAttemptStatusSchema = z.discriminatedUnion("status", [
  z.object({
    status: z.literal("pending"),
  }),
  z.object({
    status: z.literal("completed"),
  }),
  z.object({
    status: z.literal("failed"),
    error: z.string(),
  }),
  z.object({
    status: z.literal("canceled"),
  }),
  z.object({
    status: z.literal("not_found"),
  }),
]);
export type LoopbackAttemptStatus = z.infer<typeof LoopbackAttemptStatusSchema>;

export const SessionsRealtimeUpdateSchema = ApiUpdateContainerSchema.extend({
  body: z.union([
    ApiUpdateNewSessionSchema,
    ApiDeleteSessionSchema,
    ApiUpdateAccountSchema,
    z.object({
      t: z.literal("new-message"),
      sid: z.string(),
      message: RemoteMessageRecordSchema,
    }),
    z.object({
      t: z.literal("update-session"),
      id: z.string(),
      metadata: SessionMetadataUpdateSchema.nullish(),
      agentState: SessionAgentStateUpdateSchema.nullish(),
    }),
    z.object({
      t: z.literal("new-artifact"),
      artifactId: z.string(),
      header: z.string(),
      headerVersion: z.number(),
      body: z.string().optional(),
      bodyVersion: z.number().optional(),
      dataEncryptionKey: z.string(),
      seq: z.number(),
      createdAt: z.number(),
      updatedAt: z.number(),
    }),
    z.object({
      t: z.literal("update-artifact"),
      artifactId: z.string(),
      header: SessionMetadataUpdateSchema.optional(),
      body: SessionMetadataUpdateSchema.optional(),
    }),
    z.object({
      t: z.literal("delete-artifact"),
      artifactId: z.string(),
    }),
    ApiUpdateMachineStateSchema,
  ]),
});
export type SessionsRealtimeUpdate = z.infer<typeof SessionsRealtimeUpdateSchema>;

function summarizeZodError(error: z.ZodError): string {
  return error.issues
    .map((issue: z.ZodIssue) => {
      const path = issue.path.length > 0 ? issue.path.join(".") : "<root>";
      return `${path}: ${issue.message}`;
    })
    .join("; ");
}

export function parseWithSchema<T>(
  schema: z.ZodType<T>,
  value: unknown,
  label: string,
): T {
  const parsed = schema.safeParse(value);
  if (!parsed.success) {
    throw new Error(`${label} is invalid: ${summarizeZodError(parsed.error)}`);
  }
  return parsed.data;
}

export function safeParseWithSchema<T>(
  schema: z.ZodType<T>,
  value: unknown,
): T | null {
  const parsed = schema.safeParse(value);
  return parsed.success ? parsed.data : null;
}
