import type { PinnedItem } from "../stores/dock";
import type { Settings } from "../stores/settings";

async function getInvoke() {
  try {
    const mod = await import("@tauri-apps/api/core");
    return mod.invoke;
  } catch {
    return null;
  }
}

export async function getPinnedItems(): Promise<PinnedItem[]> {
  const fn = await getInvoke();
  if (fn) return fn("get_pinned_items");
  return [
    { id: "1", path: "explorer.exe", label: "文件资源管理器", icon_path: null },
    { id: "2", path: "chrome.exe", label: "Google Chrome", icon_path: null },
    { id: "3", path: "code.exe", label: "VS Code", icon_path: null },
    { id: "4", path: "terminal.exe", label: "终端", icon_path: null },
    { id: "5", path: "spotify.exe", label: "Spotify", icon_path: null },
  ];
}

export async function getSettings(): Promise<Settings> {
  const fn = await getInvoke();
  if (fn) return fn("get_settings");
  return { position: "bottom", auto_hide: false, icon_size: 48, magnification: true };
}

export async function addPinnedItem(path: string): Promise<PinnedItem> {
  const fn = await getInvoke();
  if (fn) return fn("add_pinned_item", { path });
  return { id: Date.now().toString(), path, label: path.split("\\").pop() || "App", icon_path: null };
}
