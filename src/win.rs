//! 通用 Win32 辅助函数。

use std::ffi::c_void;
use std::ffi::OsStr;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Gdi::{CreateFontIndirectW, HFONT};
use windows_sys::Win32::UI::HiDpi::GetDpiForSystem;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, SystemParametersInfoW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK,
    NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS,
};

/// 把 &str 转成以 0 结尾的宽字符缓冲。
pub fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/// 按 DPI 缩放逻辑像素（设计基准 96 DPI）。
pub fn scale(value: i32, dpi: u32) -> i32 {
    (value as f64 * dpi as f64 / 96.0).round() as i32
}

/// 取系统界面字体（经典 Win32 做法：NONCLIENTMETRICS.lfMessageFont）。
pub unsafe fn system_font(dpi: u32) -> HFONT {
    let mut metrics: NONCLIENTMETRICSW = zeroed();
    metrics.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
    if SystemParametersInfoW(
        SPI_GETNONCLIENTMETRICS,
        metrics.cbSize,
        &mut metrics as *mut _ as *mut c_void,
        0,
    ) == 0
    {
        return 0;
    }

    let mut log_font = metrics.lfMessageFont;
    let system_dpi = GetDpiForSystem();
    if dpi != 0 && system_dpi != 0 && dpi != system_dpi {
        log_font.lfHeight =
            (log_font.lfHeight as f64 * dpi as f64 / system_dpi as f64).round() as i32;
    }
    CreateFontIndirectW(&log_font)
}

pub unsafe fn message_box(owner: HWND, text: &str, title: &str, icon: u32) {
    let text = wide(text);
    let title = wide(title);
    MessageBoxW(owner, text.as_ptr(), title.as_ptr(), MB_OK | icon);
}

pub const ICON_INFO: u32 = MB_ICONINFORMATION;
pub const ICON_ERROR: u32 = MB_ICONERROR;
