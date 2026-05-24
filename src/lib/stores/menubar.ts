import { writable } from "svelte/store";

export const menuBarVisible = writable(true);
export const activeApp = writable<{ name: string; icon?: string } | null>(null);
export const searchOpen = writable(false);
export const notificationPanelOpen = writable(false);

export interface Notification {
  id: string;
  title: string;
  body: string;
  time: Date;
}

export const notifications = writable<Notification[]>([]);

export async function setupMenuBarListeners() {
  try {
    const { listen } = await import("@tauri-apps/api/event");
    listen<{ hwnd: number; title: string; app_name: string; icon_base64: string | null }>(
      "foreground-changed",
      (event) => {
        activeApp.set({ name: event.payload.app_name, icon: event.payload.icon_base64 ?? undefined });
      }
    );
    listen("toggle-search", () => {
      searchOpen.update((v) => !v);
    });
  } catch {
    activeApp.set({ name: "文件资源管理器" });
  }
}
