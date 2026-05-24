use std::sync::OnceLock;
use tauri::AppHandle;

pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

#[cfg(target_os = "windows")]
mod imp {
    use tauri::Emitter;
    use super::APP_HANDLE;
    use super::super::process;
    use windows::Win32::Foundation::*;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::UI::Accessibility::*;

    pub fn start_listener() {
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
                    let _ = TranslateMessage(&msg);
                    DispatchMessageA(&msg);
                }

                let _ = UnhookWinEvent(hook);
            }
        });
    }

    unsafe extern "system" fn event_proc(
        _hhook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        _id_obj: i32,
        _id_child: i32,
        _id_event_thread: u32,
        _dwms_event_time: u32,
    ) {
        if !IsWindow(Some(hwnd)).as_bool() || !IsWindowVisible(hwnd).as_bool() {
            return;
        }

        match event {
            EVENT_OBJECT_CREATE | EVENT_OBJECT_SHOW => {
                if let Some(handle) = APP_HANDLE.get() {
                    if let Some(info) = process::window_info_from_hwnd(hwnd) {
                        let _ = handle.emit("window-created", info);
                    }
                }
            }
            EVENT_OBJECT_DESTROY | EVENT_OBJECT_HIDE => {
                if let Some(handle) = APP_HANDLE.get() {
                    let _ = handle.emit("window-destroyed", hwnd.0 as u64);
                }
            }
            EVENT_SYSTEM_FOREGROUND => {
                if let Some(handle) = APP_HANDLE.get() {
                    if let Some(info) = process::window_info_from_hwnd(hwnd) {
                        let _ = handle.emit("foreground-changed", info);
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(target_os = "windows")]
pub fn start_window_event_listener() {
    imp::start_listener();
}

#[cfg(not(target_os = "windows"))]
pub fn start_window_event_listener() {
}
