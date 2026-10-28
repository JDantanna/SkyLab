//! Local mission geometry. Working coordinates use a projected CRS in metres.
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
pub mod desktop;
pub mod editing;
pub mod installation;
pub mod projection;
pub mod self_update;
pub mod updates;
pub type Result<T> = std::result::Result<T, String>;
pub type Point = [f64; 2];
const EPS: f64 = 1e-7;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Crs {
    pub zone: u8,
    pub south: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Polygon {
    pub outer: Vec<Point>,
    pub holes: Vec<Vec<Point>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Line {
    pub name: String,
    pub points: Vec<Point>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub version: u8,
    pub crs: Crs,
    pub polygons: Vec<Polygon>,
    pub lines: Vec<Line>,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub spacing: f64,
    pub angle: f64,
    pub offset: f64,
    pub min_length: f64,
    pub prefix: String,
    pub first: u32,
    pub increment: u32,
    pub alternate: bool,
}
fn finite(p: Point) -> bool {
    p.iter().all(|v| v.is_finite())
}
pub fn distance(a: Point, b: Point) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
pub fn length(l: &Line) -> f64 {
    l.points.windows(2).map(|w| distance(w[0], w[1])).sum()
}
fn sub(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1]]
}
fn cross(a: Point, b: Point) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn lerp(a: Point, b: Point, t: f64) -> Point {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
pub fn valid_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_- .".contains(&b))
        && s.trim() == s
        && s.parse::<f64>().is_err()
        && !s.contains(' ')
}
impl Crs {
    pub fn validate(&self) -> Result<()> {
        if let Some(id) = &self.authority {
            return projection::validate(id);
        }
        if !(1..=60).contains(&self.zone) {
            return Err("Zona UTM deve estar entre 1 e 60.".into());
        }
        Ok(())
    }
    /// Snyder series, restricted to the ordinary six-degree UTM zone.
    pub fn forward(&self, p: Point) -> Result<Point> {
        self.validate()?;
        if let Some(id) = &self.authority {
            return projection::transform("EPSG:4326", id, p);
        }
        let [lon, lat] = p;
        let lon0 = self.zone as f64 * 6. - 183.;
        if !finite(p)
            || !(-80.0..=84.0).contains(&lat)
            || (lon - lon0).abs() > 3.000001
            || (lat.abs() > 1e-9 && (lat < 0.) != self.south)
        {
            return Err("Coordenada fora da zona/hemisfério UTM selecionados (WGS84).".into());
        }
        let a = 6378137.;
        let e = 0.0066943799901413165;
        let ep = e / (1. - e);
        let r = lat.to_radians();
        let d = (lon - lon0).to_radians();
        let n = a / (1. - e * r.sin().powi(2)).sqrt();
        let t = r.tan().powi(2);
        let c = ep * r.cos().powi(2);
        let aa = r.cos() * d;
        let m = a
            * ((1. - e / 4. - 3. * e * e / 64. - 5. * e.powi(3) / 256.) * r
                - (3. * e / 8. + 3. * e * e / 32. + 45. * e.powi(3) / 1024.) * (2. * r).sin()
                + (15. * e * e / 256. + 45. * e.powi(3) / 1024.) * (4. * r).sin()
                - 35. * e.powi(3) / 3072. * (6. * r).sin());
        let x = 500000.
            + 0.9996
                * n
                * (aa
                    + (1. - t + c) * aa.powi(3) / 6.
                    + (5. - 18. * t + t * t + 72. * c - 58. * ep) * aa.powi(5) / 120.);
        let y = 0.9996
            * (m + n
                * r.tan()
                * (aa * aa / 2.
                    + (5. - t + 9. * c + 4. * c * c) * aa.powi(4) / 24.
                    + (61. - 58. * t + t * t + 600. * c - 330. * ep) * aa.powi(6) / 720.))
            + if self.south { 10000000. } else { 0. };
        Ok([x, y])
    }
    pub fn inverse(&self, p: Point) -> Result<Point> {
        self.validate()?;
        if let Some(id) = &self.authority {
            return projection::transform(id, "EPSG:4326", p);
        }
        if !finite(p)
            || !(100000.0..=900000.0).contains(&p[0])
            || !(0.0..=10000000.0).contains(&p[1])
        {
            return Err("Coordenada UTM fora dos limites suportados.".into());
        }
        let a = 6378137.;
        let e: f64 = 0.0066943799901413165;
        let ep = e / (1. - e);
        let e1 = (1. - (1. - e).sqrt()) / (1. + (1. - e).sqrt());
        let m = (p[1] - if self.south { 10000000. } else { 0. }) / 0.9996;
        let mu = m / (a * (1. - e / 4. - 3. * e * e / 64. - 5. * e.powi(3) / 256.));
        let r = mu
            + (3. * e1 / 2. - 27. * e1.powi(3) / 32.) * (2. * mu).sin()
            + (21. * e1 * e1 / 16. - 55. * e1.powi(4) / 32.) * (4. * mu).sin()
            + 151. * e1.powi(3) / 96. * (6. * mu).sin()
            + 1097. * e1.powi(4) / 512. * (8. * mu).sin();
        let n = a / (1. - e * r.sin().powi(2)).sqrt();
        let rr = a * (1. - e) / (1. - e * r.sin().powi(2)).powf(1.5);
        let t = r.tan().powi(2);
        let c = ep * r.cos().powi(2);
        let d = (p[0] - 500000.) / (n * 0.9996);
        let lat = r
            - (n * r.tan() / rr)
                * (d * d / 2. - (5. + 3. * t + 10. * c - 4. * c * c - 9. * ep) * d.powi(4) / 24.
                    + (61. + 90. * t + 298. * c + 45. * t * t - 252. * ep - 3. * c * c)
                        * d.powi(6)
                        / 720.);
        let lon = (d - (1. + 2. * t + c) * d.powi(3) / 6.
            + (5. - 2. * c + 28. * t - 3. * c * c + 8. * ep + 24. * t * t) * d.powi(5) / 120.)
            / r.cos();
        let ll = [
            self.zone as f64 * 6. - 183. + lon.to_degrees(),
            lat.to_degrees(),
        ];
        self.forward(ll)?;
        Ok(ll)
    }
}
fn on_segment(p: Point, a: Point, b: Point) -> bool {
    let ab = sub(b, a);
    let ap = sub(p, a);
    cross(ab, ap).abs() <= EPS * distance(a, b).max(1.)
        && p[0] >= a[0].min(b[0]) - EPS
        && p[0] <= a[0].max(b[0]) + EPS
        && p[1] >= a[1].min(b[1]) - EPS
        && p[1] <= a[1].max(b[1]) + EPS
}
fn intersects(a: Point, b: Point, c: Point, d: Point) -> bool {
    if on_segment(a, c, d) || on_segment(b, c, d) || on_segment(c, a, b) || on_segment(d, a, b) {
        return true;
    }
    let x = cross(sub(b, a), sub(c, a));
    let y = cross(sub(b, a), sub(d, a));
    let z = cross(sub(d, c), sub(a, c));
    let w = cross(sub(d, c), sub(b, c));
    x * y < 0. && z * w < 0.
}
fn in_ring(p: Point, r: &[Point]) -> bool {
    let mut inside = false;
    for w in r.windows(2) {
        let (a, b) = (w[0], w[1]);
        if on_segment(p, a, b) {
            return true;
        }
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside
        }
    }
    inside
}
fn in_polygon(p: Point, g: &Polygon) -> bool {
    in_ring(p, &g.outer) && !g.holes.iter().any(|h| in_ring(p, h))
}
fn rings_cross(a: &[Point], b: &[Point]) -> bool {
    a.windows(2)
        .any(|x| b.windows(2).any(|y| intersects(x[0], x[1], y[0], y[1])))
}
fn validate_ring(r: &[Point]) -> Result<()> {
    if r.len() < 4 || r.len() > 2001 || r.first() != r.last() {
        return Err("Anel deve ser fechado, com 3 a 2000 vértices.".into());
    }
    let origin = r[0];
    let area: f64 = r
        .windows(2)
        .map(|w| cross(sub(w[0], origin), sub(w[1], origin)))
        .sum();
    if area.abs() < 0.01 {
        return Err("Polígono sem área útil.".into());
    }
    for (i, w) in r.windows(2).enumerate() {
        if distance(w[0], w[1]) < EPS {
            return Err("Vértices consecutivos repetidos.".into());
        }
        for (j, v) in r.windows(2).enumerate().skip(i + 2) {
            if i == 0 && j == r.len() - 2 {
                continue;
            }
            if intersects(w[0], w[1], v[0], v[1]) {
                return Err("Polígono possui auto-interseção.".into());
            }
        }
    }
    Ok(())
}
impl Project {
    pub fn empty(crs: Crs) -> Self {
        Self {
            version: if crs.authority.is_some() { 2 } else { 1 },
            crs,
            polygons: vec![],
            lines: vec![],
            warnings: vec![],
        }
    }
    pub fn validate(&self) -> Result<()> {
        self.validate_with_lines(&self.lines)
    }
    fn validate_with_lines(&self, lines: &[Line]) -> Result<()> {
        if ![1, 2].contains(&self.version) || (self.crs.authority.is_some() && self.version != 2) {
            return Err("Versão de projeto não suportada.".into());
        }
        self.crs.validate()?;
        let vertices: usize = self
            .polygons
            .iter()
            .map(|g| g.outer.len() + g.holes.iter().map(Vec::len).sum::<usize>())
            .sum();
        if vertices > 5000
            || lines.len() > 20000
            || lines.iter().map(|l| l.points.len()).sum::<usize>() > 200000
        {
            return Err("Projeto excede os limites desta versão.".into());
        }
        for g in &self.polygons {
            validate_ring(&g.outer)?;
            for (i, h) in g.holes.iter().enumerate() {
                validate_ring(h)?;
                if !in_ring(h[0], &g.outer) || rings_cross(h, &g.outer) {
                    return Err("Furo fora ou tocando o contorno.".into());
                }
                for other in g.holes.iter().skip(i + 1) {
                    if rings_cross(h, other) || in_ring(h[0], other) || in_ring(other[0], h) {
                        return Err("Furos sobrepostos ou aninhados.".into());
                    }
                }
            }
        }
        let mut names = std::collections::HashSet::new();
        for l in lines {
            if !valid_name(&l.name) || !names.insert(&l.name) {
                return Err(format!(
                    "Nome inválido ou repetido: {}. Use ASCII, sem espaços, até 64 caracteres.",
                    l.name
                ));
            }
            if l.points.len() < 2 || length(l) < 0.01 {
                return Err("Linha precisa de ao menos dois pontos distintos.".into());
            }
        }
        for p in self
            .polygons
            .iter()
            .flat_map(|g| g.outer.iter().chain(g.holes.iter().flatten()))
            .chain(lines.iter().flat_map(|l| l.points.iter()))
        {
            self.crs.inverse(*p)?;
        }
        Ok(())
    }
}
pub fn import_mis(text: &str, crs: Crs) -> Result<Project> {
    let mut p = Project::empty(crs);
    let mut current: Option<Line> = None;
    for (i, s) in text.trim_start_matches('\u{feff}').lines().enumerate() {
        let s = s.trim();
        if s.is_empty() {
            continue;
        }
        let mut cols = s.split_whitespace();
        let (first, second, extra) = (cols.next(), cols.next(), cols.next());
        if let (Some(x), Some(y), None) = (first, second, extra) {
            let x = x
                .parse::<f64>()
                .map_err(|_| format!("MIS: coordenada inválida na linha {}", i + 1))?;
            let y = y
                .parse::<f64>()
                .map_err(|_| format!("MIS: coordenada inválida na linha {}", i + 1))?;
            current
                .as_mut()
                .ok_or("MIS: coordenadas sem nome de linha")?
                .points
                .push([x, y]);
        } else if first.is_some() && second.is_none() && valid_name(s) {
            if let Some(l) = current.take() {
                p.lines.push(l)
            }
            current = Some(Line {
                name: s.into(),
                points: vec![],
            });
        } else {
            return Err(format!("MIS: registro inválido na linha {}", i + 1));
        }
    }
    if let Some(l) = current {
        p.lines.push(l)
    }
    if p.lines.is_empty() {
        return Err("MIS vazio.".into());
    }
    p.warnings.push(
        "MIS não informa datum ou zona. A projeção selecionada deve ser conferida na origem."
            .into(),
    );
    p.validate()?;
    Ok(p)
}
fn coords(n: roxmltree::Node, crs: &Crs) -> Result<Vec<Point>> {
    let s = n
        .descendants()
        .find(|n| n.has_tag_name("coordinates"))
        .and_then(|n| n.text())
        .ok_or("Geometria sem coordinates.")?;
    s.split_whitespace()
        .map(|c| {
            let mut parts = c.split(',');
            let (Some(lon), Some(lat)) = (parts.next(), parts.next()) else {
                return Err("Tupla KML inválida.".into());
            };
            let altitude = parts.next();
            if parts.next().is_some() {
                return Err("Tupla KML inválida.".into());
            }
            if altitude.is_some_and(|alt| !alt.parse::<f64>().is_ok_and(f64::is_finite)) {
                return Err("Altitude KML inválida.".into());
            }
            crs.forward([
                lon.parse().map_err(|_| "Longitude inválida")?,
                lat.parse().map_err(|_| "Latitude inválida")?,
            ])
        })
        .collect()
}
pub fn import_kml(text: &str, crs: Crs) -> Result<Project> {
    let doc = roxmltree::Document::parse(text).map_err(|e| format!("KML/XML inválido: {e}"))?;
    let mut p = Project::empty(crs);
    if !doc.root_element().has_tag_name("kml") {
        return Err("Documento não é KML.".into());
    }
    let mut used_names = std::collections::HashSet::new();
    for pm in doc.descendants().filter(|n| n.has_tag_name("Placemark")) {
        let explicit_name = pm
            .children()
            .find(|n| n.has_tag_name("name"))
            .and_then(|n| n.text())
            .filter(|s| !s.trim().is_empty());
        let attribute_id = pm
            .descendants()
            .find(|n| {
                n.has_tag_name("SimpleData")
                    && n.attribute("name")
                        .is_some_and(|s| s.eq_ignore_ascii_case("id"))
            })
            .and_then(|n| n.text());
        let name = explicit_name
            .map(str::to_owned)
            .or_else(|| {
                attribute_id.map(|s| {
                    if s.bytes().all(|b| b.is_ascii_digit()) {
                        format!("L{s}")
                    } else {
                        s.to_owned()
                    }
                })
            })
            .unwrap_or_else(|| format!("LINHA_{:06}", p.lines.len() + 1));
        let geometries: Vec<_> = pm
            .descendants()
            .filter(|n| n.has_tag_name("LineString"))
            .collect();
        for (i, n) in geometries.iter().enumerate() {
            let mut nm = if geometries.len() > 1 {
                format!("{name}_{}", i + 1)
            } else {
                name.to_string()
            };
            if !valid_name(&nm) {
                nm = format!("LINHA_{:06}", p.lines.len() + 1);
                p.warnings.push(format!(
                    "Nome KML '{name}' substituído por {nm} para compatibilidade MIS."
                ))
            }
            let base = nm.clone();
            let mut suffix = 2;
            while used_names.contains(&nm) {
                nm = format!("{}_{suffix}", &base[..base.len().min(55)]);
                suffix += 1;
            }
            if nm != base {
                p.warnings.push(format!(
                    "Nome repetido '{base}' importado como '{nm}'. Confira a numeração."
                ));
            }
            if explicit_name.is_none() {
                p.warnings.push(format!("Linha sem <name>: adotado '{nm}' a partir do ID ou sequência. Confira a numeração."));
            }
            used_names.insert(nm.clone());
            p.lines.push(Line {
                name: nm,
                points: coords(*n, &p.crs)?,
            });
        }
        for n in pm.descendants().filter(|n| n.has_tag_name("Polygon")) {
            let outer = n
                .children()
                .find(|n| n.has_tag_name("outerBoundaryIs"))
                .ok_or("Polígono sem contorno externo")?;
            let mut g = Polygon {
                outer: coords(outer, &p.crs)?,
                holes: vec![],
            };
            for h in n.children().filter(|n| n.has_tag_name("innerBoundaryIs")) {
                g.holes.push(coords(h, &p.crs)?)
            }
            p.polygons.push(g);
        }
    }
    if p.lines.is_empty() && p.polygons.is_empty() {
        return Err("KML sem Polygon ou LineString. KMZ deve ser descompactado antes.".into());
    }
    if p.polygons.is_empty() {
        p.warnings.push("Este KML contém linhas, sem bloco poligonal. Importe um polígono para gerar ou recortar a malha.".into())
    }
    if !p.polygons.is_empty() {
        p.warnings.push("Polígonos importados são blocos de levantamento; apenas anéis internos são furos. Arquivos de zonas proibidas não devem ser adicionados como blocos.".into());
    }
    if doc
        .descendants()
        .any(|n| n.has_tag_name("Point") || n.has_tag_name("Track") || n.has_tag_name("Model"))
    {
        p.warnings.push("Geometrias Point, Track e Model não foram importadas; somente Polygon e LineString são suportados.".into());
    }
    p.warnings.push(
        "KML convertido de WGS84; altitude e estilos não são usados no planejamento 2D.".into(),
    );
    p.validate()?;
    Ok(p)
}
/// Split every segment at all boundary crossings; retain intervals in the union of polygons.
pub fn clip_line(l: &Line, polys: &[Polygon]) -> Vec<Vec<Point>> {
    let mut result: Vec<Vec<Point>> = vec![];
    let mut active: Vec<Point> = vec![];
    let mut ts = Vec::new();
    for pair in l.points.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let ab = sub(b, a);
        ts.clear();
        ts.extend([0., 1.]);
        for g in polys {
            for ring in std::iter::once(&g.outer).chain(g.holes.iter()) {
                for w in ring.windows(2) {
                    let cd = sub(w[1], w[0]);
                    let den = cross(ab, cd);
                    if den.abs() > EPS {
                        let t = cross(sub(w[0], a), cd) / den;
                        let u = cross(sub(w[0], a), ab) / den;
                        if (0.0..=1.0).contains(&t) && (-EPS..=1. + EPS).contains(&u) {
                            ts.push(t)
                        }
                    } else if cross(ab, sub(w[0], a)).abs() < EPS {
                        let axis = if ab[0].abs() > ab[1].abs() { 0 } else { 1 };
                        if ab[axis].abs() > EPS {
                            for q in w {
                                let t = (q[axis] - a[axis]) / ab[axis];
                                if t > 0. && t < 1. {
                                    ts.push(t)
                                }
                            }
                        }
                    }
                }
            }
        }
        ts.sort_by(f64::total_cmp);
        ts.dedup_by(|a, b| (*a - *b).abs() < 1e-10);
        for w in ts.windows(2) {
            let x = lerp(a, b, w[0]);
            let y = lerp(a, b, w[1]);
            if distance(x, y) < EPS {
                continue;
            }
            if polys
                .iter()
                .any(|g| in_polygon(lerp(a, b, (w[0] + w[1]) / 2.), g))
            {
                if !active.is_empty() && distance(*active.last().unwrap(), x) > EPS {
                    result.push(std::mem::take(&mut active))
                }
                if active.is_empty() {
                    active.push(x)
                }
                active.push(y);
            } else if !active.is_empty() {
                result.push(std::mem::take(&mut active))
            }
        }
    }
    if !active.is_empty() {
        result.push(active)
    }
    result
}
pub fn generate(p: &Project, s: &Settings) -> Result<Vec<Line>> {
    p.validate()?;
    if p.polygons.is_empty() {
        return Err("Importe um KML com Polygon para gerar linhas.".into());
    }
    if ![s.spacing, s.angle, s.offset, s.min_length]
        .iter()
        .all(|v| v.is_finite())
        || s.spacing < 1.
        || s.min_length < 0.
        || s.offset < 0.
        || s.offset >= s.spacing
        || s.increment == 0
        || s.prefix.len() > 20
    {
        return Err("Use espaçamento ≥ 1 m, deslocamento entre 0 e espaçamento, comprimento ≥ 0 e incremento positivo.".into());
    }
    let r = s.angle.to_radians();
    let along = [r.sin(), r.cos()];
    let across = [r.cos(), -r.sin()];
    let dot = |p: Point, v: Point| p[0] * v[0] + p[1] * v[1];
    let mut vmin = f64::INFINITY;
    let mut vmax = f64::NEG_INFINITY;
    let mut umin = f64::INFINITY;
    let mut umax = f64::NEG_INFINITY;
    for q in p.polygons.iter().flat_map(|g| &g.outer) {
        vmin = vmin.min(dot(*q, across));
        vmax = vmax.max(dot(*q, across));
        umin = umin.min(dot(*q, along));
        umax = umax.max(dot(*q, along));
    }
    let count = ((vmax - vmin) / s.spacing).ceil() as usize;
    if count > 20000 {
        return Err("Mais de 20 mil linhas candidatas; aumente o espaçamento.".into());
    }
    let mut out = vec![];
    for i in 0..=count {
        let v = vmin + s.offset + i as f64 * s.spacing;
        if v > vmax + EPS {
            break;
        }
        let point = |u: f64| [along[0] * u + across[0] * v, along[1] * u + across[1] * v];
        let l = Line {
            name: String::new(),
            points: vec![point(umin - 1.), point(umax + 1.)],
        };
        for points in clip_line(&l, &p.polygons) {
            let number = s
                .first
                .checked_add(
                    (out.len() as u32)
                        .checked_mul(s.increment)
                        .ok_or("Numeração excedida")?,
                )
                .ok_or("Numeração excedida")?;
            let mut line = Line {
                name: format!("{}{number:06}", s.prefix),
                points,
            };
            let len = length(&line);
            if len + EPS < s.min_length || len < 0.01 {
                continue;
            }
            if s.alternate && out.len() % 2 == 1 {
                line.points.reverse()
            }
            out.push(line);
            if out.len() > 20000 {
                return Err("Mais de 20 mil segmentos gerados.".into());
            }
        }
    }
    if out.is_empty() {
        return Err("Nenhuma linha gerada com esses parâmetros.".into());
    }
    p.validate_with_lines(&out)?;
    Ok(out)
}
pub fn trim_line(l: &Line, start: f64, end: f64) -> Result<Line> {
    let total = length(l);
    if !start.is_finite() || !end.is_finite() || start < 0. || end <= start || end > total + EPS {
        return Err("Corte deve satisfazer 0 ≤ início < fim ≤ comprimento da linha.".into());
    }
    let mut acc = 0.;
    let mut pts = vec![];
    for w in l.points.windows(2) {
        let d = distance(w[0], w[1]);
        if d > EPS && acc + d > start && acc < end {
            let a = lerp(w[0], w[1], ((start - acc) / d).max(0.));
            let b = lerp(w[0], w[1], ((end - acc) / d).min(1.));
            if pts.is_empty() {
                pts.push(a)
            }
            pts.push(b)
        }
        acc += d;
        if acc >= end {
            break;
        }
    }
    Ok(Line {
        name: l.name.clone(),
        points: pts,
    })
}
pub fn export_mis(p: &Project) -> Result<String> {
    p.validate()?;
    if p.lines.is_empty() {
        return Err("Sem linhas para exportar.".into());
    }
    let mut s = String::new();
    for l in &p.lines {
        s.push_str(&l.name);
        s.push_str("\r\n");
        for q in &l.points {
            write!(s, "{:.2} {:.2}\r\n", q[0], q[1]).expect("writing to String cannot fail");
        }
    }
    import_mis(&s, p.crs.clone()).map_err(|e| format!("MIS arredondado não é válido: {e}"))?;
    Ok(s)
}
pub fn export_kml(p: &Project) -> Result<String> {
    p.validate()?;
    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><kml xmlns=\"http://www.opengis.net/kml/2.2\"><Document>",
    );
    for l in &p.lines {
        write!(
            s,
            "<Placemark><name>{}</name><LineString><coordinates>",
            l.name
        )
        .expect("writing to String cannot fail");
        for q in &l.points {
            let v = p.crs.inverse(*q)?;
            write!(s, "{:.10},{:.10},0 ", v[0], v[1]).expect("writing to String cannot fail");
        }
        s += "</coordinates></LineString></Placemark>";
    }
    s += "</Document></kml>";
    Ok(s)
}
