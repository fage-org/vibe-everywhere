import { z } from "zod";

export const GitHubProfileSchema = z.object({
  id: z.number(),
  login: z.string(),
  name: z.string(),
  avatar_url: z.string(),
  email: z.string().optional(),
  bio: z.string().nullable(),
});

export const ImageRefSchema = z.object({
  width: z.number(),
  height: z.number(),
  thumbhash: z.string(),
  path: z.string(),
  url: z.string(),
});

export const FeedBodySchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("friend_request"), uid: z.string() }),
  z.object({ kind: z.literal("friend_accepted"), uid: z.string() }),
  z.object({ kind: z.literal("text"), text: z.string() }),
]);

export const RelationshipStatusSchema = z.enum([
  "none",
  "requested",
  "pending",
  "friend",
  "rejected",
]);

export const UserProfileSchema = z.object({
  id: z.string(),
  firstName: z.string(),
  lastName: z.string().nullable(),
  avatar: z.object({
    path: z.string(),
    url: z.string(),
    width: z.number().optional(),
    height: z.number().optional(),
    thumbhash: z.string().optional(),
  }).nullable(),
  username: z.string(),
  bio: z.string().nullable(),
  status: RelationshipStatusSchema,
});

export const UserResponseSchema = z.object({
  user: UserProfileSchema,
});

export const MetadataSchema = z.object({
  models: z.array(z.object({
    code: z.string(),
    value: z.string(),
    description: z.string().nullish(),
  })).optional(),
  currentModelCode: z.string().optional(),
  operatingModes: z.array(z.object({
    code: z.string(),
    value: z.string(),
    description: z.string().nullish(),
  })).optional(),
  currentOperatingModeCode: z.string().optional(),
  thoughtLevels: z.array(z.object({
    code: z.string(),
    value: z.string(),
    description: z.string().nullish(),
  })).optional(),
  currentThoughtLevelCode: z.string().optional(),
  path: z.string(),
  host: z.string(),
  version: z.string().optional(),
  name: z.string().optional(),
  os: z.string().optional(),
  summary: z.object({
    text: z.string(),
    updatedAt: z.number(),
  }).optional(),
  machineId: z.string().optional(),
  claudeSessionId: z.string().optional(),
  codexThreadId: z.string().optional(),
  tools: z.array(z.string()).optional(),
  slashCommands: z.array(z.string()).optional(),
  homeDir: z.string().optional(),
  happyHomeDir: z.string().optional(),
  hostPid: z.number().optional(),
  flavor: z.string().nullish(),
  sandbox: z.unknown().nullish(),
  dangerouslySkipPermissions: z.boolean().nullish(),
  lifecycleState: z.string().optional(),
  lifecycleStateSince: z.number().optional(),
  archivedBy: z.string().optional(),
  archiveReason: z.string().optional(),
});

export const AgentStateSchema = z.object({
  controlledByUser: z.boolean().nullish(),
  requests: z.record(z.string(), z.object({
    tool: z.string(),
    arguments: z.unknown(),
    createdAt: z.number().nullish(),
  })).nullish(),
  completedRequests: z.record(z.string(), z.object({
    tool: z.string(),
    arguments: z.unknown(),
    createdAt: z.number().nullish(),
    completedAt: z.number().nullish(),
    status: z.enum(["canceled", "denied", "approved"]),
    reason: z.string().nullish(),
    mode: z.string().nullish(),
    allowedTools: z.array(z.string()).nullish(),
    decision: z.enum(["approved", "approved_for_session", "denied", "abort"]).nullish(),
  })).nullish(),
});

export const MachineMetadataSchema = z.object({
  host: z.string(),
  platform: z.string(),
  happyCliVersion: z.string(),
  happyHomeDir: z.string(),
  homeDir: z.string(),
  username: z.string().optional(),
  arch: z.string().optional(),
  displayName: z.string().optional(),
  daemonLastKnownStatus: z.enum(["running", "shutting-down"]).optional(),
  daemonLastKnownPid: z.number().optional(),
  shutdownRequestedAt: z.number().optional(),
  shutdownSource: z.enum(["happy-app", "happy-cli", "os-signal", "unknown"]).optional(),
  cliAvailability: z.object({
    claude: z.boolean(),
    codex: z.boolean(),
    gemini: z.boolean(),
    openclaw: z.boolean(),
    detectedAt: z.number(),
  }).optional(),
  resumeSupport: z.object({
    rpcAvailable: z.boolean(),
    requiresSameMachine: z.boolean(),
    requiresHappyAgentAuth: z.boolean(),
    happyAgentAuthenticated: z.boolean(),
    detectedAt: z.number(),
  }).optional(),
});

