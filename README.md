# VT Hub Manager

VT Hub Manager is a desktop workspace for managing AI coding tools, projects, rules, skills, presets, and generated `AGENTS.md` context.

VT Hub Manager 是一个面向 AI 编程工具的桌面工作台，用于统一管理项目、规则、技能、预设和生成的 `AGENTS.md` 工作上下文。

## 中文说明

### 项目介绍

VT Hub Manager 基于 Vue 3、Tauri 2 和 Rust 构建，适合用于本地管理 AI 编程工具的项目上下文。

主要能力：

- 管理本地项目，并生成或维护项目 `AGENTS.md`
- 管理 Rules，并将规则绑定到项目或工具
- 管理 Skills，沉淀可复用的能力说明
- 管理 Presets，维护工具配置和模型参数
- 查看操作历史、备份记录和诊断信息
- 支持中文 / English 双语界面切换

### 技术栈

- Vue 3 + TypeScript + Vite
- Pinia + Vue Router + Vue I18n
- Ant Design Vue
- Tauri 2 + Rust

### 环境要求

请先安装：

- Node.js 22 或更新版本
- pnpm 10 或更新版本
- Rust stable
- Tauri 2 所需系统依赖

安装 pnpm：

```bash
npm install -g pnpm
```

Tauri 环境要求参考：

```text
https://tauri.app/start/prerequisites/
```

### 快速入手

克隆项目：

```bash
git clone https://github.com/twj0415/vt-agent-hub.git
cd vt-agent-hub
```

安装依赖：

```bash
pnpm install
```

启动桌面开发模式：

```bash
pnpm tauri:dev
```

只启动前端开发服务：

```bash
pnpm dev
```

### 常用命令

```bash
pnpm dev          # 启动 Vite 开发服务
pnpm tauri:dev    # 启动 Tauri 桌面开发模式
pnpm build        # 类型检查并构建前端
pnpm typecheck    # TypeScript 类型检查
pnpm test         # 运行前端测试
pnpm tauri build  # 构建桌面应用安装包
```

### 基本使用流程

1. 打开应用，进入 Projects，创建或导入本地项目。
2. 进入 Rules，创建或导入项目规则。
3. 将规则绑定到项目或工具。
4. 在 Projects 中预览即将生成的 `AGENTS.md`。
5. 确认内容后应用到项目目录。
6. 在 Skills 和 Presets 中维护可复用能力和工具配置。
7. 在 History 中查看操作记录、备份和诊断结果。
8. 在 Settings 中切换语言、主题，并查看数据路径。

### 中英双语切换

应用内置中文和英文界面。

切换路径：

```text
Settings -> Appearance and Language -> Language
```

可选语言：

- 中文
- English

### 项目结构

```text
.
├── src/                 # 前端源码
│   ├── app/             # 应用入口、路由、主题和全局样式
│   ├── features/        # 跨页面功能模块
│   ├── pages/           # Projects、Rules、Skills、Presets、History、Settings
│   └── shared/          # API、组件、状态、工具函数和类型
├── src-tauri/           # Tauri / Rust 后端
│   ├── src/             # Rust 服务、命令、数据访问和工具适配器
│   ├── Cargo.toml       # Rust 依赖配置
│   └── tauri.conf.json  # Tauri 应用配置
├── package.json         # 前端依赖和脚本
├── pnpm-lock.yaml       # pnpm 锁文件
└── vite.config.ts       # Vite 配置
```

## English

### Overview

VT Hub Manager is a local desktop workspace for AI coding tools. It helps manage projects, rules, skills, presets, history, settings, and generated `AGENTS.md` context files.

### Features

- Manage local projects
- Generate and maintain project `AGENTS.md` files
- Create, import, and bind rules
- Manage reusable Skills
- Manage tool Presets and configuration assets
- Review history, backups, and diagnostics
- Switch between Chinese and English

### Tech Stack

- Vue 3 + TypeScript + Vite
- Pinia + Vue Router + Vue I18n
- Ant Design Vue
- Tauri 2 + Rust

### Requirements

Install these tools first:

- Node.js 22 or later
- pnpm 10 or later
- Rust stable
- System dependencies required by Tauri 2

Install pnpm:

```bash
npm install -g pnpm
```

Tauri prerequisites:

```text
https://tauri.app/start/prerequisites/
```

### Quick Start

Clone the repository:

```bash
git clone https://github.com/twj0415/vt-agent-hub.git
cd vt-agent-hub
```

Install dependencies:

```bash
pnpm install
```

Start the desktop app in development mode:

```bash
pnpm tauri:dev
```

Start only the frontend dev server:

```bash
pnpm dev
```

### Scripts

```bash
pnpm dev          # Start the Vite dev server
pnpm tauri:dev    # Start the Tauri desktop app in dev mode
pnpm build        # Type-check and build the frontend
pnpm typecheck    # Run TypeScript type checking
pnpm test         # Run frontend tests
pnpm tauri build  # Build desktop installers/packages
```

### Basic Workflow

1. Open Projects and create or import a local project.
2. Create or import rules in Rules.
3. Bind rules to projects or tools.
4. Preview the generated `AGENTS.md` content in Projects.
5. Apply the generated output after review.
6. Manage reusable Skills and tool Presets.
7. Check History for operations, backups, and diagnostics.
8. Use Settings to switch language, theme, and inspect storage paths.

### Language Switching

The app supports Chinese and English.

Switch language from:

```text
Settings -> Appearance and Language -> Language
```

Available languages:

- 中文
- English

### Project Structure

```text
.
├── src/                 # Frontend source code
│   ├── app/             # App bootstrap, router, theme, and global styles
│   ├── features/        # Cross-page feature modules
│   ├── pages/           # Projects, Rules, Skills, Presets, History, Settings
│   └── shared/          # API, components, stores, utilities, and types
├── src-tauri/           # Tauri / Rust backend
│   ├── src/             # Rust services, commands, repositories, and adapters
│   ├── Cargo.toml       # Rust dependencies
│   └── tauri.conf.json  # Tauri app configuration
├── package.json         # Frontend dependencies and scripts
├── pnpm-lock.yaml       # pnpm lockfile
└── vite.config.ts       # Vite configuration
```

