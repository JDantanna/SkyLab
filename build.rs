use std::{env, fs, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=assets/app.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=native/vendor/proj");
    let mut files = String::from("{\n");
    for entry in fs::read_dir(root.join("native/vendor/proj")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy();
            let path = path.to_string_lossy().replace('\\', "/");
            files.push_str(&format!("{{ let bytes=include_bytes!({path:?}); let file=dir.join({name:?}); if std::fs::read(&file).ok().as_deref()!=Some(bytes.as_slice()) {{ std::fs::write(file,bytes).map_err(|e|e.to_string())?; }} }}\n"));
        }
    }
    files.push_str("}\n");
    fs::write(out.join("proj_files.rs"), files).unwrap();
    println!("cargo:rerun-if-changed=native/Host.cs");
    println!("cargo:rerun-if-changed=native/vendor/webview2");
    let sdk = root.join("native/vendor/webview2");
    let win = |path: PathBuf| path.to_string_lossy().replace('/', "\\");
    let csc = PathBuf::from(env::var_os("WINDIR").unwrap())
        .join("Microsoft.NET/Framework64/v4.0.30319/csc.exe");
    let output = Command::new(csc)
        .args([
            "/nologo",
            "/target:winexe",
            "/platform:x64",
            "/optimize+",
            "/codepage:65001",
            "/warnaserror+",
        ])
        .arg(format!("/out:{}", win(out.join("SkyLab.Window.exe"))))
        .arg(format!("/win32icon:{}", win(root.join("assets/app.ico"))))
        .args([
            "/reference:System.Windows.Forms.dll",
            "/reference:System.Drawing.dll",
            "/reference:System.Net.Http.dll",
            "/reference:System.Core.dll",
        ])
        .arg(format!(
            "/reference:{}",
            win(sdk.join("Microsoft.Web.WebView2.Core.dll"))
        ))
        .arg(format!(
            "/reference:{}",
            win(sdk.join("Microsoft.Web.WebView2.WinForms.dll"))
        ))
        .arg(win(root.join("native/Host.cs")))
        .output()
        .expect("Compilador .NET Framework necessário");
    assert!(
        output.status.success(),
        "Falha ao compilar a janela WebView2: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let display_version = version.strip_suffix(".0").unwrap_or(&version);
    let numbers = version
        .split('.')
        .map(|s| s.parse::<u16>().unwrap_or(0).to_string())
        .collect::<Vec<_>>()
        .join(",");
    let icon = root
        .join("assets/app.ico")
        .to_string_lossy()
        .replace('\\', "/");
    let rc = format!(
        r#"1 ICON "{icon}"
1 VERSIONINFO
FILEVERSION {numbers},0
PRODUCTVERSION {numbers},0
FILEFLAGSMASK 0x3fL
FILEFLAGS 0
FILEOS 0x40004L
FILETYPE 0x1L
BEGIN
 BLOCK "StringFileInfo"
 BEGIN
  BLOCK "040904b0"
  BEGIN
   VALUE "CompanyName", "SkyTEM - uso interno\0"
   VALUE "FileDescription", "SkyTEM SkyLab\0"
   VALUE "FileVersion", "{display_version}\0"
   VALUE "ProductName", "SkyTEM SkyLab\0"
   VALUE "ProductVersion", "{display_version}\0"
  END
 END
 BLOCK "VarFileInfo"
 BEGIN
  VALUE "Translation", 0x409, 1200
 END
END
"#
    );
    let input = out.join("skylab.rc");
    let result = out.join("skylab.res");
    fs::write(&input, rc).unwrap();
    let kits = PathBuf::from(env::var_os("ProgramFiles(x86)").expect("Windows SDK necessário"))
        .join("Windows Kits/10/bin");
    let mut versions: Vec<_> = fs::read_dir(kits)
        .expect("Windows SDK não encontrado")
        .filter_map(|e| e.ok())
        .map(|e| e.path().join("x64/rc.exe"))
        .filter(|p| p.is_file())
        .collect();
    versions.sort();
    let compiler = versions
        .last()
        .expect("rc.exe não encontrado no Windows SDK");
    let status = Command::new(compiler)
        .arg("/nologo")
        .arg("/fo")
        .arg(&result)
        .arg(input)
        .status()
        .expect("Falha ao executar rc.exe");
    assert!(
        status.success(),
        "Falha ao compilar ícone/versão do produto"
    );
    println!("cargo:rustc-link-arg-bins={}", result.display());
}
