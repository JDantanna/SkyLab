#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::{Child, Command};

pub fn hidden(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000);
    }
    cmd
}

/// Materialize the pinned Microsoft WebView2 SDK and our native window in a per-build cache.
#[cfg(windows)]
pub fn host_command() -> crate::Result<Command> {
    use std::{
        fs,
        hash::{Hash, Hasher},
        path::PathBuf,
    };
    let exe = include_bytes!(concat!(env!("OUT_DIR"), "/SkyLab.Window.exe"));
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    exe.hash(&mut hash);
    let root = std::env::var_os("SKYLAB_NATIVE_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("SkyTEM-SkyLab-Window"))
        .join(format!("{:x}", hash.finish()));
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let files: [(&str, &[u8]); 7] = [
        ("SkyLab.Window.exe", exe),
        (
            "Microsoft.Web.WebView2.Core.dll",
            include_bytes!("../native/vendor/webview2/Microsoft.Web.WebView2.Core.dll"),
        ),
        (
            "Microsoft.Web.WebView2.WinForms.dll",
            include_bytes!("../native/vendor/webview2/Microsoft.Web.WebView2.WinForms.dll"),
        ),
        (
            "WebView2Loader.dll",
            include_bytes!("../native/vendor/webview2/WebView2Loader.dll"),
        ),
        ("app.ico", include_bytes!("../assets/app.ico")),
        (
            "LICENSE-WebView2.txt",
            include_bytes!("../native/vendor/webview2/LICENSE.txt"),
        ),
        (
            "NOTICE-WebView2.txt",
            include_bytes!("../native/vendor/webview2/NOTICE.txt"),
        ),
    ];
    for (name, bytes) in files {
        let file = root.join(name);
        if fs::symlink_metadata(&file).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err("Arquivo de janela foi substituído por link.".into());
        }
        if fs::read(&file).ok().as_deref() != Some(bytes) {
            fs::write(&file, bytes).map_err(|e| e.to_string())?;
        }
    }
    Ok(Command::new(root.join("SkyLab.Window.exe")))
}

/// Open a real WebView2 window, whose native controls are integrated into the page header.
pub fn open(url: &str, setup: bool) -> Option<Child> {
    #[cfg(windows)]
    {
        let result = host_command().and_then(|mut command| {
            command.arg(url);
            if setup {
                command.arg("setup");
            }
            hidden(&mut command).spawn().map_err(|e| e.to_string())
        });
        match result {
            Ok(child) => Some(child),
            Err(e) => {
                show_error(&e);
                None
            }
        }
    }
    #[cfg(not(windows))]
    {
        Command::new("xdg-open").arg(url).spawn().ok()
    }
}

pub fn show_error(message: &str) {
    eprintln!("SkyLab: {message}");
    #[cfg(windows)]
    {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn MessageBoxW(
                hwnd: *mut std::ffi::c_void,
                text: *const u16,
                caption: *const u16,
                kind: u32,
            ) -> i32;
        }
        let text: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
        let title: Vec<u16> = "SkyTEM SkyLab".encode_utf16().chain(Some(0)).collect();
        // Both pointers reference NUL-terminated UTF-16 buffers alive through the call.
        unsafe {
            MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), 0x10);
        }
    }
}
