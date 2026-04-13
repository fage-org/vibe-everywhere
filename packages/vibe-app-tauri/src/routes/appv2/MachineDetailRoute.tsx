import { useEffect, useState, useMemo, useCallback } from "react";
import { SettingsSurface, type SettingSection } from "../../components/routes";
import { useDesktopState } from "../../useDesktopState";
import type { DesktopMachine } from "../../desktop-client";
import type { SpawnSessionRpcResult } from "../../desktop-wire";
import { useTranslation } from "react-i18next";
import { Body, Title3 } from "../../components/ui/Typography";
import { tokens } from "../../design-system/tokens";

/**
 * MachineDetailRouteProps - Props for the machine detail route
 */
export interface MachineDetailRouteProps {
  machineId: string;
  onNavigateToSession?: (sessionId: string) => void;
}

/**
 * Check if a machine is online based on activeAt timestamp
 */
function isMachineOnline(machine: DesktopMachine | null): boolean {
  if (!machine) return false;
  // Consider online if active within last 60 seconds
  const sixtySecondsAgo = Date.now() / 1000 - 60;
  return machine.activeAt > sixtySecondsAgo;
}

/**
 * MachineDetailRoute - Machine detail page
 *
 * Displays machine information, daemon status, CLI availability,
 * and allows spawning new sessions.
 */
