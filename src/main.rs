#![windows_subsystem = "windows"]

mod fix;
mod win;

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::null;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::{
    CreateSolidBrush, DeleteObject, GetDC, GetDeviceCaps, GetSysColor, ReleaseDC, SetBkColor,
    SetTextColor, UpdateWindow, COLOR_BTNFACE, COLOR_BTNTEXT, HBRUSH, HDC, HFONT, LOGPIXELSX,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::SystemServices::{SS_ETCHEDHORZ, SS_LEFT};
use windows_sys::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX,
};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const CLASS_NAME: &str = "FnnMainWindow";
const APP_NAME: &str = "FixNewNotepad";
const APP_VERSION: &str = env!("FNN_VERSION");
const PROJECT_URL: &str = "https://github.com/NeetheCheeBao/FixNewNotepad";

const ID_FIX_MENU: i32 = 1001;
const ID_FIX_ASSOC: i32 = 1002;
const ID_COMBO: i32 = 1003;
const ID_APPLY_ENC: i32 = 1004;
const ID_CHECK_UPDATE: i32 = 1005;

const ENCODINGS: [&str; 6] = [
    "ANSI",
    "UTF-16 LE",
    "UTF-16 BE",
    "UTF-8",
    "带有BOM的UTF-8",
    "GB18030",
];

// 设计稿尺寸（96 DPI 下的逻辑像素）
const CLIENT_W: i32 = 420;
const CLIENT_H: i32 = 144;

struct App {
    hwnd: HWND,
    dpi: u32,
    font: HFONT,
    brush: HBRUSH,
    btn_menu: HWND,
    btn_assoc: HWND,
    combo: HWND,
    btn_apply: HWND,
    label: HWND,
    hint: HWND,
    separator: HWND,
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            if self.font != 0 {
                DeleteObject(self.font);
            }
            if self.brush != 0 {
                DeleteObject(self.brush);
            }
        }
    }
}

impl App {
    unsafe fn new(dpi: u32) -> Self {
        Self {
            hwnd: 0,
            dpi,
            font: win::system_font(dpi),
            brush: CreateSolidBrush(GetSysColor(COLOR_BTNFACE)),
            btn_menu: 0,
            btn_assoc: 0,
            combo: 0,
            btn_apply: 0,
            label: 0,
            hint: 0,
            separator: 0,
        }
    }

    fn s(&self, value: i32) -> i32 {
        win::scale(value, self.dpi)
    }

