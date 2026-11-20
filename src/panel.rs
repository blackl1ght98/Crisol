//! Panel local mínimo (solo std) para ver el catálogo y dar feedback sobre las
//! recetas aplicadas. Alternativa a GTK4 sin crates ni libs del sistema.
//! Incluye historial, estadísticas y export/import del feedback.

use crate::catalog;
use crate::paths::Paths;
use crate::recipe;
use crate::ui;
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn escape(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '&' => "&amp;".into(),
            '<' => "&lt;".into(),
            '>' => "&gt;".into(),
            '"' => "&quot;".into(),
            _ => c.to_string(),
        })
        .collect()
}

fn feedback_path(paths: &Paths) -> PathBuf {
    paths.config.join("feedback.tsv")
}

fn history_path(paths: &Paths) -> PathBuf {
    paths.config.join("feedback.log")
}

fn load_feedback(paths: &Paths) -> Vec<(String, f32)> {
    let Ok(text) = fs::read_to_string(feedback_path(paths)) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() == 2 {
                Some((f[0].to_string(), f[1].parse::<f32>().ok()?))
            } else {
                None
            }
        })
        .collect()
}

fn save_feedback(paths: &Paths, entries: &[(String, f32)]) -> Result<(), String> {
    let mut text = String::new();
    for (id, delta) in entries {
        text.push_str(&format!("{id}\t{delta}\n"));
    }
    fs::write(feedback_path(paths), text).map_err(|e| format!("No se pudo guardar feedback: {e}"))
}

#[derive(Clone, Debug)]
struct HistoryEntry {
    ts: u64,
    app: String,
    verdict: String,
    recipes: String,
}

fn load_history(paths: &Paths) -> Vec<HistoryEntry> {
    let Ok(text) = fs::read_to_string(history_path(paths)) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() != 4 {
                return None;
            }
            Some(HistoryEntry {
                ts: f[0].parse().unwrap_or(0),
                app: f[1].to_string(),
                verdict: f[2].to_string(),
                recipes: f[3].to_string(),
            })
        })
        .collect()
}

fn append_history(paths: &Paths, app: &str, verdict: &str, recipes: &[String]) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut text = fs::read_to_string(history_path(paths)).unwrap_or_default();
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&format!("{ts}\t{app}\t{verdict}\t{}\n", recipes.join(";")));
    let _ = fs::write(history_path(paths), text);
}

#[derive(Clone, Debug)]
struct RecipeStat {
    id: String,
    ok: u32,
    bad: u32,
}

fn stats(paths: &Paths) -> Vec<RecipeStat> {
    let mut by_id: BTreeMap<String, RecipeStat> = BTreeMap::new();
    for entry in load_history(paths) {
        if entry.verdict != "ok" && entry.verdict != "bad" {
            continue;
        }
        for id in entry.recipes.split(';').filter(|s| !s.is_empty()) {
            let stat = by_id.entry(id.to_string()).or_insert_with(|| RecipeStat {
                id: id.to_string(),
                ok: 0,
                bad: 0,
            });
            if entry.verdict == "ok" {
                stat.ok += 1;
            } else {
                stat.bad += 1;
            }
        }
    }
    by_id.into_values().collect()
}

fn apply_feedback(paths: &Paths, id: &str, good: bool) -> Result<(), String> {
    let mut entries = catalog::load(&paths.catalog);
    let Some(app) = entries.iter_mut().find(|e| e.id == id) else {
        return Err(format!("App desconocida: {id}"));
    };
    let app_id = app.id.clone();
    app.last_status = if good {
        "ok-confirmado".to_string()
    } else {
        "revisar".to_string()
    };
    let recipe_ids: Vec<String> = app
        .recipes
        .split(';')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    catalog::save(&paths.catalog, &entries)?;

    if !recipe_ids.is_empty() {
        let mut feedback = load_feedback(paths);
        let delta = if good { 0.05 } else { -0.10 };
        for rid in recipe_ids.iter() {
            if let Some((_, value)) = feedback.iter_mut().find(|(id, _)| id == rid) {
                *value = (*value + delta).clamp(-0.95, 0.95);
            } else {
                feedback.push((rid.clone(), delta));
            }
        }
        save_feedback(paths, &feedback)?;
    }
    append_history(paths, &app_id, if good { "ok" } else { "bad" }, &recipe_ids);
    Ok(())
}

/// Exporta `feedback.tsv` a un archivo (o a `feedback-export.tsv`).
pub fn export_feedback(paths: &Paths, out: Option<&Path>) -> Result<(), String> {
    let dest = out
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("feedback-export.tsv"));
    let text = fs::read_to_string(feedback_path(paths)).unwrap_or_default();
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&dest, text).map_err(|e| format!("No se pudo exportar: {e}"))?;
    ui::ok(&format!("Feedback exportado a {}", dest.display()));
    Ok(())
}

