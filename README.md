# 🦞 OpenClaw Manager

**One-click installer & management GUI for [OpenClaw](https://github.com/miaoxworld/OpenClawInstaller)** — the open-source AI assistant framework.

Built with **Tauri 2.0 + React 18 + TypeScript + Rust** for native performance on every desktop platform.

![Platform](https://img.shields.io/badge/platform-macOS%20|%20Windows%20|%20Linux-blue)
![Tauri](https://img.shields.io/badge/Tauri-2.0-orange)
![React](https://img.shields.io/badge/React-18-61DAFB)
![Rust](https://img.shields.io/badge/Rust-1.70+-red)

---

## ✨ Key Features

### 🚀 One-Click Setup Wizard
Skip the terminal entirely. The built-in setup wizard automatically detects your environment, installs Node.js and OpenClaw, and initializes everything — all from the GUI.

- Automatic detection of Node.js, Git, and OpenClaw
- One-click installation of missing prerequisites
- Cross-platform support (Windows, macOS, Linux)
- Guided environment setup with real-time status updates

### 📊 Dashboard & Service Control
Real-time monitoring and full lifecycle management of the OpenClaw service.

![Dashboard](pic/dashboard.png)

- Live service status (port, PID, memory usage, uptime)
- **Start / Stop / Restart / Kill All** actions
- Embedded system requirements checker
- Real-time log viewer with auto-refresh

### 🧩 MCP Management
Full [Model Context Protocol](https://modelcontextprotocol.io/) server management with integrated **mcporter** support.

- Add, edit, remove, enable/disable MCP servers
- One-click **mcporter** install/uninstall
- Test MCP server connectivity
- Automatic sync to `~/.mcporter/mcporter.json` for seamless OpenClaw integration
- Support for stdio and SSE transport types

### 📚 Skills Management
Browse, install, and manage OpenClaw skills via **ClawHub**.

- One-click **ClawHub** install/uninstall
- Browse available skills from the ClawHub registry
- Install and uninstall individual skills
- View skill metadata (name, description, version)

### 🤖 AI Model Configuration
Flexible multi-provider AI configuration with custom endpoint support.

![AI Configuration](pic/ai.png)

- **14+ AI providers**: Anthropic, OpenAI, DeepSeek, Google Gemini, Moonshot, Z.AI (GLM), and more
- Custom API endpoints — compatible with any OpenAI-format service
- One-click primary model switching
- API key management

### 📱 Message Channels
Connect OpenClaw to multiple messaging platforms for omnichannel AI.

<table>
  <tr>
    <td width="50%">
      <img src="pic/telegram.png" alt="Telegram Configuration">
      <p align="center"><b>Telegram Bot</b></p>
    </td>
    <td width="50%">
      <img src="pic/feishu.png" alt="Feishu Configuration">
      <p align="center"><b>Feishu Bot</b></p>
    </td>
  </tr>
</table>

- **Telegram** — Bot Token, private chat & group policies
- **Feishu** — App ID/Secret, WebSocket, multi-region deployment
- **Discord, Slack, WhatsApp, iMessage, WeChat, DingTalk** — and more

### 🤖 Multi-Agent Routing
Run multiple specialized AI agents with intelligent message routing and nested subagent orchestration.

- **Agent management** — create, edit, clone, and delete agents with isolated workspaces
- **Default Agent toggle** — designate a primary agent that uses the main workspace (`~/.openclaw/workspace`)
- **Quick Setup Wizard** — 3-step guided flow: select bot → configure agent → set personality
- **Subagent configuration** — per-agent `allowAgents` list to control which agents can be spawned as subagents
- **Global subagent defaults** — `maxSpawnDepth`, `maxChildrenPerAgent`, `maxConcurrent` limits
- **Routing rules** — bind agents to specific Telegram bot accounts and channels
- **Personality editor** — inline `SOUL.md` editor per agent
- **Agent routing test** — real-time routing resolution preview

#### Nested Subagent Example
```
researchbot (depth 0)
    └─→ orchestrator subagent (depth 1)
            ├─→ worker 1 (depth 2) — research topic A
            ├─→ worker 2 (depth 2) — research topic B
            └─→ worker 3 (depth 2) — research topic C
```

#### Directory Structure
```
~/.openclaw/
├── workspace/              ← default agent workspace
├── workspace-coder/        ← per-agent workspace
├── agents/
│   ├── main/
│   │   ├── agent/          ← agent state
│   │   └── sessions/       ← session data
│   └── coder/
│       ├── agent/
│       └── sessions/
```

### 📋 Application Logs
Built-in structured log viewer with filtering, color-coded levels, and export.

- Filter by level: Debug, Info, Warning, Error
- Color-coded source modules (App, Service, Config, AI, etc.)
- One-click log export and clear

### 🔄 Auto-Update
Automatic update detection for OpenClaw with one-click upgrade.

- Checks npm registry for the latest OpenClaw version
- Compare and display current vs. latest version
- One-click update from within the app

### 🧪 Testing & Diagnostics
Comprehensive system, AI, and channel connectivity testing.

- System environment checks
- AI provider connection tests
- Channel connectivity verification

---

## 📁 Project Structure

```
openclaw-manager/
├── src-tauri/                 # Rust Backend
│   ├── src/
│   │   ├── main.rs            # Entry point
│   │   ├── commands/
│   │   │   ├── config.rs      # Configuration & MCP sync
│   │   │   ├── diagnostics.rs # Diagnostics & testing
│   │   │   ├── installer.rs   # Environment detection & one-click installs
│   │   │   ├── process.rs     # Process management
│   │   │   ├── service.rs     # Service lifecycle
│   │   │   └── skills.rs      # ClawHub & skills management
│   │   ├── models/            # Data models
│   │   └── utils/             # Platform helpers & shell utilities
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                       # React Frontend
│   ├── App.tsx                # Root app with setup wizard & update banner
│   ├── components/
│   │   ├── Layout/            # Sidebar navigation & header
│   │   ├── Dashboard/         # Service status, quick actions, system info
│   │   ├── MCP/               # MCP server management (mcporter)
│   │   ├── Skills/            # Skills management (ClawHub)
│   │   ├── AIConfig/          # AI provider configuration
│   │   ├── Channels/          # Messaging channel configuration
│   │   ├── Agents/            # Multi-agent routing & subagent config
│   │   ├── Testing/           # Diagnostics & connectivity tests
│   │   ├── Logs/              # Structured log viewer
│   │   ├── Setup/             # One-click setup wizard
│   │   └── Settings/          # App settings
│   ├── hooks/                 # Custom React hooks
│   ├── lib/                   # Tauri API bridge & logger
│   ├── stores/                # Zustand state management
│   └── styles/
│       └── globals.css
│
├── package.json
├── vite.config.ts
└── tailwind.config.js
```

---

## 🛠️ Tech Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Frontend | React 18 | UI framework |
| State | Zustand | Lightweight reactive state |
| Styling | TailwindCSS | Utility-first CSS |
| Animation | Framer Motion | Smooth transitions & micro-interactions |
| Icons | Lucide React | Consistent icon set |
| Backend | Rust | High-performance system operations |
| Desktop | Tauri 2.0 | Native cross-platform shell |

---

## 🚀 Quick Start (Development)

### Prerequisites

| Tool | Version | Download |
|------|---------|----------|
| **Node.js** | >= 18.0 | [nodejs.org](https://nodejs.org/) |
| **Rust** | >= 1.70 | [rustup.rs](https://rustup.rs/) |
| **pnpm** or npm | Latest | Comes with Node.js |

<details>
<summary><b>Platform-specific dependencies</b></summary>

**macOS**
```bash
xcode-select --install
```

**Windows**
- [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) *(pre-installed on Windows 10/11)*

**Linux (Ubuntu/Debian)**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Linux (Fedora)**
```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file libxdo-devel
```
</details>

### Clone & Run

```bash
git clone https://github.com/MrFadiAi/openclaw-one-click-installer.git
cd openclaw-one-click-installer

npm install          # Install dependencies
npm run tauri:dev    # Launch in development mode (hot-reload)
```

> **Note:** First build compiles all Rust dependencies and takes **3–5 minutes**. Subsequent runs are much faster.

### Build Release

```bash
npm run tauri:build
```

Output in `src-tauri/target/release/bundle/`:

| Platform | Formats |
|----------|---------|
| macOS | `.dmg`, `.app` |
| Windows | `.msi`, `.exe` |
| Linux | `.deb`, `.AppImage` |

---

## 🔧 Development Commands

```bash
npm run tauri:dev          # Full desktop app with hot-reload
npm run dev                # Frontend only (browser)
npm run build              # Build frontend
npm run tauri:build        # Build desktop release

cd src-tauri && cargo check   # Check Rust code
cd src-tauri && cargo test    # Run Rust tests
```

---

## 🍎 macOS Troubleshooting

<details>
<summary><b>"Damaged, cannot be opened" error</b></summary>

macOS Gatekeeper may block unsigned apps.

**Remove quarantine attribute (recommended):**
```bash
xattr -cr /Applications/OpenClaw\ Manager.app
```

**Or allow via System Preferences:**
1. Open **System Preferences** > **Privacy & Security**
2. Find the blocked app → Click **Open Anyway**
</details>

<details>
<summary><b>Permission issues</b></summary>

Grant **Full Disk Access**:
1. **System Preferences** > **Privacy & Security** > **Full Disk Access**
2. Add **OpenClaw Manager**
</details>

---

## 🎨 Design Philosophy

- **Dark Theme** — Eye-friendly for extended sessions
- **Modern UI** — Frosted glass, gradients, smooth animations
- **Responsive** — Adapts to any desktop window size
- **Native Performance** — Rust backend with minimal memory footprint

---

## 🤝 Contributing

1. Fork the project
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📄 License

MIT License — See [LICENSE](LICENSE) for details.

## 🔗 Related Links

- [OpenClaw Manager](https://github.com/MrFadiAi/openclaw-one-click-installer) — This project (GUI)
- [OpenClawInstaller](https://github.com/miaoxworld/OpenClawInstaller) — CLI installer
- [Tauri Documentation](https://tauri.app/)
- [React Documentation](https://react.dev/)

---

**Made with ❤️ by the OpenClaw Community**
