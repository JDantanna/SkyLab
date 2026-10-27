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
        eprintln!("SkyLab: {e}");
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
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))?;
    let actual = listener.local_addr()?.port();
    let origin = format!("http://127.0.0.1:{actual}");
    let server = tiny_http::Server::from_listener(listener, None).map_err(|e| e.to_string())?;
    println!(
        "SkyLab Rust • aplicativo local\nAbra {origin}\nMantenha esta janela aberta. Ctrl+C encerra.\nValide datum, projeção e missão antes de uso em voo."
    );
    if !args.iter().any(|s| s == "--no-open") {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer.exe")
                .arg(&origin)
                .spawn();
        }
    }
    for mut req in server.incoming_requests() {
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
        let (status, mime, body) = match (req.method().as_str(), req.url()) {
            ("GET", "/") => (
                200,
                "text/html; charset=utf-8",
                include_str!("../web/index.html").to_string(),
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
    }
    Ok(())
}