/// Importa un `feedback.tsv` compartido y registró la operación en el historial.
pub fn import_feedback(paths: &Paths, src: &Path) -> Result<(), String> {
    let text =
        fs::read_to_string(src).map_err(|e| format!("No se pudo leer {}: {e}", src.display()))?;
    let mut entries = Vec::new();
    for line in text.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 2 {
            continue;
        }
        let delta = f[1]
            .parse::<f32>()
            .map_err(|_| format!("Delta invalido para '{}': {}", f[0], f[1]))?;
        entries.push((f[0].to_string(), delta));
    }
    if entries.is_empty() {
        return Err(format!("No hay entradas validas en {}", src.display()));
    }
    save_feedback(paths, &entries)?;
    let ids: Vec<String> = entries.iter().map(|(id, _)| id.clone()).collect();
    append_history(paths, "import", "import", &ids);
    ui::ok(&format!(
        "Importadas {} entradas de feedback desde {}",
        ids.len(),
        src.display()
    ));
    Ok(())
}

/// Muestra por terminal el resumen de votos por receta.
pub fn show_stats(paths: &Paths) -> Result<(), String> {
    let rows = stats(paths);
    if rows.is_empty() {
        ui::info("Todavía no hay votos registrados (los imports no cuentan como votos)");
        return Ok(());
    }
    println!("{:<34} {:>5} {:>5} {:>8}", "RECETA", "OK", "BAD", "% OK");
    for r in rows {
        let total = r.ok + r.bad;
        let pct = if total == 0 {
            0.0
        } else {
            r.ok as f32 * 100.0 / total as f32
        };
        println!("{:<34} {:>5} {:>5} {:>7.0}%", r.id, r.ok, r.bad, pct);
    }
    Ok(())
}

fn html(paths: &Paths) -> String {
    let apps = catalog::load(&paths.catalog);
    let recipes = recipe::parse_all(include_str!("../recipes/core.recipe"), &paths.recipes);
    let stats = stats(paths);
    let history = load_history(paths);
    let mut page = String::from(
        "<!doctype html><html lang='es'><head><meta charset='utf-8'>\
         <title>Crisol · panel</title><style>\
         body{font:14px system-ui,sans-serif;background:#0f172a;color:#e2e8f0;margin:2rem;max-width:960px}\
         h1{color:#38bdf8}h2{margin-top:2rem;border-bottom:1px solid #334155;padding-bottom:.3rem}\
         table{border-collapse:collapse;width:100%}td,th{padding:.5rem;border-bottom:1px solid #1e293b;text-align:left}\
         a{color:#4ade80;text-decoration:none;margin-right:.6rem}a.bad{color:#f87171}\
         .muted{color:#94a3b8}code{color:#fbbf24}</style></head><body>\
         <h1>Crisol</h1><p class='muted'>Panel local de apps y recetas. Escuchar solo en 127.0.0.1.</p>\
         <p><a href='/export'>Descargar feedback.tsv</a> · <code>crisol feedback stats</code></p>",
    );
    page.push_str("<h2>Aplicaciones instaladas</h2>");
    if apps.is_empty() {
        page.push_str("<p class='muted'>Todavía no hay apps instaladas.</p>");
    } else {
        page.push_str("<table><tr><th>Nombre</th><th>ID</th><th>Estado</th><th>Recetas</th><th>Acciones</th></tr>");
        for app in apps {
            page.push_str(&format!(
                "<tr><td>{}</td><td><code>{}</code></td><td>{}</td><td class='muted'>{}</td>\
                 <td><a href='/ok?id={}'>✔ Funcionó</a><a class='bad' href='/bad?id={}'>✘ No funcionó</a></td></tr>",
                escape(&app.name),
                escape(&app.id),
                escape(&app.last_status),
                escape(&app.recipes),
                escape(&app.id),
                escape(&app.id)
            ));
        }
        page.push_str("</table>");
    }
    page.push_str("<h2>Recetas (score = confianza×100 + prioridad)</h2>");
    page.push_str("<table><tr><th>ID</th><th>Score</th><th>Origen</th></tr>");
    for r in recipes {
        page.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{:.0}</td><td class='muted'>{}</td></tr>",
            escape(&r.id),
            r.score(),
            escape(&r.source)
        ));
    }
    page.push_str("</table>");

    page.push_str("<h2>Estadísticas de feedback</h2>");
    if stats.is_empty() {
        page.push_str("<p class='muted'>Sin votos todavía.</p>");
    } else {
        page.push_str("<table><tr><th>Receta</th><th>OK</th><th>No</th><th>% OK</th></tr>");
        for r in stats {
            let total = r.ok + r.bad;
            let pct = if total == 0 {
                0.0
            } else {
                r.ok as f32 * 100.0 / total as f32
            };
            page.push_str(&format!(
                "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{:.0}%</td></tr>",
                escape(&r.id),
                r.ok,
                r.bad,
                pct
            ));
        }
        page.push_str("</table>");
    }

    page.push_str("<h2>Historial reciente</h2>");
    if history.is_empty() {
        page.push_str("<p class='muted'>Sin historial todavía.</p>");
    } else {
        page.push_str(
            "<table><tr><th>Fecha</th><th>App</th><th>Veredicto</th><th>Recetas</th></tr>",
        );
        for e in history.iter().rev().take(20) {
            page.push_str(&format!(
                "<tr><td class='muted'>{}</td><td>{}</td><td>{}</td><td class='muted'>{}</td></tr>",
                e.ts,
                escape(&e.app),
                escape(&e.verdict),
                escape(&e.recipes)
            ));
        }
        page.push_str("</table>");
    }
    page.push_str("</body></html>");
    page
}

fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body.as_bytes());
    let _ = stream.flush();
}

fn handle(stream: &mut TcpStream, paths: &Paths) {
    let mut buf = [0u8; 8192];
    let Ok(n) = stream.read(&mut buf) else {
        return;
    };
    let request = String::from_utf8_lossy(&buf[..n]).to_string();
    let first = request.lines().next().unwrap_or("");
    let target = first.split_whitespace().nth(1).unwrap_or("/");
    let (route, query) = target.split_once('?').unwrap_or((target, ""));

    match route {
        "/export" => {
            let text = fs::read_to_string(feedback_path(paths)).unwrap_or_default();
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Disposition: attachment; filename=\"feedback.tsv\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                text.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(text.as_bytes());
            let _ = stream.flush();
        }
        "/ok" | "/bad" => {
            let id = query
                .split('&')
                .find_map(|p| p.strip_prefix("id="))
                .map(|s| s.replace("%20", " "))
                .unwrap_or_default();
            match apply_feedback(paths, &id, route == "/ok") {
                Ok(()) => {
                    ui::info(&format!(
                        "Feedback de '{}': {}",
                        id,
                        if route == "/ok" { "+0.05" } else { "-0.10" }
                    ));
                    let redirect =
                        "HTTP/1.1 302 Found\r\nLocation: /\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    let _ = stream.write_all(redirect.as_bytes());
                    let _ = stream.flush();
                }
                Err(e) => respond(stream, "404 Not Found", "text/plain", &escape(&e)),
            }
        }
        _ => {
            let page = html(paths);
            respond(stream, "200 OK", "text/html", &page);
        }
    }
}

pub fn serve(paths: &Paths, port: u16) -> Result<(), String> {
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("No se pudo escuchar en 127.0.0.1:{port}: {e}"))?;
    ui::ok(&format!("Panel disponible en http://127.0.0.1:{port}"));
    ui::info("Pulsa Ctrl+C para cerrar");
    for mut stream in listener.incoming().flatten() {
        handle(&mut stream, paths);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_paths(name: &str) -> (Paths, PathBuf) {
        let root = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&root);
        let config = root.join("config/crisol");
        let data = root.join("data/crisol");
        let cache = root.join("cache/crisol");
        fs::create_dir_all(&config).unwrap();
        fs::create_dir_all(&data).unwrap();
        let paths = Paths {
            home: root.clone(),
            data: data.clone(),
            config: config.clone(),
            cache,
            prefixes: data.join("prefixes"),
            backups: data.join("backups"),
            logs: data.join("logs"),
            recipes: config.join("recipes"),
            catalog: data.join("catalog.tsv"),
            applications: root.join("data/applications"),
            icons: root.join("data/icons/hicolor/scalable/apps"),
            runtime: None,
        };
        (paths, root)
    }

    #[test]
    fn history_and_stats() {
        let (paths, root) = temp_paths("crisol_panel_test");
        fs::write(
            history_path(&paths),
            "100\tapp1\tok\treceta-a;receta-b\n101\tapp2\tbad\treceta-a\n102\tapp3\timport\treceta-c\n",
        )
        .unwrap();
        let rows = stats(&paths);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, "receta-a");
        assert_eq!((rows[0].ok, rows[0].bad), (1, 1));
        assert_eq!(load_history(&paths).len(), 3);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn export_and_import_feedback() {
        let (paths, root) = temp_paths("crisol_panel_io_test");
        let src = root.join("compartido.tsv");
        fs::write(&src, "receta-x\t-0.1\nreceta-y\t0.05\n").unwrap();
        import_feedback(&paths, &src).unwrap();
        let out = root.join("exportado.tsv");
        export_feedback(&paths, Some(&out)).unwrap();
        assert_eq!(
            fs::read_to_string(&out).unwrap(),
            "receta-x\t-0.1\nreceta-y\t0.05\n"
        );
        assert!(load_history(&paths)[0].verdict == "import");
        fs::remove_dir_all(root).unwrap();
    }
}
