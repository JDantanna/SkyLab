#![cfg_attr(windows, windows_subsystem = "windows")]
fn main() {
    if let Err(error) =
        skylab_rust::setup::run(Some(include_bytes!("../../payload/SkyLab-Rust.exe")))
    {
        skylab_rust::desktop::show_error(&error.to_string());
        std::process::exit(1);
    }
}
