//! File activation and a local MRU list. Contents never enter the recent-file database.
use crate::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fs,
    io::Read,
    path::{Path, PathBuf},
};
#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: u64,
    pub path: PathBuf,
    pub name: String,
}
#[derive(Clone, Serialize)]
pub struct Pending {
    pub id: u64,
    pub new_project: bool,
}
pub struct Files {
    root: PathBuf,
    entries: Vec<Entry>,
    recent: Vec<u64>,
    queue: VecDeque<Pending>,
    next: u64,
}
const LIMIT: u64 = 8 * 1024 * 1024;
impl Default for Files {
    fn default() -> Self {
        Self::at(
            std::env::var_os("SKYLAB_FILES_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    std::env::var_os("LOCALAPPDATA")
                        .map(PathBuf::from)
                        .unwrap_or_else(std::env::temp_dir)
                        .join("SkyTEM/SkyLab/Files")
                }),
        )
    }
}
impl Files {
    pub fn at(root: PathBuf) -> Self {
        let mut this = Self {
            root,
            entries: vec![],
            recent: vec![],
            queue: VecDeque::new(),
            next: 1,
        };
        if this.safe().is_ok()
            && let Ok(file) = fs::File::open(this.root.join("recent.json"))
        {
            let mut data = String::new();
            if file.take(128 * 1024).read_to_string(&mut data).is_ok()
                && let Ok(paths) = serde_json::from_str::<Vec<PathBuf>>(&data)
            {
                for path in paths.into_iter().take(12) {
                    if let Ok(id) = this.authorize(&path, false) {
                        this.recent.push(id);
                    }
                }
            }
        }
        this
    }
    fn safe(&self) -> Result<()> {
        for path in [
            &self.root,
            &self.root.join("recent.json"),
            &self.root.join("recent.new"),
        ] {
            if fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
                return Err("Recent-file storage cannot be a link".into());
            }
        }
        Ok(())
    }
    fn authorize(&mut self, path: &Path, require_file: bool) -> Result<u64> {
        if !path.is_absolute() {
            return Err("An absolute file path is required".into());
        }
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !["mis", "mission", "kml", "kmz", "json", "skylab"].contains(&ext.as_str()) {
            return Err("Choose KML, KMZ, MIS or a SkyLab project".into());
        }
        let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if require_file {
            let meta = fs::metadata(&path)
                .map_err(|_| "File unavailable. It may have been moved or deleted.")?;
            if !meta.is_file() || meta.len() > LIMIT {
                return Err("File must be smaller than 8 MB".into());
            }
        }
        if let Some(entry) = self.entries.iter().find(|e| e.path == path) {
            return Ok(entry.id);
        }
        if self.entries.len() >= 256 {
            return Err("Too many pending files; restart SkyLab".into());
        }
        let id = self.next;
        self.next += 1;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        self.entries.push(Entry { id, path, name });
        Ok(id)
    }
    pub fn enqueue(&mut self, paths: &[PathBuf], new_project: bool) -> Result<()> {
        if paths.len() > 32 || self.queue.len() + paths.len() > 64 {
            return Err("Too many files to open at once".into());
        }
        let mut ids = vec![];
        for path in paths {
            ids.push(self.authorize(path, true)?);
        }
        for id in ids {
            self.queue.push_back(Pending { id, new_project });
        }
        Ok(())
    }
    pub fn pending(&self) -> Option<&Pending> {
        self.queue.front()
    }
    pub fn acknowledge(&mut self, id: u64) {
        if self.queue.front().is_some_and(|p| p.id == id) {
            self.queue.pop_front();
        }
    }
    pub fn recent(&self) -> Vec<&Entry> {
        self.recent
            .iter()
            .filter_map(|id| self.entries.iter().find(|e| e.id == *id))
            .collect()
    }
    pub fn remember(&mut self, id: u64) -> Result<()> {
        if !self.entries.iter().any(|e| e.id == id) {
            return Err("Unknown recent file".into());
        }
        self.recent.retain(|i| *i != id);
        self.recent.insert(0, id);
        self.recent.truncate(12);
        self.save()
    }
    pub fn forget(&mut self, id: u64) -> Result<()> {
        self.recent.retain(|i| *i != id);
        self.save()
    }
    fn save(&self) -> Result<()> {
        self.safe()?;
        fs::create_dir_all(&self.root).map_err(|e| e.to_string())?;
        let paths: Vec<_> = self.recent().into_iter().map(|e| &e.path).collect();
        fs::write(
            self.root.join("recent.new"),
            serde_json::to_vec(&paths).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        fs::rename(self.root.join("recent.new"), self.root.join("recent.json"))
            .map_err(|e| e.to_string())
    }
    pub fn read(&self, id: u64) -> Result<serde_json::Value> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.id == id)
            .ok_or("Unknown file")?;
        let file = fs::File::open(&entry.path)
            .map_err(|_| "File unavailable. It may have been moved or deleted.")?;
        if !file.metadata().map_err(|e| e.to_string())?.is_file() {
            return Err("Not a file".into());
        }
        let mut bytes = vec![];
        file.take(LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > LIMIT {
            return Err("File must be smaller than 8 MB".into());
        }
        let size = bytes.len();
        let text = if entry
            .path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("kmz"))
        {
            crate::kmz::read(&base64(&bytes))?
        } else {
            String::from_utf8(bytes).map_err(|_| "File must use UTF-8")?
        };
        Ok(serde_json::json!({"name":entry.name,"size":size,"text":text}))
    }
}
fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for group in bytes.chunks(3) {
        let a = group[0] as usize;
        let b = group.get(1).copied().unwrap_or(0) as usize;
        let c = group.get(2).copied().unwrap_or(0) as usize;
        out.push(TABLE[a >> 2] as char);
        out.push(TABLE[((a & 3) << 4) | (b >> 4)] as char);
        out.push(if group.len() > 1 {
            TABLE[((b & 15) << 2) | (c >> 6)] as char
        } else {
            '='
        });
        out.push(if group.len() > 2 {
            TABLE[c & 63] as char
        } else {
            '='
        });
    }
    out
}
