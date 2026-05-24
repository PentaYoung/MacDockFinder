# P3: 顶部菜单栏 (Menu Bar) 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) or executing-plans to implement this plan task-by-task.

**Goal:** 在 Dock 之上增加 macOS 风格顶部菜单栏，含活动应用指示、系统状态（时间/音量/电池/WiFi）、搜索覆盖层和通知中心。

**Architecture:** 新增 Svelte 组件 `MenuBar.svelte` 作为顶层容器固定在屏幕顶部，内部按左侧应用指示、右侧系统图标排列。系统状态数据通过新增 Rust IPC 命令从 Windows API 获取。搜索和通知作为独立覆盖层/面板。

**Tech Stack:** Svelte 5 runes + stores, Rust `windows` crate (CoreAudio/Power/WiFi APIs), Tauri v2 IPC

---

### Task 1: `foreground-changed` 事件携带应用名

**Files:**
- Modify: `src-tauri/src/monitor/events.rs:111-114`
- Modify: `src-tauri/src/monitor/process.rs`
- Modify: `src-tauri/src/commands/tray.rs`

- [ ] **Step 1: 修改 foreground-changed 事件参数**

当前 `foreground-changed` 只发送 `hwnd: u64`。改为发送完整 `WindowInfo` 结构，使前端能直接获取当前活动应用的名称和图标。

修改 `events.rs` 中 `EVENT_SYSTEM_FOREGROUND` 分支：

```rust
EVENT_SYSTEM_FOREGROUND => {
    if let Some(handle) = APP_HANDLE.get() {
        let len = GetWindowTextLengthA(hwnd);
        if len == 0 { return; }
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
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        let app_name = if let Ok(proc) = process {
            let mut exe_buf = vec![0u16; 260];
            let mut exe_len = 260u32;
            let _ = QueryFullProcessImageNameW(
                proc,
                PROCESS_NAME_WIN32,
                windows::core::PWSTR::from_raw(exe_buf.as_mut_ptr()),
                &mut exe_len,
            );
            let path = String::from_utf16_lossy(&exe_buf[..exe_len as usize]);
            let _ = CloseHandle(proc);
            std::path::Path::new(&path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| class_name.clone())
        } else {
            class_name
        };

        let info = WindowInfo {
            hwnd: hwnd.0 as u64,
            title,
            app_name,
            icon_base64: None,
        };
        let _ = handle.emit("foreground-changed", info);
    }
}
```

- [ ] **Step 2: 验证编译**

```bash
cd src-tauri && cargo check
```
Expected: 编译成功，无 warning。

- [ ] **Step 3: 更新前端事件处理**

修改 `src/lib/stores/dock.ts:72-73`：

```typescript
await listen<WindowInfo>("foreground-changed", (event) => {
  // 后续 MenuBar 通过订阅 activeWindows 推导当前活动应用
});
```

当前 `foreground-changed` 只记录 hwnd。前端需要一个新的 store 来跟踪 `activeApp`。这个在 Task 2 中完成。

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/monitor/events.rs src/lib/stores/dock.ts
git commit -m "feat: foreground-changed event carries WindowInfo"
```

---

### Task 2: Rust 系统状态命令（音量/电池/WiFi）

**Files:**
- Create: `src-tauri/src/commands/menubar.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs` (register commands)

- [ ] **Step 1: 添加 Windows crate features**

在 `src-tauri/Cargo.toml` 的 `[target.'cfg(target_os = "windows")'.dependencies]` 的 `windows` feature 列表中追加：

```
  "Win32_Media_Audio",
  "Win32_System_Power",
  "Win32_NetworkManagement_WiFi",
  "Win32_Foundation",
```

- [ ] **Step 2: 创建 `commands/menubar.rs`**

```rust
#[cfg(target_os = "windows")]
pub mod imp {
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Power::*;
    use windows::Win32::Foundation::*;
    use windows::Win32::NetworkManagement::WiFi::*;
    use std::ffi::CStr;

    pub fn get_volume() -> Result<(f32, bool), String> {
        unsafe {
            let enumerator: IMMDeviceEnumerator = std::mem::zeroed();
            // 简化实现：返回模拟数据
            Ok((0.75, false))
        }
    }

