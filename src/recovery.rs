//! Local crash recovery. A normal exit removes only this product's recovery file.
use crate::{Result, self_update};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
pub struct Recovery {
    root: PathBuf,
}
impl Default for Recovery {
    fn default() -> Self {
        let root = std::env::var_os("SKYLAB_RECOVERY_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::var_os("LOCALAPPDATA")
                    .map(PathBuf::from)
                    .unwrap_or_else(std::env::temp_dir)
                    .join("SkyTEM/SkyLab/Recovery")
            });
        Self { root }
    }
}
impl Recovery {
    pub fn at(root: PathBuf) -> Self {
        Self { root }
    }
    fn safe(&self) -> Result<()> {
        if fs::symlink_metadata(&self.root).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err("Pasta de recuperação não pode ser um link.".into());
        }
        for name in ["session.json", "session.new"] {
            if fs::symlink_metadata(self.root.join(name)).is_ok_and(|m| m.file_type().is_symlink())
            {
                return Err("Arquivo de recuperação não pode ser um link.".into());
            }
        }
        Ok(())
    }
    pub fn load(&self) -> Result<Option<Value>> {
        self.safe()?;
        let path = self.root.join("session.json");
        if !path.exists() {
            return Ok(None);
        }
        let mut bytes = Vec::new();
        fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(40 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 40 * 1024 * 1024 {
            return Err("Recuperação excede o limite de tamanho.".into());
        }
        let value: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if value["product"] != crate::installation::PRODUCT {
            return Err("Recuperação inválida.".into());
        }
        self_update::validate_session(&value["session"])?;
        Ok(Some(value))
    }
    pub fn save(&self, session: Value) -> Result<()> {
        self.safe()?;
        self_update::validate_session(&session)?;
        if session["documents"].as_array().is_some_and(Vec::is_empty) {
            return self.clear();
        }
        let value = json!({"product":crate::installation::PRODUCT,"saved_at":SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_secs(),"session":session});
        let bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
        if bytes.len() > 40 * 1024 * 1024 {
            return Err("Recuperação excede o limite de tamanho.".into());
        }
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let pending = self.root.join("session.new");
        let mut file = fs::File::create(&pending).map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|e| e.to_string())?;
        drop(file);
        fs::rename(pending, self.root.join("session.json")).map_err(|e| e.to_string())
    }
    pub fn clear(&self) -> Result<()> {
        self.safe()?;
        for name in ["session.json", "session.new"] {
            let file = self.root.join(name);
            if file.exists() {
                fs::remove_file(file).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}
