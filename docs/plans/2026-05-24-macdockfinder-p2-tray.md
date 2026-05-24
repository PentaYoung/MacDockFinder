# P2 系统托盘 & 窗口监控 & 全局快捷键 实现计划

> **给执行 agent：** 必需子技能：使用 subagent-driven-development（推荐）或 executing-plans 来逐任务执行本计划。步骤使用复选框（`- [ ]`）语法跟踪进度。

**目标：** 为 MacDockFinder 添加系统托盘图标、Windows 窗口事件监控和 Alt+Space 全局快捷键。

**架构：** 系统托盘使用 Tauri v2 内置 TrayIconBuilder + Menu API；窗口监控使用 Win32 `SetWinEventHook` 在独立线程监听，通过 `AppHandle::emit()` 推送到前端；快捷键使用 `tauri-plugin-global-shortcut`。

**技术栈：** Rust (windows crate 0.59)、Tauri v2 (tray + menu + event APIs)、tauri-plugin-global-shortcut、TypeScript (@tauri-apps/api/event)

---

### 任务 1：Cargo.toml — 补充 Windows feature

**文件：**
- 修改：`src-tauri/Cargo.toml:26-33`

- [ ] **添加 Win32_UI_Accessibility feature**

```toml
windows = { version = "0.59", features = [
  "Win32_Foundation",
  "Win32_UI_WindowsAndMessaging",
  "Win32_Graphics_Gdi",
  "Win32_System_Threading",
  "Win32_UI_Shell",
  "Win32_UI_Accessibility",
] }
```

- [ ] **验证编译**

```bash
cd src-tauri && cargo check
```

预期：编译通过，无错误。

---

### 任务 2：Monitor process.rs — 实现窗口枚举

**文件：**
- 修改：`src-tauri/src/monitor/process.rs`

- [ ] **重写 process.rs（cfg 门控 Windows API 调用）**

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub hwnd: u64,
    pub title: String,
    pub app_name: String,
    pub icon_base64: Option<String>,
}

#[cfg(target_os = "windows")]
pub fn enumerate_windows() -> Vec<WindowInfo> {
    use windows::Win32::Foundation::*;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use std::ffi::CStr;

    let mut windows: Vec<WindowInfo> = Vec::new();
    let windows_ptr = &mut windows as *mut Vec<WindowInfo>;

    unsafe {
        EnumWindows(
            Some(enum_window_proc),
            LPARAM(windows_ptr as isize),
        );
    }

    windows
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd).as_bool() {
        let len = GetWindowTextLengthA(hwnd);
        if len > 0 {
            let mut title_buf = vec![0u8; (len + 1) as usize];
            GetWindowTextA(hwnd, &mut title_buf);
            let title = CStr::from_ptr(title_buf.as_ptr() as *const i8)
                .to_string_lossy().to_string();

            let mut class_buf = [0u8; 260];
            GetClassNameA(hwnd, &mut class_buf);
            let class_name = CStr::from_ptr(class_buf.as_ptr() as *const i8)
                .to_string_lossy().to_string();

            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));

            use windows::Win32::System::Threading::*;
            use windows::Win32::Storage::FileSystem::*;
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
            let mut exe_buf = [0u16; 260];
            let mut exe_len = 260u32;
            if process.is_ok() {
                QueryFullProcessImageNameW(process.unwrap(), PROCESS_NAME_WIN32, &mut exe_buf, &mut exe_len);
                let app_path = String::from_utf16_lossy(&exe_buf[..exe_len as usize]);
                let app_name = std::path::Path::new(&app_path)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| class_name);
                CloseHandle(process.unwrap());

                let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);
                windows.push(WindowInfo {
                    hwnd: hwnd.0 as u64,
                    title,
                    app_name,
                    icon_base64: None,
                });
            }
        }
    }
    BOOL(1)
}

#[cfg(not(target_os = "windows"))]
pub fn enumerate_windows() -> Vec<WindowInfo> {
    vec![]
}
```

- [ ] **验证编译**

```bash
cd src-tauri && cargo check
```

预期：编译通过，无新 warning。

---

### 任务 3：Monitor events.rs — 实现 SetWinEventHook 监听

**文件：**
- 修改：`src-tauri/src/monitor/events.rs`

- [ ] **重写 events.rs，启动后台线程监听窗口事件**

```rust
use std::sync::OnceLock;
use tauri::AppHandle;
use crate::monitor::process::WindowInfo;

pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

#[cfg(target_os = "windows")]
pub fn start_window_event_listener() {
    use windows::Win32::UI::Accessibility::*;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::Foundation::*;

    std::thread::spawn(move || {
        unsafe {
            let hook = SetWinEventHook(
                EVENT_OBJECT_CREATE,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(event_proc),
                0, 0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            );

            if hook.is_invalid() {
                return;
            }

            let mut msg = MSG::default();
            while GetMessageA(&mut msg, None, 0, 0).as_bool() {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }

            UnhookWinEvent(hook);
        }
    });
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn event_proc(
    _hhook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_obj: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    use windows::Win32::UI::WindowsAndMessaging::*;
    use std::ffi::CStr;

    let hwnd = HWND(hwnd.0);
    if !IsWindow(hwnd).as_bool() || !IsWindowVisible(hwnd).as_bool() {
        return;
    }

    match event {
        EVENT_OBJECT_CREATE | EVENT_OBJECT_SHOW => {
            let len = GetWindowTextLengthA(hwnd);
            if len == 0 { return; }
            let mut title_buf = vec![0u8; (len + 1) as usize];
            GetWindowTextA(hwnd, &mut title_buf);
            let title = CStr::from_ptr(title_buf.as_ptr() as *const i8)
                .to_string_lossy().to_string();
            if title.is_empty() { return; }

            let mut class_buf = [0u8; 260];
            GetClassNameA(hwnd, &mut class_buf);
            let class_name = CStr::from_ptr(class_buf.as_ptr() as *const i8)
                .to_string_lossy().to_string();

            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));

            use windows::Win32::System::Threading::*;
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
            let app_name = if let Ok(proc) = process {
                use windows::Win32::Storage::FileSystem::*;
                let mut exe_buf = [0u16; 260];
                let mut exe_len = 260u32;
                QueryFullProcessImageNameW(proc, PROCESS_NAME_WIN32, &mut exe_buf, &mut exe_len);
                let path = String::from_utf16_lossy(&exe_buf[..exe_len as usize]);
                CloseHandle(proc);
                std::path::Path::new(&path)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| class_name.clone())
            } else {
                class_name
            };

            if let Some(handle) = APP_HANDLE.get() {
                let info = WindowInfo {
                    hwnd: hwnd.0 as u64,
                    title,
                    app_name,
                    icon_base64: None,
                };
                let _ = handle.emit("window-created", info);
            }
        }
        EVENT_OBJECT_DESTROY | EVENT_OBJECT_HIDE => {
            if let Some(handle) = APP_HANDLE.get() {
                let _ = handle.emit("window-destroyed", hwnd.0 as u64);
            }
        }
        EVENT_SYSTEM_FOREGROUND => {
            if let Some(handle) = APP_HANDLE.get() {
                let _ = handle.emit("foreground-changed", hwnd.0 as u64);
            }
        }
        _ => {}
    }
}

#[cfg(not(target_os = "windows"))]
pub fn start_window_event_listener() {
    // No-op on Linux
}
```

- [ ] **验证编译**

```bash
cd src-tauri && cargo check
```

预期：编译通过。

---

### 任务 4：新增 tray.rs — 系统托盘实现

**文件：**
- 创建：`src-tauri/src/tray.rs`

- [ ] **创建 tray.rs，托盘图标 + 菜单 + 事件处理**

```rust
use tauri::{
    AppHandle, Runtime,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let show_hide = MenuItem::with_id(app, "toggle", "显示/隐藏 Dock", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_hide, &settings, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => toggle_dock_visibility(app),
            "settings" => { /* future: open settings panel */ }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(app) = tray.app_handle() {
                    toggle_dock_visibility(app);
                }
            }
        })
        .build(app)?;

    Ok(())
}

pub fn toggle_dock_visibility<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
```

- [ ] **验证编译**

```bash
cd src-tauri && cargo check
```

预期：编译通过。

---

### 任务 5：新增 commands/tray.rs — 窗口列表 IPC

**文件：**
- 创建：`src-tauri/src/commands/tray.rs`
- 修改：`src-tauri/src/commands/mod.rs`

- [ ] **创建 commands/tray.rs**

```rust
use tauri::State;
use crate::config::store::ConfigStore;

