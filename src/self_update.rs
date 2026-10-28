//! Verified background updates and a separate apply worker, after the running app exits.
use crate::{
    Project, Result, desktop,
    installation::{self, Options},
    updates,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[derive(Clone, Serialize, Default)]
pub struct Progress {
    pub phase: String,
    pub percent: u8,
    pub bytes: u64,
    pub total: u64,
    pub error: Option<String>,
}
#[derive(Clone)]
pub struct Manager {
    inner: Arc<Mutex<Job>>,
}
#[derive(Default)]
struct Job {
    progress: Progress,
    stage: Option<PathBuf>,
    version: Option<String>,
    revision: Option<u32>,
}
#[derive(Serialize, Deserialize)]
struct Plan {
    product: String,
    root: PathBuf,
    parent: u32,
    version: String,
    revision: u32,
    test_mode: bool,
}
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn validate_session(value: &Value) -> Result<()> {
    let docs = value
        .get("documents")
        .and_then(Value::as_array)
        .ok_or("Sessão de atualização inválida.")?;
    if docs.is_empty() && value["active"].is_null() {
        return Ok(());
    }
    if docs.is_empty() || docs.len() > 3 {
        return Err("Sessão precisa ter de 1 a 3 projetos.".into());
    }
    let mut ids = std::collections::HashSet::new();
    for doc in docs {
        let id = doc["id"]
            .as_u64()
            .filter(|id| *id > 0)
            .ok_or("Identificador de projeto inválido.")?;
        if !ids.insert(id) {
            return Err("Projeto repetido na sessão.".into());
        }
        let state = &doc["state"];
        if !state.is_object() {
            return Err("Estado de projeto inválido.".into());
        }
        if !state["project"].is_null() {
            let project: Project =
                serde_json::from_value(state["project"].clone()).map_err(error)?;
            project.validate()?;
        }
    }
    if !ids.contains(&value["active"].as_u64().ok_or("Aba ativa inválida.")?) {
        return Err("Aba ativa ausente.".into());
    }
    Ok(())
}
fn options() -> Result<Options> {
    let exe = std::env::current_exe().map_err(error)?;
    let root = exe
        .parent()
        .ok_or("Pasta do programa não encontrada.")?
        .to_owned();
    if exe.file_name().and_then(|s| s.to_str()) != Some(installation::APP) {
        return Err(
            "A atualização interna requer o SkyLab instalado. Use o instalador inicial uma vez."
                .into(),
        );
    }
    let o = Options {
        root,
        desktop: false,
        test_mode: std::env::var("SKYLAB_UPDATE_TEST_MODE").as_deref() == Ok("1"),
    };
    if !o.installed() {
        return Err("Instalação SkyLab não reconhecida para atualização interna.".into());
    }
    Ok(o)
}
fn new_stage() -> Result<PathBuf> {
    let cache = cache_root();
    fs::create_dir_all(&cache).map_err(error)?;
    if fs::symlink_metadata(&cache)
        .map_err(error)?
        .file_type()
        .is_symlink()
    {
        return Err("Pasta de atualização não pode ser um link.".into());
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(error)?
        .as_nanos();
    let path = cache.join(format!("update-{}-{now}", std::process::id()));
    fs::create_dir(&path).map_err(error)?;
    Ok(path)
}
fn cache_root() -> PathBuf {
    std::env::var_os("SKYLAB_UPDATE_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("SkyTEM-SkyLab-Updates"))
}
fn owned_session(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()) == Some("session.json")
        && path.parent().is_some_and(|stage| {
            stage
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("update-"))
                && fs::canonicalize(stage).ok().is_some_and(|p| {
                    fs::canonicalize(cache_root())
                        .ok()
                        .is_some_and(|cache| p.parent() == Some(cache.as_path()))
                })
        })
}
impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}
impl Manager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Job {
                progress: Progress {
                    phase: "idle".into(),
                    ..Progress::default()
                },
                ..Job::default()
            })),
        }
    }
    pub fn progress(&self) -> Progress {
        self.inner.lock().unwrap().progress.clone()
    }
    pub fn start(&self) -> Result<()> {
        options()?;
        {
            let mut job = self.inner.lock().unwrap();
            if matches!(
                job.progress.phase.as_str(),
                "checking" | "downloading" | "ready" | "applying"
            ) {
                return Err("Já existe uma atualização em andamento.".into());
            }
            job.progress = Progress {
                phase: "checking".into(),
                ..Progress::default()
            };
        }
        let this = self.clone();
        std::thread::spawn(move || {
            if let Err(error) = this.download() {
                let mut job = this.inner.lock().unwrap();
                job.progress.phase = "error".into();
                job.progress.error = Some(error);
                if let Some(stage) = job.stage.take() {
                    for name in ["package.zip", "package.zip.part", installation::APP] {
                        let file = stage.join(name);
                        if fs::symlink_metadata(&file)
                            .is_ok_and(|m| m.is_file() && !m.file_type().is_symlink())
                        {
                            let _ = fs::remove_file(file);
                        }
                    }
                    let _ = fs::remove_dir(stage);
                }
            }
        });
        Ok(())
    }
    fn download(&self) -> Result<()> {
        let check = updates::check()?;
        if check.status != "available" {
            return Err("Nenhuma atualização posterior disponível.".into());
        }
        let url = check
            .package_url
            .ok_or("Esta publicação não tem pacote de atualização interna.")?;
        let hash = check
            .package_sha256
            .filter(|s| updates::valid_sha256(s))
            .ok_or("Pacote sem verificação de integridade.")?;
        if !url.starts_with("https://github.com/JDantanna/SkyLab/releases/download/")
            || !updates::safe_https(&url)
        {
            return Err("Origem do pacote de atualização não autorizada.".into());
        }
        let stage = new_stage()?;
        {
            let mut job = self.inner.lock().unwrap();
            job.stage = Some(stage.clone());
            job.version = check.latest_version;
            job.revision = check.latest_revision;
            job.progress.phase = "downloading".into();
        }
        #[cfg(windows)]
        {
            let mut command = desktop::host_command()?;
            let test_package = if std::env::var("SKYLAB_UPDATE_TEST_MODE").as_deref() == Ok("1") {
                std::env::var_os("SKYLAB_UPDATE_TEST_PACKAGE")
            } else {
                None
            };
            if let Some(package) = test_package {
                command.arg("--unpack-update").arg(package);
            } else {
                command.args(["--download-update", &url]);
            }
            command
                .arg(&hash)
                .arg(&stage)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let mut child = desktop::hidden(&mut command).spawn().map_err(error)?;
            for line in
                BufReader::new(child.stdout.take().ok_or("Sem progresso do download.")?).lines()
            {
                let value: Value = serde_json::from_str(&line.map_err(error)?).map_err(error)?;
                let mut job = self.inner.lock().unwrap();
                job.progress.bytes = value["bytes"].as_u64().unwrap_or(0);
                job.progress.total = value["total"].as_u64().unwrap_or(0);
                job.progress.percent = value["percent"].as_u64().unwrap_or(0).min(95) as u8;
            }
            let output = child.wait_with_output().map_err(error)?;
            if !output.status.success() {
                return Err(format!(
                    "Não foi possível preparar a atualização: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
        #[cfg(not(windows))]
        {
            let _ = (&stage, &hash);
            return Err("Atualização interna disponível no Windows.".into());
        }
        #[cfg(windows)]
        {
            if !stage.join(installation::APP).is_file() {
                return Err("Aplicativo ausente no pacote.".into());
            }
            let output = desktop::hidden(
                Command::new(stage.join(installation::APP)).arg("--update-identity"),
            )
            .output()
            .map_err(error)?;
            if !output.status.success() {
                return Err("O aplicativo do pacote não iniciou corretamente.".into());
            }
            let identity: Value = serde_json::from_slice(&output.stdout)
                .map_err(|_| "Identidade do aplicativo inválida.")?;
            let expected_version = self
                .inner
                .lock()
                .unwrap()
                .version
                .clone()
                .ok_or("Versão ausente.")?;
            if identity["product"] != installation::PRODUCT
                || identity["revision"].as_u64() != check.latest_revision.map(u64::from)
                || updates::version(
                    identity["version"]
                        .as_str()
                        .ok_or("Versão inválida no aplicativo.")?,
                )? != updates::version(&expected_version)?
            {
                return Err("O aplicativo não corresponde à versão publicada.".into());
            }
            let mut job = self.inner.lock().unwrap();
            job.progress.phase = "ready".into();
            job.progress.percent = 95;
            Ok(())
        }
    }
    pub fn apply(&self, session: Value) -> Result<()> {
        validate_session(&session)?;
        let o = options()?;
        let mut job = self.inner.lock().unwrap();
        if job.progress.phase != "ready" {
            return Err("A atualização ainda não está pronta.".into());
        }
        let stage = job.stage.as_ref().ok_or("Pacote ausente.")?;
        let plan = Plan {
            product: installation::PRODUCT.into(),
            root: o.root,
            parent: std::process::id(),
            version: job.version.clone().ok_or("Versão ausente.")?,
            revision: job.revision.ok_or("Revisão ausente.")?,
            test_mode: o.test_mode,
        };
        fs::write(
            stage.join("session.json"),
            serde_json::to_vec(&session).map_err(error)?,
        )
        .map_err(error)?;
        fs::write(
            stage.join("plan.json"),
            serde_json::to_vec(&plan).map_err(error)?,
        )
        .map_err(error)?;
        desktop::hidden(
            Command::new(stage.join(installation::APP))
                .arg("--apply-update")
                .arg(stage.join("plan.json"))
                .stdout(Stdio::null())
                .stderr(Stdio::null()),
        )
        .spawn()
        .map_err(error)?;
        job.progress.phase = "applying".into();
        job.progress.percent = 100;
        Ok(())
    }
}
#[cfg(windows)]
fn wait_parent(pid: u32) -> Result<()> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
        fn WaitForSingleObject(handle: *mut std::ffi::c_void, ms: u32) -> u32;
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        fn GetLastError() -> u32;
    }
    if pid == 0 || pid == std::process::id() {
        return Err("Processo de atualização inválido.".into());
    }
    // The handle identifies the original parent even if Windows later reuses its PID.
    unsafe {
        let handle = OpenProcess(0x00100000, 0, pid);
        if handle.is_null() {
            return if GetLastError() == 87 {
                Ok(())
            } else {
                Err("Não foi possível aguardar o programa encerrar.".into())
            };
        }
        let status = WaitForSingleObject(handle, 30_000);
        CloseHandle(handle);
        if status != 0 {
            return Err("O programa não encerrou. A atualização não foi aplicada.".into());
        }
    }
    Ok(())
}
#[cfg(not(windows))]
fn wait_parent(_: u32) -> Result<()> {
    Err("Atualização disponível no Windows.".into())
}
pub fn apply_plan(path: &Path) -> Result<()> {
    let stage = path.parent().ok_or("Plano inválido.")?;
    let plan: Plan = serde_json::from_slice(&read_limited(path, 16384)?).map_err(error)?;
    if plan.product != installation::PRODUCT
        || plan.version != updates::display_version() && plan.version != env!("CARGO_PKG_VERSION")
        || plan.revision != updates::build_revision()
    {
        return Err("A versão do pacote não corresponde à publicação.".into());
    }
    let source = std::env::current_exe().map_err(error)?;
    if fs::canonicalize(&source).map_err(error)?
        != fs::canonicalize(stage.join(installation::APP)).map_err(error)?
    {
        return Err("Executável fora do pacote de atualização.".into());
    }
    let session = load_session(&stage.join("session.json"))?;
    let _ = session;
    let result = wait_parent(plan.parent).and_then(|()| {
        installation::replace_application(
            &Options {
                root: plan.root.clone(),
                desktop: false,
                test_mode: plan.test_mode,
            },
            &source,
        )
    });
    let mut command = Command::new(plan.root.join(installation::APP));
    command
        .arg("--restore-session")
        .arg(stage.join("session.json"));
    if plan.test_mode {
        command.args(["--no-open", "--port", "0"]);
    }
    let log = fs::File::create(stage.join("restart.log")).map_err(error)?;
    command.stdout(Stdio::from(log));
    let restarted = desktop::hidden(&mut command).spawn().map_err(error)?;
    fs::write(
        stage.join("result.json"),
        serde_json::to_vec(
            &json!({"ok":result.is_ok(),"error":result.as_ref().err(),"pid":restarted.id()}),
        )
        .map_err(error)?,
    )
    .map_err(error)?;
    result
}
fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    fs::File::open(path)
        .map_err(error)?
        .take(limit + 1)
        .read_to_end(&mut data)
        .map_err(error)?;
    if data.len() as u64 > limit {
        return Err("Sessão de atualização muito grande.".into());
    }
    Ok(data)
}
pub fn load_session(path: &Path) -> Result<Value> {
    if !owned_session(path) {
        return Err("Sessão fora da pasta de atualização reconhecida.".into());
    }
    let value: Value =
        serde_json::from_slice(&read_limited(path, 40 * 1024 * 1024)?).map_err(error)?;
    validate_session(&value)?;
    Ok(value)
}
pub fn finish_restore(path: PathBuf) {
    if !owned_session(&path) {
        return;
    }
    std::thread::spawn(move || {
        let Some(stage) = path.parent() else {
            return;
        };
        let Ok(plan_bytes) = read_limited(&stage.join("plan.json"), 16384) else {
            return;
        };
        let Ok(plan) = serde_json::from_slice::<Plan>(&plan_bytes) else {
            return;
        };
        if plan.product != installation::PRODUCT {
            return;
        }
        for _ in 0..10 {
            for name in [
                "package.zip",
                "package.zip.part",
                installation::APP,
                "session.json",
                "plan.json",
                "result.json",
                "restart.log",
            ] {
                let file = stage.join(name);
                if fs::symlink_metadata(&file)
                    .is_ok_and(|m| m.is_file() && !m.file_type().is_symlink())
                {
                    let _ = fs::remove_file(file);
                }
            }
            if fs::remove_dir(stage).is_ok() {
                return;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}
