# Vibe Everywhere

[![CI](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml)
[![Release](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml/badge.svg)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

[中文](./README.md) | [English](./README.en.md)

Vibe Everywhere 是面向自建环境的远程 AI 控制面。系统由 `vibe-relay`、`vibe-agent` 和控制端组成，用于在远程机器上执行 AI Session，并通过统一入口完成设备注册、会话调度、工作区浏览、Git 检查、预览访问和必要的高级连接操作。

本文件面向最终用户和部署人员，提供系统概览、二进制安装方式、启动入口、关键配置语义和标准使用流程。

## 概述

标准工作流程如下：

1. 在客户端和 Agent 可访问的主机上部署 `vibe-relay`。
2. 在目标执行节点上启动 `vibe-agent`。
3. 通过桌面端、Android 或自建 Web 客户端连接 relay。
4. 选择设备并创建 AI Session。
5. 在同一控制面中检查工作区、Git 状态、预览和执行结果。

## 组件说明

| 组件 | 作用 | 典型部署位置 |
| --- | --- | --- |
| `vibe-relay` | 控制面入口；负责认证、设备注册、Session 路由、状态汇总和对外 API | 服务器、工作站、云主机 |
| `vibe-agent` | 运行在目标机器上；负责执行 Provider CLI、工作区访问、Git 检查、预览桥接和高级连接 | 需要执行 AI 任务的目标机器 |
| 控制端 | 连接 relay，发起和管理 AI Session，查看设备与结果 | 桌面端、Android、自建 Web 客户端 |

## 功能范围

当前版本支持：

- AI Session 的创建、执行、取消和事件流查看
- 设备注册、在线状态上报和 Provider 可用性展示
- 工作区目录浏览和文本文件预览
- Git 状态、变更文件和最近提交检查
- 预览访问、Shell 和高级连接能力
- 中文和英文界面
- 浅色、深色和系统主题

## 快速开始

### 前置条件

部署前请先确认以下信息：

- relay 的客户端访问地址，例如 `https://relay.example.com` 或 `http://203.0.113.10:8787`
- 人类控制端使用的控制面 token，即 `VIBE_RELAY_ACCESS_TOKEN`
- Agent 首次注册使用的 enrollment token，即 `VIBE_RELAY_ENROLLMENT_TOKEN`
- 目标机器上的至少一个 AI Provider CLI，例如 `codex`、`claude` 或 `opencode`

### 1. 下载或更新 CLI 二进制

`scripts/install-relay.sh` 和 `scripts/install-relay.ps1` 用于安装、更新或卸载 CLI 二进制。默认安装 `vibe-relay` 和 `vibe-agent`，也可以通过 `--component` 或 `-Component` 仅处理其中一个组件。

#### Linux

直连 GitHub：

```bash
curl -fsSL https://raw.githubusercontent.com/fage-ac-org/vibe-everywhere/main/scripts/install-relay.sh -o install-relay.sh
bash install-relay.sh install --no-gh-proxy
```

中国网络环境推荐：

```bash
curl -fsSL https://ghfast.top/https://raw.githubusercontent.com/fage-ac-org/vibe-everywhere/main/scripts/install-relay.sh -o install-relay.sh
bash install-relay.sh install
```

常用命令：

```bash
bash install-relay.sh install
bash install-relay.sh install --component relay
bash install-relay.sh install --component agent
bash install-relay.sh update --release-tag v0.1.8
bash install-relay.sh uninstall
bash install-relay.sh uninstall --component agent
```

默认安装路径：

- `/usr/local/bin/vibe-relay`
- `/usr/local/bin/vibe-agent`

#### Windows

直连 GitHub：

```powershell
Invoke-WebRequest `
  -Uri "https://raw.githubusercontent.com/fage-ac-org/vibe-everywhere/main/scripts/install-relay.ps1" `
  -OutFile ".\install-relay.ps1"
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command install -NoGhProxy
```

中国网络环境推荐：

```powershell
Invoke-WebRequest `
  -Uri "https://ghfast.top/https://raw.githubusercontent.com/fage-ac-org/vibe-everywhere/main/scripts/install-relay.ps1" `
  -OutFile ".\install-relay.ps1"
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command install
```

常用命令：

```powershell
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command install
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command install -Component relay
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command install -Component agent
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command update -ReleaseTag v0.1.8
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command uninstall
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command uninstall -Component agent
```

默认安装路径：

- `C:\Program Files\Vibe Everywhere\vibe-relay.exe`
- `C:\Program Files\Vibe Everywhere\vibe-agent.exe`
- `C:\Program Files\Vibe Everywhere\Packet.dll`
- `C:\Program Files\Vibe Everywhere\wintun.dll`
- `C:\Program Files\Vibe Everywhere\WinDivert64.sys`
- `C:\Program Files\Vibe Everywhere\WinDivert.dll`（如果归档中包含）

说明：

- Windows 下 `vibe-relay.exe` 和 `vibe-agent.exe` 需要与 side-by-side 运行时文件放在同一目录。

加速说明：

- 两个安装脚本默认使用 `https://ghfast.top/` 作为 GitHub URL 前缀，加速脚本内部的 release 解析和归档下载。
- 如果当前网络环境可以直接访问 GitHub，Linux 请使用 `--no-gh-proxy`，Windows 请使用 `-NoGhProxy`。
- 如果需要替换为其他代理前缀，Linux 使用 `--gh-proxy <url>`，Windows 使用 `-GhProxy <url>`。

### 2. 配置并启动 Relay

独立启动说明见：

- 中文：[docs/relay-startup.zh-CN.md](./docs/relay-startup.zh-CN.md)
- English: [docs/relay-startup.md](./docs/relay-startup.md)

最小前台启动示例：

```bash
export VIBE_RELAY_HOST=0.0.0.0
export VIBE_RELAY_PORT=8787
export VIBE_PUBLIC_RELAY_BASE_URL=https://relay.example.com
export VIBE_RELAY_ACCESS_TOKEN=change-control-token
export VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token
vibe-relay
```

健康检查：

```bash
curl https://relay.example.com/api/health
```

### 3. 启动 Agent

如果已通过安装脚本安装 CLI，可直接使用安装后的 `vibe-agent`。如果未使用安装脚本，也可以从 Release 页面下载 CLI 包并解压后运行。

```bash
VIBE_RELAY_URL=https://relay.example.com \
VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token \
VIBE_DEVICE_NAME=build-node-01 \
vibe-agent
```

操作说明：

- Linux 默认路径是 `/usr/local/bin/vibe-agent`
- Windows 默认路径是 `C:\Program Files\Vibe Everywhere\vibe-agent.exe`
- Windows 下必须保留归档中的 side-by-side 运行时文件，不要只复制 `vibe-agent.exe`
- `VIBE_RELAY_URL` 必须指向 Agent 实际可访问的 relay 地址
- Agent 首次注册成功后，会在工作目录下写入 `.vibe-agent/identity.json`
- 后续重启优先复用该设备凭证，而不是重复使用控制面 token

### 4. 连接控制端

首次连接建议按以下顺序执行：

1. 打开桌面端或 Android 客户端。
2. 输入 relay 地址。
3. 输入 `VIBE_RELAY_ACCESS_TOKEN`。
4. 确认至少一台设备在线，且该设备至少一个 Provider 处于可用状态。
5. 创建 AI Session。

## 配置语义

### Relay 监听地址与对外地址

`bind` 配置和 `public origin` 配置不是同一个概念。

| 配置 | 作用 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `VIBE_RELAY_HOST` | relay 本地监听地址 | `0.0.0.0` | 控制服务绑定到哪个网卡地址 |
| `VIBE_RELAY_PORT` | relay 本地监听端口 | `8787` | 控制服务监听的 TCP 端口 |
| `VIBE_PUBLIC_RELAY_BASE_URL` | 客户端访问 relay 时使用的对外地址 | 无公网默认值 | 用于控制端连接信息、预览和对外链接生成 |
| `VIBE_RELAY_FORWARD_HOST` | 预览和转发对外主机名 | 尽可能从 `VIBE_PUBLIC_RELAY_BASE_URL` 推导 | 用于客户端可访问的预览地址 |

关键规则：

- `VIBE_PUBLIC_RELAY_BASE_URL` 不会改变 relay 实际监听端口。
- 如果 relay 监听在 `8787`，且客户端直接访问该端口，则 `VIBE_PUBLIC_RELAY_BASE_URL` 必须写成包含端口的形式，例如 `http://203.0.113.10:8787`。
- `0.0.0.0` 只适合作为监听地址，不适合作为客户端访问地址。
- `127.0.0.1` 和 `localhost` 只适用于同机开发。

## 认证模型

推荐采用人类控制端和 Agent 分离的认证模型。

| 配置或文件 | 用途 | 使用方 |
| --- | --- | --- |
| `VIBE_RELAY_ACCESS_TOKEN` | 控制面认证 | 桌面端、Android、自建 Web 客户端 |
| `VIBE_RELAY_ENROLLMENT_TOKEN` | 首次设备注册 | `vibe-agent` |
| `.vibe-agent/identity.json` | 已发放设备凭证持久化文件 | `vibe-agent` 重启后的身份复用 |

行为说明：

- Agent 首次注册成功后，会使用发放的设备凭证执行心跳、任务领取、工作区请求和预览桥接。
- 删除 `.vibe-agent/identity.json` 会触发下一次启动时重新注册。
- 如果未设置 `VIBE_RELAY_ENROLLMENT_TOKEN`，relay 仍可接受控制面 token 作为兼容注册路径，但不建议在常规部署中使用该模式。

## Agent、Overlay 与 EasyTier 端口

### 默认模式

在默认 relay-polling 模式下，agent 不提供固定的公网控制面监听端口。它主要通过出站请求与 relay 交互。

### Overlay 模式

当设置 `VIBE_EASYTIER_NETWORK_NAME` 启用 EasyTier overlay 后，agent 会启动以下 bridge 监听端口：

| 端口 | 功能 | 可覆盖变量 |
| --- | --- | --- |
| `19090` | Shell bridge | `VIBE_AGENT_SHELL_BRIDGE_PORT` |
| `19091` | Port-forward bridge | `VIBE_AGENT_PORT_FORWARD_BRIDGE_PORT` |
| `19092` | Task bridge | `VIBE_AGENT_TASK_BRIDGE_PORT` |

这些端口用于 relay 与 agent 之间的 overlay 内部链路，不是浏览器或手机客户端的直接入口。

### EasyTier listener 默认行为

| 侧别 | 条件 | 默认行为 |
| --- | --- | --- |
| relay | 启用嵌入式 EasyTier，且未设置 `VIBE_EASYTIER_LISTENERS` | 默认监听 TCP/UDP `11010` |
| agent | 启用嵌入式 EasyTier | 默认 `VIBE_EASYTIER_NO_LISTENER=true`，不接受入站 EasyTier peer |
| agent | 设置 `VIBE_EASYTIER_NO_LISTENER=false`，且未设置 `VIBE_EASYTIER_LISTENERS` | 默认监听 TCP/UDP `11010` |

## 标准使用流程

推荐按以下流程使用系统：

1. 配置 relay 地址和控制面 token。
2. 确认目标设备在线。
3. 检查目标设备的 Provider 可用性。
4. 创建 AI Session。
5. 查看事件流和执行结果。
6. 使用工作区浏览、Git 检查和预览确认输出。
7. 仅在需要人工干预时使用 Shell 或高级连接能力。

## 故障排查

| 现象 | 优先检查项 |
| --- | --- |
| Agent 已启动，但控制端看不到设备 | `VIBE_RELAY_URL` 是否可达；`VIBE_RELAY_ENROLLMENT_TOKEN` 是否正确；relay `/api/health` 是否正常 |
| 设备在线，但 Provider 不可用 | 目标机器是否安装 Provider CLI；该 CLI 是否在 Agent 进程的 `PATH` 中 |
| 需要重新注册设备 | 删除 `.vibe-agent/identity.json` 后重启 Agent |
| 预览链接不可访问 | `VIBE_PUBLIC_RELAY_BASE_URL` 和 `VIBE_RELAY_FORWARD_HOST` 是否正确；客户端是否能访问该地址 |

## 系统结构

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

## 相关文档

- 自建部署文档：[docs/self-hosted.md](./docs/self-hosted.md)
- Relay 启动说明（中文）：[docs/relay-startup.zh-CN.md](./docs/relay-startup.zh-CN.md)
- Relay Startup Guide (English): [docs/relay-startup.md](./docs/relay-startup.md)
- 发布下载：[GitHub Releases](https://github.com/fage-ac-org/vibe-everywhere/releases)