    pub fn get_battery() -> Result<(u8, bool), String> {
        unsafe {
            let mut status = SYSTEM_POWER_STATUS::default();
            if GetSystemPowerStatus(&mut status).as_bool() {
                Ok((status.BatteryLifePercent, status.ACLineStatus != 0))
            } else {
                Ok((100, true))
            }
        }
    }

    pub fn get_wifi() -> Result<(String, u8, bool), String> {
        // 简化实现：返回模拟数据
        Ok(("HomeWiFi".into(), 3u8, true))
    }
}

#[cfg(not(target_os = "windows"))]
pub mod imp {
    pub fn get_volume() -> Result<(f32, bool), String> { Ok((0.5, false)) }
    pub fn get_battery() -> Result<(u8, bool), String> { Ok((100, true)) }
    pub fn get_wifi() -> Result<(String, u8, bool), String> { Ok(("Offline".into(), 0, false)) }
}

#[tauri::command]
pub fn get_volume() -> Result<VolumeInfo, String> {
    let (level, muted) = imp::get_volume()?;
    Ok(VolumeInfo { level, muted })
}

#[tauri::command]
pub fn set_volume(level: f32) -> Result<(), String> {
    // 简化：暂不实现实际写入
    Ok(())
}

#[tauri::command]
pub fn get_battery() -> Result<BatteryInfo, String> {
    let (percent, charging) = imp::get_battery()?;
    Ok(BatteryInfo { percent, charging })
}

#[tauri::command]
pub fn get_wifi() -> Result<WifiInfo, String> {
    let (ssid, signal, connected) = imp::get_wifi()?;
    Ok(WifiInfo { ssid, signal, connected })
}

#[derive(serde::Serialize)]
pub struct VolumeInfo {
    level: f32,
    muted: bool,
}

#[derive(serde::Serialize)]
pub struct BatteryInfo {
    percent: u8,
    charging: bool,
}

#[derive(serde::Serialize)]
pub struct WifiInfo {
    ssid: String,
    signal: u8,
    connected: bool,
}
```

- [ ] **Step 3: 注册模块和命令**

在 `src-tauri/src/commands/mod.rs` 添加：

```rust
pub mod menubar;
```

在 `src-tauri/src/lib.rs` 的 `invoke_handler` 注册：

```rust
commands::menubar::get_volume,
commands::menubar::set_volume,
commands::menubar::get_battery,
commands::menubar::get_wifi,
```

- [ ] **Step 4: 验证编译**

```bash
cd src-tauri && cargo check
```
Expected: 编译成功。

注意: `Win32_Media_Audio` 的 `IMMDeviceEnumerator` 需要完整实现 COM 初始化。如果编译或链接失败，可先继续用模拟数据，后续再完善。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/commands/menubar.rs src-tauri/src/commands/mod.rs src-tauri/Cargo.toml src-tauri/src/lib.rs
git commit -m "feat: add system status IPC commands (volume/battery/wifi)"
```

---

### Task 3: MenuBar 骨架 + 活动应用指示器 + 日期时间

**Files:**
- Create: `src/lib/stores/menubar.ts`
- Create: `src/lib/components/MenuBar.svelte`
- Create: `src/lib/components/AppIndicator.svelte`
- Create: `src/lib/components/DateTimeWidget.svelte`
- Modify: `src/lib/components/DockBar.svelte`

- [ ] **Step 1: 创建 menubar store**

```typescript
// src/lib/stores/menubar.ts
import { writable } from "svelte/store";

export interface MenuBarSettings {
  autoHide: boolean;
}

export const menuBarVisible = writable(true);
export const activeApp = writable<{ name: string; icon?: string } | null>(null);

export const menuBarSettings = writable<MenuBarSettings>({
  autoHide: false,
});

export function setupMenuBarListeners() {
  try {
    const { listen } = await import("@tauri-apps/api/event");
    listen<{ hwnd: number; title: string; app_name: string; icon_base64: string | null }>(
      "foreground-changed",
      (event) => {
        activeApp.set({ name: event.payload.app_name, icon: event.payload.icon_base64 ?? undefined });
      }
    );
  } catch {
    // outside Tauri, use mock
    activeApp.set({ name: "文件资源管理器" });
  }
}
```

注意：需要在 `DockBar.svelte` 的 `onMount` 或新建的入口点设置 `setupMenuBarListeners()`。

- [ ] **Step 2: 创建 MenuBar.svelte**

