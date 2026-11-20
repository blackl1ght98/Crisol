use std::env;
use std::fs;
use std::path::PathBuf;

/// Todas las rutas que usa Crisol. Se derivan de XDG_* con fallback a $HOME.
#[derive(Clone, Debug)]
pub struct Paths {
    pub home: PathBuf,
    pub data: PathBuf,
    pub config: PathBuf,
    pub cache: PathBuf,
    pub prefixes: PathBuf,
    pub backups: PathBuf,
    pub logs: PathBuf,
    pub recipes: PathBuf,
    pub catalog: PathBuf,
    pub applications: PathBuf,
    pub icons: PathBuf,
    pub runtime: Option<PathBuf>,
}

fn xdg_dir(var: &str, fallback: &str) -> PathBuf {
    match env::var_os(var) {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => PathBuf::from(fallback),
    }
}

impl Paths {
    pub fn new() -> Result<Self, String> {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| "HOME no esta definido".to_string())?;
        let data = xdg_dir("XDG_DATA_HOME", &home.join(".local/share").to_string_lossy());
        let config = xdg_dir("XDG_CONFIG_HOME", &home.join(".config").to_string_lossy());
        let cache = xdg_dir("XDG_CACHE_HOME", &home.join(".cache").to_string_lossy());
        let data = data.join("crisol");
        let config = config.join("crisol");
        let cache = cache.join("crisol");
        let runtime = env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);

        Ok(Self {
            applications: xdg_dir("XDG_DATA_HOME", &home.join(".local/share").to_string_lossy())
                .join("applications"),
            icons: xdg_dir("XDG_DATA_HOME", &home.join(".local/share").to_string_lossy())
                .join("icons/hicolor/scalable/apps"),
            prefixes: data.join("prefixes"),
            backups: data.join("backups"),
            logs: data.join("logs"),
            recipes: config.join("recipes"),
            catalog: data.join("catalog.tsv"),
            runtime,
            home,
            data,
            config,
            cache,
        })
    }

    pub fn ensure_dirs(&self) -> Result<(), String> {
        for dir in [
            &self.data,
            &self.config,
            &self.cache,
            &self.prefixes,
            &self.backups,
            &self.logs,
            &self.recipes,
            &self.applications,
            &self.icons,
        ] {
            fs::create_dir_all(dir)
                .map_err(|e| format!("No se pudo crear {}: {e}", dir.display()))?;
        }
        Ok(())
    }
}
