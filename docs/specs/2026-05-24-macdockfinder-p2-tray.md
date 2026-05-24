# P2 系统托盘 & 窗口监控 & 全局快捷键

## 概述

为 MacDockFinder 添加系统托盘、Windows 窗口事件监控和全局快捷键功能。用户可以通过系统托盘图标控制应用，Dock 栏实时显示运行中的应用，快捷键快速切换。

## 功能需求

### 系统托盘
- 右键菜单：显示/隐藏 Dock、设置、退出
- 左键单击：切换 Dock 可见性
- 托盘图标使用现有 `icons/icon.ico`
- 应用关闭时最小化到托盘而非退出

### 窗口监控
- 枚举当前所有可见窗口，初始化运行应用列表
- 实时监听窗口创建 (`EVENT_OBJECT_CREATE`)、销毁 (`EVENT_OBJECT_DESTROY`)、焦点变化 (`EVENT_SYSTEM_FOREGROUND`)
- 通过 Tauri event 推送到前端，更新 `activeWindows` store
- 仅 Windows 实现，使用 `SetWinEventHook` + 后台消息线程
- Linux 编译时使用空实现（已有 cfg 门控）

### 全局快捷键
- 注册 `Alt+Space`，切换 Dock 显示/隐藏
- 使用已有依赖 `tauri-plugin-global-shortcut`

## 架构

```
Rust 后端
├── commands/
│   ├── dock.rs            ← 已有
│   ├── settings.rs        ← 已有
│   └── tray.rs            ← 新增：托盘 IPC 命令
├── monitor/
│   ├── mod.rs             ← 已有
│   ├── process.rs         ← 实现 enumerate_windows()
│   └── events.rs          ← 实现 SetWinEventHook 监听
├── tray.rs                ← 新增：系统托盘初始化和事件处理
├── config/store.rs        ← 已有（不变）
└── lib.rs                 ← 修改：注册托盘、快捷键、事件监听

Svelte 前端
└── lib/stores/dock.ts     ← 修改：监听 Tauri events → 更新 activeWindows
```

## 数据流

```
Windows 窗口事件 → SetWinEventHook → Rust 回调
  → app.emit("window-created"/"window-destroyed"/"foreground-changed")
  → 前端 stores/dock.ts 监听事件 → 更新 activeWindows → DockBar 渲染

用户点击托盘 → TrayIcon 事件 → 切换 Dock 可见性 / 打开设置 / 退出

快捷键 Alt+Space → tauri-plugin-global-shortcut → 切换 Dock 可见性
```

## Rust 依赖变化

`Cargo.toml` 的 `windows` crate 补充 feature:
```toml
"Win32_UI_Accessibility"   # SetWinEventHook
```

## 前端的修改

`stores/dock.ts`：
- 新增 Tauri event 监听 (`listen`)
- 监听 `window-created` / `window-destroyed` / `foreground-changed`
- 更新 `activeWindows` store

`stores/settings.ts`：
- 新增是否最小化到托盘的设置项

无新组件创建，使用现有 DockBar 渲染运行中的应用列表。

## 不包含（延期）

- 窗口预览缩略图（鼠标悬停截图）— 延期到 P3+
- 可配置快捷键 — 暂用固定 `Alt+Space`
