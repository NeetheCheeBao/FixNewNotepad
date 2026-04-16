fn main() {
    slint_build::compile("src/main.slint").expect("Slint build failed");

    if std::env::var("TARGET").unwrap().contains("windows") {
        let mut res = winres::WindowsResource::new();
        res.set_version_info(winres::VersionInfo::FILEVERSION, 0x0001000000000000);
        res.set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0001000000000000);
        res.set("FileDescription", "FixNewNotepad");
        res.set("ProductName", "FixNewNotepad");
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#);
        res.compile().unwrap();
    }
}