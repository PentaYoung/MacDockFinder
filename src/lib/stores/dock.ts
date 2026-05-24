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

export const pinnedItems = writable<PinnedItem[]>([
  { id: "default-explorer", path: "explorer.exe", label: "文件资源管理器", icon_path: null },
  { id: "default-terminal", path: "wt.exe", label: "终端", icon_path: null },
  { id: "default-edge", path: "msedge.exe", label: "Microsoft Edge", icon_path: null },
  { id: "default-calculator", path: "calculator.exe", label: "计算器", icon_path: null },
  { id: "default-notepad", path: "notepad.exe", label: "记事本", icon_path: null },
]);
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

    const wins = await invoke<WindowInfo[]>("get_active_windows");
    activeWindows.set(wins);

    await listen<WindowInfo>("window-created", (event) => {
      activeWindows.update((wins) => {
        if (!wins.find((w) => w.hwnd === event.payload.hwnd)) {
          return [...wins, event.payload];
        }
        return wins;
      });
    });

    await listen<number>("window-destroyed", (event) => {
      activeWindows.update((wins) =>
        wins.filter((w) => w.hwnd !== event.payload)
      );
    });

    await listen<WindowInfo>("foreground-changed", () => {
    });
  } catch {
    // Running outside Tauri, no window monitoring
  }
}