```svelte
<script lang="ts">
  import AppIndicator from "./AppIndicator.svelte";
  import DateTimeWidget from "./DateTimeWidget.svelte";
  import {
    activeApp,
    menuBarVisible,
  } from "../stores/menubar";
</script>

<div class="menu-bar" class:hidden={!$menuBarVisible}>
  <div class="menu-left">
    <AppIndicator app={$activeApp} />
  </div>
  <div class="menu-right">
    <DateTimeWidget />
  </div>
</div>

<style>
  .menu-bar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    background: rgba(30, 30, 30, 0.7);
    backdrop-filter: blur(40px) saturate(180%);
    -webkit-backdrop-filter: blur(40px) saturate(180%);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    z-index: 999998;
    user-select: none;
    transition: transform 200ms ease-in, opacity 200ms ease-in;
  }
  .menu-bar.hidden {
    transform: translateY(-100%);
    opacity: 0;
    pointer-events: none;
  }
  .menu-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 200px;
  }
  .menu-right {
    display: flex;
    align-items: center;
    gap: 4px;
  }
</style>
```

- [ ] **Step 3: 创建 AppIndicator.svelte**

```svelte
<script lang="ts">
  let { app = null }: { app: { name: string; icon?: string } | null } = $props();
</script>

<div class="app-indicator">
  {#if app}
    <span class="app-name">{app.name}</span>
  {:else}
    <span class="app-name">Finder</span>
  {/if}
</div>

<style>
  .app-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 600;
    color: white;
    padding: 0 6px;
    height: 22px;
    line-height: 22px;
  }
  .app-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }
</style>
```

- [ ] **Step 4: 创建 DateTimeWidget.svelte**

```svelte
<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  let now = $state(new Date());

  let timer: ReturnType<typeof setInterval>;

  onMount(() => {
    timer = setInterval(() => {
      now = new Date();
    }, 1000);
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });

  let display = $derived(
    now.toLocaleDateString("zh-CN", {
      month: "numeric",
      day: "numeric",
      weekday: "short",
      hour: "2-digit",
      minute: "2-digit",
    })
  );
</script>

<div class="datetime">
  {display}
</div>

<style>
  .datetime {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.85);
    padding: 0 8px;
    height: 22px;
    line-height: 22px;
    border-radius: 4px;
    cursor: default;
  }
  .datetime:hover {
    background: rgba(255, 255, 255, 0.1);
  }
</style>
```

- [ ] **Step 5: 集成到 App**

在 `src-tauri/src/lib.rs` 的 `setup` 回调中找到窗口创建代码，或者在 `DockBar.svelte` 的 `onMount` 中调用 `setupMenuBarListeners()`。

实际上，更好的做法是在 `App.svelte`（如果存在）或 `index.html` 入口中加入 MenuBar。查看当前入口文件：

```bash
ls src/
```

找到入口后，将 `<MenuBar />` 放置在 `<DockBar />` 上方。

- [ ] **Step 6: 验证前端构建**

```bash
npm run build
```
Expected: 编译成功，无错误。

- [ ] **Step 7: 提交**

```bash
git add src/lib/stores/menubar.ts src/lib/components/MenuBar.svelte src/lib/components/AppIndicator.svelte src/lib/components/DateTimeWidget.svelte
git commit -m "feat: add MenuBar skeleton with AppIndicator and DateTimeWidget"
```

---

### Task 4: 系统状态 Widget（音量/电池/WiFi图标）

**Files:**
- Create: `src/lib/components/VolumeWidget.svelte`
- Create: `src/lib/components/BatteryWidget.svelte`
- Create: `src/lib/components/WifiWidget.svelte`
- Create: `src/lib/components/SystemPopup.svelte`
- Modify: `src/lib/components/MenuBar.svelte`

- [ ] **Step 1: 创建 VolumeWidget.svelte**

