# Vibe Everywhere

[![CI](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml)
[![Release](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml/badge.svg)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

[中文](./README.md) | [English](./README.en.md)

Rust-first 的远程 AI 控制平面：`Rust relay + Rust agent + Vue 3.5 + Tauri 2 app`。

它不是传统远程桌面，而是一个以 AI Session 为中心的远程开发控制系统。服务端负责设备注册、状态维护和控制面 API，Agent 运行在被控设备上执行 AI 会话、工作区相关操作和高级诊断能力，控制端可以通过 Web、Tauri 桌面壳和 Android 壳连接整个系统。

## 项目状态

- 当前定位：个人版 MVP / 开源实验项目
- 当前主流程：设备选择、AI Session 发起、事件流监督、高级 Terminal / Preview 工具
- 当前技术方向：以 Rust 为核心，控制端统一走 Vue + Tauri，服务端和 Agent 统一协议
- 当前移动端：Android arm64 APK / AAB 已打通，iOS 待补齐
- 当前适用场景：个人远程 AI 工作台、自托管多设备控制面、跨平台实验性远程协作

## 核心能力

- Rust workspace 架构，协议、服务端、Agent、桌面端共享同一仓库
- `vibe-relay` 提供 Axum API、设备状态管理、AI Session 调度、Terminal 与 Preview 控制面
- `vibe-agent` 提供设备注册、轮询执行、Provider 适配、Workspace 根目录运行时以及高级 Terminal / Tunnel 能力
- `vibe-app` 提供 Vue 3.5 控制台，当前以 AI Session 工作台为主，`src-tauri` 提供桌面壳和 Android 移动壳
- 支持 `Codex`、`Claude Code`、`OpenCode` Provider 接入
- 支持 Relay-first AI Session、Terminal、TCP 预览/转发
- 支持基于 EasyTier 的 Overlay 辅助传输
- 支持 Tauri Android arm64 调试 APK、release APK 与 AAB 构建
- 支持 SSE / WebSocket / Tunnel 等多种实时通道

## 架构概览

```text
┌──────────────────────────────────────────────────────────┐
│                     Control App                          │
│           Vue 3.5 Web UI / Tauri Desktop Shell          │
└───────────────────────────┬──────────────────────────────┘
                            │ HTTP / SSE / WebSocket
┌───────────────────────────▼──────────────────────────────┐
│                      vibe-relay                          │
│   device registry · task control · shell · port proxy   │
│   auth · persistence · overlay-aware transport choice    │
└───────────────────────────┬──────────────────────────────┘
                            │ HTTP polling / bridge / tunnel
┌───────────────────────────▼──────────────────────────────┐
│                      vibe-agent                          │
│ provider adapters · task runtime · shell runtime         │
│ port-forward runtime · embedded overlay node             │
└───────────────────────────┬──────────────────────────────┘
                            │ local process / local TCP
                    ┌───────▼────────┐
                    │ target machine │
                    └────────────────┘
```

## 仓库结构

```text
.
├── apps
│   ├── vibe-relay        # Relay API / control plane
│   ├── vibe-agent        # Device agent / runtimes / providers
│   └── vibe-app          # Vue control app
│       └── src-tauri     # Tauri desktop shell
│           └── gen/android  # Generated Tauri Android project
├── crates
│   └── vibe-core         # Shared protocol / models
├── scripts               # Smoke tests and helper scripts
└── TESTING.md            # Testing strategy and validation matrix
```

## 快速开始

### 依赖要求

- Rust stable toolchain
- Node.js 20+
- `protobuf-compiler` 或可用的 `protoc`
- Linux 下构建 Tauri 时需要 WebKitGTK / GTK 相关开发包
- Android 构建需要 JDK 17、Android SDK cmdline-tools，以及 `platforms;android-36`、`build-tools;35.0.0`、`ndk;25.2.9519653`
- Windows 下如果要启用 EasyTier / Overlay 相关能力，建议安装 Npcap，并启用 WinPcap API-compatible Mode
- 如果要实际执行 AI 任务，需要本机至少安装一个 Provider CLI
  - `codex`
  - `claude`
  - `opencode`

### 1. 克隆仓库

```bash
git clone https://github.com/fage-ac-org/vibe-everywhere.git
cd vibe-everywhere
```

### 2. 启动 relay

```bash
cargo run -p vibe-relay
```

默认监听 `http://127.0.0.1:8787`。

如果你希望开启单用户访问控制，可以在启动前设置：

```bash
export VIBE_RELAY_ACCESS_TOKEN=change-me
```

### 3. 启动 agent

```bash
cargo run -p vibe-agent -- --relay-url http://127.0.0.1:8787
```

如果没有安装任何 Provider CLI，设备仍然可以注册上线，但 AI 任务能力会显示不可用。

### 4. 启动 Web 控制台

```bash
cd apps/vibe-app
npm ci
npm run dev
```

默认访问地址：

- `http://127.0.0.1:1420`

如果 relay 开启了访问令牌，可在页面右上区域填入 token，或者在前端环境变量中设置：

```bash
export VITE_RELAY_BASE_URL=http://127.0.0.1:8787
export VITE_RELAY_ACCESS_TOKEN=change-me
```

### 5. 启动桌面壳

```bash
cd apps/vibe-app
npm ci
npm run tauri dev
```

Tauri 桌面壳会读取：

- `VIBE_PUBLIC_RELAY_BASE_URL`
- `VIBE_RELAY_ACCESS_TOKEN`

### 6. 构建 Android 测试包

如果你要从 Android 手机远程控制服务器，可以直接构建 Tauri Android 包：

```bash
rustup target add aarch64-linux-android

export JAVA_HOME=/path/to/jdk-17
export ANDROID_HOME=$HOME/Android/Sdk
export ANDROID_SDK_ROOT=$ANDROID_HOME
export NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653
export ANDROID_NDK_HOME=$NDK_HOME

cd apps/vibe-app
npm ci
npm run android:doctor
npm run android:build:debug:apk
```

默认调试 APK 输出路径：

- `apps/vibe-app/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`

如果你要生成 release 产物：

```bash
cd apps/vibe-app
npm run android:build:apk
npm run android:build:aab
```

对应输出路径：

- `apps/vibe-app/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`
- `apps/vibe-app/src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab`

如果你要让 release APK / AAB 直接带签名，可以在
`apps/vibe-app/src-tauri/gen/android/app/keystore.properties`
写入签名信息，或在构建时导出环境变量：

```properties
storeFile=/absolute/path/to/vibe-everywhere-release.jks
storePassword=your-store-password
keyAlias=vibe-everywhere
keyPassword=your-key-password
```

支持的环境变量如下，且优先级高于 `keystore.properties`：

- `VIBE_ANDROID_KEYSTORE_PATH`
- `VIBE_ANDROID_KEYSTORE_PASSWORD`
- `VIBE_ANDROID_KEY_ALIAS`
- `VIBE_ANDROID_KEY_PASSWORD`

当签名配置齐全后，再执行：

```bash
cd apps/vibe-app
npm run android:build:apk
npm run android:build:aab
```

此时 release APK 输出路径会变成：

- `apps/vibe-app/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`

如果你希望 GitHub Actions 在发布时自动签名，需要在仓库或组织的
Actions Secrets 中配置以下 4 个 Secret：

- `VIBE_ANDROID_KEYSTORE_BASE64`
- `VIBE_ANDROID_KEYSTORE_PASSWORD`
- `VIBE_ANDROID_KEY_ALIAS`
- `VIBE_ANDROID_KEY_PASSWORD`

其中 `VIBE_ANDROID_KEYSTORE_BASE64` 是 keystore 文件内容的 Base64。
例如本地可以这样生成：

```bash
base64 -w 0 /absolute/path/to/vibe-everywhere-release.jks
```

macOS 如果没有 `-w` 参数，可以用：

```bash
base64 < /absolute/path/to/vibe-everywhere-release.jks | tr -d '\n'
```

配置完成后，GitHub 的 release 工作流会自动把 keystore 解码到 runner
临时目录，并通过 `VIBE_ANDROID_*` 环境变量注入构建。未配置这些 Secret
时，工作流仍会继续执行，但 release APK 会保持 `unsigned`。

注意：

- Android / iOS 控制端默认不会预填 `127.0.0.1:8787`；首次启动请手动填写 relay 所在机器的局域网 IP 或 HTTPS 公网地址，除非你显式设置了 `VIBE_PUBLIC_RELAY_BASE_URL`
- 手机上的 relay 地址应该配置为 `http://<服务器局域网IP>:8787` 或 HTTPS 公网地址，不要使用 `http://127.0.0.1:8787`
- 当前 Android 包默认允许 HTTP 明文流量，方便自托管局域网 relay；如果对外发布，仍然建议使用 HTTPS
- 如果 `tauri android build` 报 NDK `source.properties` 缺失，说明 SDK 里有半安装状态的 NDK；先运行 `npm run android:doctor`，再重装对应 NDK 或显式导出 `NDK_HOME`
- `apps/vibe-app/src-tauri/gen/android/app/keystore.properties` 和任何 `.jks` / `.keystore` 文件都不应该提交到仓库

### 7. 验证链路

完成以上步骤后，你应该可以看到：

- 控制台能连上 relay
- Agent 设备出现在设备列表里
- 如果安装了可用 Provider，可以创建并执行任务
- 可以创建 Shell Session
- 可以创建 TCP 端口转发

## 开发命令

```bash
cargo check -p vibe-relay -p vibe-agent -p vibe-app
cargo test --workspace --all-targets -- --nocapture
cd apps/vibe-app && npm ci && npm run build
```

常用启动方式：

```bash
cargo run -p vibe-relay
cargo run -p vibe-agent -- --relay-url http://127.0.0.1:8787
cd apps/vibe-app && npm run dev
cd apps/vibe-app && npm run tauri dev
cd apps/vibe-app && npm run android:build:debug:apk
cd apps/vibe-app && npm run android:build:apk
cd apps/vibe-app && npm run android:build:aab
```

## 测试

完整测试方案见 [TESTING.md](./TESTING.md)。

当前推荐的本地最小验证集：

```bash
cargo fmt --all --check
cargo check -p vibe-relay -p vibe-agent -p vibe-app
cargo test --workspace --all-targets -- --nocapture
cd apps/vibe-app && npm ci && npm run build
./scripts/dual-process-smoke.sh relay_polling
```

涉及 Android 移动端改动时，建议额外执行：

```bash
cd apps/vibe-app && npm run android:build:debug:apk
cd apps/vibe-app && npm run android:build:apk
cd apps/vibe-app && npm run android:build:aab
```

涉及 Overlay / EasyTier / Shell / 端口转发传输路径时，建议额外执行：

```bash
./scripts/dual-process-smoke.sh overlay
```

## GitHub Actions

仓库内置两套工作流：

- `CI`
  - 触发时机：`push` 到 `main`、`pull_request`、手动触发
  - 执行内容：Rust 格式检查、workspace 编译、workspace 测试、前端构建、`relay_polling` 烟测、Windows Rust 编译与 Tauri MSI 打包兼容性校验、Android debug APK 构建与产物上传
- `Release`
  - 触发时机：推送 `v*` tag
  - 执行内容：完整验证、best-effort `overlay` 烟测、Linux / Windows CLI 与 Tauri 桌面包构建、Android debug APK / release APK / AAB 构建、GitHub Release 资产上传

发布方式示例：

```bash
git tag v0.1.0
git push origin v0.1.0
```

Release 工作流会上传类似以下资产：

- `vibe-everywhere-cli-x86_64-unknown-linux-gnu.tar.gz`
- `vibe-everywhere-desktop-x86_64-unknown-linux-gnu.tar.gz`
- `vibe-everywhere-cli-x86_64-pc-windows-msvc.zip`
- `vibe-everywhere-desktop-x86_64-pc-windows-msvc.zip`
- `vibe-everywhere-android-arm64-debug.apk`
- `vibe-everywhere-android-arm64-release-unsigned.apk`
- `vibe-everywhere-android-arm64-release.aab`
- `SHA256SUMS.txt`

说明：

- 当前仓库还没有内置 Android 发布签名密钥，所以 release APK 默认为 `unsigned`
- 如果要直接安装测试版，请优先使用 debug APK

## 常用环境变量

### relay

- `VIBE_RELAY_HOST`
- `VIBE_RELAY_PORT`
- `VIBE_PUBLIC_RELAY_BASE_URL`
- `VIBE_RELAY_ACCESS_TOKEN`
- `VIBE_RELAY_STATE_FILE`
- `VIBE_RELAY_FORWARD_HOST`
- `VIBE_RELAY_FORWARD_BIND_HOST`

### agent

- `VIBE_RELAY_URL`
- `VIBE_RELAY_ACCESS_TOKEN`
- `VIBE_DEVICE_NAME`
- `VIBE_DEVICE_ID`
- `VIBE_WORKING_ROOT`
- `VIBE_CODEX_COMMAND`
- `VIBE_CLAUDE_COMMAND`
- `VIBE_OPENCODE_COMMAND`

### overlay

- `VIBE_EASYTIER_RELAY_ENABLED`
- `VIBE_EASYTIER_NETWORK_NAME`
- `VIBE_EASYTIER_NETWORK_SECRET`
- `VIBE_EASYTIER_BOOTSTRAP_URL`
- `VIBE_EASYTIER_LISTENERS`

### frontend / desktop

- `VITE_RELAY_BASE_URL`
- `VITE_RELAY_ACCESS_TOKEN`
- `VIBE_PUBLIC_RELAY_BASE_URL`
- `VIBE_RELAY_ACCESS_TOKEN`

### android signing

- `VIBE_ANDROID_KEYSTORE_PATH`
- `VIBE_ANDROID_KEYSTORE_PASSWORD`
- `VIBE_ANDROID_KEY_ALIAS`
- `VIBE_ANDROID_KEY_PASSWORD`
- `VIBE_ANDROID_KEYSTORE_BASE64`（仅 GitHub Actions Secret 使用）

## 路线图

- 增强认证、审计和生产化部署能力
- 补充前端自动化测试和协议 round-trip 测试
- 补齐 iOS 客户端和移动端发布自动化链路
- 继续压缩 `main.rs` 中的聚合逻辑，稳定模块边界
- 扩展更完整的文件同步、工作区浏览和通知能力
- 完善桌面端和移动端体验

## 贡献

欢迎提交 Issue 和 Pull Request。

建议在 PR 中包含：

- 变更背景和目标
- 受影响的 crate / app
- 已执行的验证命令
- 如果涉及 UI，附带截图
- 如果涉及环境变量或系统依赖，明确说明

提交信息建议使用 Conventional Commits，例如：

```text
feat(agent): add claude stream-json mapping
fix(relay): keep overlay transport fallback stable
docs(readme): rewrite project overview and quick start
```

## 许可证

本项目使用 [MIT License](./LICENSE)。
