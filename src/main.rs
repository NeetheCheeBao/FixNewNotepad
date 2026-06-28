#![windows_subsystem = "windows"]

slint::include_modules!();
use winreg::enums::*;
use winreg::RegKey;
use std::ptr;
use std::process::Command;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::fs;
use windows_sys::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK, MB_ICONINFORMATION};

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;

    ui.on_open_url(|| {
        let _ = open::that("https://github.com/NeetheCheeBao/FixNewNotepad");
    });

    ui.on_fix_menu_clicked(move || {
        if repair_new_menu() {
            refresh_shell();
            show_message_box("“新建文本文档”已尝试恢复。\n如果未立即生效，请重启电脑。");
        } else {
            show_message_box("修复失败，请确保以管理员身份运行。");
        }
    });

    ui.on_fix_association_clicked(move || {
        if repair_file_associations() {
            clean_user_choice();
            refresh_shell();
            restart_explorer();
            show_message_box("经典记事本关联已恢复！\n\n1. 已清除 UWP 劫持\n2. 已重置 .txt 打开方式\n3. 资源管理器已重启");
        } else {
            show_message_box("修复失败，请确保以管理员身份运行。");
        }
    });

    ui.on_fix_encoding_clicked(move |encoding| {
        if set_default_encoding(encoding.as_str()) {
            refresh_shell();
            show_message_box(&format!("已将新建文本文档编码设置为: {}", encoding));
        } else {
            show_message_box("设置编码失败，请确保以管理员身份运行。");
        }
    });

    ui.run()?;
    Ok(())
}

fn repair_new_menu() -> bool {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    
    if let Ok((key, _)) = hkcr.create_subkey(".txt") {
        let _ = key.set_value("", &"txtfile");
        if let Ok((shell_new, _)) = key.create_subkey("ShellNew") {
            let _ = shell_new.set_value("NullFile", &"");
            let _ = shell_new.delete_value("FileName");
        }
    } else {
        return false;
    }

    if let Ok((key, _)) = hkcr.create_subkey("txtfile") {
        let _ = key.set_value("", &"文本文档");
    }

    true
}

fn repair_file_associations() -> bool {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    let paths_to_clean = [
        r"txtfile\shell\open\command",
        r"SystemFileAssociations\.txt\shell\open\command",
        r"Applications\notepad.exe\shell\open\command",
    ];

    for path in paths_to_clean {
        if let Ok(key) = hkcr.open_subkey_with_flags(path, KEY_ALL_ACCESS) {
            let _ = key.delete_value("DelegateExecute");
        }
    }

    if let Ok((key, _)) = hkcr.create_subkey(r"txtfile\shell\open\command") {
        let _ = key.set_value("", &"\"C:\\Windows\\notepad.exe\" \"%1\"");
    }
    
    if let Ok((key, _)) = hkcr.create_subkey("txtfile") {
        let _ = key.set_value("", &"文本文档");
        if let Ok((icon_key, _)) = key.create_subkey("DefaultIcon") {
            let _ = icon_key.set_value("", &"%SystemRoot%\\system32\\imageres.dll,-102");
        }
    }

    true
}

fn clean_user_choice() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.txt";
    if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_ALL_ACCESS) {
        let _ = key.delete_subkey_all("UserChoice");
        let _ = key.delete_subkey_all("OpenWithList");
        return true;
    }
    true
}

fn set_default_encoding(encoding: &str) -> bool {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let shell_new_key_path = r".txt\ShellNew";

    if encoding == "ANSI" {
        if let Ok((key, _)) = hkcr.create_subkey(shell_new_key_path) {
            let _ = key.set_value("NullFile", &"");
            let _ = key.delete_value("FileName");
            return true;
        } else {
            return false;
        }
    }

    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let shell_new_dir = PathBuf::from(&system_root).join("ShellNew");
    
    if !shell_new_dir.exists() {
        if fs::create_dir_all(&shell_new_dir).is_err() {
            return false;
        }
    }

    let template_path = shell_new_dir.join("txtfile.txt");
    let content: &[u8] = match encoding {
        "UTF-16 LE" => &[0xFF, 0xFE],
        "UTF-16 BE" => &[0xFE, 0xFF],
        "带有BOM的UTF-8" => &[0xEF, 0xBB, 0xBF],
        "UTF-8" | "GB18030" | _ => &[],
    };

    if fs::write(&template_path, content).is_err() {
        return false;
    }

    if let Ok((key, _)) = hkcr.create_subkey(shell_new_key_path) {
        let _ = key.delete_value("NullFile");
        if key.set_value("FileName", &"txtfile.txt").is_err() {
            return false;
        }
    } else {
        return false;
    }

    true
}

fn refresh_shell() {
    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED as i32, SHCNF_IDLIST as u32, ptr::null(), ptr::null());
    }
}

fn restart_explorer() {
    let _ = Command::new("taskkill").args(["/F", "/IM", "explorer.exe"]).creation_flags(CREATE_NO_WINDOW).status();
    std::thread::sleep(std::time::Duration::from_millis(500));
    let _ = Command::new("explorer.exe").creation_flags(CREATE_NO_WINDOW).spawn();
}

fn show_message_box(text: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let wide_text: Vec<u16> = OsStr::new(text).encode_wide().chain(Some(0)).collect();
    let wide_title: Vec<u16> = OsStr::new("提示").encode_wide().chain(Some(0)).collect();

    unsafe {
        MessageBoxW(0, wide_text.as_ptr(), wide_title.as_ptr(), MB_OK | MB_ICONINFORMATION);
    }
}