//! Fallback a máquina virtual cuando Wine no puede (anti-cheat de kernel, DRM, drivers).
//! Genera scripts para QEMU/KVM con virtio, 9p y, si se pide, VFIO (GPU passthrough).
//! No instala nada por nosotros: `vm setup` deja un script para ejecutar con sudo.

use crate::paths::Paths;
use crate::runner;
use crate::ui;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn vms_dir(paths: &Paths) -> PathBuf {
    paths.data.join("vms")
}

fn vm_dir(paths: &Paths, name: &str) -> PathBuf {
    vms_dir(paths).join(name)
}

fn is_kvm_available() -> bool {
    Path::new("/dev/kvm").exists()
}

/// Muestra las rutas de datos del usuario con `~` para que el comando copiable
/// sea corto. El shell expande `~` antes de ejecutar `sudo`, así que funciona.
fn short_path(paths: &Paths, path: &Path) -> String {
    let full = path.to_string_lossy();
    let home = paths.home.to_string_lossy();
    match full.strip_prefix(home.as_ref()) {
        Some(rest) if rest.starts_with('/') => format!("~{rest}"),
        _ => full.to_string(),
    }
}

fn ovmf_path() -> String {
    for candidate in [
        "/usr/share/ovmf/x64/OVMF.fd",
        "/usr/share/ovmf/OVMF.fd",
        "/usr/share/edk2/x64/OVMF.fd",
    ] {
        if Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    "/usr/share/ovmf/x64/OVMF.fd".to_string()
}

pub fn doctor(paths: &Paths) -> Result<(), String> {
    ui::header("Diagnostico de virtualizacion");
    for bin in ["qemu-system-x86_64", "qemu-img", "virt-manager"] {
        match runner::which(bin) {
            Some(p) => ui::ok(&format!("{bin}: {}", p.display())),
            None => ui::warn(&format!("{bin}: no encontrado (ejecuta crisol vm setup)")),
        }
    }
    if is_kvm_available() {
        ui::ok("/dev/kvm: presente (aceleracion KVM disponible)");
    } else {
        ui::warn("/dev/kvm: ausente (la VM sera lenta o no arrancara)");
    }
    let ovmf = ovmf_path();
    if Path::new(&ovmf).exists() {
        ui::ok(&format!("OVMF: {ovmf}"));
    } else {
        ui::warn(&format!("OVMF no encontrado en {ovmf}"));
    }

    if let Ok(cmdline) = fs::read_to_string("/proc/cmdline") {
        if cmdline.contains("iommu=pt")
            || cmdline.contains("amd_iommu=on")
            || cmdline.contains("intel_iommu=on")
        {
            ui::ok("IOMMU habilitado en el kernel (posible GPU passthrough)");
        } else {
            ui::detail("IOMMU no habilitado: solo hace falta para GPU passthrough; si lo quieres, añade 'amd_iommu=on iommu=pt' o 'intel_iommu=on iommu=pt' al kernel");
        }
    }

    let mut found = 0;
    if let Ok(devices) = fs::read_dir("/sys/bus/pci/devices") {
        for entry in devices.flatten() {
            let path = entry.path();
            let Ok(group) = fs::read_link(path.join("iommu_group")) else {
                continue;
            };
            let group = group
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let vendor = fs::read_to_string(path.join("vendor")).unwrap_or_default();
            let device = fs::read_to_string(path.join("device")).unwrap_or_default();
            let addr = entry.file_name().to_string_lossy().to_string();
            ui::detail(&format!(
                "PCI {} -> grupo {} ({}{})",
                addr,
                group,
                vendor.trim(),
                device.trim()
            ));
            found += 1;
        }
    }
    if found == 0 {
        ui::detail("No se encontraron dispositivos con iommu_group");
    }
    ui::info(&format!("VMs guardadas en: {}", vms_dir(paths).display()));
    Ok(())
}

pub fn setup(paths: &Paths) -> Result<(), String> {
    let script = vms_dir(paths).join("setup-vm.sh");
    fs::create_dir_all(script.parent().unwrap()).map_err(|e| e.to_string())?;
    let body = r#"#!/usr/bin/env bash
set -euo pipefail
# Ejecutar con sudo. CachyOS/Arch:
pacman -S --needed qemu ovmf qemu-img virt-manager virtio-win
usermod -aG kvm "$SUDO_USER"
echo 'Para GPU passthrough añade al arranque: amd_iommu=on iommu=pt  (o intel_iommu=on iommu=pt)'
echo 'Despues: newgrp kvm  o  cerrar sesion y volver a entrar.'
"#;
    fs::write(&script, body)
        .map_err(|e| format!("No se pudo escribir {}: {e}", script.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&script, fs::Permissions::from_mode(0o755));
    }
    ui::ok(&format!("Script de instalacion: {}", script.display()));
    ui::info("Copia y pega ESTA linea (sudo no ve ~/.local/bin, por eso se usa la ruta con ~):");
    ui::detail(&format!("sudo {}", short_path(paths, &script)));
    ui::info("No escribas 'sudo crisol ...': sudo no encuentra el binario y Crisol no debe correr como root.");
    ui::info("Después: newgrp kvm (o cierra sesión), crisol vm doctor y crisol vm create windows --iso /ruta/Windows.iso");
    Ok(())
}

pub fn list(paths: &Paths) -> Result<(), String> {
    let dir = vms_dir(paths);
    let Ok(entries) = fs::read_dir(&dir) else {
        ui::info("No hay VMs creadas");
        return Ok(());
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    if names.is_empty() {
        ui::info("No hay VMs creadas");
    } else {
        for name in names {
            ui::info(&format!("{name}: {}", vm_dir(paths, &name).display()));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn create(
    paths: &Paths,
    name: &str,
    iso: Option<&str>,
    disk_gb: u32,
    ram_gb: u32,
    cpus: u32,
    gpu_pci: Option<&str>,
) -> Result<(), String> {
    if name.is_empty() || name.contains('/') {
        return Err("El nombre de la VM no es válido".to_string());
    }
    let dir = vm_dir(paths, name);
    let share = dir.join("share");
    fs::create_dir_all(&share).map_err(|e| format!("No se pudo crear {}: {e}", dir.display()))?;
    let disk = dir.join("disk.qcow2");

    if !disk.exists() {
        if let Some(qemu_img) = runner::which("qemu-img") {
            let status = Command::new(qemu_img)
                .args(["create", "-f", "qcow2"])
                .arg(&disk)
                .arg(format!("{disk_gb}G"))
                .status()
                .map_err(|e| format!("qemu-img fallo: {e}"))?;
            if !status.success() {
                return Err("qemu-img no pudo crear el disco".to_string());
            }
        } else {
            ui::warn("qemu-img no está instalado; crea el disco manualmente:");
            ui::detail(&format!(
                "qemu-img create -f qcow2 {} {disk_gb}G",
                disk.display()
            ));
        }
    }

    // Si no se indicó una ISO, el script la busca en ~/ISO/Windows.iso (o en
    // la variable ISO) para no tener que regenerar la VM al conseguirla.
    let (iso_setup, iso_arg) = match iso {
        Some(i) => (String::new(), format!("  -cdrom '{i}' \\\n")),
        None => (
            "ISO=\"${ISO:-$HOME/ISO/Windows.iso}\"\nCDROM=()\n[[ -f \"$ISO\" ]] && CDROM=(-cdrom \"$ISO\")\n"
                .to_string(),
            "  \"${CDROM[@]}\" \\\n".to_string(),
        ),
    };
    let gpu_line = if let Some(pci) = gpu_pci {
        format!("  -device vfio-pci,host={pci} \\\n  -vga none \\\n")
    } else {
        "  -device virtio-gpu-pci \\\n".to_string()
    };
    let start = format!(
        "#!/usr/bin/env bash\n\
         set -euo pipefail\n\
         DIR=\"{}\"\n\
         {}\
         qemu-system-x86_64 \\\n\
         \x20 -enable-kvm -machine q35,accel=kvm -cpu host \\\n\
         \x20 -m {}G -smp {} \\\n\
         \x20 -bios {} \\\n\
         \x20 -drive file=\"$DIR/disk.qcow2\",format=qcow2,if=virtio \\\n\
         {}\
         \x20 -netdev user,id=net0 -device virtio-net-pci,netdev=net0 \\\n\
         {}\
         \x20 -audiodev pipewire,id=a0 -device ich9-intel-hda -device hda-duplex,audiodev=a0 \\\n\
         \x20 -virtfs local,path=\"$DIR/share\",mount_tag=crisol,security_model=passthrough,id=fs0 \\\n\
         \x20 -display gtk,gl=on -usb -device usb-tablet\n",
        dir.display(),
        iso_setup,
        ram_gb,
        cpus,
        ovmf_path(),
        iso_arg,
        gpu_line
    );
    let start_script = dir.join("start.sh");
    fs::write(&start_script, start)
        .map_err(|e| format!("No se pudo escribir {}: {e}", start_script.display()))?;

    if let Some(pci) = gpu_pci {
        let bind = dir.join("bind-vfio.sh");
        let body = format!(
            "#!/usr/bin/env bash\n\
             set -euo pipefail\n\
             # Ejecutar con sudo ANTES de arrancar la VM.\n\
             GPU=\"{pci}\"\n\
             DEVICES=$(ls /sys/bus/pci/devices/$GPU/iommu_group/devices 2>/dev/null || echo \"$GPU\")\n\
             modprobe vfio\n\
             modprobe vfio_pci\n\
             for dev in $DEVICES; do\n\
             \x20 VENDOR=$(cat /sys/bus/pci/devices/$dev/vendor)\n\
             \x20 DEVICE=$(cat /sys/bus/pci/devices/$dev/device)\n\
             \x20 echo \"$VENDOR $DEVICE\" > /sys/bus/pci/drivers/vfio-pci/new_id 2>/dev/null || true\n\
             \x20 echo vfio-pci > /sys/bus/pci/devices/$dev/driver_override 2>/dev/null || true\n\
             \x20 echo \"$dev\" > /sys/bus/pci/drivers/vfio-pci/bind 2>/dev/null || true\n\
             done\n\
             echo 'GPU enlazada a vfio-pci'\n"
        );
        fs::write(&bind, body)
            .map_err(|e| format!("No se pudo escribir {}: {e}", bind.display()))?;
        make_executable(&bind);
        ui::warn(&format!(
            "GPU passthrough: ejecuta sudo {} antes de arrancar",
            bind.display()
        ));
    }
    make_executable(&start_script);

    let readme = dir.join("README.txt");
    let _ = fs::write(
        &readme,
        format!(
            "VM '{name}' creada por Crisol\n\n\
             1) Arranca: {}\n\
             2) Instala Windows desde la ISO indicada.\n\
             3) Instala los drivers virtio (virtio-win) dentro del guest.\n\
             4) Comparte la carpeta 'share': net use Z: \\\\.\\crisol\n\
             5) Ejecuta Z:\\INSTALAR.cmd para instalar el .exe.\n\n\
             El anti-cheat y los drivers de kernel funcionan aquí porque es un Windows real.\n",
            start_script.display()
        ),
    );

    ui::ok(&format!("VM '{name}' creada en {}", dir.display()));
    ui::info(&format!("Arrancar: {}", start_script.display()));
    if !is_kvm_available() {
        ui::warn("No hay /dev/kvm: ejecuta crisol vm setup y reinicia sesión");
    }
    Ok(())
}

fn make_executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
    }
}

/// Línea completa que se ejecutará dentro de la VM para el instalador dado.
fn install_command(exe: &Path) -> String {
    let name = exe
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    if name.to_lowercase().ends_with(".msi") {
        format!("msiexec /i \"{name}\" /qn /norestart")
    } else if name.to_lowercase().contains("inno") {
        format!("\"{name}\" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART")
    } else {
        format!("\"{name}\" /S")
    }
}

/// Pasa un instalador a la VM por defecto. Si no existe, explica cómo crearla.
pub fn handoff(paths: &Paths, exe: &Path) -> Result<(), String> {
    if !vm_dir(paths, "windows").exists() {
        ui::warn("No existe la VM por defecto 'windows'");
        ui::info("Crea primero una VM con una ISO de Windows:");
        ui::detail("crisol vm create windows --iso /ruta/Windows.iso --ram 8 --disk 60");
        ui::detail("crisol vm doctor        # comprueba QEMU, /dev/kvm y OVMF");
        ui::info("Después vuelve a ejecutar: crisol install <archivo> --vm");
        return Ok(());
    }
    install_into(paths, "windows", exe)
}

/// Copia un instalador a la carpeta compartida de una VM concreta.
pub fn install(paths: &Paths, name: &str, exe: &Path) -> Result<(), String> {
    if !vm_dir(paths, name).exists() {
        return Err(format!("No existe la VM '{name}'. Usa crisol vm list"));
    }
    install_into(paths, name, exe)
}

/// Imprime el camino completo desde aquí hasta ejecutar el archivo en Windows.
/// Crisol no puede arrancar Windows ni hacer clic dentro del guest, así que los
/// pasos tienen que ser explícitos y copiables; un simple "ya está copiado"
/// deja al usuario sin saber qué hacer.
fn print_next_steps(paths: &Paths, vm: &str, exe: &Path, cmd: &Path) {
    let dir = vm_dir(paths, vm);
    let exe_name = exe
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let cmd_name = cmd
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "INSTALAR.cmd".into());

    ui::header("Como continuar en la VM");

    ui::info("1) Comprueba el entorno (solo la primera vez):");
    ui::detail("crisol vm doctor");
    if runner::which("qemu-system-x86_64").is_none() {
        ui::warn("Falta QEMU: ejecuta 'crisol vm setup' y después el script que indica CON sudo (pide contraseña).");
    }
    if !is_kvm_available() {
        ui::warn("Falta /dev/kvm: la VM irá muy lenta. Usa 'crisol vm setup' y vuelve a entrar en sesión.");
    }
    let disk = dir.join("disk.qcow2");
    if !disk.exists() {
        ui::warn(&format!(
            "Falta {}: crisol vm create {vm} --iso /ruta/Windows.iso --ram 8 --disk 60",
            short_path(paths, &disk)
        ));
    }

    ui::info(&format!("2) Arranca Windows: crisol vm start {vm}"));
    ui::detail(&format!(
        "   Alternativa: {}",
        short_path(paths, &dir.join("start.sh"))
    ));

    ui::info(
        "3) Si es la primera vez, instala Windows desde la ISO y los drivers virtio (virtio-win).",
    );

    ui::info("4) Dentro de Windows abre un Simbolo del sistema y ejecuta el bloque final.");

    ui::info(&format!(
        "5) El archivo {exe_name} ya esta en Z:\\ y se abrira desde ahi."
    ));

    ui::info("--- BLOQUE PARA COPIAR: AQUI (si algo falta, ejecuta estas lineas) ---");
    ui::detail("crisol vm doctor");
    ui::detail("crisol vm setup");
    ui::detail(&format!(
        "sudo {}   # pide contraseña; Crisol no lo ejecuta",
        short_path(paths, &vms_dir(paths).join("setup-vm.sh"))
    ));
    ui::detail("newgrp kvm   # o cierra sesion y vuelve a entrar");
    ui::detail(&format!(
        "crisol vm create {vm} --iso /ruta/Windows.iso --ram 8 --disk 60"
    ));
    ui::detail(&format!("crisol vm start {vm}"));

    ui::info("--- BLOQUE PARA COPIAR: DENTRO DE WINDOWS (Simbolo del sistema) ---");
    ui::detail("net use Z: \\\\.\\crisol");
    ui::detail("Z:");
    ui::detail("cd \\");
    ui::detail(&cmd_name);
    ui::detail("   (tambien puedes hacer doble clic en Z:\\INSTALAR.cmd)");

    ui::info("Si la VM no arranca, ejecuta 'crisol vm doctor' y comparte el resultado.");
}

fn install_into(paths: &Paths, name: &str, exe: &Path) -> Result<(), String> {
    let exe = fs::canonicalize(exe).unwrap_or_else(|_| exe.to_path_buf());
    let dir = vm_dir(paths, name);
    let share = dir.join("share");
    fs::create_dir_all(&share).ok();
    let file_name = exe
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .ok_or_else(|| "El archivo no tiene nombre".to_string())?;
    let dest = share.join(&file_name);
    fs::copy(&exe, &dest).map_err(|e| format!("No se pudo copiar a la VM: {e}"))?;
    let cmd = share.join("INSTALAR.cmd");
    let command = install_command(&exe);
    let _ = fs::write(
        &cmd,
        format!(
            "@echo off\n\
             net use Z: \\\\.\\crisol >nul 2>&1\n\
             cd /d Z:\\\n\
             echo Preparando {file_name}...\n\
             {command}\n\
             echo Resultado: %ERRORLEVEL%\n\
             pause\n"
        ),
    );
    ui::ok(&format!("Instalador copiado a {}", dest.display()));
    ui::ok(&format!("Script generado: {}", cmd.display()));
    if !is_kvm_available() || runner::which("qemu-system-x86_64").is_none() {
        ui::warn("Esta maquina aun no puede arrancar la VM; revisa el paso 1.");
    }
    print_next_steps(paths, name, &dest, &cmd);
    Ok(())
}

/// Permite `crisol vm start <name>` sin recordar la ruta del script.
pub fn start(paths: &Paths, name: &str) -> Result<(), String> {
    let script = vm_dir(paths, name).join("start.sh");
    if !script.exists() {
        return Err(format!("No existe la VM '{name}'. Usa crisol vm list"));
    }
    let status = Command::new(&script)
        .status()
        .map_err(|e| format!("No se pudo arrancar la VM: {e}"))?;
    if !status.success() {
        ui::warn("QEMU terminó con error. Comprueba crisol vm doctor");
    }
    Ok(())
}
