//! 注册表修复逻辑（与原实现保持一致，改为返回可读错误）。

use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::ptr;

use windows_sys::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
use winreg::enums::*;
use winreg::RegKey;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// 修复右键菜单中的「新建文本文档」。
pub fn repair_new_menu() -> Result<(), String> {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    let (key, _) = hkcr
        .create_subkey(".txt")
        .map_err(|e| format!("无法打开 HKCR\\.txt：{e}"))?;
    key.set_value("", &"txtfile")
        .map_err(|e| format!("写入 .txt 默认值失败：{e}"))?;

    let (shell_new, _) = key
        .create_subkey("ShellNew")
        .map_err(|e| format!("无法创建 .txt\\ShellNew：{e}"))?;
    shell_new
        .set_value("NullFile", &"")
        .map_err(|e| format!("写入 NullFile 失败：{e}"))?;
    let _ = shell_new.delete_value("FileName");

    let (txtfile, _) = hkcr
        .create_subkey("txtfile")
        .map_err(|e| format!("无法打开 HKCR\\txtfile：{e}"))?;
    let _ = txtfile.set_value("", &"文本文档");

    Ok(())
}

/// 还原经典记事本（清除 UWP 劫持、重置打开方式与图标）。
pub fn restore_classic_notepad() -> Result<(), String> {
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

    let (cmd_key, _) = hkcr
        .create_subkey(r"txtfile\shell\open\command")
        .map_err(|e| format!("无法写入打开命令：{e}"))?;
    cmd_key
        .set_value("", &"\"C:\\Windows\\notepad.exe\" \"%1\"")
        .map_err(|e| format!("写入打开命令失败：{e}"))?;

    let (txtfile, _) = hkcr
        .create_subkey("txtfile")
        .map_err(|e| format!("无法打开 HKCR\\txtfile：{e}"))?;
    let _ = txtfile.set_value("", &"文本文档");

    let (icon_key, _) = txtfile
        .create_subkey("DefaultIcon")
        .map_err(|e| format!("无法写入图标项：{e}"))?;
    icon_key
        .set_value("", &"%SystemRoot%\\system32\\imageres.dll,-102")
        .map_err(|e| format!("写入图标失败：{e}"))?;

    clean_user_choice();

    Ok(())
}

/// 清除资源管理器记住的 .txt 打开方式。
fn clean_user_choice() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.txt";
    if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_ALL_ACCESS) {
        let _ = key.delete_subkey_all("UserChoice");
        let _ = key.delete_subkey_all("OpenWithList");
    }
}

/// 设置新建文本文档的默认编码。
pub fn set_default_encoding(encoding: &str) -> Result<(), String> {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let shell_new_path = r".txt\ShellNew";

    if encoding == "ANSI" {
        let (key, _) = hkcr
            .create_subkey(shell_new_path)
            .map_err(|e| format!("无法打开 .txt\\ShellNew：{e}"))?;
        key.set_value("NullFile", &"")
            .map_err(|e| format!("写入 NullFile 失败：{e}"))?;
        let _ = key.delete_value("FileName");
        return Ok(());
    }

    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let shell_new_dir = PathBuf::from(&system_root).join("ShellNew");
    if !shell_new_dir.exists() {
        fs::create_dir_all(&shell_new_dir).map_err(|e| format!("无法创建 ShellNew 目录：{e}"))?;
    }

    let template_path = shell_new_dir.join("txtfile.txt");
    let content: &[u8] = match encoding {
        "UTF-16 LE" => &[0xFF, 0xFE],
        "UTF-16 BE" => &[0xFE, 0xFF],
        "带有BOM的UTF-8" => &[0xEF, 0xBB, 0xBF],
        _ => &[],
    };
    fs::write(&template_path, content).map_err(|e| format!("写入模板文件失败：{e}"))?;

    let (key, _) = hkcr
        .create_subkey(shell_new_path)
        .map_err(|e| format!("无法打开 .txt\\ShellNew：{e}"))?;
    let _ = key.delete_value("NullFile");
    key.set_value("FileName", &"txtfile.txt")
        .map_err(|e| format!("写入 FileName 失败：{e}"))?;

    Ok(())
}

/// 通知外壳刷新文件关联。
pub fn refresh_shell() {
    unsafe {
        SHChangeNotify(
            SHCNE_ASSOCCHANGED as i32,
            SHCNF_IDLIST as u32,
            ptr::null(),
            ptr::null(),
        );
    }
}

/// 重启资源管理器，让右键菜单立即生效。
pub fn restart_explorer() {
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "explorer.exe"])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    std::thread::sleep(std::time::Duration::from_millis(500));
    let _ = Command::new("explorer.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}
