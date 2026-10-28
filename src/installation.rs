//! Per-user installation. Only explicitly owned product files are replaced or removed.
use crate::{Result, desktop};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};
pub const PRODUCT: &str = "skytem.skylab.local";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP: &str = "SkyLab-Rust.exe";
pub const UNINSTALL: &str = "Uninstall.exe";
#[derive(Clone)]
pub struct Options {
    pub root: PathBuf,
    pub desktop: bool,
    pub test_mode: bool,
}
#[derive(Serialize, Deserialize)]
struct Manifest {
    product: String,
    version: String,
    root: PathBuf,
}
impl Options {
    pub fn current_user() -> Result<Self> {
        let local = std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA não está disponível")?;
        Ok(Self {
            root: PathBuf::from(local)
                .join("Programs")
                .join("SkyTEM")
                .join("SkyLab"),
            desktop: true,
            test_mode: false,
        })
    }
    pub fn test(root: PathBuf) -> Self {
        Self {
            root,
            desktop: true,
            test_mode: true,
        }
    }
    pub fn installed(&self) -> bool {
        read_manifest(&self.root).is_ok() && self.root.join(APP).is_file()
    }
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn read_manifest(root: &Path) -> Result<Manifest> {
    let m: Manifest =
        serde_json::from_slice(&fs::read(root.join("installation.json")).map_err(err)?)
            .map_err(err)?;
    if m.product != PRODUCT
        || fs::canonicalize(&m.root).map_err(err)? != fs::canonicalize(root).map_err(err)?
    {
        return Err("A pasta não pertence a uma instalação SkyLab reconhecida.".into());
    }
    Ok(m)
}
fn safe_root(o: &Options, existing: bool) -> Result<()> {
    if !o.root.is_absolute() || o.root.parent().is_none() {
        return Err("Destino de instalação inválido.".into());
    }
    if let Ok(meta) = fs::symlink_metadata(&o.root) {
        if meta.file_type().is_symlink() || !meta.is_dir() {
            return Err("O destino não pode ser um link ou arquivo.".into());
        }
        let nonempty = fs::read_dir(&o.root).map_err(err)?.next().is_some();
        if existing || nonempty {
            read_manifest(&o.root)?;
        }
    } else if existing {
        return Err("Instalação não encontrada.".into());
    }
    for file in [
        APP,
        UNINSTALL,
        "installation.json",
        "THIRD-PARTY-NOTICES.txt",
        "SkyLab-Rust.previous.exe",
        "SkyLab-Rust.new.exe",
        "Uninstall.new.exe",
    ] {
        if let Ok(meta) = fs::symlink_metadata(o.root.join(file))
            && (meta.file_type().is_symlink() || !meta.is_file())
        {
            return Err(
                "Um arquivo do produto foi substituído por link ou diretório; operação recusada."
                    .into(),
            );
        }
    }
    Ok(())
}
fn run_script(script: &str, o: &Options) -> Result<()> {
    #[cfg(windows)]
    {
        let output = desktop::hidden(
            Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", script])
                .env("SKYLAB_INSTALL_ROOT", &o.root)
                .env("SKYLAB_VERSION", crate::updates::display_version())
                .env("SKYLAB_DESKTOP", if o.desktop { "1" } else { "0" })
                .env("SKYLAB_TEST_MODE", if o.test_mode { "1" } else { "0" }),
        )
        .output()
        .map_err(err)?;
        if !output.status.success() {
            return Err(format!(
                "Não foi possível registrar os atalhos: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = (script, o);
        Err("Instalação disponível somente no Windows.".into())
    }
}
pub fn install(o: &Options, app: &[u8], setup: &[u8]) -> Result<()> {
    install_with_progress(o, app, setup, |_, _| {})
}
pub fn install_with_progress<F: Fn(&str, u8)>(
    o: &Options,
    app: &[u8],
    setup: &[u8],
    progress: F,
) -> Result<()> {
    install_source(o, app, SetupSource::Bytes(setup), progress)
}
/// Copy the running setup image directly, without allocating another setup-sized buffer.
pub fn install_from_executable<F: Fn(&str, u8)>(
    o: &Options,
    app: &[u8],
    setup: &Path,
    progress: F,
) -> Result<()> {
    install_source(o, app, SetupSource::File(setup), progress)
}
enum SetupSource<'a> {
    Bytes(&'a [u8]),
    File(&'a Path),
}
impl SetupSource<'_> {
    fn validate(&self) -> Result<()> {
        let valid = match self {
            Self::Bytes(bytes) => bytes.starts_with(b"MZ"),
            Self::File(path) => {
                let mut magic = [0; 2];
                fs::File::open(path)
                    .map_err(err)?
                    .read_exact(&mut magic)
                    .map_err(err)?;
                magic == *b"MZ"
            }
        };
        if valid {
            Ok(())
        } else {
            Err("Pacote de instalação inválido.".into())
        }
    }
    fn copy_to(&self, destination: &Path) -> Result<()> {
        match self {
            Self::Bytes(bytes) => fs::write(destination, bytes).map_err(err),
            Self::File(path) => fs::copy(path, destination).map(|_| ()).map_err(err),
        }
    }
}
fn install_source<F: Fn(&str, u8)>(
    o: &Options,
    app: &[u8],
    setup: SetupSource<'_>,
    progress: F,
) -> Result<()> {
    progress("Verificando instalação", 5);
    safe_root(o, false)?;
    check_app_closed(o)?;
    if !app.starts_with(b"MZ") {
        return Err("Pacote de instalação inválido.".into());
    }
    setup.validate()?;
    progress("Preparando pasta", 15);
    fs::create_dir_all(&o.root).map_err(err)?;
    // Ownership is recorded before copying so an interrupted installation can be repaired.
    let manifest = Manifest {
        product: PRODUCT.into(),
        version: VERSION.into(),
        root: o.root.clone(),
    };
    fs::write(
        o.root.join("installation.json"),
        serde_json::to_vec_pretty(&manifest).map_err(err)?,
    )
    .map_err(err)?;
    let target = o.root.join(APP);
    let new = o.root.join("SkyLab-Rust.new.exe");
    let previous = o.root.join("SkyLab-Rust.previous.exe");
    progress("Copiando aplicativo", 35);
    fs::write(&new, app).map_err(err)?;
    if previous.exists() {
        fs::remove_file(&previous).map_err(err)?;
    }
    if target.exists() {
        fs::rename(&target, &previous)
            .map_err(|_| "Feche o SkyLab antes de atualizar e tente novamente.".to_string())?;
    }
    if let Err(e) = fs::rename(&new, &target) {
        if previous.exists() {
            let _ = fs::rename(&previous, &target);
        }
        return Err(err(e));
    }
    progress("Preparando desinstalador", 60);
    fs::write(
        o.root.join("THIRD-PARTY-NOTICES.txt"),
        include_bytes!("../assets/THIRD-PARTY-NOTICES.txt"),
    )
    .map_err(err)?;
    let uninstaller = o.root.join(UNINSTALL);
    let unew = o.root.join("Uninstall.new.exe");
    setup.copy_to(&unew)?;
    if uninstaller.exists() {
        fs::remove_file(&uninstaller)
            .map_err(|_| "Feche a janela de desinstalação antes de atualizar.".to_string())?;
    }
    fs::rename(unew, uninstaller).map_err(err)?;
    progress("Registrando aplicativo e atalhos", 80);
    run_script(include_str!("../scripts/register.ps1"),o).map_err(|e|format!("Arquivos instalados, mas o registro não terminou. Execute o instalador novamente para reparar. {e}"))?;
    progress("Instalação concluída", 100);
    Ok(())
}
pub fn uninstall(o: &Options) -> Result<()> {
    uninstall_with_progress(o, |_, _| {})
}
pub fn uninstall_with_progress<F: Fn(&str, u8)>(o: &Options, progress: F) -> Result<()> {
    progress("Verificando instalação", 5);
    safe_root(o, true)?;
    check_app_closed(o)?;
    progress("Removendo atalhos e registro", 25);
    run_script(include_str!("../scripts/unregister.ps1"), o)?;
    progress("Removendo arquivos do aplicativo", 50);
    for name in [
        APP,
        "SkyLab-Rust.previous.exe",
        "SkyLab-Rust.new.exe",
        "Uninstall.new.exe",
        "THIRD-PARTY-NOTICES.txt",
    ] {
        let p = o.root.join(name);
        if p.exists() {
            fs::remove_file(p)
                .map_err(|_| "Feche o SkyLab e tente remover novamente.".to_string())?;
        }
    }
    let own = o.root.join(UNINSTALL);
    let current = std::env::current_exe().map_err(err)?;
    if own.exists()
        && fs::canonicalize(&own).map_err(err)? == fs::canonicalize(&current).map_err(err)?
    {
        // The running uninstaller is deleted after it exits; no recursive directory deletion.
        let script = "$ErrorActionPreference='Stop'; Wait-Process -Id ([int]$env:SKYLAB_PARENT_PID) -ErrorAction SilentlyContinue; $ownedFile=Join-Path $env:SKYLAB_INSTALL_ROOT 'Uninstall.exe'; for($attempt=0;$attempt -lt 60;$attempt++){try{if(Test-Path -LiteralPath $ownedFile){Remove-Item -LiteralPath $ownedFile -Force};break}catch{Start-Sleep -Milliseconds 500}}; if (-not (Get-ChildItem -LiteralPath $env:SKYLAB_INSTALL_ROOT -Force | Select-Object -First 1)) { Remove-Item -LiteralPath $env:SKYLAB_INSTALL_ROOT }";
        desktop::hidden(
            Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", script])
                .env("SKYLAB_INSTALL_ROOT", &o.root)
                .env("SKYLAB_PARENT_PID", std::process::id().to_string()),
        )
        .spawn()
        .map_err(err)?;
    } else if own.exists() {
        fs::remove_file(own).map_err(err)?;
    }
    progress("Finalizando remoção", 90);
    fs::remove_file(o.root.join("installation.json")).map_err(err)?;
    let _ = fs::remove_dir(&o.root); // Keep any user-created or unknown files.
    progress("Remoção concluída", 100);
    Ok(())
}

fn check_app_closed(o: &Options) -> Result<()> {
    let app = o.root.join(APP);
    if app.exists() {
        fs::OpenOptions::new().write(true).open(app).map_err(|_|"Feche o SkyLab antes de continuar; verifique também a permissão de escrita na pasta.".to_owned())?;
    }
    Ok(())
}
/// Update only the owned application. Shortcuts, uninstaller and personal files stay in place.
pub fn replace_application(o: &Options, source: &Path) -> Result<()> {
    safe_root(o, true)?;
    check_app_closed(o)?;
    let old_manifest = fs::read(o.root.join("installation.json")).map_err(err)?;
    let old = read_manifest(&o.root)?;
    let mut magic = [0; 2];
    fs::File::open(source)
        .map_err(err)?
        .read_exact(&mut magic)
        .map_err(err)?;
    if magic != *b"MZ" {
        return Err("Pacote de atualização inválido.".into());
    }
    let target = o.root.join(APP);
    let new = o.root.join("SkyLab-Rust.new.exe");
    let previous = o.root.join("SkyLab-Rust.previous.exe");
    fs::copy(source, &new).map_err(err)?;
    if previous.exists() {
        fs::remove_file(&previous).map_err(err)?;
    }
    fs::rename(&target, &previous).map_err(err)?;
    let result = (|| {
        fs::rename(&new, &target).map_err(err)?;
        fs::write(
            o.root.join("installation.json"),
            serde_json::to_vec_pretty(&Manifest {
                product: PRODUCT.into(),
                version: VERSION.into(),
                root: o.root.clone(),
            })
            .map_err(err)?,
        )
        .map_err(err)?;
        run_script(include_str!("../scripts/update-registration.ps1"), o)
    })();
    if let Err(error) = result {
        if target.exists() {
            fs::remove_file(&target).map_err(err)?;
        }
        fs::rename(&previous, &target).map_err(err)?;
        fs::write(o.root.join("installation.json"), old_manifest).map_err(err)?;
        // Restore the previous display version even if registration failed partway through.
        #[cfg(windows)]
        {
            let _ = desktop::hidden(
                Command::new("powershell.exe")
                    .args([
                        "-NoProfile",
                        "-NonInteractive",
                        "-Command",
                        include_str!("../scripts/update-registration.ps1"),
                    ])
                    .env("SKYLAB_INSTALL_ROOT", &o.root)
                    .env(
                        "SKYLAB_VERSION",
                        old.version.strip_suffix(".0").unwrap_or(&old.version),
                    )
                    .env("SKYLAB_TEST_MODE", if o.test_mode { "1" } else { "0" }),
            )
            .output();
        }
        return Err(format!(
            "Atualização não aplicada; versão anterior restaurada. {error}"
        ));
    }
    Ok(())
}
