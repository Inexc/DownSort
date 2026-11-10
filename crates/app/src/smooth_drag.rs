#![cfg(windows)]

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Dwm::{DWM_TIMING_INFO, DwmGetCompositionTimingInfo};
use windows::Win32::Graphics::Gdi::{
    EnumDisplaySettingsW, GetMonitorInfoW, MonitorFromWindow, RedrawWindow, DEVMODEW,
    ENUM_CURRENT_SETTINGS, MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
    RDW_INTERNALPAINT, RDW_INVALIDATE, RDW_UPDATENOW,
};
use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};
use windows::Win32::UI::Shell::{
    DefSubclassProc, ExtractIconExW, RemoveWindowSubclass, SetWindowSubclass,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, PostMessageW,
    SendMessageW, SetWindowTextW, ICON_BIG, ICON_SMALL, WM_DESTROY, WM_DISPLAYCHANGE,
    WM_DPICHANGED, WM_ENTERSIZEMOVE, WM_EXITSIZEMOVE, WM_MOVE, WM_MOVING, WM_SETICON, WM_TIMER,
};

const SUBCLASS_ID: usize = 0x444F574E; // "DOWN"

#[derive(Clone, Copy)]
struct SendHwnd(isize);
unsafe impl Send for SendHwnd {}
unsafe impl Sync for SendHwnd {}

struct SubclassState {
    is_in_size_move: Arc<AtomicBool>,
    current_hz: Arc<AtomicU32>,
    last_drag_paint: Instant,
}

/// Request 1ms system timer precision from Windows OS
pub fn init_system_timer_resolution() {
    unsafe {
        let _ = timeBeginPeriod(1);
    }
}

/// Cleanup timer resolution on application exit
#[allow(dead_code)]
pub fn cleanup_system_timer_resolution() {
    unsafe {
        let _ = timeEndPeriod(1);
    }
}

/// Query the live refresh rate (Hz) of the monitor where the window is currently located
pub fn get_monitor_refresh_rate(hwnd: HWND) -> u32 {
    unsafe {
        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFOEXW::default();
        mi.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(hmon, &mut mi as *mut _ as *mut MONITORINFO).as_bool() {
            let mut devmode = DEVMODEW::default();
            devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
            if EnumDisplaySettingsW(
                PCWSTR(mi.szDevice.as_ptr()),
                ENUM_CURRENT_SETTINGS,
                &mut devmode,
            )
            .as_bool()
                && devmode.dmDisplayFrequency >= 30
            {
                return devmode.dmDisplayFrequency;
            }
        }

        // Fallback to DWM composition refresh rate
        let mut timing_info = DWM_TIMING_INFO {
            cbSize: std::mem::size_of::<DWM_TIMING_INFO>() as u32,
            ..Default::default()
        };
        if DwmGetCompositionTimingInfo(HWND::default(), &mut timing_info).is_ok()
            && timing_info.rateRefresh.uiDenominator > 0
            && timing_info.rateRefresh.uiNumerator > 0
        {
            let hz = timing_info.rateRefresh.uiNumerator / timing_info.rateRefresh.uiDenominator;
            if hz >= 30 {
                return hz;
            }
        }

        180 // Default to high-refresh 180Hz
    }
}

