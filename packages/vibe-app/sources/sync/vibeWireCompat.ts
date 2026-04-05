import { z } from 'zod';

export const SessionMessageContentSchema = z.object({
    c: z.string(),
    t: z.literal('encrypted'),
});
export type SessionMessageContent = z.infer<typeof SessionMessageContentSchema>;

export const ApiMessageSchema = z.object({
    id: z.string(),
    seq: z.number(),
    localId: z.string().nullish(),
    content: SessionMessageContentSchema,
    createdAt: z.number(),
    updatedAt: z.number(),
});
export type ApiMessage = z.infer<typeof ApiMessageSchema>;

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
    t: z.literal('new-message'),
    sid: z.string(),
    message: ApiMessageSchema,
});

export const ApiUpdateSessionStateSchema = z.object({
    t: z.literal('update-session'),
    id: z.string(),
    metadata: VersionedEncryptedValueSchema.nullish(),
    agentState: VersionedNullableEncryptedValueSchema.nullish(),
});

export const ApiUpdateMachineStateSchema = z.object({
    t: z.literal('update-machine'),
    machineId: z.string(),
    metadata: VersionedMachineEncryptedValueSchema.nullish(),
    daemonState: VersionedMachineEncryptedValueSchema.nullish(),
    active: z.boolean().optional(),
    activeAt: z.number().optional(),
});

export const VoiceTokenAllowedSchema = z.object({
    allowed: z.literal(true),
    token: z.string(),
    agentId: z.string(),
    elevenUserId: z.string(),
    usedSeconds: z.number(),
    limitSeconds: z.number(),
});

export const VoiceTokenDeniedSchema = z.object({
    allowed: z.literal(false),
    reason: z.enum(['voice_limit_reached', 'subscription_required']),
    usedSeconds: z.number(),
    limitSeconds: z.number(),
    agentId: z.string(),
});

export const VoiceTokenResponseSchema = z.discriminatedUnion('allowed', [
    VoiceTokenAllowedSchema,
    VoiceTokenDeniedSchema,
]);
export type VoiceTokenResponse = z.infer<typeof VoiceTokenResponseSchema>;
