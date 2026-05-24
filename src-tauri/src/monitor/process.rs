use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub hwnd: u64,
    pub title: String,
    pub app_name: String,
    pub icon_base64: Option<String>,
}

#[cfg(windows)]
mod imp {
    use windows::Win32::Foundation::*;
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::System::Threading::*;
    use std::ffi::CStr;

    pub fn window_info_from_hwnd(hwnd: HWND) -> Option<super::WindowInfo> {
        let len = unsafe { GetWindowTextLengthA(hwnd) };
        if len == 0 { return None; }
        let mut title_buf = vec![0u8; (len + 1) as usize];
        unsafe { GetWindowTextA(hwnd, &mut title_buf); }
        let title = unsafe { CStr::from_ptr(title_buf.as_ptr() as *const i8) }
            .to_string_lossy().to_string();
        if title.is_empty() { return None; }

        let mut class_buf = [0u8; 260];
        unsafe { GetClassNameA(hwnd, &mut class_buf); }
        let class_name = unsafe { CStr::from_ptr(class_buf.as_ptr() as *const i8) }
            .to_string_lossy().to_string();

        let mut pid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }

        let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) };
        let app_name = if let Ok(proc) = process {
            let mut exe_buf = vec![0u16; 260];
            let mut exe_len = 260u32;
            unsafe {
                let _ = QueryFullProcessImageNameW(
                    proc,
                    PROCESS_NAME_WIN32,
                    windows::core::PWSTR::from_raw(exe_buf.as_mut_ptr()),
                    &mut exe_len,
                );
            }
            let path = String::from_utf16_lossy(&exe_buf[..exe_len as usize]);
            unsafe { let _ = CloseHandle(proc); }
            std::path::Path::new(&path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| class_name.clone())
        } else {
            class_name
        };

        Some(super::WindowInfo {
            hwnd: hwnd.0 as u64,
            title,
            app_name,
            icon_base64: None,
        })
    }

    pub fn enumerate_windows() -> Vec<super::WindowInfo> {
        let mut windows: Vec<super::WindowInfo> = Vec::new();
        let windows_ptr = &mut windows as *mut Vec<super::WindowInfo>;

        unsafe {
            let _ = EnumWindows(
                Some(enum_window_proc),
                LPARAM(windows_ptr as isize),
            );
        }

        windows
    }

    unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if IsWindowVisible(hwnd).as_bool() {
            if let Some(info) = window_info_from_hwnd(hwnd) {
                let windows = &mut *(lparam.0 as *mut Vec<super::WindowInfo>);
                windows.push(info);
            }
        }
        BOOL(1)
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn enumerate_windows() -> Vec<super::WindowInfo> {
        vec![]
    }
}

pub fn enumerate_windows() -> Vec<WindowInfo> {
    imp::enumerate_windows()
}

#[cfg(windows)]
pub fn window_info_from_hwnd(hwnd: windows::Win32::Foundation::HWND) -> Option<WindowInfo> {
    imp::window_info_from_hwnd(hwnd)
}
