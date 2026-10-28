use crate::Result;
use serde::{Deserialize, Serialize};
use std::{fs, io::Read, path::Path};
pub fn display_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
        .strip_suffix(".0")
        .unwrap_or(env!("CARGO_PKG_VERSION"))
}
#[derive(Deserialize)]
struct Config {
    manifest: String,
}
#[derive(Deserialize)]
struct Manifest {
    version: String,
    revision: Option<u32>,
    download_url: Option<String>,
    notes: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct Check {
    pub status: String,
    pub current_version: String,
    pub current_revision: u32,
    pub latest_version: Option<String>,
    pub latest_revision: Option<u32>,
    pub download_url: Option<String>,
    pub notes: Option<String>,
    pub message: String,
}
pub fn build_revision() -> u32 {
    env!("SKYLAB_BUILD_REVISION")
        .parse()
        .expect("validated build revision")
}
fn release_version(tag: &str) -> Result<(&str, u32)> {
    let tag = tag.strip_prefix('v').unwrap_or(tag);
    let (number, revision) = if let Some((number, revision)) = tag.split_once("-r") {
        if revision.is_empty() || !revision.bytes().all(|b| b.is_ascii_digit()) {
            return Err("Revisão de atualização inválida.".into());
        }
        (
            number,
            revision
                .parse::<u32>()
                .map_err(|_| "Revisão de atualização inválida.")?,
        )
    } else {
        (tag, 0)
    };
    version(number)?;
    Ok((number, revision))
}
fn version(s: &str) -> Result<[u32; 3]> {
    let mut n = [0; 3];
    let parts: Vec<_> = s.split('.').collect();
    if !(2..=3).contains(&parts.len()) {
        return Err("Versão do manifesto inválida.".into());
    }
    for (i, p) in parts.iter().enumerate() {
        if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
            return Err("Versão do manifesto inválida.".into());
        }
        n[i] = p.parse().map_err(|_| "Versão do manifesto inválida.")?;
    }
    Ok(n)
}
pub fn safe_https(s: &str) -> bool {
    s.starts_with("https://")
        && s.len() < 2048
        && !s
            .bytes()
            .any(|b| b.is_ascii_control() || b.is_ascii_whitespace() || b == b'"')
        && !s[8..].split('/').next().unwrap_or("").is_empty()
        && !s[8..].split('/').next().unwrap_or("").contains('@')
}
fn read(path: &Path) -> Result<String> {
    let mut text = String::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(1048577)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.len() > 1048576 {
        return Err("Manifesto de atualização muito grande.".into());
    }
    Ok(text)
}
pub fn check_at(path: &Path) -> Result<Check> {
    let current = display_version().to_owned();
    if !path.exists() {
        return Ok(Check {
            status: "not_configured".into(),
            current_version: current,
            current_revision: build_revision(),
            latest_version: None,
            latest_revision: None,
            download_url: None,
            notes: None,
            message: "Fonte de atualização não configurada.".into(),
        });
    }
    let config: Config = serde_json::from_str(&read(path)?)
        .map_err(|e| format!("Configuração de atualização inválida: {e}"))?;
    let text = if config.manifest.starts_with("https://") {
        if !safe_https(&config.manifest) {
            return Err("Endereço de atualização inválido.".into());
        }
        #[cfg(windows)]
        {
            let output = crate::desktop::hidden(
                crate::desktop::host_command()?.args(["--fetch-update", &config.manifest]),
            )
            .output()
            .map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err(format!(
                    "Não foi possível consultar a atualização: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            String::from_utf8(output.stdout).map_err(|_| "Resposta de atualização inválida")?
        }
        #[cfg(not(windows))]
        {
            return Err("Consulta HTTPS disponível no Windows.".into());
        }
    } else {
        if config.manifest.contains("://") {
            return Err("A fonte remota deve usar HTTPS.".into());
        }
        let source = Path::new(&config.manifest);
        read(&if source.is_absolute() {
            source.to_owned()
        } else {
            path.parent().unwrap_or(Path::new(".")).join(source)
        })?
    };
    let manifest: Manifest = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|e| format!("Manifesto inválido: {e}"))?;
    if manifest
        .download_url
        .as_ref()
        .is_some_and(|s| !safe_https(s))
    {
        return Err("O endereço para obter a atualização deve usar HTTPS.".into());
    }
    if manifest.notes.as_ref().is_some_and(|s| s.len() > 4096) {
        return Err("Notas de atualização muito extensas.".into());
    }
    let newer = (version(&manifest.version)?, manifest.revision.unwrap_or(0))
        > (version(env!("CARGO_PKG_VERSION"))?, build_revision());
    Ok(Check {
        status: if newer { "available" } else { "up_to_date" }.into(),
        current_version: current,
        current_revision: build_revision(),
        latest_version: Some(manifest.version),
        latest_revision: manifest.revision,
        download_url: manifest.download_url,
        notes: manifest.notes,
        message: if newer {
            "Há uma atualização disponível."
        } else {
            "Nenhuma versão posterior encontrada na fonte configurada."
        }
        .into(),
    })
}
pub fn config_path() -> std::path::PathBuf {
    std::env::var_os("SKYLAB_UPDATES_CONFIG")
        .map(Into::into)
        .unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap_or_default()
                .parent()
                .unwrap_or(Path::new("."))
                .join("updates.json")
        })
}
pub fn check() -> Result<Check> {
    let path = config_path();
    if std::env::var_os("SKYLAB_UPDATES_CONFIG").is_some() || path.exists() {
        return check_at(&path);
    }
    #[cfg(windows)]
    {
        let output = crate::desktop::hidden(crate::desktop::host_command()?.args([
            "--fetch-release",
            "https://api.github.com/repos/JDantanna/SkyLab/releases/latest",
        ]))
        .output()
        .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "Não foi possível consultar o GitHub: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        github_release(
            std::str::from_utf8(&output.stdout).map_err(|_| "Resposta de atualização inválida")?,
        )
    }
    #[cfg(not(windows))]
    {
        Err("Consulta HTTPS disponível no Windows.".into())
    }
}

