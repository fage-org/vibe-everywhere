<script setup lang="ts">
import { computed, onMounted } from "vue";
import { storeToRefs } from "pinia";
import {
  getRelayBaseUrlPlaceholder,
  isLoopbackRelayBaseUrl,
  isMobileControlClient
} from "../lib/runtime";
import { useControlStore } from "../stores/control";

const store = useControlStore();
const {
  appConfig,
  devices,
  draft,
  errorMessage,
  eventState,
  health,
  portForwardDraft,
  portForwardScope,
  portForwardStatusFilter,
  portForwards,
  relayAccessTokenInput,
  relayInput,
  selectedDevice,
  selectedPortForward,
  selectedShellSession,
  selectedShellSessionDetail,
  selectedTask,
  selectedTaskDetail,
  shellDraft,
  shellScope,
  shellSessions,
  shellSocketState,
  shellStatusFilter,
  taskScope,
  taskStatusFilter,
  tasks,
  visiblePortForwards,
  visibleShellSessions,
  visibleTasks
} = storeToRefs(store);

const canSubmit = computed(
  () =>
    Boolean(selectedDevice.value) &&
    Boolean(draft.value.provider) &&
    Boolean(draft.value.prompt.trim())
);
const canCancel = computed(() =>
  Boolean(
    selectedTask.value &&
      !["succeeded", "failed", "canceled"].includes(selectedTask.value.status)
  )
);
const selectedDeviceSupportsShell = computed(
  () => selectedDevice.value?.capabilities.includes("shell") ?? false
);
const canOpenShell = computed(
  () => Boolean(selectedDevice.value) && selectedDeviceSupportsShell.value
);
const canSendShellInput = computed(() => {
  if (!selectedShellSession.value) {
    return false;
  }

  return (
    Boolean(shellDraft.value.input.trim()) &&
    !isShellTerminal(selectedShellSession.value.status) &&
    !selectedShellSession.value.closeRequested
  );
});
const canCloseShell = computed(() => {
  if (!selectedShellSession.value) {
    return false;
  }

  return (
    !isShellTerminal(selectedShellSession.value.status) &&
    !selectedShellSession.value.closeRequested
  );
});
const canCreatePortForward = computed(() => {
  const targetPort = Number.parseInt(portForwardDraft.value.targetPort.trim(), 10);
  return (
    Boolean(selectedDevice.value) &&
    Boolean(portForwardDraft.value.targetHost.trim()) &&
    Number.isInteger(targetPort) &&
    targetPort > 0 &&
    targetPort <= 65535
  );
});
const canClosePortForward = computed(() =>
  Boolean(
    selectedPortForward.value &&
      !["closed", "failed", "close_requested"].includes(selectedPortForward.value.status)
  )
);
const shellTimeline = computed(() => {
  const detail = selectedShellSessionDetail.value;
  if (!detail) {
    return [];
  }

  return [
    ...detail.inputs.map((input) => ({
      key: `input-${input.seq}`,
      stream: "stdin",
      label: "stdin",
      data: input.data,
      timestampEpochMs: input.timestampEpochMs,
      order: input.seq
    })),
    ...detail.outputs.map((output) => ({
      key: `output-${output.seq}`,
      stream: output.stream,
      label: output.stream,
      data: output.data,
      timestampEpochMs: output.timestampEpochMs,
      order: output.seq + 1000000
    }))
  ].sort((left, right) => {
    if (left.timestampEpochMs === right.timestampEpochMs) {
      return left.order - right.order;
    }
    return left.timestampEpochMs - right.timestampEpochMs;
  });
});
const relayPlaceholder = getRelayBaseUrlPlaceholder();
const showMobileRelayHint = computed(() => isMobileControlClient());
const showLoopbackRelayWarning = computed(
  () =>
    showMobileRelayHint.value &&
    Boolean(relayInput.value.trim()) &&
    isLoopbackRelayBaseUrl(relayInput.value)
);