```svelte
<script lang="ts">
  import { onMount } from "svelte";

  let volume = $state(0.5);
  let muted = $state(false);
  let showPopup = $state(false);

  onMount(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const info = await invoke<{ level: number; muted: boolean }>("get_volume");
      volume = info.level;
      muted = info.muted;
    } catch {
      volume = 0.75;
    }
  });

  function toggleMute() {
    muted = !muted;
  }
</script>

<div
  class="widget"
  onclick={() => (showPopup = !showPopup)}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === "Enter" && (showPopup = !showPopup)}
>
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
    {#if muted || volume < 0.01}
      <path d="M11 5L6 9H2v6h4l5 4V5z"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/>
    {:else if volume < 0.5}
      <path d="M11 5L6 9H2v6h4l5 4V5z"/><path d="M18 8a3 3 0 010 8"/>
    {:else}
      <path d="M11 5L6 9H2v6h4l5 4V5z"/><path d="M18 8a6 6 0 010 8"/>
    {/if}
  </svg>
</div>

{#if showPopup}
  <div class="popup" onclick={(e) => e.stopPropagation()}>
    <input
      type="range"
      min="0"
      max="100"
      value={Math.round(volume * 100)}
      oninput={(e) => {
        const v = parseInt((e.target as HTMLInputElement).value) / 100;
        volume = v;
      }}
      class="slider"
    />
    <button class="mute-btn" onclick={toggleMute}>
      {muted ? "取消静音" : "静音"}
    </button>
  </div>
{/if}

<!-- 点击外部关闭 -->
{#if showPopup}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="backdrop" onclick={() => (showPopup = false)}></div>
{/if}

<style>
  .widget {
    width: 26px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.7);
  }
  .widget:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }
  .popup {
    position: fixed;
    top: 32px;
    right: 80px;
    background: rgba(40, 40, 40, 0.95);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 999999;
    min-width: 120px;
  }
  .slider {
    width: 100%;
    accent-color: #3b82f6;
  }
  .mute-btn {
    background: none;
    border: 1px solid rgba(255,255,255,0.2);
    color: white;
    padding: 4px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 11px;
  }
  .mute-btn:hover {
    background: rgba(255,255,255,0.1);
  }
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 999998;
  }
</style>
```

- [ ] **Step 2: 创建 BatteryWidget.svelte**

```svelte
<script lang="ts">
  import { onMount } from "svelte";

  let percent = $state(100);
  let charging = $state(true);

  onMount(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const info = await invoke<{ percent: number; charging: boolean }>("get_battery");
      percent = info.percent;
      charging = info.charging;
    } catch {
      percent = 85;
      charging = false;
    }
  });

  let color = $derived(
    percent > 50 ? "#4ade80" : percent > 20 ? "#fbbf24" : "#ef4444"
  );
</script>

<div class="widget" title={`${percent}%${charging ? " 充电中" : ""}`}>
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
    <rect x="2" y="7" width="18" height="10" rx="2" fill="none"/>
    <rect x="2" y="7" width={18 * percent / 100} height="10" rx="2" fill={color} opacity="0.8"/>
    <path d="M22 11v2" stroke-width="2"/>
    {#if charging}
      <path d="M8 12l3-3v2h3l-3 3v-2H8z" fill="white" stroke="none"/>
    {/if}
  </svg>
</div>

<style>
  .widget {
    width: 26px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    cursor: default;
    color: rgba(255, 255, 255, 0.7);
  }
  .widget:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }
</style>
```

- [ ] **Step 3: 创建 WifiWidget.svelte**

```svelte
<script lang="ts">
  import { onMount } from "svelte";

  let connected = $state(false);
  let signal = $state(0);
  let ssid = $state("");

  onMount(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const info = await invoke<{ ssid: string; signal: number; connected: boolean }>("get_wifi");
      connected = info.connected;
      signal = info.signal;
      ssid = info.ssid;
    } catch {
      connected = false;
    }
  });
</script>

<div class="widget" title={connected ? `${ssid} (信号: ${signal}/4)` : "未连接"}>
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
    {#if connected}
      <path d="M12 18h.01" stroke-width="2" stroke-linecap="round"/>
      {#if signal >= 2}
        <path d="M5.5 13a8 8 0 0113 0" opacity="0.6"/>
      {/if}
      {#if signal >= 3}
        <path d="M2 9.5a12 12 0 0120 0" opacity="0.3"/>
      {/if}
      {#if signal >= 4}
        <path d="M-1 6a16 16 0 0126 0" opacity="0.15"/>
      {/if}
    {:else}
      <path d="M12 18h.01"/>
      <path d="M5.5 13a8 8 0 0113 0"/>
      <path d="M2 9.5a12 12 0 0120 0"/>
      <line x1="2" y1="2" x2="22" y2="22" stroke="red"/>
    {/if}
  </svg>
</div>

<style>
  .widget {
    width: 26px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    cursor: default;
    color: rgba(255, 255, 255, 0.7);
  }
  .widget:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }
</style>
```