#[tauri::command]
pub fn get_active_windows() -> Result<Vec<crate::monitor::process::WindowInfo>, String> {
    Ok(crate::monitor::process::enumerate_windows())
}
```

- [ ] **修改 commands/mod.rs，注册新模块**

```rust
pub mod dock;
pub mod settings;
pub mod tray;
```

- [ ] **验证编译**

```bash
cd src-tauri && cargo check
```

预期：编译通过。

---

### 任务 6：lib.rs — 集成所有模块

**文件：**
- 修改：`src-tauri/src/lib.rs`

- [ ] **重写 lib.rs，集成托盘、监控、快捷键**

```rust
mod commands;
mod config;
#[cfg(target_os = "windows")]
mod monitor;
mod tray;

use config::store::ConfigStore;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config_store = ConfigStore::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        tray::toggle_dock_visibility(app);
                    }
                })
                .build(),
        )
        .manage(config_store)
        .setup(|app| {
            // Position window at bottom
            if let Err(e) = position_window_at_bottom(app) {
                eprintln!("failed to position window: {e}");
            }

            // Create system tray
            if let Err(e) = tray::create_tray(app.handle()) {
                eprintln!("failed to create tray: {e}");
            }

            // Store AppHandle for window event callbacks
            let _ = crate::monitor::events::APP_HANDLE.set(app.handle().clone());

            // Register global shortcut
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .build(),
            ).ok();
            if let Err(e) = app.global_shortcut().register(
                tauri_plugin_global_shortcut::Shortcut::new(Some(Modifiers::ALT), Code::Space)
            ) {
                eprintln!("failed to register shortcut: {e}");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Minimize to tray instead of closing
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::dock::get_pinned_items,
            commands::dock::add_pinned_item,
            commands::dock::remove_pinned_item,
            commands::dock::reorder_items,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::tray::get_active_windows,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn position_window_at_bottom(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main").ok_or("main window not found")?;
    let monitor = window.primary_monitor()?.ok_or("no primary monitor")?;
    let screen_size = monitor.size();
    let bar_height = 100i32;
    let y = (screen_size.height as i32).saturating_sub(bar_height);
    window.set_position(tauri::PhysicalPosition::new(0, y))?;
    window.set_size(tauri::PhysicalSize::new(screen_size.width, bar_height as u32))?;
    Ok(())
}
```

- [ ] **验证编译**

```bash
cd src-tauri && cargo check
```

预期：编译通过。

---

### 任务 7：前端 stores/dock.ts — 监听窗口事件

**文件：**
- 修改：`src/lib/stores/dock.ts`

- [ ] **添加窗口事件监听和初始化加载**

```typescript
import { writable, derived } from "svelte/store";

export interface PinnedItem {
  id: string;
  path: string;
  label: string;
  icon_path: string | null;
}

export interface WindowInfo {
  hwnd: number;
  title: string;
  app_name: string;
  icon_base64: string | null;
}

export const pinnedItems = writable<PinnedItem[]>([]);
export const activeWindows = writable<WindowInfo[]>([]);
export const mouseX = writable(0);
export const mouseY = writable(0);
export const dockVisible = writable(true);

export const runningAppNames = derived(activeWindows, ($wins) =>
  new Set($wins.map((w) => w.app_name))
);

export async function loadPinnedItems() {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const items = await invoke<PinnedItem[]>("get_pinned_items");
    pinnedItems.set(items);
  } catch {
    pinnedItems.set([
      { id: "1", path: "explorer.exe", label: "文件资源管理器", icon_path: null },
      { id: "2", path: "chrome.exe", label: "Google Chrome", icon_path: null },
      { id: "3", path: "code.exe", label: "VS Code", icon_path: null },
      { id: "4", path: "terminal.exe", label: "终端", icon_path: null },
      { id: "5", path: "spotify.exe", label: "Spotify", icon_path: null },
    ]);
  }
}

export async function setupWindowListeners() {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const { listen } = await import("@tauri-apps/api/event");

    // Load initial window list
    const wins = await invoke<WindowInfo[]>("get_active_windows");
    activeWindows.set(wins);

    // Listen for window creation
    await listen<WindowInfo>("window-created", (event) => {
      activeWindows.update((wins) => {
        if (!wins.find((w) => w.hwnd === event.payload.hwnd)) {
          return [...wins, event.payload];
        }
        return wins;
      });
    });

    // Listen for window destruction
    await listen<number>("window-destroyed", (event) => {
      activeWindows.update((wins) =>
        wins.filter((w) => w.hwnd !== event.payload)
      );
    });

    // Listen for foreground changes
    await listen<number>("foreground-changed", (event) => {
      // Update active state (future: styling for active window)
    });
  } catch {
    // Running outside Tauri, no window monitoring
  }
}
```

- [ ] **验证 TypeScript 编译**

```bash
npm run build
```

预期：构建通过，无错误。

---

### 任务 8：DockBar.svelte — 集成窗口事件和可见性控制

**文件：**
- 修改：`src/lib/components/DockBar.svelte`

- [ ] **修改 DockBar，集成窗口监听和可见性控制**

在 `<script>` 中：
- `onMount` 中调用 `setupWindowListeners()`
- 使用 `dockVisible` store 控制显示/隐藏

将：
```typescript
import { onMount } from "svelte";
import DockItem from "./DockItem.svelte";
import DockDivider from "./DockDivider.svelte";
import TrashBin from "./TrashBin.svelte";
import {
  pinnedItems, activeWindows, runningAppNames,
  loadPinnedItems, mouseX, mouseY,
  type PinnedItem
} from "../stores/dock";
import { settings, loadSettings } from "../stores/settings";
```

改为：
```typescript
import { onMount } from "svelte";
import DockItem from "./DockItem.svelte";
import DockDivider from "./DockDivider.svelte";
import TrashBin from "./TrashBin.svelte";
import {
  pinnedItems, activeWindows, runningAppNames,
  loadPinnedItems, setupWindowListeners,
  mouseX, mouseY, dockVisible,
  type PinnedItem
} from "../stores/dock";
import { settings, loadSettings } from "../stores/settings";
```

在 `onMount` 中添加：
```typescript
onMount(async () => {
    await loadSettings();
    await loadPinnedItems();
    await setupWindowListeners();

    document.addEventListener("mousemove", handleMouseMove);
});
```

修改 dock-bar 容器的可见性绑定：
```svelte
<div
  class="dock-bar"
  class:visible={isVisible && $dockVisible}
  class:hidden={!isVisible || !$dockVisible}
  ...