    /// 创建子控件，style 中无需包含 WS_CHILD / WS_VISIBLE。
    unsafe fn add(&self, class: &str, text: &str, style: u32, id: i32) -> HWND {
        let class = win::wide(class);
        let text = win::wide(text);
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            text.as_ptr(),
            WS_CHILD | WS_VISIBLE | style,
            0,
            0,
            10,
            10,
            self.hwnd,
            id as HMENU,
            GetModuleHandleW(null()),
            null(),
        );
        if hwnd != 0 {
            SendMessageW(hwnd, WM_SETFONT, self.font as WPARAM, 1);
        }
        hwnd
    }

    unsafe fn create_children(&mut self) -> bool {
        let button = WS_TABSTOP | BS_PUSHBUTTON as u32;

        self.btn_menu = self.add("BUTTON", "修复新建文本文档", button | WS_GROUP, ID_FIX_MENU);
        self.btn_assoc = self.add("BUTTON", "还原经典记事本", button, ID_FIX_ASSOC);
        self.separator = self.add("STATIC", "", SS_ETCHEDHORZ, 0);
        self.label = self.add("STATIC", "新建文本文档默认编码：", SS_LEFT, 0);
        self.combo = self.add(
            "COMBOBOX",
            "",
            WS_TABSTOP | WS_VSCROLL | CBS_DROPDOWNLIST as u32,
            ID_COMBO,
        );
        self.btn_apply = self.add("BUTTON", "应用", button, ID_APPLY_ENC);
        self.hint = self.add(
            "STATIC",
            "修改后对新建的 .txt 文件生效。",
            SS_LEFT,
            0,
        );

        if self.combo != 0 {
            for encoding in ENCODINGS {
                let text = win::wide(encoding);
                SendMessageW(self.combo, CB_ADDSTRING, 0, text.as_ptr() as LPARAM);
            }
            SendMessageW(self.combo, CB_SETCURSEL, 0, 0);
        }

        [
            self.btn_menu,
            self.btn_assoc,
            self.separator,
            self.label,
            self.combo,
            self.btn_apply,
            self.hint,
        ]
        .iter()
        .all(|handle| *handle != 0)
    }

    unsafe fn layout(&self) {
        if self.btn_menu == 0 {
            return;
        }
        let s = |v: i32| self.s(v);

        MoveWindow(self.btn_menu, s(12), s(12), s(188), s(28), 1);
        MoveWindow(self.btn_assoc, s(220), s(12), s(188), s(28), 1);
        MoveWindow(self.separator, s(12), s(52), s(396), s(1), 1);
        MoveWindow(self.label, s(12), s(62), s(240), s(18), 1);
        MoveWindow(self.combo, s(12), s(84), s(188), s(160), 1);
        MoveWindow(self.btn_apply, s(208), s(84), s(84), s(26), 1);
        MoveWindow(self.hint, s(12), s(116), s(396), s(16), 1);
    }

    unsafe fn apply_font(&self, font: HFONT) {
        for child in [
            self.btn_menu,
            self.btn_assoc,
            self.separator,
            self.label,
            self.combo,
            self.btn_apply,
            self.hint,
        ] {
            if child != 0 {
                SendMessageW(child, WM_SETFONT, font as WPARAM, 1);
            }
        }
    }

    unsafe fn set_dpi(&mut self, dpi: u32) {
        if dpi == 0 || dpi == self.dpi {
            return;
        }
        self.dpi = dpi;

        let old = self.font;
        self.font = win::system_font(dpi);
        if self.font != 0 {
            self.apply_font(self.font);
        }
        if old != 0 {
            DeleteObject(old);
        }
    }

    unsafe fn on_command(&mut self, id: i32) {
        match id {
            ID_FIX_MENU => self.fix_new_menu(),
            ID_FIX_ASSOC => self.fix_association(),
            ID_APPLY_ENC => self.apply_encoding(),
            ID_CHECK_UPDATE => {
                let _ = open::that(PROJECT_URL);
            }
            _ => {}
        }
    }

    unsafe fn fix_new_menu(&self) {
        match fix::repair_new_menu() {
            Ok(()) => {
                fix::refresh_shell();
                win::message_box(
                    self.hwnd,
                    "「新建文本文档」已尝试恢复。\n\n如果右键菜单没有立即刷新，请注销或重启电脑。",
                    "修复完成",
                    win::ICON_INFO,
                );
            }
            Err(err) => win::message_box(
                self.hwnd,
                &format!("修复失败：\n{err}\n\n请确认程序以管理员身份运行。"),
                "修复失败",
                win::ICON_ERROR,
            ),
        }
    }

    unsafe fn fix_association(&self) {
        match fix::restore_classic_notepad() {
            Ok(()) => {
                fix::refresh_shell();
                fix::restart_explorer();
                win::message_box(
                    self.hwnd,
                    "经典记事本关联已恢复！\n\n1. 已清除 UWP 劫持\n2. 已重置 .txt 打开方式\n3. 资源管理器已重启",
                    "还原完成",
                    win::ICON_INFO,
                );
            }
            Err(err) => win::message_box(
                self.hwnd,
                &format!("还原失败：\n{err}\n\n请确认程序以管理员身份运行。"),
                "还原失败",
                win::ICON_ERROR,
            ),
        }
    }

    unsafe fn apply_encoding(&self) {
        let index = SendMessageW(self.combo, CB_GETCURSEL, 0, 0) as i32;
        let encoding = ENCODINGS
            .get(index.max(0) as usize)
            .copied()
            .unwrap_or("ANSI");

        match fix::set_default_encoding(encoding) {
            Ok(()) => {
                fix::refresh_shell();
                win::message_box(
                    self.hwnd,
                    &format!("已将新建文本文档编码设置为：{encoding}"),
                    "设置完成",
                    win::ICON_INFO,
                );
            }
            Err(err) => win::message_box(
                self.hwnd,
                &format!("设置编码失败：\n{err}\n\n请确认程序以管理员身份运行。"),
                "设置失败",
                win::ICON_ERROR,
            ),
        }
    }
}

