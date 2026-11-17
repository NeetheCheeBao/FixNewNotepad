<div align="center">
<img src="assets/icon.ico" width="128" alt="logo" />
<h1>FixNewNotepad</h1>

> 专为解决 Windows 11 系统中右键菜单“新建文本文档”丢失的问题，且支持还原经典记事本。

![Platform](https://img.shields.io/badge/Platform-Windows%2011-0078D4?style=for-the-badge&logo=windows11&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-2021-000000?style=for-the-badge&logo=rust&logoColor=white)
![UI](https://img.shields.io/badge/UI-Win32%20GUI-0F6CBD?style=for-the-badge)
![Portable](https://img.shields.io/badge/Portable-Single%20EXE-2EA043?style=for-the-badge&logo=files&logoColor=white)
![License](https://img.shields.io/badge/License-GPL%20v3.0-BD0000?style=for-the-badge&logo=gnu&logoColor=white)

</div>

## 📸 工具演示

| 主程序 |
| --- |
| ![demo](screenshot/demo1.png) |

## ⬇️ 下载发行版

[![Releases](https://img.shields.io/badge/Download-Releases-7C25FF?style=for-the-badge&logo=github&logoColor=white)](https://github.com/NeetheCheeBao/FixNewNotepad/releases)

## 🔧 本地构建

1. 克隆仓库

```bash
git clone https://github.com/NeetheCheeBao/FixNewNotepad.git
```

```bash
gh repo clone NeetheCheeBao/FixNewNotepad
```

2. 一键编译

```bash
.\build.bat
```

## 📂 项目结构

```text
FixNewNotepad
|   .gitignore
|   build.bat               # 一键编译脚本
|   build.rs                # 构建脚本：嵌入图标与清单、注入版本号
|   Cargo.lock              # 依赖版本锁定文件
|   Cargo.toml              # 项目元信息与依赖配置
|   LICENSE
|   README.md
|   version.ini             # 版本号配置
|
+---assets
|       icon.ico            # 图标
|
+---screenshot
|       demo1.png           # 截图
|
\---src
        fix.rs              # 注册表修复逻辑（新建菜单 / 关联 / 编码）
        main.rs             # 程序入口：窗口、菜单与控件
        win.rs              # Win32 辅助函数（宽字符、DPI、系统字体）
```

## ⚖️ 许可证

本项目采用 GNU General Public License v3.0 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件