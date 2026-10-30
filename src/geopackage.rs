//! Read-only GeoPackage vector access using the already bundled SQLite runtime.
use crate::Result;
#[derive(Debug)]
pub(crate) enum Cell {
    Text(String),
    Blob(Vec<u8>),
    Null,
}
impl Cell {
    pub fn text(&self) -> Result<&str> {
        if let Self::Text(s) = self {
            Ok(s)
        } else {
            Err("Invalid GeoPackage metadata".into())
        }
    }
    pub fn blob(&self) -> Result<&[u8]> {
        if let Self::Blob(b) = self {
            Ok(b)
        } else {
            Err("Invalid GeoPackage geometry".into())
        }
    }
}
pub(crate) fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
#[cfg(windows)]
mod native {
    use super::*;
    use std::{
        ffi::{CStr, CString, c_char, c_void},
        os::windows::ffi::OsStrExt,
        sync::OnceLock,
    };
    type Handle = *mut c_void;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LoadLibraryExW(name: *const u16, file: Handle, flags: u32) -> Handle;
        fn GetProcAddress(module: Handle, name: *const u8) -> Handle;
    }
    struct Api {
        open: unsafe extern "C" fn(*const c_char, *mut Handle, i32, *const c_char) -> i32,
        close: unsafe extern "C" fn(Handle) -> i32,
        prepare: unsafe extern "C" fn(
            Handle,
            *const c_char,
            i32,
            *mut Handle,
            *mut *const c_char,
        ) -> i32,
        step: unsafe extern "C" fn(Handle) -> i32,
        finalize: unsafe extern "C" fn(Handle) -> i32,
        count: unsafe extern "C" fn(Handle) -> i32,
        kind: unsafe extern "C" fn(Handle, i32) -> i32,
        text: unsafe extern "C" fn(Handle, i32) -> *const c_char,
        blob: unsafe extern "C" fn(Handle, i32) -> *const c_void,
        bytes: unsafe extern "C" fn(Handle, i32) -> i32,
        error: unsafe extern "C" fn(Handle) -> *const c_char,
    }
    fn api() -> Result<&'static Api> {
        static API: OnceLock<Result<Api>> = OnceLock::new();
        API.get_or_init(|| unsafe {
            let path = crate::projection::runtime_directory()?.join("sqlite3.dll");
            let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            let module = LoadLibraryExW(wide.as_ptr(), std::ptr::null_mut(), 0x1100);
            if module.is_null() {
                return Err("Cannot load bundled SQLite runtime".into());
            }
            macro_rules! get {
                ($n:literal,$ty:ty) => {{
                    let p = GetProcAddress(module, concat!($n, "\0").as_ptr());
                    if p.is_null() {
                        return Err(format!("Missing SQLite function {}", $n));
                    }
                    std::mem::transmute::<Handle, $ty>(p)
                }};
            }
            Ok(Api {
                open: get!(
                    "sqlite3_open_v2",
                    unsafe extern "C" fn(*const c_char, *mut Handle, i32, *const c_char) -> i32
                ),
                close: get!("sqlite3_close", unsafe extern "C" fn(Handle) -> i32),
                prepare: get!(
                    "sqlite3_prepare_v2",
                    unsafe extern "C" fn(
                        Handle,
                        *const c_char,
                        i32,
                        *mut Handle,
                        *mut *const c_char,
                    ) -> i32
                ),
                step: get!("sqlite3_step", unsafe extern "C" fn(Handle) -> i32),
                finalize: get!("sqlite3_finalize", unsafe extern "C" fn(Handle) -> i32),
                count: get!("sqlite3_column_count", unsafe extern "C" fn(Handle) -> i32),
                kind: get!(
                    "sqlite3_column_type",
                    unsafe extern "C" fn(Handle, i32) -> i32
                ),
                text: get!(
                    "sqlite3_column_text",
                    unsafe extern "C" fn(Handle, i32) -> *const c_char
                ),
                blob: get!(
                    "sqlite3_column_blob",
                    unsafe extern "C" fn(Handle, i32) -> *const c_void
                ),
                bytes: get!(
                    "sqlite3_column_bytes",
                    unsafe extern "C" fn(Handle, i32) -> i32
                ),
                error: get!(
                    "sqlite3_errmsg",
                    unsafe extern "C" fn(Handle) -> *const c_char
                ),
            })
        })
        .as_ref()
        .map_err(Clone::clone)
    }
    pub struct Database {
        handle: Handle,
        api: &'static Api,
    }
    impl Drop for Database {
        fn drop(&mut self) {
            unsafe {
                (self.api.close)(self.handle);
            }
        }
    }
    impl Database {
        pub fn open(path: &std::path::Path) -> Result<Self> {
            let api = api()?;
            let name =
                CString::new(path.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
            let mut handle = std::ptr::null_mut();
            let result =
                unsafe { (api.open)(name.as_ptr(), &mut handle, 1 | 0x8000, std::ptr::null()) };
            if result != 0 {
                if !handle.is_null() {
                    unsafe {
                        (api.close)(handle);
                    }
                }
                return Err("Cannot open GeoPackage read-only".into());
            }
            let db = Self { handle, api };
            db.query("PRAGMA trusted_schema=OFF")?;
            Ok(db)
        }
        pub fn query(&self, sql: &str) -> Result<Vec<Vec<Cell>>> {
            struct Statement {
                h: Handle,
                a: &'static Api,
            }
            impl Drop for Statement {
                fn drop(&mut self) {
                    unsafe {
                        (self.a.finalize)(self.h);
                    }
                }
            }
            let sql = CString::new(sql).map_err(|e| e.to_string())?;
            let mut h = std::ptr::null_mut();
            unsafe {
                if (self.api.prepare)(self.handle, sql.as_ptr(), -1, &mut h, std::ptr::null_mut())
                    != 0
                {
                    return Err(CStr::from_ptr((self.api.error)(self.handle))
                        .to_string_lossy()
                        .into_owned());
                }
            }
            if h.is_null() {
                return Ok(vec![]);
            }
            let stmt = Statement { h, a: self.api };
            let mut rows = vec![];
            let mut total = 0usize;
            loop {
                let code = unsafe { (self.api.step)(stmt.h) };
                if code == 101 {
                    break;
                }
                if code != 100 {
                    return Err("Cannot read GeoPackage vector data".into());
                }
                if rows.len() >= 20001 {
                    return Err("GeoPackage exceeds the feature limit".into());
                }
                let mut row = vec![];
                for i in 0..unsafe { (self.api.count)(stmt.h) } {
                    let n = unsafe { (self.api.bytes)(stmt.h, i) };
                    if !(0..=4_000_000).contains(&n) {
                        return Err("GeoPackage value exceeds the size limit".into());
                    }
                    total += n as usize;
                    if total > 32 * 1024 * 1024 {
                        return Err("GeoPackage vector selection exceeds 32 MB".into());
                    }
                    let value = unsafe {
                        match (self.api.kind)(stmt.h, i) {
                            5 => Cell::Null,
                            4 => {
                                let ptr = (self.api.blob)(stmt.h, i);
                                Cell::Blob(if n == 0 {
                                    vec![]
                                } else {
                                    if ptr.is_null() {
                                        return Err("Invalid GeoPackage geometry".into());
                                    }
                                    std::slice::from_raw_parts(ptr as *const u8, n as usize)
                                        .to_vec()
                                })
                            }
                            _ => {
                                let ptr = (self.api.text)(stmt.h, i);
                                if ptr.is_null() {
                                    return Err("Invalid GeoPackage text".into());
                                }
                                Cell::Text(CStr::from_ptr(ptr).to_string_lossy().into_owned())
                            }
                        }
                    };
                    row.push(value);
                }
                rows.push(row);
            }
            Ok(rows)
        }
    }
}
#[cfg(windows)]
pub(crate) use native::Database;
#[cfg(not(windows))]
pub(crate) struct Database;
#[cfg(not(windows))]
impl Database {
    pub fn open(_: &std::path::Path) -> Result<Self> {
        Err("GeoPackage import requires Windows".into())
    }
    pub fn query(&self, _: &str) -> Result<Vec<Vec<Cell>>> {
        Err("GeoPackage import requires Windows".into())
    }
}
