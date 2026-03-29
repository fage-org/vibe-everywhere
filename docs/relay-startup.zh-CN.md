# Relay 启动说明

最后更新：2026-03-29

本文档说明在 CLI 二进制安装完成后如何启动 `vibe-relay`。`scripts/install-relay.sh` 和
`scripts/install-relay.ps1` 现在只负责安装、更新或卸载 CLI 二进制文件，不再创建服务、写入
环境变量文件或直接启动 relay 进程。

## 启动方式

支持以下启动方式：

- 前台直接启动，适合本地测试或手工运维
- Linux `systemd`
- Windows PowerShell 启动脚本和计划任务

## 最小配置

共享部署场景下，最小实用配置如下：

```bash
export VIBE_RELAY_HOST=0.0.0.0
export VIBE_RELAY_PORT=8787
export VIBE_PUBLIC_RELAY_BASE_URL=https://relay.example.com
export VIBE_RELAY_ACCESS_TOKEN=change-control-token
export VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token
vibe-relay
```

如果仅用于本机开发，可以省略 `VIBE_PUBLIC_RELAY_BASE_URL`，或将其设为本机回环地址。若供远程桌面端、
Android 或多台机器访问，必须使用客户端真实可达的地址。

## Linux 启动

### 前台启动

```bash
export VIBE_RELAY_HOST=0.0.0.0
export VIBE_RELAY_PORT=8787
export VIBE_PUBLIC_RELAY_BASE_URL=https://relay.example.com
export VIBE_RELAY_ACCESS_TOKEN=change-control-token
export VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token
vibe-relay
```

### `systemd` 示例

环境变量文件示例：

```ini
# /etc/vibe-relay/relay.env
VIBE_RELAY_HOST=0.0.0.0
VIBE_RELAY_PORT=8787
VIBE_PUBLIC_RELAY_BASE_URL=https://relay.example.com
VIBE_RELAY_ACCESS_TOKEN=change-control-token
VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token
VIBE_RELAY_STATE_FILE=/var/lib/vibe-relay/relay-state.json
```

服务文件示例：

```ini
# /etc/systemd/system/vibe-relay.service
[Unit]
Description=Vibe Everywhere Relay
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=/etc/vibe-relay/relay.env
WorkingDirectory=/var/lib/vibe-relay
ExecStart=/usr/local/bin/vibe-relay
Restart=always
RestartSec=2
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

启用方式：

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now vibe-relay
sudo systemctl status vibe-relay
```

## Windows 启动

### 前台启动

```powershell
$env:VIBE_RELAY_HOST = "0.0.0.0"
$env:VIBE_RELAY_PORT = "8787"
$env:VIBE_PUBLIC_RELAY_BASE_URL = "https://relay.example.com"
$env:VIBE_RELAY_ACCESS_TOKEN = "change-control-token"
$env:VIBE_RELAY_ENROLLMENT_TOKEN = "change-agent-enrollment-token"
& "C:\Program Files\Vibe Everywhere\vibe-relay.exe"
```

### PowerShell 启动脚本示例

```powershell
# C:\ProgramData\Vibe Everywhere\relay-env.ps1
$env:VIBE_RELAY_HOST = "0.0.0.0"
$env:VIBE_RELAY_PORT = "8787"
$env:VIBE_PUBLIC_RELAY_BASE_URL = "https://relay.example.com"
$env:VIBE_RELAY_ACCESS_TOKEN = "change-control-token"
$env:VIBE_RELAY_ENROLLMENT_TOKEN = "change-agent-enrollment-token"
$env:VIBE_RELAY_STATE_FILE = "$env:ProgramData\Vibe Everywhere\state\relay-state.json"
```

```powershell
# C:\ProgramData\Vibe Everywhere\Start-VibeRelay.ps1
. "C:\ProgramData\Vibe Everywhere\relay-env.ps1"
& "C:\Program Files\Vibe Everywhere\vibe-relay.exe"
```

### 计划任务示例

```powershell
$action = New-ScheduledTaskAction `
  -Execute "powershell.exe" `
  -Argument '-NoProfile -ExecutionPolicy Bypass -File "C:\ProgramData\Vibe Everywhere\Start-VibeRelay.ps1"'
$trigger = New-ScheduledTaskTrigger -AtStartup
$principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest

Register-ScheduledTask -TaskName VibeRelay -Action $action -Trigger $trigger -Principal $principal -Force
Start-ScheduledTask -TaskName VibeRelay
```

## 核心环境变量

