# P3：顶部菜单栏 (Menu Bar)

macOS 风格顶部菜单栏，与现有的 Dock 栏构成完整桌面美化体验。

---

## 布局

全宽横条，固定在屏幕顶部，高度 28px，z-index 高于 Dock。

```
┌──────────────────────────────────────────────────────────────┐
│  [icon] 应用名    │         日期时间  🔍  🔔  🔋  🔊  📶   │
│  ← 左侧区域 →     │              ← 右侧系统托盘 →            │
└──────────────────────────────────────────────────────────────┘
```

## 功能模块

### 1. 左侧：活动应用指示器
- 显示当前前台窗口的应用图标 + 应用名
- 数据来源：已有 `monitor/events.rs` 的 `foreground-changed` 事件
- 纯展示，无交互

### 2. 右侧：系统托盘

**日期时间**
- 格式：`5月24日 周一 14:30`
- 使用 `Intl.DateTimeFormat`，每分钟更新
- 点击弹出日历面板（简略月视图）

**WiFi 状态**
- Rust IPC 命令 `get_wifi_status` → Windows API `WlanGetAvailableNetworkList`
- 前端显示: 信号格图标 (0-4 格), 连接状态
- 点击弹出面板: SSID、信号强度、IP 地址

**音量控制**
- Rust IPC 命令 `get_volume` / `set_volume`
- Windows API: `IAudioEndpointVolume`
- 前端显示: 扬声器图标 (有/无声)
- 点击弹出面板: 滑块 0-100%, 静音切换

**电池状态**
- Rust IPC 命令 `get_battery_status`
- Windows API: `GetSystemPowerStatus`
- 前端显示: 电池图标 (充电/放电/百分比)
- 鼠标悬停显示百分比

**通知中心入口**
- 点击滑出右侧通知面板
- 通知列表: 标题 + 正文 + 时间
- 支持清除单条/全部

**搜索入口**
- 点击触发全屏 Spotlight 覆盖层
- 覆盖层: 半透明蒙版，居中搜索框
- 搜索范围: 已固定应用、已安装应用（开始菜单枚举）、计算器
- 结果列表: 图标 + 名称，回车打开

### 3. 搜索覆盖层 (Spotlight)

```
┌──────────────────────────────────────────────────────────────┐
│░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│░░░░░░░░░░░░░░░░    🔍  搜索应用...    ░░░░░░░░░░░░░░░░░░░░│
│░░░░░░░░░░░░░░░░  ───────────────────  ░░░░░░░░░░░░░░░░░░░░│
│░░░░░░░░░░░░░░░░  文件资源管理器       ░░░░░░░░░░░░░░░░░░░░│
│░░░░░░░░░░░░░░░░  Microsoft Edge      ░░░░░░░░░░░░░░░░░░░░│
│░░░░░░░░░░░░░░░░  Code                ░░░░░░░░░░░░░░░░░░░░│
│░░░░░░░░░░░░░░░░                      ░░░░░░░░░░░░░░░░░░░░│
└──────────────────────────────────────────────────────────────┘
```

- 快捷键 `Ctrl+Space` 或点击搜索图标触发
- 输入过滤结果列表（模糊匹配）
- 上下键导航，回车打开选中项
- ESC 或点击外部关闭

## 组件树

```
MenuBar.svelte
├── AppIndicator.svelte          # 左侧应用图标+名称
├── SystemTray.svelte            # 右侧系统托盘容器
│   ├── DateTimeWidget.svelte    # 日期时间 + 日历面板
│   ├── WifiWidget.svelte        # WiFi 状态 + 面板
│   ├── VolumeWidget.svelte      # 音量图标 + 滑块面板
│   ├── BatteryWidget.svelte     # 电池图标 + 提示
│   ├── NotificationBell.svelte  # 通知铃铛
│   └── SearchButton.svelte      # 搜索按钮
├── NotificationPanel.svelte     # 右侧滑出通知面板
└── SearchOverlay.svelte         # 全屏搜索覆盖层
```

## Rust 后端新增

```rust
// src-tauri/src/commands/menubar.rs
#[tauri::command]
fn get_volume() -> Result<VolumeInfo, String>     // 当前音量 + 静音状态

#[tauri::command]
fn set_volume(level: f32) -> Result<(), String>   // 设置音量 0.0-1.0

#[tauri::command]
fn get_battery() -> Result<BatteryInfo, String>   // 电量% + 充电状态

#[tauri::command]
fn get_wifi() -> Result<WifiInfo, String>         // SSID + 信号强度

#[tauri::command]
fn search_apps(query: String) -> Vec<AppResult>   // 搜索已安装应用

struct VolumeInfo { level: f32, muted: bool }
struct BatteryInfo { percent: u8, charging: bool }
struct WifiInfo { ssid: String, signal: u8, connected: bool }
struct AppResult { name: String, path: String, icon: Option<String> }
```

注：WiFi API 需要链接 `wlanapi.lib`，音量用 `windows` crate 的 `CoreAudio` API，电池用 `GetSystemPowerStatus`。

## 前端 Store

```typescript
// src/lib/stores/menubar.ts
menuVisible: Writable<boolean>          // 菜单栏可见性
activeApp: Writable<{name: string, icon?: string} | null>  // 当前活动应用
volume: Writable<{level: number, muted: boolean}>          // 音量状态
battery: Writable<{percent: number, charging: boolean}>    // 电池状态
wifi: Writable<{ssid: string, signal: number} | null>      // WiFi 状态
notifications: Writable<Notification[]>                     // 通知列表
searchOpen: Writable<boolean>                               // 搜索覆盖层
notificationPanelOpen: Writable<boolean>                    // 通知面板
```

## 数据流

```
Windows 事件 → monitor/events.rs → foreground-changed → AppIndicator 更新
系统状态  → Rust IPC (get_volume/battery/wifi) → SystemTray 组件
搜索查询  → SearchOverlay → IPC search_apps → 结果列表
用户操作  → 点击系统图标 → 弹出面板 → IPC 写入 (set_volume)
```

## 样式

- 背景: `rgba(30, 30, 30, 0.7)` + `backdrop-filter: blur(40px)`（与 Dock 一致）
- 字体: 12px system-ui, 白色
- 图标: 16px SVG inline
- 悬停高亮: 半透明白色背景
- 屏幕适配: `position: fixed; top: 0; width: 100vw`

## 实现顺序

1. MenuBar 骨架 + 活动应用指示器 + 日期时间
2. 系统状态 (音量/电池/WiFi) — Rust + 前端
3. 搜索覆盖层
4. 通知中心
5. 全局快捷键 + 自动隐藏
