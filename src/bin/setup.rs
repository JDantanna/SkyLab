#![cfg_attr(windows, windows_subsystem = "windows")]
use serde::Serialize;
use serde_json::json;
use skylab_rust::{
    desktop,
    installation::{self, Options},
    updates,
};
use std::{
    io::Read,
    net::{Ipv4Addr, TcpListener},
    sync::{Arc, Mutex},
};
#[derive(Clone, Serialize)]
struct Progress {
    running: bool,
    done: bool,
    percent: u8,
    stage: String,
    error: Option<String>,
}
fn main() {
    if let Err(e) = run() {
        desktop::show_error(&e.to_string());
        std::process::exit(1)
    }
}
fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let uninstall = args.iter().any(|s| s == "--uninstall");
    let mut options = if let Some(i) = args.iter().position(|s| s == "--test-root") {
        Options::test(
            std::path::absolute(args.get(i + 1).ok_or("Falta pasta de teste")?)?.join("SkyLab"),
        )
    } else {
        Options::current_user()?
    };
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let port = listener.local_addr()?.port();
    let origin = format!("http://127.0.0.1:{port}");
    let server = tiny_http::Server::from_listener(listener, None).map_err(|e| e.to_string())?;
    let progress = Arc::new(Mutex::new(Progress {
        running: false,
        done: false,
        percent: 0,
        stage: "Revisar".into(),
        error: None,
    }));
    println!("SkyLab Setup {origin}");
    let mut window = if !args.iter().any(|s| s == "--no-open") {
        desktop::open(&origin, true)
    } else {
        None
    };
    let mut last_seen = std::time::Instant::now();
    let mut page_seen = false;
    loop {
        if window
            .as_mut()
            .is_some_and(|child| child.try_wait().ok().flatten().is_some())
            && !progress.lock().unwrap().running
        {
            break;
        }
        let Some(mut req) = server.recv_timeout(std::time::Duration::from_secs(1))? else {
            if window.is_none()
                && !progress.lock().unwrap().running
                && last_seen.elapsed().as_secs() > if page_seen { 90 } else { 600 }
            {
                break;
            }
            continue;
        };
        let expected = format!("127.0.0.1:{port}");
        let host = req
            .headers()
            .iter()
            .find(|h| h.field.equiv("Host"))
            .map(|h| h.value.as_str());
        let from = req
            .headers()
            .iter()
            .find(|h| h.field.equiv("Origin"))
            .map(|h| h.value.as_str());
        if host != Some(expected.as_str()) || from.is_some_and(|s| s != origin) {
            let _ = req
                .respond(tiny_http::Response::from_string("Origem recusada").with_status_code(403));
            continue;
        }
        let mut exit = false;
        let mut launch = false;
        let route = req.url().to_owned();
        let method = req.method().as_str().to_owned();
        let(status,mime,bytes)=match(method.as_str(),route.as_str()){
            ("GET","/heartbeat")=>{page_seen=true;last_seen=std::time::Instant::now();(200,"application/json",b"{\"ok\":true}".to_vec())},
            ("GET","/")=>(200,"text/html; charset=utf-8",include_bytes!("../../installer/index.html").to_vec()),
            ("GET","/style.css")=>(200,"text/css; charset=utf-8",include_bytes!("../../web/style.css").to_vec()),
            ("GET","/window.css")=>(200,"text/css; charset=utf-8",include_bytes!("../../web/window.css").to_vec()),
            ("GET","/window.js")=>(200,"text/javascript; charset=utf-8",include_bytes!("../../web/window.js").to_vec()),
            ("GET","/setup.css")=>(200,"text/css; charset=utf-8",include_bytes!("../../installer/setup.css").to_vec()),
            ("GET","/setup.js")=>(200,"text/javascript; charset=utf-8",include_bytes!("../../installer/setup.js").to_vec()),
            ("GET","/brand.svg")=>(200,"image/svg+xml",include_bytes!("../../assets/skytem-white.svg").to_vec()),
            ("GET","/progress")=>(200,"application/json",json!(*progress.lock().unwrap()).to_string().into_bytes()),
            ("GET","/state")=>(200,"application/json",json!({"version":updates::display_version(),"path":options.root,"installed":options.installed(),"uninstall":uninstall,"test":options.test_mode}).to_string().into_bytes()),
            ("POST",route)if["/install","/remove","/finish"].contains(&route)=>{
                let mut body=String::new();let read=req.as_reader().take(4097).read_to_string(&mut body);
                let input=if read.is_ok()&&body.len()<=4096{serde_json::from_str::<serde_json::Value>(&body).map_err(|e|e.to_string())}else{Err("Requisição inválida".into())};
                let result=input.and_then(|v|{
                    if progress.lock().unwrap().running{return Err("Aguarde a operação em andamento.".into())}
                    match route {
                        "/install" if !uninstall=>{
                            options.desktop=v["desktop"].as_bool().unwrap_or(true);
                            let me=std::fs::read(std::env::current_exe().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
                            let opts=options.clone();let state=progress.clone();
                            *state.lock().unwrap()=Progress{running:true,done:false,percent:0,stage:"Verificando instalação".into(),error:None};
                            std::thread::spawn(move||{let state2=state.clone();let result=installation::install_with_progress(&opts,include_bytes!("../../payload/SkyLab-Rust.exe"),&me,move|stage,percent|{let mut p=state2.lock().unwrap();p.stage=stage.into();p.percent=percent;});let mut p=state.lock().unwrap();p.running=false;p.done=true;p.error=result.err();});Ok(())
                        },
                        "/remove" if uninstall=>{
                            let opts=options.clone();let state=progress.clone();
                            *state.lock().unwrap()=Progress{running:true,done:false,percent:0,stage:"Verificando instalação".into(),error:None};
                            std::thread::spawn(move||{let state2=state.clone();let result=installation::uninstall_with_progress(&opts,move|stage,percent|{let mut p=state2.lock().unwrap();p.stage=stage.into();p.percent=percent;});let mut p=state.lock().unwrap();p.running=false;p.done=true;p.error=result.err();});Ok(())
                        },
                        "/finish"=>{exit=true;launch=!uninstall&&!options.test_mode&&options.installed()&&v["launch"].as_bool().unwrap_or(false);Ok(())},
                        _=>Err("Operação indisponível neste modo.".into()),
                    }
                });
                match result{Ok(())=>(200,"application/json",b"{\"ok\":true}".to_vec()),Err(e)=>(400,"application/json",json!({"ok":false,"error":e}).to_string().into_bytes())}
            },
            _=>(404,"text/plain",b"Nao encontrado".to_vec()),
        };
        let mut response = tiny_http::Response::from_data(bytes).with_status_code(status);
        for (k, v) in [
            ("Content-Type", mime),
            ("Cache-Control", "no-store"),
            ("X-Content-Type-Options", "nosniff"),
            (
                "Content-Security-Policy",
                "default-src 'self'; style-src 'self'; script-src 'self'; img-src 'self'; connect-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'",
            ),
        ] {
            response.add_header(tiny_http::Header::from_bytes(k, v).unwrap());
        }
        let _ = req.respond(response);
        if launch {
            let _ = desktop::hidden(&mut std::process::Command::new(
                options.root.join(installation::APP),
            ))
            .spawn();
        }
        if exit {
            break;
        }
    }
    Ok(())
}
