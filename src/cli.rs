use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Options {
    pub sandbox: bool,
    pub max_attempts: usize,
    pub engine: Option<String>,
    pub yes: bool,
    pub timeout_secs: u64,
    pub exe: Option<String>,
    pub interactive: bool,
    /// Ejecuta un .exe portable como prueba; por defecto solo se registra.
    pub execute: bool,
    pub vm: bool,
    pub vm_fallback: bool,
    pub iso: Option<String>,
    pub disk: u32,
    pub ram: u32,
    pub cpus: u32,
    pub gpu: Option<String>,
    pub port: u16,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sandbox: true,
            max_attempts: 5,
            engine: None,
            yes: false,
            timeout_secs: 900,
            exe: None,
            interactive: false,
            execute: false,
            vm: false,
            vm_fallback: false,
            iso: None,
            disk: 40,
            ram: 4,
            cpus: 2,
            gpu: None,
            port: 8765,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Version,
    App(u16),
    Completions(String),
    Install(PathBuf, Options),
    /// ID de la app y permiso explícito para lanzar una app incompatible.
    Launch(String, bool),
    List,
    ListJson,
    Remove(String),
    Repair(String),
    Doctor,
    DoctorJson,
    Recipes,
    RecipesJson,
    SetupDesktop,
    Handler(PathBuf, Options),
    VmDoctor,
    VmSetup,
    VmList,
    VmStart(String),
    VmCreate {
        name: String,
        iso: Option<String>,
        disk: u32,
        ram: u32,
        cpus: u32,
        gpu: Option<String>,
    },
    VmInstall {
        name: String,
        exe: PathBuf,
    },
    Panel(u16),
    FeedbackExport(Option<PathBuf>),
    FeedbackImport(PathBuf),
    FeedbackStats,
    FeedbackStatsJson,
    RecipeImport(PathBuf),
    RecipeExport(PathBuf),
    Help,
}

