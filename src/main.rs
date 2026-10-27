#![cfg_attr(windows, windows_subsystem = "windows")]
use serde::Deserialize;
use serde_json::{Value, json};
use skylab_rust::*;
use std::{io::Read, net::TcpListener};
#[derive(Deserialize)]
struct Request {
    op: String,
    #[serde(default)]
    text: String,
    crs: Option<Crs>,
    project: Option<Project>,
    settings: Option<Settings>,
    #[serde(default)]
    append: bool,
    index: Option<usize>,
    start: Option<f64>,
    end: Option<f64>,
}
fn process(r: Request) -> Result<Value> {
    if r.op == "import-kml" || r.op == "import-mis" {
        let crs = r.crs.ok_or("Selecione a projeção.")?;
        return Ok(json!(if r.op == "import-kml" {
            import_kml(&r.text, crs)?
        } else {
            import_mis(&r.text, crs)?
        }));
    }
    if r.op == "open" {
        let p: Project =
            serde_json::from_str(&r.text).map_err(|e| format!("Projeto inválido: {e}"))?;
        p.validate()?;
        return Ok(json!(p));
    }
    let mut p = r.project.ok_or("Projeto ausente")?;
    p.validate()?;
    match r.op.as_str() {
        "validate" => {}
        "generate" => {
            let lines = generate(&p, &r.settings.ok_or("Parâmetros ausentes")?)?;
            if r.append {
                p.lines.extend(lines)
            } else {
                p.lines = lines
            }
        }
        "trim" => {
            let i = r.index.ok_or("Selecione uma linha")?;
            let l = p.lines.get(i).ok_or("Linha inexistente")?;
            p.lines[i] = trim_line(
                l,
                r.start.ok_or("Início ausente")?,
                r.end.ok_or("Fim ausente")?,
            )?
        }
        "clip" => {
            if p.polygons.is_empty() {
                return Err("Importe um polígono para recortar.".into());
            }
            let mut out = vec![];
            for (i, l) in p.lines.iter().enumerate() {
                if r.index.is_some() && r.index != Some(i) {
                    out.push(l.clone());
                    continue;
                }
                let pieces = clip_line(l, &p.polygons);
                let count = pieces.len();
                for (j, points) in pieces.into_iter().enumerate() {
                    out.push(Line {
                        name: if count > 1 {
                            format!("{}_{}", l.name, j + 1)
                        } else {
                            l.name.clone()
                        },
                        points,
                    })
                }
            }
            p.lines = out
        }
        "export-mis" => return Ok(json!({"text":export_mis(&p)?})),
        "export-kml" => return Ok(json!({"text":export_kml(&p)?})),
        _ => return Err("Operação desconhecida".into()),
    }
    p.validate()?;
    Ok(json!(p))
}
fn main() {
    if let Err(e) = run() {
        desktop::show_error(&e.to_string());
        std::process::exit(1)
    }
}
fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let port: u16 = if let Some(i) = args.iter().position(|s| s == "--port") {
        args.get(i + 1).ok_or("Falta porta")?.parse()?
    } else {
        8777
    };
    let listener = match TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)) {
        Ok(listener) => listener,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            // Open only when the listener identifies itself as this application.
            use std::io::Write;
            if let Ok(mut stream) = std::net::TcpStream::connect(("127.0.0.1", port)) {
                stream.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
                write!(
                    stream,
                    "GET /identity HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
                )?;
                let mut reply = String::new();
                stream.take(4096).read_to_string(&mut reply)?;
                if reply.contains("skytem-skylab-local-planner") {
                    if !args.iter().any(|s| s == "--no-open") {
                        desktop::open(&format!("http://127.0.0.1:{port}"));
                    }
                    return Ok(());
                }
            }
            return Err(format!("A porta {port} está ocupada por outro programa. Inicie o SkyLab com --port e outra porta.").into());
        }
        Err(e) => return Err(e.into()),
    };
    let actual = listener.local_addr()?.port();
    let origin = format!("http://127.0.0.1:{actual}");
    let server = tiny_http::Server::from_listener(listener, None).map_err(|e| e.to_string())?;
    println!(
        "SkyLab Rust • aplicativo local\nAbra {origin}\nMantenha esta janela aberta. Ctrl+C encerra.\nValide datum, projeção e missão antes de uso em voo."
    );
    if !args.iter().any(|s| s == "--no-open") {
        desktop::open(&origin);
    }
    let mut last_seen = std::time::Instant::now();
    let mut page_seen = false;
    loop {
        let Some(mut req) = server.recv_timeout(std::time::Duration::from_secs(1))? else {
            if last_seen.elapsed().as_secs() > if page_seen { 90 } else { 600 } {
                break;
            }
            continue;
        };
        let expected_host = format!("127.0.0.1:{actual}");
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
        if host != Some(expected_host.as_str()) || from.is_some_and(|s| s != origin) {
            let _ = req
                .respond(tiny_http::Response::from_string("Origem recusada").with_status_code(403));
            continue;
        }
        if req.method().as_str() == "GET" && req.url() == "/brand.jpg" {
            let mut response = tiny_http::Response::from_data(
                include_bytes!("../assets/skytem-logo.jpg").to_vec(),
            );
            response
                .add_header(tiny_http::Header::from_bytes("Content-Type", "image/jpeg").unwrap());
            let _ = req.respond(response);
            continue;
        }
        let mut shutdown = false;
        let (status, mime, body) = match (req.method().as_str(), req.url()) {
            ("GET", "/heartbeat") => {
                page_seen = true;
                last_seen = std::time::Instant::now();
                (200, "application/json", "{\"ok\":true}".into())
            }
            ("GET", "/identity") => (200, "text/plain", "skytem-skylab-local-planner".into()),
            ("POST", "/shutdown") => {
                shutdown = true;
                (200, "application/json", "{\"ok\":true}".into())
            }
            ("GET", "/") => (
                200,
                "text/html; charset=utf-8",
                include_str!("../web/index.html").replace("0.2.0", env!("CARGO_PKG_VERSION")),
            ),
            ("GET", "/app.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/app.js").to_string(),
            ),
            ("GET", "/style.css") => (
                200,
                "text/css; charset=utf-8",
                include_str!("../web/style.css").to_string(),
            ),
            ("POST", "/api") => {
                let mut buf = String::new();
                let read = req
                    .as_reader()
                    .take(10 * 1024 * 1024 + 1)
                    .read_to_string(&mut buf);
                let result = if read.is_err() || buf.len() > 10 * 1024 * 1024 {
                    Err("Requisição inválida ou maior que 10 MB.".into())
                } else {
                    serde_json::from_str::<Request>(&buf)
                        .map_err(|e| e.to_string())
                        .and_then(process)
                };
                match result {
                    Ok(v) => (
                        200,
                        "application/json",
                        json!({"ok":true,"data":v}).to_string(),
                    ),
                    Err(e) => (
                        400,
                        "application/json",
                        json!({"ok":false,"error":e}).to_string(),
                    ),
                }
            }
            _ => (404, "text/plain", "Não encontrado".into()),
        };
        let mut response = tiny_http::Response::from_string(body).with_status_code(status);
        for (k, v) in [
            ("Content-Type", mime),
            ("Cache-Control", "no-store"),
            ("X-Content-Type-Options", "nosniff"),
            ("Referrer-Policy", "no-referrer"),
            (
                "Content-Security-Policy",
                "default-src 'self'; style-src 'self'; script-src 'self'; connect-src 'self'; img-src 'self' data:; object-src 'none'; frame-ancestors 'none'; base-uri 'none'",
            ),
        ] {
            response.add_header(tiny_http::Header::from_bytes(k, v).unwrap());
        }
        let _ = req.respond(response);
        if shutdown {
            break;
        }
    }
    Ok(())
}