| 变量 | 默认值 | 是否建议配置 | 说明 |
| --- | --- | --- | --- |
| `VIBE_RELAY_HOST` | `0.0.0.0` | 否 | relay HTTP 监听地址 |
| `VIBE_RELAY_PORT` | `8787` | 否 | relay HTTP 监听端口 |
| `VIBE_PUBLIC_RELAY_BASE_URL` | 生产环境无默认公网值 | 建议配置 | 客户端访问 relay 时使用的对外地址 |
| `VIBE_RELAY_ACCESS_TOKEN` | 无 | 建议配置 | 桌面端、Android、Web 和运维 API 使用的控制面 token |
| `VIBE_RELAY_ENROLLMENT_TOKEN` | 无 | 建议配置 | Agent 首次注册使用的 enrollment token |
| `VIBE_RELAY_STATE_FILE` | 平台默认路径 | 否 | relay 状态文件路径 |
| `VIBE_RELAY_DEPLOYMENT_MODE` | `self_hosted` | 否 | 暴露给客户端的部署模式元数据 |
| `VIBE_RELAY_FORWARD_HOST` | 尽可能从 `VIBE_PUBLIC_RELAY_BASE_URL` 推导 | 否 | 预览和转发外链使用的公网主机名 |
| `VIBE_RELAY_FORWARD_BIND_HOST` | 与 `VIBE_RELAY_HOST` 相同 | 否 | relay 负责的预览和转发监听地址 |
| `VIBE_RELAY_FORWARD_PORT_START` | `39000` | 否 | relay 预览和转发端口范围起始值 |
| `VIBE_RELAY_FORWARD_PORT_END` | `39999` | 否 | relay 预览和转发端口范围结束值 |

## 默认身份相关变量

下列变量通常仅在高级或多租户场景下使用：

| 变量 | 默认值 | 说明 |
| --- | --- | --- |
| `VIBE_DEFAULT_TENANT_ID` | 仓库默认 tenant ID | relay 默认租户标识 |
| `VIBE_DEFAULT_USER_ID` | 仓库默认 user ID | relay 默认用户标识 |
| `VIBE_DEFAULT_USER_ROLE` | `owner` | relay 默认角色 |

## Overlay 与 EasyTier 变量

仅在启用嵌入式 EasyTier overlay 时使用：

| 变量 | 默认值 | 说明 |
| --- | --- | --- |
| `VIBE_EASYTIER_RELAY_ENABLED` | 未设置网络名时为 `false` | 启用 relay 侧嵌入式 EasyTier |
| `VIBE_EASYTIER_NETWORK_NAME` | 无 | overlay 网络名 |
| `VIBE_EASYTIER_NETWORK_SECRET` | 无 | overlay 网络密钥 |
| `VIBE_EASYTIER_BOOTSTRAP_URL` | 无 | 逗号分隔的 EasyTier peer 地址 |
| `VIBE_EASYTIER_LISTENERS` | relay 侧默认 TCP/UDP `11010` | EasyTier listener 配置 |
| `VIBE_EASYTIER_PRIVATE_MODE` | 根据网络名推导 | EasyTier private mode |
| `VIBE_EASYTIER_NO_TUN` | `false` | 在特殊环境中关闭 TUN |
| `VIBE_EASYTIER_INSTANCE_NAME` | relay 默认值 | EasyTier 实例名 |
| `VIBE_EASYTIER_HOSTNAME` | relay 默认值 | EasyTier 主机名 |
| `VIBE_AGENT_SHELL_BRIDGE_PORT` | `19090` | overlay 模式下 agent 侧 shell bridge 端口 |
| `VIBE_AGENT_PORT_FORWARD_BRIDGE_PORT` | `19091` | overlay 模式下 agent 侧 port-forward bridge 端口 |
| `VIBE_AGENT_TASK_BRIDGE_PORT` | `19092` | overlay 模式下 agent 侧 task bridge 端口 |

## 地址配置规则

- `VIBE_RELAY_HOST` 和 `VIBE_RELAY_PORT` 决定 relay 在本地监听的位置。
- `VIBE_PUBLIC_RELAY_BASE_URL` 决定客户端应访问的地址。
- `0.0.0.0` 只能用于监听地址，不能作为客户端访问地址。
- `127.0.0.1` 和 `localhost` 仅适用于同机开发。
- 如果 relay 实际监听在 `8787`，且客户端直接访问该端口，则
  `VIBE_PUBLIC_RELAY_BASE_URL` 必须包含 `:8787`。

## 健康检查

示例：

```bash
curl http://127.0.0.1:8787/api/health
curl http://203.0.113.10:8787/api/health
curl https://relay.example.com/api/health
```

## 相关文档

- [README.md](../README.md)
- [README.en.md](../README.en.md)
- [Self-Hosted Deployment Guide](./self-hosted.md)
