use crate::catalog::{self, Entry};
use crate::cli::Options;
use crate::desktop;
use crate::detector::{self, InstallerKind};
use crate::paths::Paths;
use crate::prefix::{self, Prefix};
use crate::recipe::{self, Action, MatchCtx};
use crate::runner::{self, RunOptions};
use crate::ui;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Report {
    pub id: String,
    pub name: String,
    pub exe: Option<String>,
    pub attempts: usize,
    pub applied: Vec<String>,
    pub log_tail: String,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn set_env(env: &mut Vec<(String, String)>, key: &str, val: &str) {
    if let Some(e) = env.iter_mut().find(|(k, _)| k == key) {
        e.1 = val.to_string();
    } else {
        env.push((key.to_string(), val.to_string()));
    }
}

fn resolve_engine(preferred: &Option<String>) -> String {
    let mut candidates: Vec<String> = Vec::new();
    if let Some(p) = preferred {
        candidates.push(p.clone());
    }
    candidates.extend(
        ["wine", "wine64", "wine-stable"]
            .iter()
            .map(|s| s.to_string()),
    );
    for c in candidates {
        if runner::which(&c).is_some() {
            return c;
        }
    }
    "wine".to_string()
}

/// Argumentos del instalador. Por defecto se pide modo silencioso según el
/// tipo detectado; `--interactive` fuerza la interfaz gráfica.
fn installer_args(
    installer: &Path,
    detection: &detector::Detection,
    interactive: bool,
) -> Vec<String> {
    let path = installer.to_string_lossy().to_string();
    if interactive {
        return if detection.kind == InstallerKind::Msi {
            vec!["msiexec".into(), "/i".into(), path, "/norestart".into()]
        } else {
            vec![path]
        };
    }
    match detection.kind {
        InstallerKind::Msi => vec![
            "msiexec".into(),
            "/i".into(),
            path,
            "/qn".into(),
            "/norestart".into(),
        ],
        InstallerKind::Nsis => vec![path, "/S".into()],
        InstallerKind::Inno => vec![
            path,
            "/VERYSILENT".into(),
            "/SUPPRESSMSGBOXES".into(),
            "/NORESTART".into(),
            "/NOCANCEL".into(),
        ],
        InstallerKind::InstallShield => vec![path, "/s".into(), "/v/qn".into()],
        InstallerKind::Wix => vec![path, "/quiet".into(), "/norestart".into()],
        InstallerKind::PlainExe | InstallerKind::Unknown => vec![path],
    }
}

fn walk_exes(root: &Path, dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > 10 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        // file_type() NO sigue symlinks. En un prefijo Wine, dosdevices/z:
        // apunta a '/' y seguirlo haria recursar por todo el sistema.
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() {
            continue;
        }
        let path = entry.path();
        if kind.is_dir() {
            walk_exes(root, &path, depth + 1, out);
        } else if kind.is_file()
            && path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("exe"))
                .unwrap_or(false)
        {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_path_buf());
            }
        }
    }
}

fn collect_executables(prefix: &Path) -> HashSet<String> {
    let mut all = Vec::new();
    walk_exes(prefix, prefix, 0, &mut all);
    all.into_iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect()
}

fn is_candidate(name: &str) -> bool {
    let n = name.to_lowercase();
    if !n.ends_with(".exe") {
        return false;
    }
    let blocked = [
        "unins",
        "uninstall",
        "setup",
        "update",
        "updater",
        "dotnet",
        "wineboot",
        "cmd.exe",
        "explorer.exe",
        "notepad.exe",
        "regedit.exe",
        "msiexec.exe",
        "helper",
        "service",
        "crashhandler",
        "uninstaller",
    ];
    !blocked.iter().any(|b| n.contains(b))
}