fn app_ptr(hwnd: HWND) -> *mut App {
    unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut App }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_NCCREATE => {
            let cs = lparam as *const CREATESTRUCTW;
            if !cs.is_null() && !(*cs).lpCreateParams.is_null() {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*cs).lpCreateParams as isize);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_CREATE => {
            let app = app_ptr(hwnd);
            if app.is_null() {
                return -1;
            }
            (*app).hwnd = hwnd;
            (*app).dpi = GetDpiForWindow(hwnd);
            if !(*app).create_children() {
                return -1;
            }
            (*app).layout();
            0
        }

        WM_NCDESTROY => {
            let app = app_ptr(hwnd);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            if !app.is_null() {
                drop(Box::from_raw(app));
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_SIZE => {
            let app = app_ptr(hwnd);
            if !app.is_null() {
                (*app).layout();
            }
            0
        }

        WM_DPICHANGED => {
            let app = app_ptr(hwnd);
            if !app.is_null() {
                (*app).set_dpi((wparam & 0xFFFF) as u32);
                let suggested = lparam as *const RECT;
                if !suggested.is_null() {
                    let r = *suggested;
                    SetWindowPos(
                        hwnd,
                        0,
                        r.left,
                        r.top,
                        r.right - r.left,
                        r.bottom - r.top,
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
                (*app).layout();
            }
            0
        }

        WM_CTLCOLORSTATIC => {
            let hdc = wparam as HDC;
            SetTextColor(hdc, GetSysColor(COLOR_BTNTEXT));
            SetBkColor(hdc, GetSysColor(COLOR_BTNFACE));
            let app = app_ptr(hwnd);
            if app.is_null() {
                0
            } else {
                (*app).brush as LRESULT
            }
        }

        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as i32;
            let notification = ((wparam >> 16) & 0xFFFF) as u32;
            let app = app_ptr(hwnd);
            if !app.is_null() && notification == BN_CLICKED {
                (*app).on_command(id);
            }
            0
        }

        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn load_app_icon(hinst: HINSTANCE) -> HICON {
    unsafe {
        // 资源文件中的图标组 ID 为 1
        let icon = LoadIconW(hinst, 1 as *const u16);
        if icon != 0 {
            icon
        } else {
            LoadIconW(0, IDI_APPLICATION)
        }
    }
}

/// 创建菜单栏：帮助 -> 检查更新
unsafe fn create_menu() -> HMENU {
    let menu = CreateMenu();
    let help = CreatePopupMenu();
    let item = win::wide("检查更新");
    AppendMenuW(help, MF_STRING, ID_CHECK_UPDATE as usize, item.as_ptr());
    let title = win::wide("帮助");
    AppendMenuW(menu, MF_POPUP | MF_STRING, help as usize, title.as_ptr());
    menu
}

unsafe fn run() {
    let hinst = GetModuleHandleW(null());

    let icc = INITCOMMONCONTROLSEX {
        dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_STANDARD_CLASSES,
    };
    InitCommonControlsEx(&icc);

    let screen = GetDC(0);
    let dpi = GetDeviceCaps(screen, LOGPIXELSX as i32) as u32;
    ReleaseDC(0, screen);

    let app = Box::into_raw(Box::new(App::new(dpi)));

    let class_name = win::wide(CLASS_NAME);
    let wc = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinst,
        hIcon: load_app_icon(hinst),
        hCursor: LoadCursorW(0, IDC_ARROW),
        hbrBackground: (COLOR_BTNFACE + 1) as HBRUSH,
        lpszMenuName: null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: load_app_icon(hinst),
    };
    RegisterClassExW(&wc);

    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
    let ex_style = WS_EX_CONTROLPARENT;
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: win::scale(CLIENT_W, dpi),
        bottom: win::scale(CLIENT_H, dpi),
    };
    AdjustWindowRectEx(&mut rect, style, 1, ex_style);
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;

    let mut work: RECT = zeroed();
    SystemParametersInfoW(
        SPI_GETWORKAREA,
        0,
        &mut work as *mut RECT as *mut c_void,
        0,
    );
    let x = work.left + ((work.right - work.left) - width) / 2;
    let y = work.top + ((work.bottom - work.top) - height) / 2;

    let title = win::wide(&format!("{APP_NAME}_{APP_VERSION}"));
    let menu = create_menu();
    let hwnd = CreateWindowExW(
        ex_style,
        class_name.as_ptr(),
        title.as_ptr(),
        style,
        x,
        y,
        width,
        height,
        0,
        menu,
        hinst,
        app as *const c_void,
    );
    if hwnd == 0 {
        drop(Box::from_raw(app));
        return;
    }

    ShowWindow(hwnd, SW_SHOW);
    UpdateWindow(hwnd);

    let mut msg: MSG = zeroed();
    while GetMessageW(&mut msg, 0, 0, 0) > 0 {
        if IsDialogMessageW(hwnd, &mut msg) == 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn main() {
    unsafe { run() };
}
