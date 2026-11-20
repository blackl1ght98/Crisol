use crate::paths::Paths;
use crate::runner::{self, RunOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct Prefix {
    pub id: String,
    pub path: PathBuf,
    pub snapshot: Option<PathBuf>,
}

fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in name.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
        if out.len() >= 28 {
            break;
        }
    }
    out.trim_matches('-').to_string()
}

fn fnv(text: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in text.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub fn make_id(installer: &Path) -> String {
    let name = installer
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "app".to_string());
    let base = slug(&name);
    let base = if base.is_empty() { "app".to_string() } else { base };
    let full = installer.to_string_lossy();
    let hash = format!("{:08x}", fnv(&full) & 0xffff_ffff);
    format!("{base}-{hash}")
}

pub fn base_env(prefix: &Path, cache: &Path) -> Vec<(String, String)> {
    let dxvk = cache.join("dxvk");
    fs::create_dir_all(&dxvk).ok();
    vec![
        ("WINEPREFIX".into(), prefix.to_string_lossy().to_string()),
        ("WINEARCH".into(), "win64".into()),
        ("WINEDEBUG".into(), "err+all".into()),
        ("WINEDLLOVERRIDES".into(), "mscoree,mshtml=".into()),
        ("DXVK_STATE_CACHE_PATH".into(), dxvk.to_string_lossy().to_string()),
    ]
}

pub fn create(paths: &Paths, id: &str, _sandbox: bool) -> Result<Prefix, String> {
    // wineboot solo inicializa el prefijo: no ejecuta codigo del instalador,
    // asi que se ejecuta sin bwrap (los servicios de Wine no arrancan bien dentro).
    let path = paths.prefixes.join(id);
    fs::create_dir_all(&path)
        .map_err(|e| format!("No se pudo crear el prefijo {}: {e}", path.display()))?;
    crate::ui::detail(&format!("Inicializando prefijo {}", path.display()));
    let env = base_env(&path, &paths.cache);
    let res = runner::run(&RunOptions {
        prefix: &path,
        command: "wineboot",
        args: vec!["--init".into()],
        env,
        sandbox: false,
        cache: &paths.cache,
        runtime: paths.runtime.as_deref(),
        timeout_secs: Some(240),
        label: "wineboot",
    })?;
    if res.code != Some(0) {
        crate::ui::warn("wineboot devolvio un codigo inesperado; se continua de todos modos");
    }
    crate::ui::detail("Prefijo inicializado");
    Ok(Prefix {
        id: id.to_string(),
        path,
        snapshot: None,
    })
}

impl Prefix {
    pub fn snapshot(&mut self, paths: &Paths) -> Result<(), String> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let dest = paths.backups.join(format!("{}-{}", self.id, ts));
        let status = Command::new("cp")
            .arg("-a")
            .arg(&self.path)
            .arg(&dest)
            .status()
            .map_err(|e| format!("No se pudo hacer el snapshot: {e}"))?;
        if !status.success() {
            return Err("cp -a fallo al hacer el snapshot del prefijo".to_string());
        }
        self.snapshot = Some(dest);
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), String> {
        let Some(backup) = self.snapshot.take() else {
            return Ok(());
        };
        let status = Command::new("rm")
            .arg("-rf")
            .arg(&self.path)
            .status()
            .map_err(|e| format!("No se pudo borrar el prefijo: {e}"))?;
        if !status.success() {
            return Err("rm -rf fallo durante el rollback".to_string());
        }
        let status = Command::new("mv")
            .arg(&backup)
            .arg(&self.path)
            .status()
            .map_err(|e| format!("No se pudo restaurar el prefijo: {e}"))?;
        if !status.success() {
            return Err("mv fallo durante el rollback".to_string());
        }
        Ok(())
    }

    pub fn discard_snapshot(&mut self, paths: &Paths) {
        if let Some(backup) = self.snapshot.take() {
            if backup.starts_with(&paths.backups) {
                let _ = fs::remove_dir_all(&backup);
            }
        }
    }
}