fn find_installed_exe(
    prefix: &Prefix,
    before: &HashSet<String>,
    start: u64,
    override_rel: Option<&str>,
) -> Option<PathBuf> {
    let mut all = Vec::new();
    walk_exes(&prefix.path, &prefix.path, 0, &mut all);

    if let Some(ov) = override_rel {
        if let Some(found) = all
            .iter()
            .find(|p| p.to_string_lossy().replace('\\', "/") == ov)
        {
            return Some(found.clone());
        }
    }

    let mut scored: Vec<(i32, PathBuf)> = all
        .into_iter()
        .filter_map(|rel| {
            let name = rel.file_name()?.to_string_lossy().to_string();
            if !is_candidate(&name) {
                return None;
            }
            let abs = prefix.path.join(&rel);
            let modified = fs::metadata(&abs)
                .and_then(|m| m.modified())
                .map(|t| {
                    t.duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            let recent = modified >= start.saturating_sub(5);
            let new = !before.contains(&rel.to_string_lossy().replace('\\', "/"));
            if !recent && !new {
                return None;
            }
            let rel_s = rel.to_string_lossy().replace('\\', "/");
            let mut score: i32 = 0;
            if rel_s.contains("program files") {
                score -= 20;
            }
            score += rel_s.matches('/').count() as i32 * 2;
            score += (name.len() / 4) as i32;
            let low = name.to_lowercase();
            if low.contains("install") || low.contains("setup") {
                score += 60;
            }
            Some((score, rel))
        })
        .collect();
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    scored.into_iter().next().map(|(_, p)| p)
}

fn log_tail(log: &str, n: usize) -> String {
    let lines: Vec<&str> = log.lines().collect();
    let from = lines.len().saturating_sub(n);
    lines[from..].join("\n")
}

pub fn install(paths: &Paths, opts: &Options, installer_path: &Path) -> Result<Report, String> {
    let _ = opts.yes;
    let installer = fs::canonicalize(installer_path)
        .map_err(|e| format!("No se encontro {}: {e}", installer_path.display()))?;
    let detection = detector::detect(&installer)?;
    let id = prefix::make_id(&installer);
    let name = installer
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "app".into());

    ui::header("Analizando instalador");
    ui::info(&format!("Tipo: {:?}", detection.kind));
    ui::info(&format!("Arquitectura: {:?}", detection.arch));
    ui::detail(&format!("Pistas: {}", detection.hints.join(", ")));

    ui::step(1, 6, "Creando prefijo aislado");
    let mut prefix = prefix::create(paths, &id, opts.sandbox)?;
    prefix.snapshot(paths)?;
    ui::detail("Snapshot del prefijo listo");
    let start = now_secs();
    let before = collect_executables(&prefix.path);
    ui::detail(&format!("Ejecutables previos detectados: {}", before.len()));

    let recipes = recipe::parse_all(include_str!("../recipes/core.recipe"), &paths.recipes);
    let mut env = prefix::base_env(&prefix.path, &paths.cache);
    let mut overrides: Vec<String> = vec!["mscoree,mshtml=".into()];
    let mut engine = resolve_engine(&opts.engine);
    if let Some(wanted) = &opts.engine {
        if !runner::which(wanted).is_some() {
            ui::warn(&format!(
                "Motor '{wanted}' no encontrado; se usa '{engine}'"
            ));
        }
    }

    let mut used: Vec<String> = Vec::new();
    let mut applied: Vec<String> = Vec::new();
    let mut last_log = String::new();
    let mut exit_code = None;
    let mut exe_override: Option<String> = opts.exe.clone();
    let mut success = false;
    let mut attempts = 0;

    for attempt in 1..=opts.max_attempts {
        attempts = attempt;
        let short_engine = Path::new(&engine)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| engine.clone());
        ui::step(
            2,
            6,
            &format!(
                "Ejecutando instalador (intento {}/{}, motor {})",
                attempt, opts.max_attempts, short_engine
            ),
        );
        ui::detail(&format!(
            "Comando: {engine} {}",
            installer_args(&installer, &detection, opts.interactive).join(" ")
        ));
        set_env(&mut env, "WINEDLLOVERRIDES", &overrides.join(";"));
        let result = runner::run(&RunOptions {
            prefix: &prefix.path,
            command: &engine,
            args: installer_args(&installer, &detection, opts.interactive),
            env: env.clone(),
            sandbox: opts.sandbox,
            cache: &paths.cache,
            runtime: paths.runtime.as_deref(),
            timeout_secs: Some(opts.timeout_secs),
            label: &format!("wine/{attempt}"),
        })?;
        let log_file = paths.logs.join(format!("{id}-{attempt}.log"));
        if let Err(e) = fs::write(&log_file, &result.log) {
            ui::warn(&format!("No se pudo guardar el log: {e}"));
        } else {
            ui::detail(&format!("Log: {}", log_file.display()));
        }
        last_log = result.log.clone();
        exit_code = result.code;
        if result.timed_out {
            ui::warn("El instalador fue detenido por tiempo limite");
        }

        if exit_code == Some(0) {
            success = true;
            break;
        }

        let ctx = MatchCtx {
            log: &last_log,
            detector: &detection.hints,
            exit_code,
            prefix: &prefix.path,
        };
        let selected = recipe::select(&recipes, &ctx, &used, 3);
        if selected.is_empty() {
            ui::err("No hay recetas que coincidan con este fallo");
            break;
        }

        ui::step(3, 6, "Aplicando recetas de compatibilidad");
        for r in selected {
            used.push(r.id.clone());
            applied.push(format!("{} [{}]", r.id, r.source));
            if let Some(msg) = &r.message {
                ui::info(msg);
            }
            for a in r
                .actions
                .iter()
                .filter(|a| matches!(a, Action::Reset(true)))
            {
                let _ = a;
                prefix.rollback()?;
                prefix.snapshot(paths)?;
                ui::info("Prefijo restaurado al estado previo");
            }
            for action in &r.actions {
                match action {
                    Action::Reset(_) => {}
                    Action::Retry => {}
                    Action::Winetricks(verbs) => {
                        let cmd = runner::which("winetricks")
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "winetricks".into());
                        let mut args = vec!["--unattended".to_string()];
                        args.extend(verbs.clone());
                        match runner::run(&RunOptions {
                            prefix: &prefix.path,
                            command: &cmd,
                            args,
                            env: env.clone(),
                            sandbox: opts.sandbox,
                            cache: &paths.cache,
                            runtime: paths.runtime.as_deref(),
                            timeout_secs: Some(opts.timeout_secs),
                            label: "winetricks",
                        }) {
                            Ok(res) => {
                                let log_file =
                                    paths.logs.join(format!("{id}-{attempt}-winetricks.log"));
                                let _ = fs::write(&log_file, &res.log);
                                if res.code == Some(0) {
                                    ui::ok(&format!("winetricks {} aplicado", verbs.join(" ")));
                                } else {
                                    ui::warn(&format!(
                                        "winetricks {} devolvio {:?}",
                                        verbs.join(" "),
                                        res.code
                                    ));
                                }
                            }
                            Err(e) => ui::warn(&format!("winetricks fallo: {e}")),
                        }
                    }
                    Action::Override(pairs) => {
                        for (k, v) in pairs {
                            overrides.push(format!("{k}={v}"));
                        }
                    }
                    Action::Env(pairs) => {
                        for (k, v) in pairs {
                            if k == "WINEDLLOVERRIDES" {
                                overrides = v
                                    .split(';')
                                    .filter(|s| !s.is_empty())
                                    .map(|s| s.to_string())
                                    .collect();
                            } else {
                                set_env(&mut env, k, v);
                            }
                        }
                    }
                    Action::Engine(wanted) => {
                        let new_engine = resolve_engine(&Some(wanted.clone()));
                        if new_engine != engine {
                            ui::info(&format!("Cambiando motor a {new_engine}"));
                            engine = new_engine;
                        }
                    }
                    Action::SetExe(rel) => exe_override = Some(rel.clone()),
                }
            }
        }
    }

    if !success {
        let applied_str = if applied.is_empty() {
            "ninguna".to_string()
        } else {
            applied.join(", ")
        };
        let mut msg = format!(
            "La instalacion fallo despues de {attempts} intentos (exit {:?}).\nRecetas aplicadas: {}\n\nUltimas lineas del log:\n{}",
            exit_code,
            applied_str,
            log_tail(&last_log, 20)
        );
        msg.push_str("\n\nEl prefijo se conserva en ");
        msg.push_str(&prefix.path.to_string_lossy());
        if opts.vm_fallback {
            ui::info("Agotados los intentos: pasando el instalador a la VM de Windows...");
            let _ = crate::vm::handoff(paths, &installer);
        } else {
            ui::info(
                "Si el fallo es por anti-cheat/DRM/drivers, un Windows real suele resolverlo:",
            );
            ui::detail("crisol vm create windows --iso /ruta/Windows.iso");
            ui::detail(&format!("crisol install {} --vm", installer.display()));
            ui::detail("o vuelve a intentar con: --vm-fallback");
        }
        return Err(msg);
    }

    ui::step(4, 6, "Localizando el ejecutable instalado");
    let exe_rel =
        find_installed_exe(&prefix, &before, start, exe_override.as_deref()).ok_or_else(|| {
            format!(
                "La instalacion termino pero no se encontro un .exe razonable en {}",
                prefix.path.display()
            )
        })?;
    let exe_abs = prefix.path.join(&exe_rel);
    ui::ok(&format!("Ejecutable: {}", exe_rel.display()));

    ui::step(5, 6, "Integrando en el escritorio");
    let entry = Entry {
        id: id.clone(),
        name: name.clone(),
        exe: exe_abs.to_string_lossy().to_string(),
        args: String::new(),
        prefix: prefix.path.to_string_lossy().to_string(),
        installer: installer.to_string_lossy().to_string(),
        icon: desktop::icon_for(paths, &exe_abs, &id),
        engine: engine.clone(),
        created: now_secs(),
        last_status: "ok".into(),
        recipes: applied
            .iter()
            .filter_map(|s| s.split(' ').next())
            .collect::<Vec<_>>()
            .join(";"),
    };
    let desktop_file = desktop::write_app(paths, &entry)?;
    ui::detail(&format!("Acceso directo: {}", desktop_file.display()));

    ui::step(6, 6, "Guardando en el catalogo");
    catalog::upsert(&paths.catalog, entry)?;
    prefix.discard_snapshot(paths);
    ui::ok(&format!("'{name}' instalado. ID: {id}"));
    desktop::notify("Crisol", &format!("{name} instalado y listo para usar"));

    if !applied.is_empty() {
        ui::info(&format!("Recetas aplicadas: {}", applied.join(", ")));
    }

    Ok(Report {
        id,
        name,
        exe: Some(exe_abs.to_string_lossy().to_string()),
        attempts,
        applied,
        log_tail: log_tail(&last_log, 10),
    })
}

