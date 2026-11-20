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
            ui::detail("IOMMU no habilitado: añade 'amd_iommu=on iommu=pt' o 'intel_iommu=on iommu=pt' al kernel");
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
    ui::info("Ejecútalo con: sudo <script>   (necesita contraseña; no lo hago yo por ti)");
    ui::info("Después: crisol vm doctor y crisol vm create windows --iso /ruta/Windows.iso");
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

fn install_into(paths: &Paths, name: &str, exe: &Path) -> Result<(), String> {
    let exe = fs::canonicalize(exe).unwrap_or_else(|_| exe.to_path_buf());
    let dir = vm_dir(paths, name);
    let share = dir.join("share");
    fs::create_dir_all(&share).ok();
    let name = exe
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .ok_or_else(|| "El archivo no tiene nombre".to_string())?;
    let dest = share.join(&name);
    fs::copy(&exe, &dest).map_err(|e| format!("No se pudo copiar a la VM: {e}"))?;
    let cmd = share.join("INSTALAR.cmd");
    let _ = fs::write(
        &cmd,
        format!(
            "@echo off\nnet use Z: \\\\.\\crisol >nul 2>&1\ncd /d Z:\\\nstart \"\" {}\n",
            install_command(&exe)
        ),
    );
    ui::ok(&format!("Instalador copiado a {}", dest.display()));
    ui::info("Arranca la VM (crisol vm start windows), instala los drivers virtio y ejecuta Z:\\INSTALAR.cmd");
    ui::detail("Si el anti-cheat exige GPU passthrough, usa --gpu-pci al crear la VM");
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
