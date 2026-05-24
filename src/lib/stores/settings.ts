import { writable } from "svelte/store";

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

export async function loadSettings() {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const s = await invoke<Settings>("get_settings");
    settings.set(s);
  } catch {
    // Running outside Tauri (dev mode), use defaults
  }
}
