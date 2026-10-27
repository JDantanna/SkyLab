#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::Command;

pub fn hidden(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000);
    }
    cmd
}

/// Open the existing local UI in a separate Edge application window, with browser fallback.
pub fn open(url: &str) {
    #[cfg(windows)]
    {
        for env in ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"] {
            if let Some(root) = std::env::var_os(env) {
                let edge =
                    std::path::PathBuf::from(root).join("Microsoft/Edge/Application/msedge.exe");
                if edge.is_file()
                    && hidden(
                        Command::new(edge)
                            .arg(format!("--app={url}"))
                            .arg("--window-size=1380,920"),
                    )
                    .spawn()
                    .is_ok()
                {
                    return;
                }
            }
        }
        let _ = hidden(Command::new("explorer.exe").arg(url)).spawn();
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("xdg-open").arg(url).spawn();
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
