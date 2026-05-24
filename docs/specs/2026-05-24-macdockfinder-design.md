# MacDockFinder — macOS 风格 Dock 栏 for Windows (Win11)

## 概述

在 Windows 11 上构建一个全功能 macOS 风格桌面美化套件，包含 Dock 栏、顶部菜单栏、Launchpad、系统监控等模块。采用 Rust + Tauri v2 + Svelte 技术栈，分 5 个阶段迭代开发。

---

## 第一阶段：Dock 栏（P1）

### 功能需求

- 屏幕底部 Dock 栏，支持多显示器识别
- 应用图标支持放大动画（鱼眼效果），60fps 平滑过渡
- 拖拽添加/排序应用图标
- 右键菜单：退出、选项、最近文档
- 自动隐藏/始终显示模式切换
- 垃圾桶图标（拖拽移除应用）
- 运行中的应用显示小圆点指示器
- 鼠标悬停显示窗口预览（缩略图浮窗）
- 分隔线区分固定应用和运行中的应用
- 设置面板入口

### 架构

```
Tauri 主进程 (Rust)
  ├── 进程/窗口枚举 (Windows API)
  ├── 配置管理 (JSON 文件)
  ├── 全局快捷键
  └── IPC 命令桥

Tauri WebView (Svelte)
  ├── DockBar (容器、布局、自动隐藏)
  ├── DockItem (图标、动画、拖拽)
  ├── DockDivider
  ├── DockPreview (窗口预览)
  ├── DockMenu (右键菜单)
  ├── TrashBin
  └── SettingsPanel
```

### Rust 后端模块

```
src-tauri/src/
├── main.rs
├── commands/
│   ├── mod.rs
│   ├── dock.rs       — get_pinned_items, add_pinned_item, remove_item, reorder
│   ├── windows.rs    — get_windows, focus_window, launch_app
│   └── settings.rs   — get_settings, update_settings
├── monitor/
│   ├── mod.rs
│   ├── process.rs    — Windows API 枚举进程/窗口
│   └── events.rs     — 监听窗口创建/关闭
├── config/
│   ├── mod.rs
│   └── store.rs      — JSON 读写 + 文件监听
└── state.rs          — Tauri 应用状态
```

### 关键 Rust 依赖

- `windows` / `windows-sys` — Windows API（Win11 SDK, build >= 22000）
- `serde` + `serde_json` — 配置序列化
- `tauri-plugin-shell` — 启动外部进程
- `tauri-plugin-global-shortcut` — 全局快捷键
- `tauri-plugin-fs` — 文件系统操作

### 数据流

```
Windows 事件 → Rust 监听 → IPC → Svelte stores → 组件更新
用户操作 → Svelte → IPC invoke → Rust 执行 → 结果回传
```

### 动画与视觉效果

- 鱼眼放大算法：以鼠标位置为中心，按距离递减缩放 (1.0x ~ 1.6x)，easeOutBack 缓动
- Dock 栏背景：Win11 Mica 材质 + backdrop-blur 模糊
- 圆角：Fluent Design 规范 (border-radius: 8px)
- 自动隐藏：300ms 延迟滑出/滑入，200ms ease-in/out 过渡

### 配置存储 (JSON)

```json
{
  "position": "bottom",
  "autoHide": true,
  "iconSize": 48,
  "magnification": true,
  "pinnedItems": [
    { "id": "uuid", "path": "C:\\...", "label": "应用名" }
  ]
}
```

### Win11 适配

- 检测 build >= 22000 启用 Mica 材质
- 使用 `SetWindowCompositionAttribute` 实现透明背景
- 圆角遵循 Win11 Fluent Design 规范
- 避开 Win11 右下角"显示桌面"冲突区域
- 可选替换/隐藏 Win11 原生任务栏

---

## 后续阶段规划

### P2：系统托盘 & 窗口管理
- 最小化到系统托盘
- 全局快捷键注册
- 监控窗口创建/关闭/焦点事件
- 提供窗口预览缩略图

### P3：顶部菜单栏
- 时间/日期显示
- WiFi、音量、电池状态（Win11 API）
- 通知中心集成
- 搜索功能（Spotlight 风格）

### P4：Launchpad
- 全屏应用网格
- 应用搜索
- 文件夹分组
- iPad 风格布局

### P5：系统监控 & 主题
- CPU/RAM/GPU 使用率
- 网络速度监控
- 动态壁纸
- 皮肤系统
- Steam Workshop 集成

---

## 开发约束

- 语言：Rust 后端 + TypeScript 前端
- 框架：Tauri v2 + Svelte 5 + Vite
- 目标平台：Windows 11 (build >= 22000)
- 开发环境：Linux（交叉编译至 Windows）
- 最小可行产品：P1 Dock 栏可运行
