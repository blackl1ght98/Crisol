use std::env;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub struct RunOptions<'a> {
    pub prefix: &'a Path,
    pub command: &'a str,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub sandbox: bool,
    pub cache: &'a Path,
    pub runtime: Option<&'a Path>,
    pub timeout_secs: Option<u64>,
    pub label: &'a str,
}

pub struct RunResult {
    pub code: Option<i32>,
    pub log: String,
    pub timed_out: bool,
}

pub fn which(name: &str) -> Option<PathBuf> {
    if name.contains('/') {
        let p = PathBuf::from(name);
        return p.is_file().then_some(p);
    }
    let paths = env::var_os("PATH")?;
    for dir in env::split_paths(&paths) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn build_command(opts: &RunOptions, sandbox: bool) -> Option<Command> {
    let mut cmd = if sandbox {
        let bwrap = which("bwrap")?;
        let mut b = Command::new(bwrap);
        b.arg("--die-with-parent")
            .arg("--ro-bind").arg("/").arg("/")
            .arg("--dev").arg("/dev")
            .arg("--proc").arg("/proc")
            .arg("--bind").arg("/tmp").arg("/tmp")
            .arg("--bind").arg(opts.prefix).arg(opts.prefix)
            .arg("--bind").arg(opts.cache).arg(opts.cache);
        if let Some(rt) = opts.runtime {
            if rt.exists() {
                b.arg("--bind").arg(rt).arg(rt);
            }
        }
        b.arg(opts.command);
        b
    } else {
        Command::new(opts.command)
    };
    cmd.args(&opts.args).envs(opts.env.iter().cloned());
    Some(cmd)
}

/// Redirige stdout y stderr a un archivo en lugar de pipes. Así un proceso hijo
/// que se quede colgado no bloquea los hilos lectores cuando matamos al padre.
fn spawn_command(opts: &RunOptions, sandbox: bool, sink: &File) -> Result<Child, String> {
    let out = sink
        .try_clone()
        .map_err(|e| format!("No se pudo clonar el archivo de log: {e}"))?;
    let err = sink
        .try_clone()
        .map_err(|e| format!("No se pudo clonar el archivo de log: {e}"))?;
    let mut cmd = build_command(opts, sandbox)
        .ok_or_else(|| format!("No se encontro el comando {}", opts.command))?;
    cmd.stdout(Stdio::from(out)).stderr(Stdio::from(err));
    cmd.spawn()
        .map_err(|e| format!("No se pudo ejecutar {}: {e}", opts.command))
}

fn safe_label(label: &str) -> String {
    label
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

pub fn run(opts: &RunOptions) -> Result<RunResult, String> {
    fs::create_dir_all(opts.cache).ok();
    let capture = opts.cache.join(format!(
        "capture-{}-{}.log",
        std::process::id(),
        safe_label(opts.label)
    ));
    if let Some(parent) = capture.parent() {
        fs::create_dir_all(parent).ok();
    }
    let sink = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&capture)
        .map_err(|e| format!("No se pudo crear {}: {e}", capture.display()))?;

    let sandbox_wanted = opts.sandbox && which("bwrap").is_some();
    let mut child = match spawn_command(opts, sandbox_wanted, &sink) {
        Ok(c) => c,
        Err(e) if sandbox_wanted => {
            crate::ui::warn(&format!("bwrap fallo ({e}); se reintenta sin aislamiento"));
            spawn_command(opts, false, &sink)?
        }
        Err(e) => return Err(e),
    };

    let mut status: Option<ExitStatus> = None;
    let mut timed_out = false;
    if let Some(secs) = opts.timeout_secs {
        let start = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(s)) => {
                    status = Some(s);
                    break;
                }
                Ok(None) => {}
                Err(e) => return Err(format!("Error esperando a {}: {e}", opts.command)),
            }
            if start.elapsed() >= Duration::from_secs(secs) {
                let _ = child.kill();
                let _ = child.wait();
                timed_out = true;
                break;
            }
            thread::sleep(Duration::from_millis(200));
        }
    } else {
        status = Some(child.wait().map_err(|e| format!("Error esperando a {}: {e}", opts.command))?);
    }

    let log = fs::read_to_string(&capture).unwrap_or_default();
    let lines: Vec<&str> = log.lines().collect();
    let from = lines.len().saturating_sub(30);
    for line in &lines[from..] {
        println!("\x1b[2m[{}]\x1b[0m {}", opts.label, line);
    }
    if timed_out {
        crate::ui::warn(&format!(
            "{} supero el limite de {}s y fue detenido",
            opts.label,
            opts.timeout_secs.unwrap_or(0)
        ));
    }

    Ok(RunResult {
        code: status.and_then(|s| s.code()),
        log,
        timed_out,
    })
}