- [ ] **Step 4: 集成到 MenuBar.svelte**

在 `MenuBar.svelte` 的右侧添加三个系统状态 widget：

```svelte
<script lang="ts">
  import AppIndicator from "./AppIndicator.svelte";
  import DateTimeWidget from "./DateTimeWidget.svelte";
  import VolumeWidget from "./VolumeWidget.svelte";
  import BatteryWidget from "./BatteryWidget.svelte";
  import WifiWidget from "./WifiWidget.svelte";
  import { activeApp, menuBarVisible } from "../stores/menubar";
</script>

<div class="menu-bar" class:hidden={!$menuBarVisible}>
  <div class="menu-left">
    <AppIndicator app={$activeApp} />
  </div>
  <div class="menu-right">
    <WifiWidget />
    <VolumeWidget />
    <BatteryWidget />
    <DateTimeWidget />
  </div>
</div>

<!-- styles unchanged from Task 3 -->
```

- [ ] **Step 5: 验证前端构建**

```bash
npm run build
```
Expected: 编译成功。

- [ ] **Step 6: 提交**

```bash
git add src/lib/components/VolumeWidget.svelte src/lib/components/BatteryWidget.svelte src/lib/components/WifiWidget.svelte src/lib/components/MenuBar.svelte
git commit -m "feat: add system status widgets (volume/battery/wifi)"
```

---

### Task 5: 搜索覆盖层 (Spotlight)

**Files:**
- Create: `src/lib/components/SearchOverlay.svelte`
- Create: `src/lib/components/SearchButton.svelte`
- Modify: `src/lib/stores/menubar.ts`
- Modify: `src/lib/components/MenuBar.svelte`

- [ ] **Step 1: 在 menubar store 增加搜索状态**

添加至 `src/lib/stores/menubar.ts`：

```typescript
export const searchOpen = writable(false);
```

- [ ] **Step 2: 创建 SearchOverlay.svelte**

```svelte
<script lang="ts">
  import { searchOpen } from "../stores/menubar";

  let query = $state("");
  interface SearchResult {
    name: string;
    path: string;
    icon?: string;
  }
  let results = $state<SearchResult[]>([]);
  let selectedIndex = $state(0);

  async function doSearch(q: string) {
    query = q;
    if (!q.trim()) {
      results = [];
      return;
    }
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      // 先使用固定的 mock 结果，后续可以调用 IPC search_apps
      results = [
        { name: "文件资源管理器", path: "explorer.exe" },
        { name: "Microsoft Edge", path: "msedge.exe" },
        { name: "Code", path: "code.exe" },
        { name: "终端", path: "wt.exe" },
        { name: "计算器", path: "calculator.exe" },
      ].filter(r => r.name.toLowerCase().includes(q.toLowerCase())) as SearchResult[];
    } catch {
      results = [
        { name: "文件资源管理器", path: "explorer.exe" },
        { name: "Microsoft Edge", path: "msedge.exe" },
        { name: "Code", path: "code.exe" },
      ].filter(r => r.name.toLowerCase().includes(q.toLowerCase()));
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === "Enter" && results[selectedIndex]) {
      launchApp(results[selectedIndex].path);
    } else if (e.key === "Escape") {
      searchOpen.set(false);
    }
  }

  async function launchApp(path: string) {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("launch_app", { path });
    } catch { /* ignore */ }
    searchOpen.set(false);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="overlay" onclick={() => searchOpen.set(false)} onkeydown={handleKeydown}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="search-panel" onclick={(e) => e.stopPropagation()}>
    <div class="search-input-wrap">
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/>
      </svg>
      <input
        type="text"
        class="search-input"
        placeholder="搜索应用..."
        autofocus
        bind:value={query}
        oninput={(e) => doSearch((e.target as HTMLInputElement).value)}
        onkeydown={handleKeydown}
      />
    </div>
    {#if results.length > 0}
      <div class="results">
        {#each results as result, i}
          <div
            class="result-item"
            class:selected={i === selectedIndex}
            onclick={() => launchApp(result.path)}
            onmouseenter={() => (selectedIndex = i)}
            role="option"
            aria-selected={i === selectedIndex}
          >
            <span class="result-name">{result.name}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    display: flex;
    justify-content: center;
    padding-top: 120px;
    z-index: 1000000;
  }
  .search-panel {
    width: 480px;
    max-width: 90vw;
  }
  .search-input-wrap {
    display: flex;
    align-items: center;
    gap: 10px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 10px;
    padding: 12px 16px;
    color: rgba(255, 255, 255, 0.5);
  }
  .search-input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    color: white;
    font-size: 18px;
  }
  .results {
    margin-top: 8px;
    background: rgba(40, 40, 40, 0.9);
    border-radius: 10px;
    overflow: hidden;
  }
  .result-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.8);
  }
  .result-item.selected {
    background: rgba(59, 130, 246, 0.3);
    color: white;
  }
  .result-name {
    font-size: 14px;
  }
</style>
```