/// Only published stable releases with an installer hosted by this repository are offered.
pub fn github_release(text: &str) -> Result<Check> {
    #[derive(Deserialize)]
    struct Asset {
        name: String,
        state: String,
        browser_download_url: String,
    }
    #[derive(Deserialize)]
    struct Release {
        #[serde(default)]
        no_release: bool,
        #[serde(default)]
        draft: bool,
        #[serde(default)]
        prerelease: bool,
        tag_name: Option<String>,
        body: Option<String>,
        #[serde(default)]
        assets: Vec<Asset>,
    }
    let release: Release =
        serde_json::from_str(text).map_err(|e| format!("Resposta do GitHub inválida: {e}"))?;
    let mut result = Check {
        status: "no_release".into(),
        current_version: display_version().into(),
        current_revision: build_revision(),
        latest_version: None,
        latest_revision: None,
        download_url: None,
        notes: None,
        message: "Nenhuma versão publicada no GitHub.".into(),
    };
    if release.no_release || release.draft || release.prerelease {
        return Ok(result);
    }
    let tag = release
        .tag_name
        .ok_or("Versão ausente na resposta do GitHub.")?;
    let (latest, revision) = release_version(&tag)?;
    let newer =
        (version(latest)?, revision) > (version(env!("CARGO_PKG_VERSION"))?, build_revision());
    result.latest_version = Some(latest.into());
    result.latest_revision = Some(revision);
    result.notes = release.body.map(|s| s.chars().take(4096).collect());
    result.download_url = release
        .assets
        .into_iter()
        .find(|asset| {
            asset.state == "uploaded"
                && asset.name.starts_with("SkyTEM-SkyLab-Setup-")
                && asset.name.ends_with(".exe")
                && safe_https(&asset.browser_download_url)
                && asset
                    .browser_download_url
                    .starts_with("https://github.com/JDantanna/SkyLab/releases/download/")
        })
        .map(|asset| asset.browser_download_url);
    let (status, message) = if !newer {
        (
            "up_to_date",
            "Nenhuma versão posterior publicada no GitHub.",
        )
    } else if result.download_url.is_some() {
        ("available", "Há uma atualização disponível.")
    } else {
        (
            "release_pending",
            "A versão foi publicada, mas o instalador ainda não está disponível.",
        )
    };
    result.status = status.into();
    result.message = message.into();
    Ok(result)
}
