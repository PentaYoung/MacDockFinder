# MacDockFinder

Rust + Tauri v2 + Svelte 5 + Vite 6 + TypeScript — macOS 风格 Dock for Windows 11。

## 架构

- **Rust 后端** (`src-tauri/src/`): 配置存储 (JSON)、IPC 命令 (`commands/dock|settings|tray|menubar.rs`)、系统托盘、窗口监控 (Windows-only EnumWindows + SetWinEventHook)
- **Svelte 前端** (`src/lib/`): DockBar/DockItem/TrashBin + MenuBar + 系统 Widget + SearchOverlay/NotificationPanel；stores (`dock.ts`, `settings.ts`, `menubar.ts`)
- **窗口**: decorationless, transparent, alwaysOnTop, skipTaskbar; 100px 贴底; 关闭 → hide + prevent_close
- **快捷键**: `Alt+Space` 切换 Dock, `Ctrl+Space` 切换搜索

## 构建

### 系统依赖

```bash
# Linux 开发
sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev \
  libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Windows 交叉编译 (WSL/Linux)
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu
```

### 构建步骤

```bash
npm install                      # 安装前端依赖
bash scripts/build-windows.sh    # 一键构建 (前端 + Rust 交叉编译)
# 等价于:
# npm run build                  # 1. 构建前端
# npx tauri build --no-bundle --target x86_64-pc-windows-gnu  # 2. 编译 Rust
```

输出: `src-tauri/target/x86_64-pc-windows-gnu/release/macdockfinder.exe`

## 关键陷阱

- **不要用 `cargo build`**: 不会嵌入前端文件，EXE 启动会连 `localhost:1420` 报错。必须用 `npx tauri build`
- **默认图标双重保障**: `seed_defaults()` 在 `config/store.rs::ConfigStore::new()` 和 `commands/dock.rs::get_pinned_items` 各有一处，空列表时各自写入

## Svelte 5 约定

- 组件用 **runes** (`$state`, `$derived`, `$effect`, `$props()`)；store 文件用 **svelte/store** (`writable`, `derived`)
- **事件回调 prop 避免原生事件名**。用 `onDragStart` 而非 `ondragstart`，子组件内显式绑定 `ondragstart={onDragStart}` (见 `DockItem.svelte:149`)

## 命令

```bash
npm run dev                     # Vite dev server (port 1420)
cd src-tauri && cargo check     # 仅检查 Rust 编译
```