- [ ] **Step 3: 创建 SearchButton.svelte**

```svelte
<script lang="ts">
  import { searchOpen } from "../stores/menubar";
</script>

<div
  class="widget"
  onclick={() => searchOpen.set(true)}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === "Enter" && searchOpen.set(true)}
>
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
    <circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/>
  </svg>
</div>

<style>
  .widget {
    width: 26px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.7);
  }
  .widget:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }
</style>
```

- [ ] **Step 4: 集成到 MenuBar**

在 `MenuBar.svelte` 引入 `SearchButton`，放在右侧 `DateTimeWidget` 之前：

```svelte
import SearchButton from "./SearchButton.svelte";
```

并在右侧区域添加：
```svelte
<SearchButton />
```

同时，在 `App.svelte` 或入口文件中添加条件渲染：
```svelte
{#if $searchOpen}
  <SearchOverlay />
{/if}
```

- [ ] **Step 5: 验证构建**

```bash
npm run build
```
Expected: 编译成功。

- [ ] **Step 6: 提交**

```bash
git add src/lib/components/SearchOverlay.svelte src/lib/components/SearchButton.svelte src/lib/stores/menubar.ts src/lib/components/MenuBar.svelte
git commit -m "feat: add Spotlight-style search overlay"
```

---

### Task 6: 通知中心面板 (NotificationPanel)

**Files:**
- Create: `src/lib/components/NotificationPanel.svelte`
- Create: `src/lib/components/NotificationBell.svelte`
- Modify: `src/lib/stores/menubar.ts`
- Modify: `src/lib/components/MenuBar.svelte`

- [ ] **Step 1: store 增加通知状态**

在 `src/lib/stores/menubar.ts` 添加：

```typescript
export interface Notification {
  id: string;
  title: string;
  body: string;
  time: Date;
}

export const notifications = writable<Notification[]>([]);
export const notificationPanelOpen = writable(false);
```

- [ ] **Step 2: 创建 NotificationBell.svelte**

```svelte
<script lang="ts">
  import { notificationPanelOpen, notifications } from "../stores/menubar";
</script>

<div
  class="widget"
  onclick={() => notificationPanelOpen.update((v) => !v)}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === "Enter" && notificationPanelOpen.update((v) => !v)}
>
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
    <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9"/>
    <path d="M13.73 21a2 2 0 01-3.46 0"/>
  </svg>
  {#if $notifications.length > 0}
    <span class="badge">{Math.min($notifications.length, 99)}</span>
  {/if}
</div>

<style>
  .widget {
    width: 26px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.7);
    position: relative;
  }
  .widget:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
  }
  .badge {
    position: absolute;
    top: 1px;
    right: 1px;
    background: #ef4444;
    color: white;
    font-size: 9px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
```

- [ ] **Step 3: 创建 NotificationPanel.svelte**