pub fn parse(args: Vec<String>) -> Command {
    if args.is_empty() {
        return Command::Help;
    }

    let mut opts = Options::default();
    let mut positional: Vec<String> = Vec::new();
    let mut json = false;
    let mut force = false;
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "--handler" | "handler" => {
                i += 1;
                if let Some(p) = args.get(i) {
                    positional.push("__handler__".to_string());
                    positional.push(p.clone());
                }
            }
            "--no-sandbox" => opts.sandbox = false,
            "--yes" | "-y" => opts.yes = true,
            "--interactive" => opts.interactive = true,
            "--execute" | "--smoke-test" => opts.execute = true,
            "--force" => force = true,
            "--vm" => opts.vm = true,
            "--vm-fallback" => opts.vm_fallback = true,
            "--iso" => {
                i += 1;
                opts.iso = args.get(i).cloned();
            }
            "--disk" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<u32>().ok()) {
                    opts.disk = v.clamp(10, 4096);
                }
            }
            "--ram" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<u32>().ok()) {
                    opts.ram = v.clamp(1, 256);
                }
            }
            "--cpus" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<u32>().ok()) {
                    opts.cpus = v.clamp(1, 128);
                }
            }
            "--gpu-pci" => {
                i += 1;
                opts.gpu = args.get(i).cloned();
            }
            "--port" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<u16>().ok()) {
                    opts.port = v;
                }
            }
            "--attempts" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    opts.max_attempts = v.clamp(1, 20);
                }
            }
            "--engine" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    opts.engine = Some(v.clone());
                }
            }
            "--timeout" => {
                i += 1;
                if let Some(v) = args.get(i).and_then(|s| s.parse::<u64>().ok()) {
                    opts.timeout_secs = v.clamp(10, 86_400);
                }
            }
            "--exe" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    opts.exe = Some(v.replace('\\', "/"));
                }
            }
            "--version" | "-V" => return Command::Version,
            "--json" => json = true,
            "--help" | "-h" | "help" => return Command::Help,
            s if s.starts_with('-') => {
                eprintln!("flag desconocida: {s}");
            }
            s => positional.push(s.to_string()),
        }
        i += 1;
    }

    if positional.is_empty() {
        return Command::Help;
    }
    if positional.first().map(|s| s.as_str()) == Some("vm") {
        let sub = positional.get(1).map(|s| s.as_str()).unwrap_or("");
        return match sub {
            "doctor" => Command::VmDoctor,
            "setup" => Command::VmSetup,
            "list" => Command::VmList,
            "start" => positional
                .get(2)
                .map(|s| Command::VmStart(s.clone()))
                .unwrap_or(Command::Help),
            "create" => {
                let name = positional.get(2).cloned().unwrap_or_default();
                if name.is_empty() {
                    Command::Help
                } else {
                    Command::VmCreate {
                        name,
                        iso: opts.iso,
                        disk: opts.disk,
                        ram: opts.ram,
                        cpus: opts.cpus,
                        gpu: opts.gpu,
                    }
                }
            }
            "install" => match (positional.get(2), positional.get(3)) {
                (Some(name), Some(exe)) => Command::VmInstall {
                    name: name.clone(),
                    exe: PathBuf::from(exe),
                },
                _ => Command::Help,
            },
            _ => Command::Help,
        };
    }

    if positional.first().map(|s| s.as_str()) == Some("feedback") {
        return match positional.get(1).map(|s| s.as_str()).unwrap_or("") {
            "export" => Command::FeedbackExport(positional.get(2).map(PathBuf::from)),
            "import" => match positional.get(2) {
                Some(p) => Command::FeedbackImport(PathBuf::from(p)),
                None => Command::Help,
            },
            "" | "stats" => {
                if json {
                    Command::FeedbackStatsJson
                } else {
                    Command::FeedbackStats
                }
            }
            _ => Command::Help,
        };
    }

    if positional.first().map(|s| s.as_str()) == Some("recipes") {
        return match positional.get(1).map(|s| s.as_str()).unwrap_or("") {
            "import" => match positional.get(2) {
                Some(p) => Command::RecipeImport(PathBuf::from(p)),
                None => Command::Help,
            },
            "export" => match positional.get(2) {
                Some(p) => Command::RecipeExport(PathBuf::from(p)),
                None => Command::Help,
            },
            "" | "list" => {
                if json {
                    Command::RecipesJson
                } else {
                    Command::Recipes
                }
            }
            _ => Command::Help,
        };
    }

    let cmd = positional[0].clone();
    let arg = positional.get(1).cloned();

    match cmd.as_str() {
        "__handler__" => match arg {
            Some(p) => Command::Handler(PathBuf::from(p), opts),
            None => Command::Help,
        },
        "install" | "i" => match arg {
            Some(p) => Command::Install(PathBuf::from(p), opts),
            None => Command::Help,
        },
        "app" | "ui" => Command::App(opts.port),
        "completions" => arg.map(Command::Completions).unwrap_or(Command::Help),
        "launch" | "run" => arg
            .map(|id| Command::Launch(id, force))
            .unwrap_or(Command::Help),
        "remove" | "rm" | "uninstall" => arg.map(Command::Remove).unwrap_or(Command::Help),
        "repair" => arg.map(Command::Repair).unwrap_or(Command::Help),
        "list" | "ls" => {
            if json {
                Command::ListJson
            } else {
                Command::List
            }
        }
        "doctor" => {
            if json {
                Command::DoctorJson
            } else {
                Command::Doctor
            }
        }
        "recipes" => {
            if json {
                Command::RecipesJson
            } else {
                Command::Recipes
            }
        }
        "setup-desktop" => Command::SetupDesktop,
        "panel" => Command::Panel(opts.port),
        _ => Command::Help,
    }
}