export function MachineDetailRoute({
  machineId,
  onNavigateToSession,
}: MachineDetailRouteProps) {
  const { t } = useTranslation("routes");
  const { status, loadMachine, sessions, spawnSessionOnMachine, stopMachineDaemon, refreshMachines, refreshSessions } = useDesktopState();
  const [machine, setMachine] = useState<DesktopMachine | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Spawn session state
  const [spawnPath, setSpawnPath] = useState("");
  const [isSpawning, setIsSpawning] = useState(false);
  const [spawnError, setSpawnError] = useState<string | null>(null);
  const [approvalNeeded, setApprovalNeeded] = useState<string | null>(null);

  // Stop daemon state
  const [isStopping, setIsStopping] = useState(false);
  const [stopError, setStopError] = useState<string | null>(null);

  useEffect(() => {
    const fetchMachine = async () => {
      if (status !== "ready") {
        setError("Not connected");
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        const result = await loadMachine(machineId);
        setMachine(result);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load machine");
      } finally {
        setLoading(false);
      }
    };

    fetchMachine();
  }, [status, machineId, loadMachine]);

  // Determine daemon status
  const daemonStatus = useMemo(() => {
    if (!machine) return "unknown";
    const metadata = machine.metadata;
    if (metadata?.daemonLastKnownStatus === "shutting-down") {
      return "stopped";
    }
    return isMachineOnline(machine) ? "likely alive" : "stopped";
  }, [machine]);

  // Get sessions for this machine
  const machineSessions = useMemo(() => {
    if (!sessions || !machineId) return [];
    return sessions.filter((session) => {
      return session.metadata?.machineId === machineId;
    });
  }, [sessions, machineId]);

  // Get recent paths from sessions
  const recentPaths = useMemo(() => {
    const paths = new Set<string>();
    machineSessions.forEach((session) => {
      if (session.metadata?.path) {
        paths.add(session.metadata.path);
      }
    });
    return Array.from(paths).sort().slice(0, 5);
  }, [machineSessions]);

  const metadata = machine?.metadata;
  const machineName = metadata?.displayName || metadata?.host || "unknown machine";
  const isOnline = isMachineOnline(machine);

  // Handle spawn session
  const handleSpawnSession = useCallback(
    async (approvedDirectory?: string) => {
      if (!isOnline || isSpawning) return;

      const targetPath = approvedDirectory || spawnPath;
      if (!targetPath.trim()) {
        setSpawnError("Please enter a working directory");
        return;
      }

      setIsSpawning(true);
      setSpawnError(null);
      setApprovalNeeded(null);

      try {
        const result: SpawnSessionRpcResult = await spawnSessionOnMachine(machineId, {
          directory: targetPath.trim(),
          approvedNewDirectoryCreation: !!approvedDirectory,
        });

        if (result.type === "success") {
          // Refresh sessions and navigate to the new session
          await refreshSessions();
          if (onNavigateToSession) {
            onNavigateToSession(result.sessionId);
          }
          setSpawnPath("");
        } else if (result.type === "requestToApproveDirectoryCreation") {
          // Need approval for directory creation
          setApprovalNeeded(result.directory);
        } else if (result.type === "error") {
          setSpawnError(result.errorMessage);
        }
      } catch (err) {
        setSpawnError(err instanceof Error ? err.message : "Failed to spawn session");
      } finally {
        setIsSpawning(false);
      }
    },
    [machineId, isOnline, spawnPath, isSpawning, spawnSessionOnMachine, refreshSessions, onNavigateToSession],
  );

  // Handle stop daemon
  const handleStopDaemon = useCallback(
    async () => {
      if (!isOnline || isStopping) return;

      setIsStopping(true);
      setStopError(null);

      try {
        await stopMachineDaemon(machineId);
        // Refresh machine state
        await refreshMachines();
        const updatedMachine = await loadMachine(machineId);
        setMachine(updatedMachine);
      } catch (err) {
        setStopError(err instanceof Error ? err.message : "Failed to stop daemon");
      } finally {
        setIsStopping(false);
      }
    },
    [machineId, isOnline, isStopping, stopMachineDaemon, refreshMachines, loadMachine],
  );

  // Handle recent path selection
  const handleSelectRecentPath = useCallback(
    (path: string) => {
      setSpawnPath(path);
      setSpawnError(null);
      setApprovalNeeded(null);
    },
    [],
  );

  // Machine information section
  const machineInfoSection: SettingSection = {
    id: "machine-info",
    title: t("machine.machineGroup"),
    settings: [
      {
        id: "host",
        label: t("machine.host"),
        type: "custom",
        value: metadata?.host || machineId,
        render: () => (
          <Body style={{ fontFamily: tokens.typography.fontFamily.mono }}>
            {metadata?.host || machineId}
          </Body>
        ),
      },
      {
        id: "machineId",
        label: t("machine.machineId"),
        type: "custom",
        value: machineId,
        render: () => (
          <Body style={{ fontFamily: tokens.typography.fontFamily.mono, fontSize: tokens.typography.fontSize.xs }}>
            {machineId}
          </Body>
        ),
      },
      ...(metadata?.username
        ? [{
            id: "username",
            label: t("machine.username"),
            type: "custom" as const,
            value: metadata.username,
            render: () => <Body>{metadata.username}</Body>,
          }]
        : []),
      ...(metadata?.homeDir
        ? [{
            id: "homeDir",
            label: t("machine.homeDirectory"),
            type: "custom" as const,
            value: metadata.homeDir,
            render: () => (
              <Body style={{ fontFamily: tokens.typography.fontFamily.mono, fontSize: tokens.typography.fontSize.sm }}>
                {metadata.homeDir}
              </Body>
            ),
          }]
        : []),
      ...(metadata?.platform
        ? [{
            id: "platform",
            label: t("machine.platform"),
            type: "custom" as const,
            value: metadata.platform,
            render: () => <Body>{metadata.platform}</Body>,
          }]
        : []),
      ...(metadata?.arch
        ? [{
            id: "arch",
            label: t("machine.architecture"),
            type: "custom" as const,
            value: metadata.arch,
            render: () => <Body>{metadata.arch}</Body>,
          }]
        : []),
      {
        id: "lastSeen",
        label: t("machine.lastSeen"),
        type: "custom",
        value: machine?.activeAt,
        render: () => (
          <Body color="secondary">
            {machine?.activeAt ? new Date(machine.activeAt * 1000).toLocaleString() : t("machine.never")}
          </Body>
        ),
      },
    ],
  };

  // Daemon status section
  const daemonSection: SettingSection = {
    id: "daemon",
    title: t("machine.daemon"),
    settings: [
      {
        id: "daemonStatus",
        label: t("machine.status"),
        type: "custom",
        value: daemonStatus,
        render: () => (
          <Body
            style={{
              color: daemonStatus === "likely alive" ? "var(--color-success)" : "var(--color-warning)",
            }}
          >
            {daemonStatus}
          </Body>
        ),
      },
      {
        id: "daemonStateVersion",
        label: t("machine.daemonStateVersion"),
        type: "custom",
        value: machine?.daemonStateVersion,
        render: () => <Body color="secondary">{machine?.daemonStateVersion ?? 0}</Body>,
      },
    ],
  };

  // CLI availability section
  const cliAvailability = metadata?.cliAvailability;
  const cliSection: SettingSection | null = cliAvailability
    ? {
        id: "cli-availability",
        title: t("machine.cliAvailability"),
        settings: [
          {
            id: "claude",
            label: "Claude",
            type: "custom",
            value: cliAvailability.claude,
            render: () => (
              <Body style={{ color: cliAvailability.claude ? "var(--color-success)" : "var(--color-text-secondary)" }}>
                {cliAvailability.claude ? t("machine.cliInstalled") : t("machine.cliNotFound")}
              </Body>
            ),
          },
          {
            id: "codex",
            label: "Codex",
            type: "custom",
            value: cliAvailability.codex,
            render: () => (
              <Body style={{ color: cliAvailability.codex ? "var(--color-success)" : "var(--color-text-secondary)" }}>
                {cliAvailability.codex ? t("machine.cliInstalled") : t("machine.cliNotFound")}
              </Body>
            ),
          },
          {
            id: "gemini",
            label: "Gemini",
            type: "custom",
            value: cliAvailability.gemini,
            render: () => (
              <Body style={{ color: cliAvailability.gemini ? "var(--color-success)" : "var(--color-text-secondary)" }}>
                {cliAvailability.gemini ? t("machine.cliInstalled") : t("machine.cliNotFound")}
              </Body>
            ),
          },
          {
            id: "openclaw",
            label: "OpenClaw",
            type: "custom",
            value: cliAvailability.openclaw,
            render: () => (
              <Body style={{ color: cliAvailability.openclaw ? "var(--color-success)" : "var(--color-text-secondary)" }}>
                {cliAvailability.openclaw ? t("machine.cliInstalled") : t("machine.cliNotFound")}
              </Body>
            ),
          },
          {
            id: "detectedAt",
            label: t("machine.lastDetected"),
            type: "custom",
            value: cliAvailability.detectedAt,
            render: () => (
              <Body color="secondary">
                {new Date(cliAvailability.detectedAt * 1000).toLocaleString()}
              </Body>
            ),
          },
        ],
      }
    : null;

  const sections = [
    machineInfoSection,
    daemonSection,
    ...(cliSection ? [cliSection] : []),
  ];

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", overflow: "auto" }}>
      {/* Header with status indicator */}
      <div
        style={{
          padding: tokens.spacing[4],
          borderBottom: "1px solid var(--border-primary)",
          display: "flex",
          alignItems: "center",
          gap: tokens.spacing[3],
        }}
      >
        <MachineStatusIndicator isOnline={isOnline} />
        <div>
          <Title3>{machineName}</Title3>
          <Body color="secondary" style={{ fontSize: tokens.typography.fontSize.sm }}>
            {isOnline ? t("status.online") : t("status.offline")}
          </Body>
        </div>
      </div>

      {error && (
        <div style={{ padding: tokens.spacing[4], backgroundColor: "var(--surface-error)" }}>
          <Body style={{ color: "var(--color-danger)" }}>{error}</Body>
        </div>
      )}

      <SettingsSurface
        sections={sections}
        loading={loading}
      />

      {/* Spawn Session Section */}
      <div
        style={{
          padding: tokens.spacing[6],
          borderTop: "1px solid var(--border-primary)",
        }}
      >
        <Title3 style={{ marginBottom: tokens.spacing[2] }}>
          {t("machine.spawnSession.title")}
        </Title3>
        <Body color="secondary" style={{ marginBottom: tokens.spacing[4], fontSize: tokens.typography.fontSize.sm }}>
          {t("machine.spawnSession.description")}
        </Body>

        {/* Path input */}
        <div style={{ marginBottom: tokens.spacing[3] }}>
          <label
            style={{
              display: "block",
              marginBottom: tokens.spacing[2],
              fontSize: tokens.typography.fontSize.sm,
              color: "var(--color-text-secondary)",
            }}
          >
            {t("machine.spawnSession.pathLabel")}
          </label>
          <input
            type="text"
            value={spawnPath}
            onChange={(e) => {
              setSpawnPath(e.target.value);
              setSpawnError(null);
              setApprovalNeeded(null);
            }}
            placeholder={t("machine.spawnSession.pathPlaceholder")}
            disabled={!isOnline || isSpawning}
            style={{
              width: "100%",
              padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
              fontSize: tokens.typography.fontSize.sm,
              fontFamily: tokens.typography.fontFamily.mono,
              borderRadius: tokens.radii.sm,
              border: "1px solid var(--border-primary)",
              backgroundColor: isOnline ? "var(--surface-primary)" : "var(--surface-disabled)",
              color: "var(--color-text-primary)",
            }}
          />
        </div>

        {/* Recent paths as quick select */}
        {recentPaths.length > 0 && (
          <div style={{ marginBottom: tokens.spacing[3] }}>
            <Body
              color="secondary"
              style={{
                fontSize: tokens.typography.fontSize.xs,
                marginBottom: tokens.spacing[2],
              }}
            >
              {t("machine.spawnSession.recentPaths")}:
            </Body>
            <div style={{ display: "flex", flexWrap: "wrap", gap: tokens.spacing[2] }}>
              {recentPaths.map((path) => (
                <button
                  key={path}
                  onClick={() => handleSelectRecentPath(path)}
                  disabled={!isOnline || isSpawning}
                  style={{
                    padding: `${tokens.spacing[1]} ${tokens.spacing[2]}`,
                    fontSize: tokens.typography.fontSize.xs,
                    fontFamily: tokens.typography.fontFamily.mono,
                    borderRadius: tokens.radii.sm,
                    border: spawnPath === path ? "1px solid var(--color-primary)" : "1px solid var(--border-primary)",
                    backgroundColor: spawnPath === path ? "var(--surface-selected)" : "var(--surface-secondary)",
                    color: "var(--color-text-secondary)",
                    cursor: isOnline && !isSpawning ? "pointer" : "default",
                  }}
                >
                  {path.length > 30 ? "..." + path.slice(-27) : path}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Error message */}
        {spawnError && (
          <div
            style={{
              padding: tokens.spacing[3],
              marginBottom: tokens.spacing[3],
              backgroundColor: "var(--surface-error)",
              borderRadius: tokens.radii.sm,
            }}
          >
            <Body style={{ color: "var(--color-danger)", fontSize: tokens.typography.fontSize.sm }}>
              {t("machine.spawnSession.errorPrefix")}{spawnError}
            </Body>
          </div>
        )}

        {/* Approval needed for directory creation */}
        {approvalNeeded && (
          <div
            style={{
              padding: tokens.spacing[3],
              marginBottom: tokens.spacing[3],
              backgroundColor: "var(--surface-warning)",
              borderRadius: tokens.radii.sm,
            }}
          >
            <Body style={{ marginBottom: tokens.spacing[2] }}>
              {t("machine.spawnSession.approveDirectoryDescription")}
            </Body>
            <Body style={{ fontFamily: tokens.typography.fontFamily.mono, marginBottom: tokens.spacing[3] }}>
              {approvalNeeded}
            </Body>
            <div style={{ display: "flex", gap: tokens.spacing[2] }}>
              <button
                onClick={() => handleSpawnSession(approvalNeeded)}
                disabled={isSpawning}
                style={{
                  padding: `${tokens.spacing[2]} ${tokens.spacing[4]}`,
                  fontSize: tokens.typography.fontSize.sm,
                  borderRadius: tokens.radii.sm,
                  border: "none",
                  backgroundColor: "var(--color-primary)",
                  color: "white",
                  cursor: isSpawning ? "wait" : "pointer",
                }}
              >
                {t("machine.spawnSession.approveDirectory")}
              </button>
              <button
                onClick={() => setApprovalNeeded(null)}
                disabled={isSpawning}
                style={{
                  padding: `${tokens.spacing[2]} ${tokens.spacing[4]}`,
                  fontSize: tokens.typography.fontSize.sm,
                  borderRadius: tokens.radii.sm,
                  border: "1px solid var(--border-primary)",
                  backgroundColor: "transparent",
                  color: "var(--color-text-secondary)",
                  cursor: "pointer",
                }}
              >
                Cancel
              </button>
            </div>
          </div>
        )}

        {/* Spawn button */}
        <button
          onClick={() => handleSpawnSession()}
          disabled={!isOnline || isSpawning || !spawnPath.trim()}
          style={{
            width: "100%",
            padding: tokens.spacing[3],
            fontSize: tokens.typography.fontSize.base,
            borderRadius: tokens.radii.md,
            border: "none",
            backgroundColor: isOnline && spawnPath.trim() && !isSpawning ? "var(--color-primary)" : "var(--surface-disabled)",
            color: isOnline && spawnPath.trim() && !isSpawning ? "white" : "var(--color-text-tertiary)",
            cursor: isOnline && spawnPath.trim() && !isSpawning ? "pointer" : "default",
            fontWeight: 500,
          }}
        >
          {isSpawning ? t("machine.actions.spawning") : t("machine.actions.spawnSession")}
        </button>

        {!isOnline && (
          <Body
            color="secondary"
            style={{
              marginTop: tokens.spacing[2],
              fontSize: tokens.typography.fontSize.xs,
              textAlign: "center",
            }}
          >
            {t("machine.offlineUnableToSpawn")}
          </Body>
        )}
      </div>

      {/* Stop Daemon Section */}
      <div
        style={{
          padding: tokens.spacing[6],
          borderTop: "1px solid var(--border-primary)",
        }}
      >
        <Title3 style={{ marginBottom: tokens.spacing[2] }}>
          {t("machine.stopDaemon.title")}
        </Title3>
        <Body color="secondary" style={{ marginBottom: tokens.spacing[4], fontSize: tokens.typography.fontSize.sm }}>
          {t("machine.stopDaemon.description")}
        </Body>

        {/* Error message */}
        {stopError && (
          <div
            style={{
              padding: tokens.spacing[3],
              marginBottom: tokens.spacing[3],
              backgroundColor: "var(--surface-error)",
              borderRadius: tokens.radii.sm,
            }}
          >
            <Body style={{ color: "var(--color-danger)", fontSize: tokens.typography.fontSize.sm }}>
              {t("machine.stopDaemon.errorPrefix")}{stopError}
            </Body>
          </div>
        )}

        {/* Stop button */}
        <button
          onClick={handleStopDaemon}
          disabled={!isOnline || isStopping || daemonStatus === "stopped"}
          style={{
            width: "100%",
            padding: tokens.spacing[3],
            fontSize: tokens.typography.fontSize.base,
            borderRadius: tokens.radii.md,
            border: daemonStatus === "stopped" ? "1px solid var(--border-primary)" : "none",
            backgroundColor:
              daemonStatus === "stopped"
                ? "transparent"
                : isOnline && !isStopping
                  ? "var(--color-danger)"
                  : "var(--surface-disabled)",
            color:
              daemonStatus === "stopped"
                ? "var(--color-text-tertiary)"
                : isOnline && !isStopping
                  ? "white"
                  : "var(--color-text-tertiary)",
            cursor: isOnline && !isStopping && daemonStatus !== "stopped" ? "pointer" : "default",
            fontWeight: 500,
          }}
        >
          {isStopping
            ? t("machine.actions.stopping")
            : daemonStatus === "stopped"
              ? "Daemon stopped"
              : t("machine.actions.stopDaemon")}
        </button>
      </div>

      {/* Recent paths section */}
      {recentPaths.length > 0 && (
        <div
          style={{
            padding: tokens.spacing[6],
            borderTop: "1px solid var(--border-primary)",
          }}
        >
          <Title3 style={{ marginBottom: tokens.spacing[4] }}>
            {t("machine.recentPaths")}
          </Title3>
          <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[2] }}>
            {recentPaths.map((path) => (
              <Body
                key={path}
                style={{
                  fontFamily: tokens.typography.fontFamily.mono,
                  fontSize: tokens.typography.fontSize.sm,
                  color: "var(--color-text-secondary)",
                }}
              >
                {path}
              </Body>
            ))}
          </div>
        </div>
      )}

      {/* Recent sessions section */}
      {machineSessions.length > 0 && (
        <div
          style={{
            padding: tokens.spacing[6],
            borderTop: "1px solid var(--border-primary)",
          }}
        >
          <Title3 style={{ marginBottom: tokens.spacing[4] }}>
            {t("machine.recentSessions")}
          </Title3>
          <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[3] }}>
            {machineSessions.slice(0, 5).map((session) => (
              <div
                key={session.id}
                style={{
                  padding: tokens.spacing[3],
                  backgroundColor: "var(--surface-secondary)",
                  borderRadius: tokens.radii.md,
                  cursor: onNavigateToSession ? "pointer" : "default",
                }}
                onClick={() => onNavigateToSession?.(session.id)}
              >
                <Body bold style={{ marginBottom: tokens.spacing[1] }}>
                  {session.metadata?.name || session.id.slice(0, 8)}
                </Body>
                <Body color="tertiary" style={{ fontSize: tokens.typography.fontSize.xs }}>
                  {new Date(session.updatedAt * 1000).toLocaleString()}
                </Body>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

/**
 * MachineStatusIndicator - Visual indicator for machine online status
 */
function MachineStatusIndicator({ isOnline }: { isOnline: boolean }) {
  return (
    <div
      style={{
        width: 12,
        height: 12,
        borderRadius: "50%",
        backgroundColor: isOnline ? "var(--color-success)" : "var(--color-text-tertiary)",
        boxShadow: isOnline
          ? "0 0 8px var(--color-success)"
          : "none",
      }}
    />
  );
}