pub fn launch(paths: &Paths, id: &str) -> Result<(), String> {
    let entries = catalog::load(&paths.catalog);
    let entry = entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("No hay ninguna app con id '{id}'"))?;
    let exe = PathBuf::from(&entry.exe);
    if !exe.exists() {
        return Err(format!("El ejecutable ya no existe: {}", exe.display()));
    }
    let engine = resolve_engine(&Some(entry.engine.clone()));
    let mut args = vec![exe.to_string_lossy().to_string()];
    args.extend(entry.args.split_whitespace().map(|s| s.to_string()));
    let env = prefix::base_env(Path::new(&entry.prefix), &paths.cache);
    ui::info(&format!("Lanzando {} con {}", entry.name, engine));
    let res = runner::run(&RunOptions {
        prefix: Path::new(&entry.prefix),
        command: &engine,
        args,
        env,
        sandbox: false,
        cache: &paths.cache,
        runtime: paths.runtime.as_deref(),
        timeout_secs: None,
        label: "app",
    })?;
    if res.code != Some(0) {
        ui::warn(&format!("La app termino con codigo {:?}", res.code));
    }
    Ok(())
}

pub fn repair(paths: &Paths, id: &str) -> Result<(), String> {
    let entries = catalog::load(&paths.catalog);
    let entry = entries
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("No hay ninguna app con id '{id}'"))?;
    let opts = Options {
        sandbox: true,
        max_attempts: 5,
        engine: Some(entry.engine.clone()),
        yes: true,
        timeout_secs: 900,
        exe: None,
        interactive: false,
        vm: false,
        vm_fallback: false,
        iso: None,
        disk: 40,
        ram: 4,
        cpus: 2,
        gpu: None,
        port: 8765,
    };
    ui::info(&format!(
        "Reparando {} desde {}",
        entry.name, entry.installer
    ));
    install(paths, &opts, Path::new(&entry.installer))?;
    Ok(())
}