pub fn print_help() {
    println!(
        "Crisol - instala .exe/.msi como si fuesen nativos, sobre Wine\n\
         \n\
         USO\n\
           crisol install <archivo.exe|archivo.msi> [opciones]\n\
           crisol launch <id> [--force]   Ejecuta una app instalada\n\
           crisol list                    Muestra el catalogo\n\
           crisol repair <id>             Repite la instalacion/receta de una app\n\
           crisol remove <id>             Borra la app y su prefijo\n\
           crisol recipes                 Muestra las recetas cargadas\n\
           crisol doctor                  Comprueba el entorno\n\
           crisol setup-desktop           Registra el doble clic en el escritorio\n\
           crisol vm doctor               Comprueba KVM, OVMF, IOMMU y PCI\n\
           crisol vm setup                Genera el script para instalar QEMU (sudo)\n\
           crisol vm create <vm> --iso ISO [--ram N] [--disk N] [--gpu-pci ADDR]\n\
           crisol vm list | vm start <vm> | vm install <vm> <archivo>\n\
           crisol app [--port 8765]            Abre el panel en el navegador\n\
            crisol panel [--port 8765]          Panel web local de feedback\n\
           crisol feedback stats               Votos y tasa de acierto por receta\n\
           crisol feedback export [archivo]    Exportar feedback.tsv\n\
           crisol feedback import <archivo>    Importar un feedback compartido\n\
           crisol recipes import <dir>         Importar recetas compartidas\n\
           crisol recipes export <dir>         Exportar recetas a un directorio\n\
           crisol --handler <archivo>     Modo interno para el gestor de archivos\n\
           crisol completions <shell>     Genera completions (bash|zsh|fish)\n\
         \n\
         OPCIONES\n\
           --no-sandbox       No usar bubblewrap (por si falla)\n\
           --attempts N       Maximo de intentos de reparacion (1-20, por defecto 5)\n\
           --engine NAME      Motor a usar: wine (por defecto), wine-staging, wine-ge...\n\
           --timeout SEG      Tiempo maximo del instalador (por defecto 900s)\n\
           --exe RUTA         Ejecutable a registrar, relativo al prefijo (ej. drive_c/Program Files/App/app.exe)\n\
           --interactive      Mostrar la UI del instalador (por defecto se usa modo silencioso)\n\
           --vm               Pasar el instalador directamente a la VM de Windows\n\
           --vm-fallback      Si Wine falla, pasar automaticamente a la VM\n\
           --port N           Puerto del panel local (por defecto 8765)\n\
           --json             Salida JSON en list/doctor/recipes/feedback stats\n\
           --version          Muestra la version\n\
           --yes              No pedir confirmacion\n\
           --force            En launch, ignora el aviso de app incompatible\n\
         \n\
         EJEMPLOS\n\
           crisol install ~/Descargas/Programa.msi\n\
           crisol install juego.exe --engine wine-ge --attempts 7\n\
           crisol launch programa-a1b2c3\n"
    );
}

/// Genera un script de completions estático (sin dependencias).
pub fn print_completions(shell: &str) {
    let cmds = "install launch list recipes doctor app panel feedback vm setup-desktop repair remove completions help";
    let flags = "--no-sandbox --yes --interactive --vm --vm-fallback --iso --disk --ram --cpus --gpu-pci --port --attempts --engine --timeout --exe --execute --smoke-test --force --json --version --help";
    match shell {
        "bash" => println!(
            "_crisol() {{\n  local cur=\"${{COMP_WORDS[COMP_CWORD]}}\"\n  COMPREPLY=( $(compgen -W \"{cmds} {flags}\" -- \"$cur\") )\n}}\ncomplete -F _crisol crisol\n"
        ),
        "zsh" => println!(
            "#compdef crisol\n_arguments '1: :({cmds})' '*:: :->args'\n"
        ),
        "fish" => println!(
            "complete -c crisol -f\ncomplete -c crisol -n '__fish_use_subcommand' -a '{cmds}'\ncomplete -c crisol -a '{flags}'\n"
        ),
        _ => println!("Shell no soportado: {shell} (usa bash, zsh o fish)"),
    }
}