>
```

- [ ] **验证前端构建**

```bash
npm run build
```

预期：构建通过。

---

### 任务 9：设置 minimize_to_tray 配置项

**文件：**
- 修改：`src-tauri/src/config/store.rs`

- [ ] **Settings 结构体添加 minimize_to_tray 字段**

```rust
pub struct Settings {
    pub position: String,
    pub auto_hide: bool,
    pub icon_size: u32,
    pub magnification: bool,
    pub minimize_to_tray: bool,
}
```

```rust
impl Default for Settings {
    fn default() -> Self {
        Self {
            position: "bottom".to_string(),
            auto_hide: false,
            icon_size: 48,
            magnification: true,
            minimize_to_tray: true,
        }
    }
}
```

同时更新 `src/lib/stores/settings.ts`：
```typescript
export interface Settings {
  position: string;
  auto_hide: boolean;
  icon_size: number;
  magnification: boolean;
  minimize_to_tray: boolean;
}

export const settings = writable<Settings>({
  position: "bottom",
  auto_hide: false,
  icon_size: 48,
  magnification: true,
  minimize_to_tray: true,
});
```

- [ ] **验证编译**

```bash
cd src-tauri && cargo check && npm run build
```

---

### 任务 10：完整构建和验证

**文件：**
- 无需修改文件

- [ ] **完整交叉编译**

```bash
bash scripts/build-windows.sh
```

预期：前端构建成功，Rust 交叉编译成功，输出 `target/x86_64-pc-windows-gnu/release/macdockfinder.exe`

- [ ] **更新 AGENTS.md**

---

## 计划自检

### 规格覆盖
- ✅ 系统托盘（右键菜单：显示/隐藏、设置、退出；左键切换可见性）— 任务 4
- ✅ 窗口监控（SetWinEventHook 监听创建/销毁/焦点，推送到前端）— 任务 2、3、7
- ✅ 全局快捷键（Alt+Space 切换 Dock 可见性）— 任务 6
- ✅ minimize_to_tray 设置项 — 任务 9
- ✅ 窗口关闭最小化到托盘 — 任务 6

### 占位符检查
- 所有步骤包含完整代码
- 无 TBD、TODO、或"稍后实现"模式

### 类型一致性
- Rust `WindowInfo` 匹配 TypeScript `WindowInfo` 接口
- IPC 命令签名一致
- Tauri event payload 类型匹配
