//! Panel local mínimo (solo std) para ver el catálogo y dar feedback sobre las
//! recetas aplicadas. Alternativa a GTK4 sin needing crates ni libs del sistema.

use crate::catalog;
use crate::paths::Paths;
use crate::recipe;
use crate::ui;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

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

fn feedback_path(paths: &Paths) -> std::path::PathBuf {
    paths.config.join("feedback.tsv")
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

fn apply_feedback(paths: &Paths, id: &str, good: bool) -> Result<(), String> {
    let mut entries = catalog::load(&paths.catalog);
    let Some(app) = entries.iter_mut().find(|e| e.id == id) else {
        return Err(format!("App desconocida: {id}"));
    };
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
        for rid in recipe_ids {
            if let Some((_, value)) = feedback.iter_mut().find(|(id, _)| *id == rid) {
                *value = (*value + delta).clamp(-0.95, 0.95);
            } else {
                feedback.push((rid, delta));
            }
        }
        save_feedback(paths, &feedback)?;
    }
    Ok(())
}

fn html(paths: &Paths) -> String {
    let apps = catalog::load(&paths.catalog);
    let recipes = recipe::parse_all(
        include_str!("../recipes/core.recipe"),
        &paths.recipes,
    );
    let mut page = String::from(
        "<!doctype html><html lang='es'><head><meta charset='utf-8'>\
         <title>Crisol · panel</title><style>\
         body{font:14px system-ui,sans-serif;background:#0f172a;color:#e2e8f0;margin:2rem;max-width:960px}\
         h1{color:#38bdf8}h2{margin-top:2rem;border-bottom:1px solid #334155;padding-bottom:.3rem}\
         table{border-collapse:collapse;width:100%}td,th{padding:.5rem;border-bottom:1px solid #1e293b;text-align:left}\
         a{color:#4ade80;text-decoration:none;margin-right:.6rem}a.bad{color:#f87171}\
         .muted{color:#94a3b8}code{color:#fbbf24}</style></head><body>\
         <h1>Crisol</h1><p class='muted'>Panel local de apps y recetas. Escuchar solo en 127.0.0.1.</p>",
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
    page.push_str("</table></body></html>");
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
    for connection in listener.incoming() {
        if let Ok(mut stream) = connection {
            handle(&mut stream, paths);
        }
    }
    Ok(())
}
