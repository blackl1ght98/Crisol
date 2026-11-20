mod catalog;
mod cli;
mod desktop;
mod detector;
mod engine;
mod panel;
mod paths;
mod peicon;
mod prefix;
mod recipe;
mod runner;
mod ui;
mod vm;

use cli::Command;
use paths::Paths;

fn main() {
    let cmd = cli::parse(std::env::args().skip(1).collect());
    if matches!(cmd, Command::Help) {
        cli::print_help();
        return;
    }

    let paths = match Paths::new() {
        Ok(p) => p,
        Err(e) => {
            ui::err(&e);
            std::process::exit(1);
        }
    };
    if let Err(e) = paths.ensure_dirs() {
        ui::err(&e);
        std::process::exit(1);
    }

    let code = match cmd {
        Command::Help => 0,
        Command::Install(path, opts) if opts.vm => {
            result_code(vm::handoff(&paths, &path))
        }
        Command::Install(path, opts) => match engine::install(&paths, &opts, &path) {
            Ok(report) => {
                ui::detail(&format!(
                    "Aplicación: {} | ID: {} | intentos: {}",
                    report.name, report.id, report.attempts
                ));
                if let Some(exe) = &report.exe {
                    ui::detail(&format!("Ejecutable: {exe}"));
                }
                if !report.applied.is_empty() {
                    ui::detail(&format!("Recetas: {}", report.applied.join(", ")));
                }
                if report.attempts > 1 {
                    ui::detail("Últimas líneas del log:");
                    for line in report.log_tail.lines().take(5) {
                        ui::detail(line);
                    }
                }
                0
            }
            Err(e) => {
                ui::err(&e);
                1
            }
        },
        Command::Handler(path, opts) => {
            if desktop::open_terminal(&path) {
                0
            } else {
                match engine::install(&paths, &opts, &path) {
                    Ok(_) => 0,
                    Err(e) => {
                        ui::err(&e);
                        1
                    }
                }
            }
        }
        Command::Launch(id) => match engine::launch(&paths, &id) {
            Ok(()) => 0,
            Err(e) => {
                ui::err(&e);
                1
            }
        },
        Command::List => result_code(engine::list(&paths)),
        Command::Remove(id) => result_code(engine::remove(&paths, &id)),
        Command::Repair(id) => result_code(engine::repair(&paths, &id)),
        Command::Doctor => result_code(engine::doctor(&paths)),
        Command::Recipes => result_code(engine::show_recipes(&paths)),
        Command::SetupDesktop => result_code(desktop::setup(&paths)),
        Command::VmDoctor => result_code(vm::doctor(&paths)),
        Command::VmSetup => result_code(vm::setup(&paths)),
        Command::VmList => result_code(vm::list(&paths)),
        Command::VmStart(name) => result_code(vm::start(&paths, &name)),
        Command::VmCreate {
            name,
            iso,
            disk,
            ram,
            cpus,
            gpu,
        } => result_code(vm::create(
            &paths,
            &name,
            iso.as_deref(),
            disk,
            ram,
            cpus,
            gpu.as_deref(),
        )),
        Command::VmInstall { name, exe } => result_code(vm::install(&paths, &name, &exe)),
        Command::Panel(port) => result_code(panel::serve(&paths, port)),
    };
    std::process::exit(code);
}

fn result_code(res: Result<(), String>) -> i32 {
    match res {
        Ok(()) => 0,
        Err(e) => {
            ui::err(&e);
            1
        }
    }
}
