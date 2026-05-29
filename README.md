<div align="center">

<img src="./src/assets/icon.png" alt="VT Hub Manager" width="120" />

# VT Hub Manager

**面向 AI 编程工具的本地桌面工作台**
统一管理 Codex / Claude / Cursor 的项目上下文、规则、技能与预设

[![Vue 3](https://img.shields.io/badge/Vue-3-42b883?logo=vuedotjs&logoColor=white)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-3178c6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Vite](https://img.shields.io/badge/Vite-6-646cff?logo=vite&logoColor=white)](https://vitejs.dev/)
[![Tauri 2](https://img.shields.io/badge/Tauri-2-ffc131?logo=tauri&logoColor=black)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![pnpm](https://img.shields.io/badge/pnpm-10-f69220?logo=pnpm&logoColor=white)](https://pnpm.io/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-2ea44f)](#-环境要求)

🌐 **中文** ｜ [English](./README.en-US.md)

</div>

---

> 🪄 一句话：把分散在各家 AI 工具里的 **项目上下文、规则、技能、Provider 配置** 拢到一个本地桌面应用里管理，每次写入都可预览、可备份、可回滚。

应用基于 **Vue 3 + TypeScript + Tauri 2 + Rust** 构建，所有数据保存在你本机的 `~/.vt-agent-hub/` 目录下，无需任何在线服务。

---

## 📑 目录

- [🎯 适合谁用](#-适合谁用)
- [✨ 核心功能](#-核心功能)
- [🤖 支持的 AI 工具](#-支持的-ai-工具)
- [🛠 环境要求](#-环境要求)
- [🚀 快速开始](#-快速开始)
- [📜 常用命令](#-常用命令)
- [🧭 基本使用流程](#-基本使用流程)
- [💾 数据存储位置](#-数据存储位置)
- [🗂 项目结构](#-项目结构)
- [❓ 常见问题](#-常见问题)
- [🤝 开发与贡献](#-开发与贡献)

---

## 🎯 适合谁用

| 你正在做什么…                                                | 这个工具能帮你…                                          |
| ------------------------------------------------------------ | --------------------------------------------------------- |
| 🧑‍💻 同时在用 **Codex / Claude / Cursor**                        | 让多个工具共用同一份项目上下文                            |
| 🗂 经常在多个本地项目之间切换                                  | 一键给每个项目准备好 `AGENTS.md` 与工具规则               |
| 🧠 想沉淀可复用的提示词                                        | 整理成 **Rules / Skills**，跨项目复用                     |
| 🔑 想集中管理多家模型的 **Provider**                          | 维护多套配置（模型 / Base URL / Token），一键切换         |
| 🛟 担心 AI 工具乱改你的配置                                    | 每次写入都 **预览 + 备份 + 回滚**，操作全程留痕           |

---

## ✨ 核心功能

### 📁 项目管理（Projects）

- 创建、导入本地项目，或 **从 Git 仓库直接导入**
- 为每个项目维护独立的规则绑定
- 预览即将写入项目目录的 `AGENTS.md`
- 一键 **Apply / Repair / Cleanup / Reset** 项目输出，写入前自动备份

### 📜 规则管理（Rules）

- 创建、编辑、删除规则
- 从本地 Markdown 文件或仓库导入规则
- 按分类排序、调整顺序
- 把一条规则绑定到 **单个项目** 或 **整个工具** 的全局上下文
- 修改前预览影响范围（哪些项目、哪些工具会受影响）

### 🧩 技能管理（Skills）

- 创建、编辑、删除技能
- 从 **GitHub 仓库** 导入技能（支持冲突检测与重命名）
- 把技能安装到具体的 AI 工具运行时
- 提供 install / uninstall / repair / mark stale 等状态操作

### ⚙️ 预设管理（Presets / Providers）

- 维护多套 Provider 配置（名称、分类、网址、备注）
- 为每个工具单独维护模型、推理参数、Base URL、凭据引用
- 从粘贴的配置文本中解析并导入 Provider
- 预览将写入工具实际配置文件的 diff
- 一键 Apply 到工具运行时，并检测线上配置漂移（drift）

### 🔐 工具与凭据管理

- 查看每个工具的安装状态、版本和健康度
- 保存、验证、清除工具凭据
- 凭据使用 **系统钥匙串 / 凭据存储**，绝不写入明文配置文件
- 对工具配置进行修复（repair）

### 🕘 历史与备份

- 查看所有写入、导入、修复操作的历史
- 浏览自动备份列表，支持 **预览 diff、还原、删除**
- 一键导出诊断信息，便于排查

### 🪄 首次导入

- 启动时自动扫描 `~/.codex` 和 `~/.claude` 已有的规则、技能、Provider
- 自由挑选要导入的条目、冲突策略和凭据策略

### 🎨 界面

- 🌍 中文 / English 双语切换
- 🌗 浅色 / 深色 / 跟随系统
- ⌘ 命令面板（Command Palette）快速跳转
- 📌 项目坞（Project Dock）快速切换当前项目

---

## 🤖 支持的 AI 工具

| 工具       | 状态       | 规则 | 预设 | 凭据 | 技能安装 | 项目 AGENTS.md |
| ---------- | ---------- | :--: | :--: | :--: | :------: | :------------: |
| 🟢 Codex   | ✅ 已启用  |  ✅  |  ✅  |  ✅  |    ✅    |       ✅       |
| 🟣 Claude  | ✅ 已启用  |  ✅  |  ✅  |  ⛔  |    ⛔    |       ✅       |
| 🟠 Cursor  | 🚧 规划中  |  ✅  |  ⛔  |  ⛔  |    ⛔    |       ✅       |

> 上表反映 Rust 后端 `src-tauri/src/core/tool_registry.rs` 中声明的工具能力。Cursor 会在后续版本开启。

---

## 🛠 环境要求

| 依赖              | 版本       | 说明                                       |
| ----------------- | ---------- | ------------------------------------------ |
| 🟩 Node.js        | ≥ 22       | 前端构建与脚本运行                         |
| 📦 pnpm           | ≥ 10       | 包管理工具                                 |
| 🦀 Rust           | stable     | Tauri 后端编译                             |
| 🖥 Tauri 2 依赖   | 见官网     | [Tauri prerequisites](https://tauri.app/start/prerequisites/) |

安装 pnpm：

```bash
npm install -g pnpm
```

---

## 🚀 快速开始

```bash
# 1️⃣  克隆仓库
git clone https://github.com/twj0415/vt-agent-hub.git
cd vt-agent-hub

# 2️⃣  安装依赖
pnpm install

# 3️⃣  启动桌面开发模式（会同时启动 Vite 和 Tauri）
pnpm tauri:dev
```

🎉 第一次启动时：

1. 应用会在 `~/.vt-agent-hub/` 下创建本地存储目录（数据库、资产库、备份、日志等）。
2. 如果检测到 `~/.codex` 或 `~/.claude` 中已有规则 / 技能 / Provider，会弹出 **首次导入** 对话框。
3. 选择要导入的内容、冲突策略后即可开始使用。

> 💡 只想跑前端调试（不连接 Tauri 后端）？
>
> ```bash
> pnpm dev   # 启动 Vite，地址固定为 http://127.0.0.1:24220
> ```

---

## 📜 常用命令

```bash
pnpm dev               # 🧪 启动 Vite 开发服务（仅前端）
pnpm tauri:dev         # 🖥  启动 Tauri 桌面开发模式
pnpm build             # 📦 类型检查并构建前端产物（dist/）
pnpm typecheck         # ✅ 仅运行 TypeScript 类型检查
pnpm test              # 🔬 运行 Vitest 前端单元测试
pnpm tauri             # 🛠 透传 Tauri CLI
pnpm tauri:build:nsis  # 🪟 构建 Windows NSIS 安装包
```

Rust 后端测试：

```bash
cd src-tauri
cargo test
```

---

## 🧭 基本使用流程

```mermaid
flowchart LR
    A[📁 创建/导入项目] --> B[📜 准备规则]
    B --> C[🔗 绑定规则到项目]
    C --> D[👀 预览 AGENTS.md]
    D --> E[💾 自动备份]
    E --> F[✅ Apply 写入项目]
    F --> G[🕘 History 中可回滚]
```

### 1️⃣ 创建或导入项目

- 进入侧栏 **Projects**
- 点击 **新建项目** 或 **从 Git 导入**
- 选择项目类型，填写本地路径或 Git URL

### 2️⃣ 准备规则

- 进入 **Assets → Rules**
- 创建一条新规则，或 **从本地文件 / 仓库导入**
- 给规则分类，调整顺序

### 3️⃣ 绑定规则到项目

- 回到 **Projects**，选中目标项目
- 打开 **规则绑定** 抽屉，勾选要应用的规则
- 选择目标工具（Codex / Claude）

### 4️⃣ 预览并应用 AGENTS.md

- 在项目卡片中点击 **预览输出**，查看即将写入的 `AGENTS.md` 的完整 diff
- 确认无误后点击 **Apply**
- 写入前会自动在 `~/.vt-agent-hub/backups/` 留下备份

### 5️⃣ 配置 Provider（可选）

- 进入 **Assets → Presets**
- 新建一个 Provider，填写名称、模型、Base URL
- 录入 Token（会进入系统凭据存储，不进数据库明文）
- 选择目标工具，预览 diff，确认后 **Apply 到实际工具配置**

### 6️⃣ 安装技能（可选）

- 进入 **Assets → Skills**
- 创建或从 GitHub 仓库导入技能
- 选择目标工具，点击 **Install**

### 7️⃣ 排查与回滚

- 进入 **History** 查看每一次操作
- 进入 **Settings → Maintenance** 查看备份列表
- 任意备份都可以 **预览 diff、一键还原**

### 8️⃣ 切换语言与主题

- 进入 **Settings → 外观与语言**
- 在 **中文 / English** 之间切换
- 在 **浅色 / 深色 / 跟随系统** 之间切换

---

## 💾 数据存储位置

所有用户数据保存在本机的 `~/.vt-agent-hub/` 目录下：

```text
~/.vt-agent-hub/
├── 🗃  app.db        # SQLite 主数据库（项目、规则、技能、Provider 等）
├── 📚 library/      # 资产库（规则、技能等可复用资源）
├── 💾 backups/      # 写入前自动备份
├── 📝 logs/         # 操作日志
├── 📸 snapshots/    # 数据快照
└── 🧰 runtime/      # 工具运行时资产
```

> 🔐 凭据（Token 等）通过 **系统凭据管理器** 存储，**不会以明文形式落盘** 到 `app.db` 或任何配置文件。

如需把数据目录指向其它位置，可通过环境变量覆盖：

| 环境变量                          | 作用                                     |
| --------------------------------- | ---------------------------------------- |
| `VT_HUB_MANAGER_STORAGE_ROOT`     | 覆盖默认的 `~/.vt-agent-hub` 存储目录    |
| `VT_HUB_MANAGER_CODEX_ROOT`       | 覆盖默认的 `~/.codex` 路径               |
| `VT_HUB_MANAGER_CLAUDE_ROOT`      | 覆盖默认的 `~/.claude` 路径              |
| `VT_HUB_MANAGER_CURSOR_ROOT`      | 覆盖默认的 `~/.cursor` 路径              |

---

## 🗂 项目结构

```text
.
├── 🎨 src/                # 前端源码
│   ├── app/              # 启动、路由、主题、全局样式
│   ├── features/         # 跨页面功能模块（首次导入、仓库导入等）
│   ├── pages/            # 主要页面：projects / tools / rules / skills / presets / history / settings
│   └── shared/           # API、组件、stores、i18n、工具函数、类型
├── 🦀 src-tauri/          # Tauri / Rust 后端
│   ├── src/
│   │   ├── adapters/        # 各 AI 工具的适配器
│   │   ├── application/     # 应用服务层
│   │   ├── commands/        # Tauri command 层
│   │   ├── core/            # 常量、路径、工具注册、状态码
│   │   ├── domain/          # 领域对象
│   │   ├── dto/             # 前后端 DTO
│   │   └── infrastructure/  # SQLite、仓储、迁移、资产仓库
│   ├── Cargo.toml
│   └── tauri.conf.json
├── 📦 package.json
├── 🔒 pnpm-lock.yaml
└── ⚙️  vite.config.ts
```

---

## ❓ 常见问题

<details>
<summary><strong>❓ 应用没有把任何文件直接写到我的项目，为什么？</strong></summary>

所有写入操作都需要先 **预览 + 确认**。在预览界面选择 Apply 之后才会真正写入，且会先创建备份。

</details>

<details>
<summary><strong>❓ 升级版本后数据会不会丢？</strong></summary>

不会。数据库通过 `infrastructure/migrations/` 下的迁移自动升级，旧版本数据会按顺序迁移到新结构。

</details>

<details>
<summary><strong>❓ 我可以同时管理多个项目和多个工具吗？</strong></summary>

可以。规则可以绑定到 **单个项目** 或 **整个工具** 的全局上下文，互不干扰。

</details>

<details>
<summary><strong>❓ Token 存在哪里？</strong></summary>

通过系统凭据管理器（Windows Credential Manager / macOS Keychain 等）存储，数据库里只保留对凭据的引用。

</details>

<details>
<summary><strong>❓ 想完全重置应用怎么办？</strong></summary>

进入 **Settings → 维护**，使用 **重置应用数据** 功能。该操作不可逆，请先导出诊断信息或备份重要内容。

</details>

---

## 🤝 开发与贡献

提交代码前请运行：

```bash
pnpm typecheck                 # ✅ TypeScript 类型检查
pnpm test                      # 🔬 前端单元测试
cd src-tauri && cargo test     # 🦀 Rust 后端测试
```

- 🦀 后端 Rust 代码遵循 `cargo fmt` / `cargo clippy` 默认风格
- 🎨 前端代码使用项目根目录下的 `.prettierrc` 进行格式化

欢迎在仓库中提 Issue 或 Pull Request 🙌

---

<div align="center">

Made with ❤️ for AI coding workflows

</div>
