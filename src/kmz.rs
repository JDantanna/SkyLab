//! Read only the KML inside an archive; never extract paths to the filesystem.
use crate::Result;
pub fn read(text: &str) -> Result<String> {
    read_archive(text, "--read-kmz")
}
pub fn read_qgz(text: &str) -> Result<String> {
    read_archive(text, "--read-qgz")
}
fn read_archive(text: &str, command_arg: &str) -> Result<String> {
    if text.len() > 11_184_812 {
        return Err("KMZ excede 8 MB.".into());
    }
    #[cfg(windows)]
    {
        use std::{io::Write, process::Stdio};
        let mut command = crate::desktop::host_command()?;
        command
            .arg(command_arg)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = crate::desktop::hidden(&mut command)
            .spawn()
            .map_err(|e| e.to_string())?;
        let input = child.stdin.take().ok_or("Não foi possível ler KMZ.")?;
        // Read stdout concurrently with the input pipe, avoiding a pipe deadlock on large archives.
        let data = text.as_bytes().to_vec();
        let writer = std::thread::spawn(move || {
            let mut input = input;
            input.write_all(&data)
        });
        let output = child.wait_with_output().map_err(|e| e.to_string())?;
        let written = writer.join().map_err(|_| "Leitura KMZ interrompida.")?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        written.map_err(|e| e.to_string())?;
        String::from_utf8(output.stdout).map_err(|_| "KML precisa usar UTF-8.".into())
    }
    #[cfg(not(windows))]
    {
        let _ = (text, command_arg);
        Err("Importação KMZ disponível no Windows.".into())
    }
}
