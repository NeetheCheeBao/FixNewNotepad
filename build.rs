use std::path::{Path, PathBuf};

const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*"/>
    </dependentAssembly>
  </dependency>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">permonitorv2,permonitor</dpiAwareness>
    </windowsSettings>
  </application>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
</assembly>"#;

/// 读取 version.ini 中的版本号，缺失时使用 1.0.0。
fn read_version() -> String {
    let text = std::fs::read_to_string("version.ini").unwrap_or_default();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            if key.trim().eq_ignore_ascii_case("version") {
                let value = value.trim();
                if !value.is_empty() {
                    return value.to_string();
                }
            }
        }
    }
    "1.0.0".to_string()
}

/// 生成只含图标与清单的 .rc 脚本。
fn write_resource_script(out_dir: &Path) -> PathBuf {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let icon = root.join("assets").join("icon.ico");
    let icon = icon.to_string_lossy().replace('\\', "/");

    let mut rc = String::new();
    rc.push_str("#pragma code_page(65001)\n");
    rc.push_str(&format!("1 ICON \"{icon}\"\n"));
    rc.push_str("1 24\n{\n");
    for line in MANIFEST.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        rc.push_str(&format!("\" {} \"\n", line.replace('"', "\"\"")));
    }
    rc.push_str("}\n");

    let path = out_dir.join("app.rc");
    std::fs::write(&path, rc).expect("写入资源脚本失败");
    path
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=version.ini");

    // 版本号在编译期写入二进制，保证产物只有一个 exe
    let version = read_version();
    println!("cargo:rustc-env=FNN_VERSION={version}");

    let is_windows = std::env::var("TARGET")
        .map(|target| target.contains("windows"))
        .unwrap_or(false);
    if !is_windows {
        return;
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("缺少 OUT_DIR"));
    let script = write_resource_script(&out_dir);

    let mut res = winres::WindowsResource::new();
    res.set_resource_file(script.to_str().expect("资源脚本路径非 UTF-8"));
    if let Err(err) = res.compile() {
        panic!("编译 Windows 资源失败: {err}");
    }
}
