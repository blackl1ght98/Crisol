use crate::catalog::Entry;
use crate::paths::Paths;
use crate::runner::which;
use std::env;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command;

const MIMES: [&str; 4] = [
    "application/x-msdownload",
    "application/x-msi",
    "application/x-ms-dos-executable",
    "application/x-wine-extension-exe",
];

fn current_bin() -> Result<PathBuf, String> {
    env::current_exe().map_err(|e| format!("No se pudo resolver el binario: {e}"))
}

fn default_icon(paths: &Paths) -> Result<PathBuf, String> {
    let icon = paths.icons.join("crisol.svg");
    if !icon.exists() {
        fs::write(&icon, include_str!("../assets/crisol.svg"))
            .map_err(|e| format!("No se pudo escribir el icono: {e}"))?;
    }
    Ok(icon)
}

/// Registra Crisol como aplicación por defecto para .exe y .msi.
pub fn setup(paths: &Paths) -> Result<(), String> {
    let bin = current_bin()?;
    let icon = default_icon(paths)?;
    let desktop = paths.applications.join("crisol.desktop");
    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Crisol\n\
         Comment=Instala ejecutables de Windows como si fuesen nativos\n\
         Exec=\"{}\" --handler %f\n\
         Icon={}\n\
         Terminal=true\n\
         MimeType={};\n\
         Categories=System;Utility;\n\
         StartupNotify=true\n",
        bin.display(),
        icon.display(),
        MIMES.join(";")
    );
    fs::write(&desktop, content)
        .map_err(|e| format!("No se pudo escribir {}: {e}", desktop.display()))?;

    for mime in MIMES {
        let _ = Command::new("xdg-mime")
            .args(["default", "crisol.desktop", mime])
            .status();
    }
    if let Some(cmd) = which("update-desktop-database") {
        let _ = Command::new(cmd)
            .arg(paths.applications.as_os_str())
            .status();
    }
    crate::ui::ok(&format!("Registrado como manejador de: {}", MIMES.join(", ")));
    Ok(())
}

fn safe_name(name: &str) -> String {
    name.chars()
        .map(|c| if c == '"' || c == '\n' || c == '\r' { ' ' } else { c })
        .collect()
}

/// Crea el .desktop de una app instalada.
pub fn write_app(paths: &Paths, entry: &Entry) -> Result<PathBuf, String> {
    let bin = current_bin()?;
    let file = paths.applications.join(format!("crisol-{}.desktop", entry.id));
    let icon = if entry.icon.is_empty() {
        default_icon(paths)?.to_string_lossy().to_string()
    } else {
        entry.icon.clone()
    };
    let mut args = entry.args.trim().to_string();
    if !args.is_empty() {
        args = format!(" {args}");
    }
    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name={}\n\
         Exec=\"{}\" launch {}{}\n\
         Icon={}\n\
         Terminal=false\n\
         Categories=Utility;\n\
         Comment=Instalado con Crisol\n",
        safe_name(&entry.name),
        bin.display(),
        entry.id,
        args,
        icon
    );
    fs::write(&file, content).map_err(|e| format!("No se pudo escribir {}: {e}", file.display()))?;
    if let Some(cmd) = which("update-desktop-database") {
        let _ = Command::new(cmd)
            .arg(paths.applications.as_os_str())
            .status();
    }
    Ok(file)
}

pub fn remove_app(paths: &Paths, id: &str) {
    let file = paths.applications.join(format!("crisol-{id}.desktop"));
    let _ = fs::remove_file(file);
    if let Some(cmd) = which("update-desktop-database") {
        let _ = Command::new(cmd)
            .arg(paths.applications.as_os_str())
            .status();
    }
}

/// Icono de la app: se intenta extraer con wrestool/icotool si están disponibles;
/// si no, se usa el icono propio de Crisol.
pub fn icon_for(paths: &Paths, exe: &Path, id: &str) -> String {
    let out = paths.icons.join(format!("{id}.ico"));
    match crate::peicon::extract_icon(exe, &out) {
        Ok(()) => {
            crate::ui::detail(&format!("Icono extraído: {}", out.display()));
            out.to_string_lossy().to_string()
        }
        Err(_) => "crisol".to_string(),
    }
}

pub fn notify(title: &str, msg: &str) {
    if let Some(bin) = which("notify-send") {
        let _ = Command::new(bin).args([title, msg]).status();
    }
}

/// Si el doble clic viene del escritorio, abre una terminal para que se vea el progreso.
/// Devuelve true si delegó la ejecución a otra terminal.
pub fn open_terminal(file: &Path) -> bool {
    if env::var_os("CRISOL_HANDLER_CHILD").is_some() || std::io::stdout().is_terminal() {
        return false; // ya estamos en una terminal (o ya delegamos una vez)
    }
    let Ok(bin) = current_bin() else {
        return false;
    };
    let terminals = ["konsole", "alacritty", "kitty", "xterm"];
    for term in terminals {
        if let Some(path) = which(term) {
            let mut cmd = Command::new(path);
            cmd.arg("-e")
                .arg(&bin)
                .arg("--handler")
                .arg(file)
                .env("CRISOL_HANDLER_CHILD", "1");
            if cmd.spawn().is_ok() {
                return true;
            }
        }
    }
    false
}
