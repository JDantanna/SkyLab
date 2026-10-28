#![cfg_attr(windows, windows_subsystem = "windows")]
use serde::Deserialize;
use serde_json::{Value, json};
use skylab_rust::*;
use std::{borrow::Cow, io::Read, net::TcpListener};
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
    stroke: Option<Vec<Point>>,
    #[serde(default)]
    geographic: bool,
}
fn process(r: Request) -> Result<Value> {
    if r.op == "inspect-crs" {
        if let Ok(p) = serde_json::from_str::<Project>(&r.text) {
            p.crs.validate()?;
            return Ok(json!({"crs":p.crs,"kind":"declared"}));
        }
        if let Ok(doc) = roxmltree::Document::parse(&r.text) {
            let mut count = 0usize;
            let (mut longitude, mut latitude) = (0., 0.);
            let (mut zones, mut hemispheres) = (0u64, 0u8);
            let zone_for = |lon: f64| (((lon + 180.) / 6.).floor() as u8 + 1).clamp(1, 60);
            for node in doc
                .descendants()
                .filter(|n| n.is_element() && n.tag_name().name() == "coordinates")
            {
                for tuple in node.text().unwrap_or("").split_whitespace() {
                    let mut parts = tuple.split(',');
                    let lon = parts.next().and_then(|s| s.parse::<f64>().ok());
                    let lat = parts.next().and_then(|s| s.parse::<f64>().ok());
                    if let (Some(lon), Some(lat)) = (lon, lat)
                        && lon.is_finite()
                        && lat.is_finite()
                        && (-180.0..=180.0).contains(&lon)
                        && (-80.0..=84.0).contains(&lat)
                    {
                        count += 1;
                        longitude += lon;
                        latitude += lat;
                        zones |= 1u64 << (zone_for(lon) - 1);
                        hemispheres |= if lat < 0. { 1 } else { 2 };
                    }
                }
            }
            if count > 0 {
                let zone = zone_for(longitude / count as f64);
                let south = latitude / (count as f64) < 0.;
                let spans = zones != 1u64 << (zone - 1) || hemispheres != if south { 1 } else { 2 };
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
    // These public operations validate their input themselves; avoid repeating polygon topology work.
    if !matches!(
        r.op.as_str(),
        "generate" | "export-mis" | "export-kml" | "cut-stroke"
    ) {
        p.validate()?;
    }
    match r.op.as_str() {
        "cut-stroke" => {
            let mut stroke = r.stroke.ok_or("Traço de corte ausente")?;
            if r.geographic {
                for point in &mut stroke {
                    *point = p.crs.forward(*point)?;
                }
            }
            return Ok(json!(editing::cut(&p, &stroke)?));
        }
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
        "validate" => return Ok(json!(p)),
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
            let mut out = Vec::with_capacity(p.lines.len());
            for (i, l) in std::mem::take(&mut p.lines).into_iter().enumerate() {
                if r.index.is_some() && r.index != Some(i) {
                    out.push(l);
                    continue;
                }
                let pieces = clip_line(&l, &p.polygons);
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
    if args.get(1).is_some_and(|s| s == "--update-identity") {
        println!(
            "{}",
            json!({"product":installation::PRODUCT,"version":updates::display_version(),"revision":updates::build_revision()})
        );
        return Ok(());
    }
    if args.get(1).is_some_and(|s| s == "--apply-update") {
        if let Err(error) = self_update::apply_plan(std::path::Path::new(
            args.get(2).ok_or("Plano de atualização ausente")?,
        )) {
            eprintln!("{error}");
        }
        return Ok(());
    }
    let updater = self_update::Manager::new();
    let mut restore_path = args
        .iter()
        .position(|s| s == "--restore-session")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from);
    let restore_session = restore_path
        .as_ref()
        .map(|path| self_update::load_session(path))
        .transpose()?;
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
    let recovery = if args.iter().any(|s| s == "--no-open")
        && std::env::var_os("SKYLAB_RECOVERY_DIR").is_none()
    {
        recovery::Recovery::at(
            std::env::temp_dir().join(format!("SkyLab-Test-Recovery-{}", std::process::id())),
        )
    } else {
        recovery::Recovery::default()
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
    let expected_host = format!("127.0.0.1:{actual}");
    let mut clean_shutdown = false;
    loop {
        if let Some(exit) = window
            .as_mut()
            .and_then(|child| child.try_wait().ok().flatten())
        {
            clean_shutdown = exit.success();
            break;
        }
        let Some(mut req) = server.recv_timeout(std::time::Duration::from_secs(1))? else {
            if window.is_none() && last_seen.elapsed().as_secs() > if page_seen { 90 } else { 600 }
            {
                break;
            }
            continue;
        };
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
        let mut shutdown = false;
        let (status, mime, body): (u16, &str, Cow<'static, str>) = match (
            req.method().as_str(),
            req.url(),
        ) {
            ("POST", "/activate") => {
                let child = desktop::open(&origin, false);
                if window.is_none() {
                    window = child;
                }
                (200, "application/json", "{\"ok\":true}".into())
            }
            ("GET", "/skytem.svg") => (
                200,
                "image/svg+xml",
                include_str!("../assets/skytem-normal.svg").into(),
            ),
            ("GET", "/brand.svg") => (
                200,
                "image/svg+xml",
                include_str!("../assets/skylab-icon.svg").into(),
            ),
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
            ("GET", "/recovery") => match recovery.load() {
                Ok(data) => (
                    200,
                    "application/json",
                    json!({"ok":true,"data":data}).to_string().into(),
                ),
                Err(error) => (
                    400,
                    "application/json",
                    json!({"ok":false,"error":error}).to_string().into(),
                ),
            },
            ("POST", "/recovery") => {
                let mut body = String::new();
                let result = req
                    .as_reader()
                    .take(40 * 1024 * 1024 + 1)
                    .read_to_string(&mut body)
                    .map_err(|e| e.to_string())
                    .and_then(|_| {
                        if body.len() > 40 * 1024 * 1024 {
                            Err("Sessão grande demais.".into())
                        } else {
                            serde_json::from_str::<Value>(&body).map_err(|e| e.to_string())
                        }
                    })
                    .and_then(|session| recovery.save(session));
                match result {
                    Ok(()) => (200, "application/json", "{\"ok\":true}".into()),
                    Err(error) => (
                        400,
                        "application/json",
                        json!({"ok":false,"error":error}).to_string().into(),
                    ),
                }
            }
            ("POST", "/recovery/clear") => match recovery.clear() {
                Ok(()) => (200, "application/json", "{\"ok\":true}".into()),
                Err(error) => (
                    400,
                    "application/json",
                    json!({"ok":false,"error":error}).to_string().into(),
                ),
            },
            ("GET", "/updates/check") => match updates::check() {
                Ok(check) => (
                    200,
                    "application/json",
                    json!({"ok":true,"data":check}).to_string().into(),
                ),
                Err(e) => (
                    400,
                    "application/json",
                    json!({"ok":false,"error":e}).to_string().into(),
                ),
            },
            ("GET", "/session/restore") => {
                let error = restore_path
                    .as_ref()
                    .and_then(|path| path.parent())
                    .and_then(|stage| std::fs::read(stage.join("result.json")).ok())
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                    .and_then(|value| value["error"].as_str().map(str::to_owned));
                (200,"application/json",json!({"ok":true,"data":restore_session.as_ref().map(|session|json!({"session":session,"error":error}))}).to_string().into())
            }
            ("POST", "/session/restored") => {
                if let Some(path) = restore_path.take() {
                    self_update::finish_restore(path);
                }
                (200, "application/json", "{\"ok\":true}".into())
            }
            ("GET", "/updates/progress") => (
                200,
                "application/json",
                json!({"ok":true,"data":updater.progress()})
                    .to_string()
                    .into(),
            ),
            ("POST", "/updates/start") => match updater.start() {
                Ok(()) => (200, "application/json", "{\"ok\":true}".into()),
                Err(error) => (
                    400,
                    "application/json",
                    json!({"ok":false,"error":error}).to_string().into(),
                ),
            },
            ("POST", "/updates/apply") => {
                let mut body = String::new();
                let result=req.as_reader().take(40*1024*1024+1).read_to_string(&mut body).map_err(|e|e.to_string()).and_then(|_|if body.len()>40*1024*1024{Err("Sessão grande demais para atualização. Salve e feche os projetos antes de tentar novamente.".into())}else{serde_json::from_str::<Value>(&body).map_err(|e|e.to_string())}).and_then(|value|updater.apply(value));
                match result {
                    Ok(()) => (200, "application/json", "{\"ok\":true}".into()),
                    Err(error) => (
                        400,
                        "application/json",
                        json!({"ok":false,"error":error}).to_string().into(),
                    ),
                }
            }
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
                )
                .into(),
            ),
            ("POST", "/shutdown") => {
                shutdown = true;
                (200, "application/json", "{\"ok\":true}".into())
            }
            ("GET", "/") => (
                200,
                "text/html; charset=utf-8",
                include_str!("../web/index.html")
                    .replace("__VERSION__", updates::display_version())
                    .into(),
            ),
            ("GET", "/home.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/home.js").into(),
            ),
            ("GET", "/hotkeys.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/hotkeys.js").into(),
            ),
            ("GET", "/recovery.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/recovery.js").into(),
            ),
            ("GET", "/xyz.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/xyz.js").into(),
            ),
            ("GET", "/keybindings.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/keybindings.js").into(),
            ),
            ("GET", "/tiles.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/tiles.js").into(),
            ),
            ("GET", "/editing.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/editing.js").into(),
            ),
            ("GET", "/updater.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/updater.js").into(),
            ),
            ("GET", "/session.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/session.js").into(),
            ),
            ("GET", "/studio.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/studio.js").into(),
            ),
            ("GET", "/app.js") => (
                200,
                "text/javascript; charset=utf-8",
                include_str!("../web/app.js").into(),
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
                include_str!("../web/style.css").into(),
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
                        json!({"ok":true,"data":v}).to_string().into(),
                    ),
                    Err(e) => (
                        400,
                        "application/json",
                        json!({"ok":false,"error":e}).to_string().into(),
                    ),
                }
            }
            _ => (404, "text/plain", "Não encontrado".into()),
        };
        // Static assets borrow embedded bytes; dynamic JSON/HTML keep their existing owned buffer.
        let bytes = body.as_bytes();
        let mut response = tiny_http::Response::new(
            tiny_http::StatusCode(status),
            vec![],
            bytes,
            Some(bytes.len()),
            None,
        );
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
            clean_shutdown = true;
            break;
        }
    }
    if clean_shutdown {
        let _ = recovery.clear();
    }
    Ok(())
}