pub fn list(paths: &Paths) -> Result<(), String> {
    let entries = catalog::load(&paths.catalog);
    if entries.is_empty() {
        ui::info("El catalogo esta vacio. Instala algo con: crisol install archivo.exe");
        return Ok(());
    }
    println!("{:<28} {:<28} {:<10} EJECUTABLE", "ID", "NOMBRE", "ESTADO");
    for e in entries {
        println!(
            "{:<28} {:<28} {:<10} {}",
            e.id,
            e.name.chars().take(28).collect::<String>(),
            e.last_status,
            e.exe
        );
    }
    Ok(())
}

pub fn remove(paths: &Paths, id: &str) -> Result<(), String> {
    let entry = catalog::remove(&paths.catalog, id)?
        .ok_or_else(|| format!("No hay ninguna app con id '{id}'"))?;
    let prefix = PathBuf::from(&entry.prefix);
    if prefix.starts_with(&paths.prefixes) {
        fs::remove_dir_all(&prefix).ok();
    }
    desktop::remove_app(paths, id);
    ui::ok(&format!("{} eliminada", entry.name));
    Ok(())
}

pub fn doctor(paths: &Paths) -> Result<(), String> {
    ui::header("Diagnostico del entorno");
    for bin in [
        "wine",
        "wineboot",
        "winetricks",
        "bwrap",
        "xdg-mime",
        "cp",
        "mv",
    ] {
        match runner::which(bin) {
            Some(p) => ui::ok(&format!("{bin}: {}", p.display())),
            None => ui::warn(&format!("{bin}: no encontrado")),
        }
    }
    let recipes = recipe::parse_all(include_str!("../recipes/core.recipe"), &paths.recipes);
    ui::info(&format!("Recetas cargadas: {}", recipes.len()));
    if let Ok(out) = Command::new("xdg-mime")
        .args(["query", "default", "application/x-msdownload"])
        .output()
    {
        let default = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if default.is_empty() {
            ui::warn("No hay manejador por defecto para .exe (ejecuta crisol setup-desktop)");
        } else {
            ui::info(&format!("Manejador de .exe: {default}"));
        }
    }
    ui::info(&format!("HOME: {}", paths.home.display()));
    ui::info(&format!("Prefijos en: {}", paths.prefixes.display()));
    ui::info(&format!("Catalogo: {}", paths.catalog.display()));
    Ok(())
}