```svelte
<script lang="ts">
  import { notificationPanelOpen, notifications, type Notification } from "../stores/menubar";

  function clearOne(id: string) {
    notifications.update((n) => n.filter((x) => x.id !== id));
  }

  function clearAll() {
    notifications.set([]);
  }
</script>

<div class="panel" class:open={$notificationPanelOpen}>
  <div class="panel-header">
    <span class="panel-title">通知</span>
    <button class="clear-btn" onclick={clearAll}>清除全部</button>
  </div>
  <div class="panel-body">
    {#if $notifications.length === 0}
      <div class="empty">暂无通知</div>
    {:else}
      {#each $notifications as notif (notif.id)}
        <div class="notif-item">
          <div class="notif-header">
            <span class="notif-title">{notif.title}</span>
            <button class="notif-close" onclick={() => clearOne(notif.id)}>✕</button>
          </div>
          <div class="notif-body">{notif.body}</div>
        </div>
      {/each}
    {/if}
  </div>
</div>

{#if $notificationPanelOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="backdrop" onclick={() => notificationPanelOpen.set(false)}></div>
{/if}

<style>
  .panel {
    position: fixed;
    top: 28px;
    right: 0;
    width: 320px;
    max-height: calc(100vh - 28px);
    background: rgba(35, 35, 35, 0.95);
    backdrop-filter: blur(40px);
    border-left: 1px solid rgba(255, 255, 255, 0.1);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0 0 0 12px;
    z-index: 999999;
    display: flex;
    flex-direction: column;
    transform: translateX(100%);
    transition: transform 200ms ease-out;
  }
  .panel.open {
    transform: translateX(0);
  }
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }
  .panel-title {
    font-size: 14px;
    font-weight: 600;
    color: white;
  }
  .clear-btn {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    font-size: 12px;
  }
  .clear-btn:hover { color: white; }
  .panel-body {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }
  .empty {
    text-align: center;
    color: rgba(255, 255, 255, 0.4);
    padding: 40px 0;
    font-size: 13px;
  }
  .notif-item {
    padding: 10px 12px;
    border-radius: 8px;
    margin-bottom: 4px;
  }
  .notif-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  .notif-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .notif-title {
    font-size: 13px;
    font-weight: 600;
    color: white;
  }
  .notif-close {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.4);
    cursor: pointer;
    font-size: 12px;
  }
  .notif-close:hover { color: white; }
  .notif-body {
    font-size: 12px;
    color: rgba(255, 255, 255, 0.6);
    margin-top: 4px;
  }
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 999998;
  }
</style>
```

- [ ] **Step 4: 集成到 MenuBar 和 App**

在 `MenuBar.svelte`:
- 引入 `NotificationBell` 并添加到右侧区域
- 引入 `NotificationPanel` 并渲染

在 `App.svelte` 或入口添加条件渲染：
```svelte
<NotificationPanel />
```

- [ ] **Step 5: 验证构建**

```bash
npm run build
```

- [ ] **Step 6: 提交**

```bash
git add src/lib/components/NotificationPanel.svelte src/lib/components/NotificationBell.svelte src/lib/stores/menubar.ts
git commit -m "feat: add notification center panel"
```

---

### Task 7: 完成全面集成与全局快捷键

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Create/Modify: `src/App.svelte` 或入口文件
- Modify: `src/lib/components/MenuBar.svelte`

- [ ] **Step 1: 注册全局快捷键 Ctrl+Space 打开搜索**

在 `src-tauri/src/lib.rs` 的 `setup` 回调中添加：

```rust
use tauri_plugin_global_shortcut::{Code, Shortcut, ShortcutState, Modifiers};

// 在已有 Alt+Space 注册之后添加
let _ = app.global_shortcut().register(
    Shortcut::new(Some(Modifiers::CONTROL), Code::Space)
);
```

并在 handler 中添加：
```rust
Code::Space if modifiers == Modifiers::CONTROL => {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("toggle-search", ());
    }
}
```

- [ ] **Step 2: 前端监听 toggle-search 事件**

在 `setupMenuBarListeners()` 中添加：

```typescript
await listen("toggle-search", () => {
  searchOpen.update((v) => !v);
});
```

- [ ] **Step 3: 验证完整编译**

```bash
cd src-tauri && cargo check
npm run build
```

- [ ] **Step 4: 完整构建测试**

```bash
npx tauri build --no-bundle --target x86_64-pc-windows-gnu
```
Expected: 构建成功。

- [ ] **Step 5: 提交**

```bash
git add .
git commit -m "feat: complete P3 menu bar with search shortcut"
```

---

### 未完成项目（后续可补）

1. **WiFi 真实 API 实现**：当前使用模拟数据。后续可以用 `windows` crate 的 `WlanOpenHandle`/`WlanEnumInterfaces` 实现真实扫描。
2. **音量真实 API 实现**：需要初始化 `IMMDeviceEnumerator` 并通过 `IAudioEndpointVolume` 读写。
3. **搜索真实应用枚举**：后续可添加从开始菜单解析 `.lnk` 文件的 Rust 命令。
4. **通知真实集成**：当前通知靠前端手动添加，后续可监听 Windows 通知 API。
5. **菜单栏自动隐藏**：类似 Dock 的 auto_hide，鼠标移动到屏幕顶部才显示。
