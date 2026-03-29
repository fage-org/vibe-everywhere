# Vibe Everywhere

[![CI](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml)
[![Release](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml/badge.svg)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

[中文](./README.md) | [English](./README.en.md)

Vibe Everywhere 是一个面向自建场景的远程 AI 控制面。你在自己的服务器、工作站或开发机上运行
`vibe-relay` 和 `vibe-agent`，再通过 Web、桌面端或 Android 客户端发起 AI Session、查看工作区、
检查 Git 状态、打开预览，并在需要时进入终端或高级连接工具。

它不是传统远程桌面。产品重点是远程 AI 开发流程的组织、状态可见性和多设备协作入口。
当前控制端默认围绕一条主流程组织：连接 relay、选择设备、发起 AI Session、审阅工作区与 Git 结果，
而设备清单和高级工具保留在次级视图中。

## 适合谁

- 想把 AI 编码任务放到远程机器执行，但仍希望集中查看状态和结果的人
- 希望自建控制面，而不是依赖托管服务的人
- 需要同时管理多台设备、多个工作区和不同 AI Provider CLI 的个人或小团队
- 想先落地一个可用 MVP，再逐步演进到更完整团队能力的团队

## 当前能力

- AI Session 创建、执行、取消、事件流展示
- 以 Session 为中心的主工作流：连接 relay、选择设备、发起会话、审阅结果
- 设备注册、在线状态和 Provider 可用性展示
- 工作区浏览、文本文件预览、Git 检视
- 预览转发能力，以及需要时的终端与高级连接能力
- Web、Tauri 桌面端、Android 控制端
- 中文 / 英文界面，浅色 / 深色 / 跟随系统主题

## 快速开始

### 1. 部署 Relay

先看自建部署说明：

- [自建部署指南](./docs/self-hosted.md)

仓库当前提供两套安装脚本：

- Linux `systemd`

```bash
curl -fsSL https://raw.githubusercontent.com/fage-ac-org/vibe-everywhere/main/scripts/install-relay.sh -o install-relay.sh
sudo RELAY_PUBLIC_BASE_URL=https://relay.example.com \
  RELAY_ACCESS_TOKEN=change-control-token \
  RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token \
  bash install-relay.sh
```

- Windows 自动启动任务

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install-relay.ps1 `
  -PublicRelayBaseUrl https://relay.example.com `
  -RelayAccessToken change-control-token `
  -RelayEnrollmentToken change-agent-enrollment-token
```

### 2. 在目标机器启动 Agent

从 GitHub Release 下载 CLI 包，解压后启动 `vibe-agent`：

```bash
VIBE_RELAY_URL=https://relay.example.com \
VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token \
VIBE_DEVICE_NAME=build-node-01 \
./vibe-agent
```

Windows 下请保留解压后的 side-by-side 运行时文件与 `vibe-agent.exe` 同目录，不要只单独拷出
`vibe-agent.exe`。

推荐做法是把人类控制端和设备注册分成两条凭证：

- Web、桌面端、Android 控制端继续使用 `VIBE_RELAY_ACCESS_TOKEN`
- Agent 使用 `VIBE_RELAY_ENROLLMENT_TOKEN` 完成首次注册
- Agent 首次注册成功后会在工作目录下写入 `.vibe-agent/identity.json`，后续重启优先复用该设备凭证

如果你希望执行 AI Session，目标机器还需要安装至少一个 Provider CLI：

- `codex`
- `claude`
- `opencode`

### 3. 打开控制端

当前可以通过以下形态连接同一套 relay：

- Web
- Tauri 桌面端
- Android

推荐顺序：

1. 先部署 relay。
2. 在目标机器启动 agent。
3. 先用 Web 或桌面端输入控制面 token，进入 Session 主流程，确认链路、设备和 Provider 都正常。
4. 需要移动访问时再安装 Android 客户端。

## 下载

- [GitHub Releases](https://github.com/fage-ac-org/vibe-everywhere/releases)

发布页面提供当前版本的 CLI、桌面端和 Android 产物。选择与你的平台匹配的文件即可。

## 产品结构

```text
┌──────────────────────────────────────────────────────────┐
│                     Control App                          │
│      Vue 3.5 Web UI / Tauri Desktop + Android Shell    │
└───────────────────────────┬──────────────────────────────┘
                            │ HTTP / SSE / WebSocket
┌───────────────────────────▼──────────────────────────────┐
│                      vibe-relay                          │
│  device registry · AI sessions · workspace · preview    │
│        auth · config · transport selection               │
└───────────────────────────┬──────────────────────────────┘
                            │ polling / stream / tunnel
┌───────────────────────────▼──────────────────────────────┐
│                      vibe-agent                          │
│ provider adapters · workspace/git runtime · shell       │
│      preview / forward runtime · overlay support         │
└───────────────────────────┬──────────────────────────────┘
                            │ local process / local TCP
                    ┌───────▼────────┐
                    │ target machine │
                    └────────────────┘
```

## 运维文档

- 自建部署与安装：[docs/self-hosted.md](./docs/self-hosted.md)