export const SessionMessageContentSchema = z.object({
  c: z.string(),
  t: z.literal("encrypted"),
});

export const ApiMessageSchema = z.object({
  id: z.string(),
  seq: z.number(),
  localId: z.string().nullish(),
  content: SessionMessageContentSchema,
  createdAt: z.number(),
  updatedAt: z.number(),
});

export const VersionedEncryptedValueSchema = z.object({
  version: z.number(),
  value: z.string(),
});

export const VersionedNullableEncryptedValueSchema = z.object({
  version: z.number(),
  value: z.string().nullable(),
});

export const VersionedMachineEncryptedValueSchema = z.object({
  version: z.number(),
  value: z.string(),
});

export const ApiUpdateNewMessageSchema = z.object({
  t: z.literal("new-message"),
  sid: z.string(),
  message: ApiMessageSchema,
});

export const ApiUpdateSessionStateSchema = z.object({
  t: z.literal("update-session"),
  id: z.string(),
  metadata: VersionedEncryptedValueSchema.nullish(),
  agentState: VersionedNullableEncryptedValueSchema.nullish(),
});

export const ApiUpdateMachineStateSchema = z.object({
  t: z.literal("update-machine"),
  machineId: z.string(),
  metadata: VersionedMachineEncryptedValueSchema.nullish(),
  daemonState: VersionedMachineEncryptedValueSchema.nullish(),
  active: z.boolean().optional(),
  activeAt: z.number().optional(),
});

export const ApiUpdateNewSessionSchema = z.object({
  t: z.literal("new-session"),
  id: z.string(),
  createdAt: z.number(),
  updatedAt: z.number(),
});

export const ApiDeleteSessionSchema = z.object({
  t: z.literal("delete-session"),
  sid: z.string(),
});

export const ApiUpdateAccountSchema = z.object({
  t: z.literal("update-account"),
  id: z.string(),
  settings: z.object({
    value: z.string().nullish(),
    version: z.number(),
  }).nullish(),
  firstName: z.string().nullish(),
  lastName: z.string().nullish(),
  avatar: ImageRefSchema.nullish(),
  github: GitHubProfileSchema.nullish(),
});

export const ApiNewArtifactSchema = z.object({
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
});

export const ApiUpdateArtifactSchema = z.object({
  t: z.literal("update-artifact"),
  artifactId: z.string(),
  header: z.object({
    value: z.string(),
    version: z.number(),
  }).optional(),
  body: z.object({
    value: z.string(),
    version: z.number(),
  }).optional(),
});

export const ApiDeleteArtifactSchema = z.object({
  t: z.literal("delete-artifact"),
  artifactId: z.string(),
});

export const ApiRelationshipUpdatedSchema = z.object({
  t: z.literal("relationship-updated"),
  fromUserId: z.string(),
  toUserId: z.string(),
  status: RelationshipStatusSchema,
  action: z.enum(["created", "updated", "deleted"]),
  fromUser: UserProfileSchema.optional(),
  toUser: UserProfileSchema.optional(),
  timestamp: z.number(),
});

export const ApiNewFeedPostSchema = z.object({
  t: z.literal("new-feed-post"),
  id: z.string(),
  body: FeedBodySchema,
  cursor: z.string(),
  createdAt: z.number(),
  repeatKey: z.string().nullable(),
});

export const ApiKvBatchUpdateSchema = z.object({
  t: z.literal("kv-batch-update"),
  changes: z.array(z.object({
    key: z.string(),
    value: z.string().nullable(),
    version: z.number(),
  })),
});

export const ApiUpdateSchema = z.union([
  ApiUpdateNewMessageSchema,
  ApiUpdateNewSessionSchema,
  ApiDeleteSessionSchema,
  ApiUpdateSessionStateSchema,
  ApiUpdateAccountSchema,
  ApiUpdateMachineStateSchema,
  ApiNewArtifactSchema,
  ApiUpdateArtifactSchema,
  ApiDeleteArtifactSchema,
  ApiRelationshipUpdatedSchema,
  ApiNewFeedPostSchema,
  ApiKvBatchUpdateSchema,
]);

export const ApiUpdateContainerSchema = z.object({
  id: z.string(),
  seq: z.number(),
  body: ApiUpdateSchema,
  createdAt: z.number(),
});
