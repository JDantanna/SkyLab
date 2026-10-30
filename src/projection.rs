//! PROJ database and offline transformations. No network or ballpark datum shifts.
use crate::{Point, Result};
use serde::Deserialize;
use std::sync::OnceLock;
pub const CATALOG: &str = include_str!("../native/vendor/proj/catalog.json");
#[derive(Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub metric: bool,
    pub deprecated: bool,
}
pub fn catalog() -> &'static Vec<Entry> {
    static C: OnceLock<Vec<Entry>> = OnceLock::new();
    C.get_or_init(|| serde_json::from_str(CATALOG).expect("embedded PROJ catalog"))
}
pub fn validate(id: &str) -> Result<()> {
    static INDEX: OnceLock<std::collections::HashMap<&'static str, &'static Entry>> =
        OnceLock::new();
    let e = INDEX
        .get_or_init(|| catalog().iter().map(|e| (e.id.as_str(), e)).collect())
        .get(id)
        .ok_or("Unknown coordinate reference system")?;
    if !e.metric || e.deprecated {
        return Err("The working CRS must be a current projected CRS in metres. Geographic, vertical and non-metric definitions remain available in the catalog for reference.".into());
    }
    Ok(())
}
#[cfg(windows)]
mod native {
    use super::*;
    use std::{
        cell::RefCell,
        ffi::{CString, c_char, c_void},
        path::PathBuf,
    };
    type Handle = *mut c_void;
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Coord {
        v: [f64; 4],
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LoadLibraryExW(name: *const u16, file: Handle, flags: u32) -> Handle;
        fn GetProcAddress(module: Handle, name: *const u8) -> Handle;
    }
    struct Api {
        context: unsafe extern "C" fn() -> Handle,
        context_destroy: unsafe extern "C" fn(Handle),
        search: unsafe extern "C" fn(Handle, i32, *const *const c_char),
        network: unsafe extern "C" fn(Handle, i32) -> i32,
        create: unsafe extern "C" fn(Handle, *const c_char) -> Handle,
        pipeline:
            unsafe extern "C" fn(Handle, Handle, Handle, Handle, *const *const c_char) -> Handle,
        normalize: unsafe extern "C" fn(Handle, Handle) -> Handle,
        destroy: unsafe extern "C" fn(Handle) -> Handle,
        trans: unsafe extern "C" fn(Handle, i32, Coord) -> Coord,
        reset: unsafe extern "C" fn(Handle) -> i32,
        errno: unsafe extern "C" fn(Handle) -> i32,
        dir: PathBuf,
    }
    fn api() -> Result<&'static Api> {
        static API: OnceLock<Result<Api>> = OnceLock::new();
        API.get_or_init(|| unsafe {
            let dir = std::env::temp_dir().join("SkyTEM-SkyLab-PROJ-9.6-r1");
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            include!(concat!(env!("OUT_DIR"), "/proj_files.rs"));
            let path: Vec<u16> = dir
                .join("proj_9.dll")
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            let module = LoadLibraryExW(path.as_ptr(), std::ptr::null_mut(), 0x1100);
            if module.is_null() {
                return Err(format!(
                    "Cannot load the offline projection engine: {}",
                    std::io::Error::last_os_error()
                ));
            }
            macro_rules! get {
                ($name:literal,$ty:ty) => {{
                    let p = GetProcAddress(module, concat!($name, "\0").as_ptr());
                    if p.is_null() {
                        return Err(format!("Missing PROJ function {}", $name));
                    }
                    std::mem::transmute::<Handle, $ty>(p)
                }};
            }
            Ok(Api {
                context: get!("proj_context_create", unsafe extern "C" fn() -> Handle),
                context_destroy: get!("proj_context_destroy", unsafe extern "C" fn(Handle)),
                search: get!(
                    "proj_context_set_search_paths",
                    unsafe extern "C" fn(Handle, i32, *const *const c_char)
                ),
                network: get!(
                    "proj_context_set_enable_network",
                    unsafe extern "C" fn(Handle, i32) -> i32
                ),
                create: get!(
                    "proj_create",
                    unsafe extern "C" fn(Handle, *const c_char) -> Handle
                ),
                pipeline: get!(
                    "proj_create_crs_to_crs_from_pj",
                    unsafe extern "C" fn(
                        Handle,
                        Handle,
                        Handle,
                        Handle,
                        *const *const c_char,
                    ) -> Handle
                ),
                normalize: get!(
                    "proj_normalize_for_visualization",
                    unsafe extern "C" fn(Handle, Handle) -> Handle
                ),
                destroy: get!("proj_destroy", unsafe extern "C" fn(Handle) -> Handle),
                trans: get!(
                    "proj_trans",
                    unsafe extern "C" fn(Handle, i32, Coord) -> Coord
                ),
                reset: get!("proj_errno_reset", unsafe extern "C" fn(Handle) -> i32),
                errno: get!("proj_errno", unsafe extern "C" fn(Handle) -> i32),
                dir,
            })
        })
        .as_ref()
        .map_err(Clone::clone)
    }
    pub fn runtime_directory() -> Result<PathBuf> {
        Ok(api()?.dir.clone())
    }
    use std::os::windows::ffi::OsStrExt;
    struct Pipeline {
        context: Handle,
        pj: Handle,
    }
    impl Drop for Pipeline {
        fn drop(&mut self) {
            if let Ok(a) = api() {
                unsafe {
                    (a.destroy)(self.pj);
                    (a.context_destroy)(self.context);
                }
            }
        }
    }
    thread_local! { static CACHE:RefCell<Vec<(String,String,Pipeline)>>=const { RefCell::new(Vec::new()) }; }
    pub fn transform(from: &str, to: &str, p: Point) -> Result<Point> {
        let a = api()?;
        CACHE.with(|cache|unsafe {
            let mut cache=cache.borrow_mut();
            let position=if let Some(position)=cache.iter().position(|(source,target,_)|source==from&&target==to){position}else{
                let context=(a.context)();
                if context.is_null(){return Err("Cannot create projection context".into());}
                let path=CString::new(a.dir.to_string_lossy().as_bytes()).map_err(|e|e.to_string())?;
                let extra=std::env::current_exe().map_err(|e|e.to_string())?.parent().ok_or("Executable directory missing")?.join("proj-data");
                let extra=CString::new(extra.to_string_lossy().as_bytes()).map_err(|e|e.to_string())?;
                let paths=[path.as_ptr(),extra.as_ptr()];
                (a.search)(context,2,paths.as_ptr());(a.network)(context,0);
                let src=CString::new(from).map_err(|e|e.to_string())?;let dst=CString::new(to).map_err(|e|e.to_string())?;
                let source=(a.create)(context,src.as_ptr());let target=(a.create)(context,dst.as_ptr());
                let options=[c"ALLOW_BALLPARK=NO".as_ptr(),c"ONLY_BEST=YES".as_ptr(),std::ptr::null()];
                let raw=if source.is_null()||target.is_null(){std::ptr::null_mut()}else{(a.pipeline)(context,source,target,std::ptr::null_mut(),options.as_ptr())};
                (a.destroy)(source);(a.destroy)(target);
                let pj=if raw.is_null(){raw}else{let normalized=(a.normalize)(context,raw);(a.destroy)(raw);normalized};
                if pj.is_null(){(a.context_destroy)(context);return Err("No accurate offline transformation is available. A datum grid may be required; no approximate transformation was applied.".into());}
                if cache.len()>=16{cache.clear();}
                cache.push((from.to_owned(),to.to_owned(),Pipeline{context,pj}));cache.len()-1
            };
            let pipe=&cache[position].2;(a.reset)(pipe.pj);
            let result=(a.trans)(pipe.pj,1,Coord{v:[p[0],p[1],0.,f64::INFINITY]});
            if (a.errno)(pipe.pj)!=0||!result.v[0].is_finite()||!result.v[1].is_finite(){return Err("Coordinate transformation failed. Check the CRS, area of use and required offline datum grids.".into());}
            Ok([result.v[0],result.v[1]])
        })
    }
}
pub fn transform(from: &str, to: &str, p: Point) -> Result<Point> {
    if from == to {
        return Ok(p);
    }
    #[cfg(windows)]
    {
        native::transform(from, to, p)
    }
    #[cfg(not(windows))]
    {
        let _ = (from, to, p);
        Err("The bundled PROJ engine requires Windows x64".into())
    }
}

#[cfg(windows)]
pub(crate) fn runtime_directory() -> Result<std::path::PathBuf> {
    native::runtime_directory()
}