pub fn show_recipes(paths: &Paths) -> Result<(), String> {
    let recipes = recipe::parse_all(include_str!("../recipes/core.recipe"), &paths.recipes);
    println!("{:<26} {:>6} {:>8}  ORIGEN", "ID", "SCORE", "REGISTRO");
    for r in recipes {
        println!(
            "{:<26} {:>6.0} {:>8}  {}",
            r.id,
            r.score(),
            r.matchers.len(),
            r.source
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detector::Arch;

    fn detection(kind: InstallerKind) -> detector::Detection {
        detector::Detection {
            kind,
            arch: Arch::X64,
            hints: Vec::new(),
        }
    }

    #[test]
    fn silent_arguments_by_installer_type() {
        let exe = Path::new("/tmp/app.exe");
        assert_eq!(
            installer_args(exe, &detection(InstallerKind::Nsis), false),
            vec!["/tmp/app.exe".to_string(), "/S".to_string()]
        );
        assert!(installer_args(exe, &detection(InstallerKind::Inno), false)
            .contains(&"/VERYSILENT".to_string()));
        assert!(
            installer_args(exe, &detection(InstallerKind::Msi), false).contains(&"/qn".to_string())
        );
        assert!(
            !installer_args(exe, &detection(InstallerKind::Nsis), true).contains(&"/S".to_string())
        );
    }
}