function formatProviderKind(value: string) {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function formatExecutionProtocol(value: string) {
  return value.toUpperCase();
}

function formatProviderSummary(kind: string, executionProtocol: string) {
  return `${formatProviderKind(kind)} · ${formatExecutionProtocol(executionProtocol)}`;
}

function formatTimestamp(value: number | null | undefined) {
  if (!value) {
    return "pending";
  }

  return new Date(value).toLocaleString();
}

function formatShellStatus(value: string, closeRequested: boolean) {
  if (closeRequested && value === "active") {
    return "close requested";
  }

  return value.replaceAll("_", " ");
}

function formatPortForwardStatus(value: string) {
  return value.replaceAll("_", " ");
}

function formatPortForwardTransport(value: string) {
  return value.replaceAll("_", " ");
}

function formatPortForwardProtocol(value: string) {
  return value.toUpperCase();
}

function formatEndpoint(host: string, port: number) {
  return `${host}:${port}`;
}

function isShellTerminal(value: string) {
  return ["succeeded", "failed", "closed"].includes(value);
}

onMounted(() => {
  void store.initialize();
});
</script>

<template>
  <main class="shell">
    <section class="hero">
      <div class="hero-copy">
        <p class="eyebrow">Vue 3.5 + Tauri 2 + Rust Relay</p>
        <h1>{{ appConfig?.appName ?? "Vibe Everywhere" }}</h1>
        <p class="lede">
          单用户多设备 AI 控制台。移动端作为控制端，桌面设备作为被控 Agent，任务通过 relay
          分发并实时回传日志。
        </p>
      </div>

      <div class="status-card">
        <span class="status-pill">SSE {{ eventState }}</span>
        <p class="status-label">Relay Base URL</p>
        <div class="relay-row">
          <input v-model="relayInput" class="relay-input" :placeholder="relayPlaceholder" />
          <button class="primary-button" @click="store.applyRelayBaseUrl">Connect</button>
        </div>
        <div class="relay-row auth-row">
          <input
            v-model="relayAccessTokenInput"
            class="relay-input"
            type="password"
            placeholder="optional access token"
          />
          <span class="subtle auth-hint">
            {{ appConfig?.requiresAuth ? "auth required" : "auth optional" }}
          </span>
        </div>
        <p
          v-if="showMobileRelayHint"
          class="subtle relay-tip"
          :class="{ warning: showLoopbackRelayWarning }"
        >
          {{
            showLoopbackRelayWarning
              ? "移动端不能连接 localhost / 127.0.0.1，请改成 relay 所在机器的局域网 IP 或 HTTPS 公网地址。"
              : "移动端请填写 relay 所在机器的局域网 IP 或 HTTPS 公网地址。"
          }}
        </p>
        <p class="status-footnote">
          在线设备 {{ health?.onlineDeviceCount ?? 0 }} / {{ health?.deviceCount ?? 0 }}，
          任务 {{ health?.taskCount ?? 0 }}，终端会话 {{ shellSessions.length }}，端口转发
          {{ portForwards.length }}
        </p>
      </div>
    </section>

    <p v-if="errorMessage" class="error-banner">{{ errorMessage }}</p>

    <section class="dashboard-grid">
      <article class="panel device-panel">
        <div class="panel-header">
          <div>
            <p class="panel-title">Devices</p>
            <h2>{{ devices.length }}</h2>
          </div>
          <button class="ghost-button" @click="store.reloadAll">Refresh</button>
        </div>

        <ul class="device-list">
          <li v-for="device in devices" :key="device.id">
            <button
              class="device-card"
              :class="{ selected: selectedDevice?.id === device.id }"
              @click="store.selectDevice(device.id)"
            >
              <div class="device-row">
                <strong>{{ device.name }}</strong>
                <span class="badge" :class="device.online ? 'online' : 'offline'">
                  {{ device.online ? "online" : "offline" }}
                </span>
              </div>
              <p>{{ device.platform }} · {{ device.metadata.arch }}</p>
              <p class="subtle">
                Overlay: {{ device.overlay.state }}
                <span v-if="device.overlay.networkName"> · {{ device.overlay.networkName }}</span>
              </p>
              <div class="provider-tags">
                <span
                  v-for="provider in device.providers"
                  :key="provider.kind"
                  class="provider-tag"
                  :class="{ unavailable: !provider.available }"
                >
                  {{ formatProviderSummary(provider.kind, provider.executionProtocol) }}
                </span>
              </div>
            </button>
          </li>
        </ul>
      </article>

      <article class="panel composer-panel">
        <div class="panel-header">
          <div>
            <p class="panel-title">Task Composer</p>
            <h2>{{ selectedDevice?.name ?? "No device selected" }}</h2>
          </div>
        </div>

        <div class="form-grid">
          <label class="field">
            <span>Provider</span>
            <select v-model="draft.provider">
              <option disabled value="">Select provider</option>
              <option
                v-for="provider in store.availableProviders"
                :key="provider.kind"
                :value="provider.kind"
              >
                {{ provider.label }}
              </option>
            </select>
          </label>

          <label class="field">
            <span>Title</span>
            <input v-model="draft.title" placeholder="Ad hoc AI task" />
          </label>

          <label class="field">
            <span>Working Dir</span>
            <input v-model="draft.cwd" placeholder="repo/path or /absolute/path" />
          </label>

          <label class="field">
            <span>Model</span>
            <input v-model="draft.model" placeholder="optional model override" />
          </label>

          <label class="field full">
            <span>Prompt</span>
            <textarea
              v-model="draft.prompt"
              rows="12"
              placeholder="Tell Codex / Claude Code / OpenCode what to do on the selected device."
            />
          </label>
        </div>

        <div class="action-row">
          <button class="primary-button" :disabled="!canSubmit" @click="store.submitTask">
            Start Task
          </button>
          <span class="subtle">
            Codex / OpenCode 已优先走 ACP；Claude Code 当前仍走 CLI。终端能力走 relay 中转轮询。
          </span>
        </div>
      </article>

      <article class="panel task-panel">
        <div class="panel-header">
          <div>
            <p class="panel-title">Tasks</p>
            <h2>{{ visibleTasks.length }} / {{ tasks.length }}</h2>
          </div>
          <button class="ghost-button" :disabled="!canCancel" @click="store.cancelSelectedTask">
            Cancel
          </button>
        </div>

        <div class="task-filter-row">
          <label class="field inline-field">
            <span>Scope</span>
            <select v-model="taskScope">
              <option value="all">All devices</option>
              <option value="selected_device">Selected device</option>
            </select>
          </label>

          <label class="field inline-field">
            <span>Status</span>
            <select v-model="taskStatusFilter">
              <option value="all">All status</option>
              <option value="pending">Pending</option>
              <option value="assigned">Assigned</option>
              <option value="running">Running</option>
              <option value="cancel_requested">Cancel requested</option>
              <option value="succeeded">Succeeded</option>
              <option value="failed">Failed</option>
              <option value="canceled">Canceled</option>
            </select>
          </label>
        </div>

        <p v-if="!visibleTasks.length" class="empty-state subtle">No tasks match current filters.</p>

        <ul class="task-list">
          <li v-for="task in visibleTasks" :key="task.id">
            <button
              class="task-card"
              :class="{ selected: selectedTask?.id === task.id }"
              @click="store.selectTask(task.id)"
            >
              <div class="device-row">
                <strong>{{ task.title }}</strong>
                <span class="badge">{{ task.status }}</span>
              </div>
              <p>
                {{ formatProviderSummary(task.provider, task.executionProtocol) }} · {{ task.deviceId }}
              </p>
              <p class="subtle">{{ task.model ?? "default model" }}</p>
            </button>
          </li>
        </ul>

        <div v-if="selectedTaskDetail" class="log-panel">
          <div class="log-header">
            <div>
              <strong>{{ selectedTaskDetail.task.title }}</strong>
              <p class="subtle">
                {{ formatProviderSummary(selectedTaskDetail.task.provider, selectedTaskDetail.task.executionProtocol) }} · {{ selectedTaskDetail.task.status }}
              </p>
            </div>
          </div>

          <div class="log-stream">
            <div
              v-for="event in selectedTaskDetail.events"
              :key="`${event.taskId}-${event.seq}`"
              class="log-line"
            >
              <span class="log-kind">{{ event.kind }}</span>
              <pre>{{ event.message }}</pre>
            </div>
          </div>
        </div>
      </article>
    </section>

    <section class="terminal-section">
      <article class="panel shell-panel">
        <div class="panel-header">
          <div>
            <p class="panel-title">Relay Shell</p>
            <h2>{{ visibleShellSessions.length }} / {{ shellSessions.length }}</h2>
            <p class="subtle">Session WS {{ shellSocketState }}</p>
          </div>
          <button class="ghost-button" @click="store.refreshShellSessionsFromPoll">
            Refresh Shells
          </button>
        </div>

        <div class="terminal-toolbar">
          <label class="field inline-field">
            <span>Scope</span>
            <select v-model="shellScope">
              <option value="all">All devices</option>
              <option value="selected_device">Selected device</option>
            </select>
          </label>

          <label class="field inline-field">
            <span>Status</span>
            <select v-model="shellStatusFilter">
              <option value="all">All status</option>
              <option value="pending">Pending</option>
              <option value="active">Active</option>
              <option value="close_requested">Close requested</option>
              <option value="succeeded">Succeeded</option>
              <option value="failed">Failed</option>
              <option value="closed">Closed</option>
            </select>
          </label>

          <label class="field terminal-cwd-field">
            <span>Shell CWD</span>
            <input v-model="shellDraft.cwd" placeholder="optional working dir" />
          </label>

          <button class="primary-button" :disabled="!canOpenShell" @click="store.createShellSession">
            Open Shell
          </button>

          <button class="ghost-button" :disabled="!canCloseShell" @click="store.closeSelectedShellSession">
            Close Session
          </button>
        </div>

        <p v-if="selectedDevice && !selectedDeviceSupportsShell" class="subtle terminal-note">
          当前选中的设备没有声明 shell capability，无法创建 relay shell session。
        </p>

        <div class="shell-grid">
          <aside class="terminal-sidebar">
            <p v-if="!visibleShellSessions.length" class="empty-state subtle">
              当前筛选条件下还没有终端会话。先选择设备并打开一个 shell。
            </p>

            <ul class="session-list">
              <li v-for="session in visibleShellSessions" :key="session.id">
                <button
                  class="task-card session-card"
                  :class="{ selected: selectedShellSession?.id === session.id }"
                  @click="store.selectShellSession(session.id)"
                >
                  <div class="device-row">
                    <strong>{{ session.deviceId }}</strong>
                    <span class="badge">
                      {{ formatShellStatus(session.status, session.closeRequested) }}
                    </span>
                  </div>
                  <p class="monospace">{{ session.cwd ?? "default shell cwd" }}</p>
                  <p class="subtle">
                    {{ formatTimestamp(session.startedAtEpochMs ?? session.createdAtEpochMs) }}
                  </p>
                </button>
              </li>
            </ul>
          </aside>

          <div class="terminal-main">
            <div v-if="selectedShellSession" class="terminal-meta">
              <div>
                <strong>Session {{ selectedShellSession.id }}</strong>
                <p class="subtle">
                  {{ formatShellStatus(selectedShellSession.status, selectedShellSession.closeRequested) }}
                  · started {{ formatTimestamp(selectedShellSession.startedAtEpochMs ?? selectedShellSession.createdAtEpochMs) }}
                </p>
              </div>
              <p class="subtle monospace">
                {{ selectedShellSession.cwd ?? selectedDevice?.metadata.cwd ?? "default shell cwd" }}
              </p>
            </div>

            <div v-if="selectedShellSessionDetail" class="terminal-stream">
              <p v-if="!shellTimeline.length" class="empty-state subtle terminal-empty">
                会话已创建，等待 agent 启动 shell 并回传输出。
              </p>

              <div
                v-for="entry in shellTimeline"
                :key="entry.key"
                class="terminal-line"
                :class="`stream-${entry.stream}`"
              >
                <div class="terminal-line-meta">
                  <span class="terminal-stream-tag">{{ entry.label }}</span>
                  <span class="subtle">{{ formatTimestamp(entry.timestampEpochMs) }}</span>
                </div>
                <pre>{{ entry.data }}</pre>
              </div>
            </div>
            <p v-else class="empty-state subtle terminal-empty">
              选择一个终端会话，或先为当前设备创建新的 relay shell。
            </p>

            <div class="terminal-input-panel">
              <textarea
                v-model="shellDraft.input"
                rows="4"
                placeholder="pwd"
                :disabled="!selectedShellSession || selectedShellSession.closeRequested"
              />
              <div class="action-row">
                <span class="subtle">
                  当前为服务端中转 shell MVP，输入输出通过 relay API 轮询同步。
                </span>
                <button class="primary-button" :disabled="!canSendShellInput" @click="store.submitShellInput">
                  Send Command
                </button>
              </div>
            </div>
          </div>
        </div>
      </article>
    </section>

    <section class="forward-section">
      <article class="panel forward-panel">
        <div class="panel-header">
          <div>
            <p class="panel-title">Port Forwards</p>
            <h2>{{ visiblePortForwards.length }} / {{ portForwards.length }}</h2>
            <p class="subtle">轮询同步 relay tunnel 与 overlay proxy 状态</p>
          </div>
          <button class="ghost-button" @click="store.refreshPortForwardsFromPoll">
            Refresh Forwards
          </button>
        </div>

        <div class="forward-toolbar">
          <label class="field inline-field">
            <span>Scope</span>
            <select v-model="portForwardScope">
              <option value="all">All devices</option>
              <option value="selected_device">Selected device</option>
            </select>
          </label>

          <label class="field inline-field">
            <span>Status</span>
            <select v-model="portForwardStatusFilter">
              <option value="all">All status</option>
              <option value="pending">Pending</option>
              <option value="active">Active</option>
              <option value="close_requested">Close requested</option>
              <option value="closed">Closed</option>
              <option value="failed">Failed</option>
            </select>
          </label>

          <label class="field">
            <span>Target Host</span>
            <input v-model="portForwardDraft.targetHost" placeholder="127.0.0.1" />
          </label>

          <label class="field">
            <span>Target Port</span>
            <input v-model="portForwardDraft.targetPort" inputmode="numeric" placeholder="3000" />
          </label>

          <button
            class="primary-button"
            :disabled="!canCreatePortForward"
            @click="store.createPortForward"
          >
            Create Forward
          </button>

          <button
            class="ghost-button"
            :disabled="!canClosePortForward"
            @click="store.closeSelectedPortForward"
          >
            Close Forward
          </button>
        </div>

        <div class="forward-grid">
          <aside class="forward-sidebar">
            <p v-if="!visiblePortForwards.length" class="empty-state subtle">
              当前还没有匹配的端口转发。选择设备并填写 target host / port 后即可创建。
            </p>

            <ul class="session-list">
              <li v-for="forward in visiblePortForwards" :key="forward.id">
                <button
                  class="task-card forward-card"
                  :class="{ selected: selectedPortForward?.id === forward.id }"
                  @click="store.selectPortForward(forward.id)"
                >
                  <div class="device-row">
                    <strong>{{ forward.deviceId }}</strong>
                    <span class="badge">{{ formatPortForwardStatus(forward.status) }}</span>
                  </div>
                  <p class="monospace">{{ formatEndpoint(forward.relayHost, forward.relayPort) }}</p>
                  <p class="monospace subtle">{{ formatEndpoint(forward.targetHost, forward.targetPort) }}</p>
                  <p class="subtle">
                    {{ formatPortForwardTransport(forward.transport) }} ·
                    {{ formatTimestamp(forward.startedAtEpochMs ?? forward.createdAtEpochMs) }}
                  </p>
                </button>
              </li>
            </ul>
          </aside>

          <div class="forward-main">
            <div v-if="selectedPortForward" class="forward-summary">
              <div class="terminal-meta">
                <div>
                  <strong>Forward {{ selectedPortForward.id }}</strong>
                  <p class="subtle">
                    {{ formatPortForwardStatus(selectedPortForward.status) }} ·
                    {{ formatPortForwardTransport(selectedPortForward.transport) }}
                  </p>
                </div>
                <p class="subtle monospace">{{ selectedPortForward.deviceId }}</p>
              </div>

              <div class="forward-route-grid">
                <div class="forward-route-card">
                  <p class="forward-route-label">Protocol</p>
                  <strong>{{ formatPortForwardProtocol(selectedPortForward.protocol) }}</strong>
                </div>
                <div class="forward-route-card">
                  <p class="forward-route-label">Relay Endpoint</p>
                  <strong class="monospace forward-route-value">
                    {{ formatEndpoint(selectedPortForward.relayHost, selectedPortForward.relayPort) }}
                  </strong>
                </div>
                <div class="forward-route-card">
                  <p class="forward-route-label">Target Endpoint</p>
                  <strong class="monospace forward-route-value">
                    {{ formatEndpoint(selectedPortForward.targetHost, selectedPortForward.targetPort) }}
                  </strong>
                </div>
              </div>

              <div class="forward-detail-grid">
                <div class="forward-detail-item">
                  <span>Created</span>
                  <strong>{{ formatTimestamp(selectedPortForward.createdAtEpochMs) }}</strong>
                </div>
                <div class="forward-detail-item">
                  <span>Started</span>
                  <strong>{{ formatTimestamp(selectedPortForward.startedAtEpochMs) }}</strong>
                </div>
                <div class="forward-detail-item">
                  <span>Finished</span>
                  <strong>{{ formatTimestamp(selectedPortForward.finishedAtEpochMs) }}</strong>
                </div>
              </div>

              <p v-if="selectedPortForward.error" class="error-inline">
                {{ selectedPortForward.error }}
              </p>

              <div class="action-row">
                <span class="subtle">
                  Relay tunnel 会在 relay 侧暴露监听地址；overlay proxy 会优先走 overlay，再由 relay 代理目标连接。
                </span>
              </div>
            </div>
            <p v-else class="empty-state subtle terminal-empty">
              选择一个端口转发查看 relay / target 路径，或先为当前设备创建新的 tunnel。
            </p>
          </div>
        </div>
      </article>
    </section>
  </main>
</template>
