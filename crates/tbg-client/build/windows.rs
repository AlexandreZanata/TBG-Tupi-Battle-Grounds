//! Embed reviewed icons, version identity and least-privilege metadata in MSVC executables.
use std::{env, fs, path::PathBuf, process::Command};

pub fn resources() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows")
        || env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc")
    {
        return;
    }
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let icon = root.join("assets/branding/tbg.ico");
    let manifest = root.join("../../deploy/native/windows/app.manifest");
    println!("cargo:rerun-if-changed={}", icon.display());
    println!("cargo:rerun-if-changed={}", manifest.display());
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let mut numbers: Vec<_> = version
        .split(['.', '-', '+'])
        .take(3)
        .map(|part| part.parse::<u16>().expect("numeric package version"))
        .collect();
    assert_eq!(numbers.len(), 3);
    numbers.push(0);
    let numeric = numbers
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let windows_path = |path: &std::path::Path| path.to_string_lossy().replace('\\', "\\\\");
    let rc = format!(
        r#"#include <windows.h>
1 ICON "{icon}"
1 RT_MANIFEST "{manifest}"
1 VERSIONINFO
FILEVERSION {numeric}
PRODUCTVERSION {numeric}
FILEOS VOS_NT_WINDOWS32
FILETYPE VFT_APP
BEGIN
 BLOCK "StringFileInfo"
 BEGIN
  BLOCK "040904B0"
  BEGIN
   VALUE "CompanyName", "Alexandre Zanata\0"
   VALUE "FileDescription", "Tupi Battle Grounds\0"
   VALUE "FileVersion", "{version}\0"
   VALUE "ProductName", "TBG\0"
   VALUE "ProductVersion", "{version}\0"
   VALUE "OriginalFilename", "tbg-client.exe\0"
   VALUE "LegalCopyright", "Alexandre Zanata - github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds\0"
  END
 END
 BLOCK "VarFileInfo"
 BEGIN
  VALUE "Translation", 0x409, 1200
 END
END
"#,
        icon = windows_path(&icon),
        manifest = windows_path(&manifest)
    );
    let source = output.join("tbg.rc");
    let resource = output.join("tbg.res");
    fs::write(&source, rc).expect("write Windows resources");
    let status = Command::new("rc.exe")
        .args(["/nologo", "/fo"])
        .arg(&resource)
        .arg(&source)
        .status()
        .expect("Windows SDK rc.exe is required for an official MSVC package");
    assert!(status.success(), "Windows resource compilation failed");
    println!("cargo:rustc-link-arg-bin=tbg-client={}", resource.display());
}