/// Window subclass procedure that forces synchronous, adaptive-rate repainting during window drag
unsafe extern "system" fn smooth_drag_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    uidsubclass: usize,
    refdata: usize,
) -> LRESULT {
    let state = &mut *(refdata as *mut SubclassState);
    match msg {
        WM_ENTERSIZEMOVE => {
            // User began dragging or resizing the window
            let hz = get_monitor_refresh_rate(hwnd);
            state.current_hz.store(hz, Ordering::Relaxed);
            state.is_in_size_move.store(true, Ordering::Relaxed);
            state.last_drag_paint = Instant::now();

            let is_moving = state.is_in_size_move.clone();
            let current_hz = state.current_hz.clone();
            let pacer_hwnd = SendHwnd(hwnd.0 as isize);

            // Dedicated high-priority pacer thread for the duration of the drag operation
            std::thread::Builder::new()
                .name("SmoothDragPacer".into())
                .spawn(move || {
                    while is_moving.load(Ordering::Relaxed) {
                        let hz = current_hz.load(Ordering::Relaxed).max(30);
                        let frame_interval = Duration::from_nanos(1_000_000_000 / hz as u64);
                        let t0 = Instant::now();
                        unsafe {
                            let hwnd = HWND(pacer_hwnd.0 as *mut _);
                            let _ = PostMessageW(hwnd, WM_TIMER, WPARAM(1), LPARAM(0));
                            let _ = RedrawWindow(
                                hwnd,
                                None,
                                None,
                                RDW_INVALIDATE | RDW_UPDATENOW | RDW_INTERNALPAINT,
                            );
                        }
                        let elapsed = t0.elapsed();
                        if elapsed < frame_interval {
                            std::thread::sleep(frame_interval - elapsed);
                        }
                    }
                })
                .ok();
        }
        WM_MOVING => {
            // Windows is actively moving the drag frame across the screen
            let hz = state.current_hz.load(Ordering::Relaxed).max(30);
            let frame_interval = Duration::from_nanos(1_000_000_000 / hz as u64);
            let now = Instant::now();
            if now.duration_since(state.last_drag_paint) >= frame_interval {
                state.last_drag_paint = now;
                unsafe {
                    // Dispatch WM_TIMER(1) so GPUI processes pending runnables and paints
                    let _ = DefSubclassProc(hwnd, WM_TIMER, WPARAM(1), LPARAM(0));
                    let _ = RedrawWindow(
                        hwnd,
                        None,
                        None,
                        RDW_INVALIDATE | RDW_UPDATENOW | RDW_INTERNALPAINT,
                    );
                }
            }
        }
        WM_MOVE => {
            // Window moved (potentially across monitors with different refresh rates, e.g. 60Hz <-> 180Hz)
            let new_hz = get_monitor_refresh_rate(hwnd);
            let old_hz = state.current_hz.swap(new_hz, Ordering::Relaxed);
            if new_hz != old_hz {
                eprintln!("[DownSort] Monitor refresh rate adapted: {} Hz -> {} Hz", old_hz, new_hz);
            }
        }
        WM_DISPLAYCHANGE | WM_DPICHANGED => {
            // Display resolution, scaling or refresh rate changed dynamically
            let new_hz = get_monitor_refresh_rate(hwnd);
            state.current_hz.store(new_hz, Ordering::Relaxed);
            eprintln!("[DownSort] Display mode changed: adapted to {} Hz", new_hz);
        }
        WM_EXITSIZEMOVE => {
            // Dragging finished
            state.is_in_size_move.store(false, Ordering::Relaxed);
            unsafe {
                let _ = DefSubclassProc(hwnd, WM_TIMER, WPARAM(1), LPARAM(0));
                let _ = RedrawWindow(
                    hwnd,
                    None,
                    None,
                    RDW_INVALIDATE | RDW_UPDATENOW | RDW_INTERNALPAINT,
                );
            }
        }
        WM_DESTROY => {
            state.is_in_size_move.store(false, Ordering::Relaxed);
            unsafe {
                let _ = RemoveWindowSubclass(hwnd, Some(smooth_drag_subclass_proc), uidsubclass);
            }
            drop(Box::from_raw(state as *mut SubclassState));
            return DefSubclassProc(hwnd, msg, wparam, lparam);
        }
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

/// Attach smooth drag subclass to the application window
pub fn attach_smooth_drag(hwnd: HWND) -> bool {
    let hz = get_monitor_refresh_rate(hwnd);
    let state = Box::new(SubclassState {
        is_in_size_move: Arc::new(AtomicBool::new(false)),
        current_hz: Arc::new(AtomicU32::new(hz)),
        last_drag_paint: Instant::now(),
    });
    let raw = Box::into_raw(state) as usize;
    unsafe {
        SetWindowSubclass(hwnd, Some(smooth_drag_subclass_proc), SUBCLASS_ID, raw).as_bool()
    }
}

/// Find the main visible HWND belonging to the current process with title "DownSort"
pub fn find_app_hwnd() -> Option<HWND> {
    struct Search {
        target_pid: u32,
        found_hwnd: Option<HWND>,
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let search = &mut *(lparam.0 as *mut Search);
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == search.target_pid && IsWindowVisible(hwnd).as_bool() {
            let mut buf = [0u16; 64];
            let len = GetWindowTextW(hwnd, &mut buf);
            let title = String::from_utf16_lossy(&buf[..len as usize]);
            if title == "DownSort" {
                search.found_hwnd = Some(hwnd);
                return BOOL(0); // Stop enumeration, target window found
            }
        }
        BOOL(1) // Continue
    }

    let mut search = Search {
        target_pid: std::process::id(),
        found_hwnd: None,
    };

    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut search as *mut Search as isize));
    }

    search.found_hwnd
}

/// Spawn background monitor that attaches the smooth drag subclass and sets high-DPI icons as soon as the window is created
pub fn spawn_smooth_drag_monitor() {
    init_system_timer_resolution();
    std::thread::Builder::new()
        .name("SmoothDragWatcher".into())
        .spawn(|| {
            for _ in 0..120 {
                // Poll every 25ms up to 3 seconds
                std::thread::sleep(Duration::from_millis(25));
                if let Some(hwnd) = find_app_hwnd() {
                    // 1. Force the OS window title to be strictly "DownSort"
                    unsafe {
                        let _ = SetWindowTextW(hwnd, w!("DownSort"));
                    }

                    // 2. Load and set crisp multi-resolution icons for Taskbar and Titlebar
                    let exe_path = std::env::current_exe().unwrap_or_default();
                    let path_hstring = HSTRING::from(exe_path.as_os_str());
                    let mut hicon_large = Default::default();
                    let mut hicon_small = Default::default();
                    let count = unsafe {
                        ExtractIconExW(
                            PCWSTR(path_hstring.as_ptr()),
                            0,
                            Some(&mut hicon_large),
                            Some(&mut hicon_small),
                            1,
                        )
                    };
                    if count > 0 {
                        unsafe {
                            if !hicon_large.is_invalid() {
                                let _ = SendMessageW(
                                    hwnd,
                                    WM_SETICON,
                                    WPARAM(ICON_BIG as usize),
                                    LPARAM(hicon_large.0 as isize),
                                );
                            }
                            if !hicon_small.is_invalid() {
                                let _ = SendMessageW(
                                    hwnd,
                                    WM_SETICON,
                                    WPARAM(ICON_SMALL as usize),
                                    LPARAM(hicon_small.0 as isize),
                                );
                            }
                        }
                    }

                    // 3. Attach subclass for 180Hz smooth dragging & adaptive refresh rate
                    let hz = get_monitor_refresh_rate(hwnd);
                    if attach_smooth_drag(hwnd) {
                        eprintln!("[DownSort] Successfully attached adaptive smooth drag subclass ({} Hz)", hz);
                        break;
                    }
                }
            }
        })
        .ok();
}
