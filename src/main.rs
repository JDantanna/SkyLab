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
    if r.op == "inspect-crs" {
        if let Ok(p) = serde_json::from_str::<Project>(&r.text) {
            p.crs.validate()?;
            return Ok(json!({"crs":p.crs,"kind":"declared"}));
        }
        if let Ok(doc) = roxmltree::Document::parse(&r.text) {
            let mut points = vec![];
            for node in doc
                .descendants()
                .filter(|n| n.is_element() && n.tag_name().name() == "coordinates")
            {
                for tuple in node.text().unwrap_or("").split_whitespace() {
                    let values: Vec<_> = tuple
                        .split(',')
                        .take(2)
                        .filter_map(|n| n.parse::<f64>().ok())
                        .collect();
                    if values.len() == 2
                        && values.iter().all(|v| v.is_finite())
                        && (-180.0..=180.0).contains(&values[0])
                        && (-80.0..=84.0).contains(&values[1])
                    {
                        points.push([values[0], values[1]]);
                    }
                }
            }
            if !points.is_empty() {
                let lon = points.iter().map(|p| p[0]).sum::<f64>() / points.len() as f64;
                let lat = points.iter().map(|p| p[1]).sum::<f64>() / points.len() as f64;
                let zone = (((lon + 180.) / 6.).floor() as u8 + 1).clamp(1, 60);
                let south = lat < 0.;
                let spans = points.iter().any(|p| {
                    ((p[0] + 180.) / 6.).floor() as u8 + 1 != zone || (p[1] < 0.) != south
                });
                return Ok(
                    json!({"crs":{"zone":zone,"south":south},"kind":"suggested","spans":spans}),
                );
            }
        }
        return Ok(json!({"crs":null,"kind":"unknown"}));
    }
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
        "reproject" => {
            let target = r.crs.ok_or("Choose the target CRS")?;
            target.validate()?;
            for g in &mut p.polygons {
                for point in g.outer.iter_mut().chain(g.holes.iter_mut().flatten()) {
                    *point = target.forward(p.crs.inverse(*point)?)?;
                }
            }
            for l in &mut p.lines {
                for point in &mut l.points {
                    *point = target.forward(p.crs.inverse(*point)?)?;
                }
            }
            p.crs = target;
            if p.crs.authority.is_some() {
                p.version = 2;
            }
        }
        "geographic" => {
            let mut count = 0usize;
            let mut geographic = |points: &mut Vec<Point>| -> Result<()> {
                let mut out = Vec::new();
                for pair in points.windows(2) {
                    let steps = (distance(pair[0], pair[1]) / 1000.).ceil().max(1.) as usize;
                    if count.saturating_add(steps) > 200000 {
                        return Err("Satellite preview exceeds the point limit. Reduce the area or line count.".into());
                    }
                    for step in 0..steps {
                        let t = step as f64 / steps as f64;
                        out.push(p.crs.inverse([
                            pair[0][0] + (pair[1][0] - pair[0][0]) * t,
                            pair[0][1] + (pair[1][1] - pair[0][1]) * t,
                        ])?);
                    }
                    count += steps;
                }
                if let Some(point) = points.last() {
                    out.push(p.crs.inverse(*point)?);
                    count += 1;
                }
                *points = out;
                Ok(())
            };
            for g in &mut p.polygons {
                geographic(&mut g.outer)?;
                for hole in &mut g.holes {
                    geographic(hole)?;
                }
            }
            for l in &mut p.lines {
                geographic(&mut l.points)?;
            }
            return Ok(json!(p));
        }
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
        8787
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
                if reply.contains("skytem-skylab-local-planner")
                    && reply.contains("native-window-v3")
                {
                    if !args.iter().any(|s| s == "--no-open") {
                        let mut activate = std::net::TcpStream::connect(("127.0.0.1", port))?;
                        write!(
                            activate,
                            "POST /activate HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        )?;
                    }
                    return Ok(());
                }
                if reply.contains("skytem-skylab-local-planner") {
                    return Err("Outra revisão do SkyLab está aberta. Encerre a revisão anterior antes de abrir esta.".into());
                }
            }
            return Err("Não foi possível iniciar esta revisão do SkyLab. Feche as outras janelas do programa e tente novamente.".into());
        }
        Err(e) => return Err(e.into()),
    };
    let actual = listener.local_addr()?.port();
    let origin = format!("http://127.0.0.1:{actual}");
    let server = tiny_http::Server::from_listener(listener, None).map_err(|e| e.to_string())?;
    println!(
        "SkyLab Rust • aplicativo local\nAbra {origin}\nMantenha esta janela aberta. Ctrl+C encerra.\nValide datum, projeção e missão antes de uso em voo."
    );
    let mut window = if !args.iter().any(|s| s == "--no-open") {
        desktop::open(&origin, false)
    } else {
        None
    };
    let mut last_seen = std::time::Instant::now();
    let mut page_seen = false;
    loop {
        if window
            .as_mut()
            .is_some_and(|child| child.try_wait().ok().flatten().is_some())
        {
            break;
        }
        let Some(mut req) = server.recv_timeout(std::time::Duration::from_secs(1))? else {
            if window.is_none() && last_seen.elapsed().as_secs() > if page_seen { 90 } else { 600 }
            {
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
        if req.method().as_str() == "GET" && req.url() == "/brand.svg" {
            let mut response = tiny_http::Response::from_data(
                include_bytes!("../assets/skylab-icon.svg").to_vec(),
            );
            response.add_header(
                tiny_http::Header::from_bytes("Content-Type", "image/svg+xml").unwrap(),
            );
            let _ = req.respond(response);
            continue;
        }
        let (status, mime, body) = match (req.method().as_str(), req.url()) {
            ("POST", "/activate") => {
                let child = desktop::open(&origin, false);
                if window.is_none() {
                    window = child;
                }
                (200, "application/json", "{\"ok\":true}".into())
            }
            ("GET", "/window.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/window.js").into(),
            ),
            ("GET", "/window.css") => (
                200,
                "text/css; charset=utf-8",
                include_str!("../web/window.css").into(),
            ),
            ("GET", "/updates/check") => match updates::check() {
                Ok(check) => (
                    200,
                    "application/json",
                    json!({"ok":true,"data":check}).to_string(),
                ),
                Err(e) => (
                    400,
                    "application/json",
                    json!({"ok":false,"error":e}).to_string(),
                ),
            },
            ("POST", "/updates/open") => match updates::check() {
                Ok(check) if check.status == "available" && check.download_url.is_some() => {
                    let url = check.download_url.unwrap();
                    #[cfg(windows)]
                    {
                        let _ =
                            desktop::hidden(std::process::Command::new("explorer.exe").arg(url))
                                .spawn();
                    }
                    (200, "application/json", "{\"ok\":true}".into())
                }
                _ => (
                    400,
                    "application/json",
                    "{\"ok\":false,\"error\":\"Nenhum endereço de atualização disponível.\"}"
                        .into(),
                ),
            },
            ("GET", "/heartbeat") => {
                page_seen = true;
                last_seen = std::time::Instant::now();
                (200, "application/json", "{\"ok\":true}".into())
            }
            ("GET", "/identity") => (
                200,
                "text/plain",
                format!(
                    "skytem-skylab-local-planner native-window-v3 {}",
                    updates::display_version()
                ),
            ),
            ("POST", "/shutdown") => {
                shutdown = true;
                (200, "application/json", "{\"ok\":true}".into())
            }
            ("GET", "/") => (
                200,
                "text/html; charset=utf-8",
                include_str!("../web/index.html")
                    .replace("__VERSION__", updates::display_version()),
            ),
            ("GET", "/app.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/app.js").to_string(),
            ),
            ("GET", "/interactions.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/interactions.js").into(),
            ),
            ("GET", "/controls.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/controls.js").into(),
            ),
            ("GET", "/flags/en.svg") => (
                200,
                "image/svg+xml",
                include_str!("../assets/flags/en.svg").into(),
            ),
            ("GET", "/flags/pt-BR.svg") => (
                200,
                "image/svg+xml",
                include_str!("../assets/flags/pt-BR.svg").into(),
            ),
            ("GET", "/flags/da.svg") => (
                200,
                "image/svg+xml",
                include_str!("../assets/flags/da.svg").into(),
            ),
            ("GET", "/workspace.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/workspace.js").into(),
            ),
            ("GET", "/languages.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/languages.js").into(),
            ),
            ("GET", "/crs-catalog") => (200, "application/json", projection::CATALOG.into()),
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
                "default-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; script-src 'self' https://maps.googleapis.com https://maps.gstatic.com; connect-src 'self' https://*.googleapis.com https://*.gstatic.com; img-src 'self' data: blob: https://*.googleapis.com https://*.gstatic.com https://*.google.com; frame-src https://*.google.com; object-src 'none'; frame-ancestors 'none'; base-uri 'none'",
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
