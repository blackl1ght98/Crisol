# VM: Windows real como último recurso

Wine no puede arreglar anti-cheat de kernel, DRM ni controladores de hardware.
Para esos casos Crisol puede **preparar** una VM con QEMU/KVM y traspasar el
instalador a un Windows real. Sigue siendo un fallback: la ruta principal sigue
siendo Wine + winetricks.

> **Entorno de desarrollo: QEMU NO está instalado aquí** (`qemu-system-x86_64` y
> `qemu-img` no están en `PATH`). `/dev/kvm` existe, pero **la VM nunca ha sido
> arrancada** ni Windows instalado. `tests/integration.sh` solo comprueba que se
> generen los scripts en un XDG temporal. Trata todo lo que sigue como una guía
> de uso no validada de extremo a extremo.

## Requisitos previos (los pones tú)

* CPU con virtualización habilitada en BIOS/UEFI (VT-x / AMD-V).
* Módulo `kvm` cargado y `/dev/kvm` accesible por tu usuario (grupo `kvm`).
* `qemu-system-x86_64`, `qemu-img`, firmware OVMF y, recomendable,
  `virt-manager`, `virtio-win` y `pipewire` (el script generado usa audio
  PipeWire y display GTK).
* Una **ISO de Windows** con licencia válida. Crisol no la descarga ni la
  instala.
* Para GPU passthrough: IOMMU activado en el kernel y un grupo VFIO aislado.

## 1. Diagnóstico

```bash
crisol vm doctor
```

Comprueba `qemu-system-x86_64`, `qemu-img` y `virt-manager` en `PATH`, la
presencia de `/dev/kvm`, la ruta de OVMF (`/usr/share/ovmf/x64/OVMF.fd` y
alternativas), los parámetros IOMMU en `/proc/cmdline` y los dispositivos PCI
con `iommu_group`. Imprime también dónde se guardan las VMs:
`~/.local/share/crisol/vms/`.

## 2. Instalar QEMU (tú, con contraseña)

```bash
crisol vm setup
sudo ~/.local/share/crisol/vms/setup-vm.sh
newgrp kvm          # o cierra sesión y vuelve a entrar
crisol vm doctor
```

`crisol vm setup` **no ejecuta sudo**: solo escribe `setup-vm.sh` con los
paquetes de Arch/CachyOS (`qemu ovmf qemu-img virt-manager virtio-win`) y
`usermod -aG kvm "$SUDO_USER"`. En Fedora el firmware suele llamarse
`edk2-ovmf`; en Debian/Ubuntu, `ovmf`. Revisa el script antes de ejecutarlo.

## 3. Crear la VM

```bash
mkdir -p ~/ISO
# Coloca aquí tu ISO de Windows, p. ej. ~/ISO/Windows.iso
crisol vm create windows --iso ~/ISO/Windows.iso --ram 8 --disk 60 --cpus 4
crisol vm list
```

Opciones: `--ram` (1-256 GB), `--disk` (10-4096 GB), `--cpus` (1-128) y
`--gpu-pci 0000:01:00.0` para VFIO. Si aún no tienes la ISO:

```bash
crisol vm create windows --ram 8 --disk 60
```

El `start.sh` generado buscará `$HOME/ISO/Windows.iso`; también puedes hacer
`ISO=/otra/ruta/Windows.iso ./start.sh`.

`create` genera en `~/.local/share/crisol/vms/windows/`:

* `disk.qcow2` (con `qemu-img create -f qcow2 ... 60G` si `qemu-img` está).
* `start.sh`: `qemu-system-x86_64 -enable-kvm -machine q35,accel=kvm -cpu host`,
  RAM/CPU elegidas, OVMF, disco virtio, `-cdrom`, red usuario + `virtio-net`,
  `virtio-gpu-pci` (o `vfio-pci` con `-vga none`), audio PipeWire, carpeta
  compartida 9p (`mount_tag=crisol`) y `-display gtk,gl=on`.
* `share/`: carpeta visible en el invitado como `Z: \\.\crisol`.
* `README.txt` con los pasos dentro del Windows invitado.

Equivalente manual para el disco, si `qemu-img` no estuviera en `PATH`:

```bash
qemu-img create -f qcow2 ~/.local/share/crisol/vms/windows/disk.qcow2 60G
```

El script `start.sh` generado es la referencia exacta: no lo edites salvo que
sepas qué estás cambiando.

## 4. Arrancar e instalar Windows

```bash
crisol vm start windows
```

Dentro del invitado:

1. Instala Windows desde la ISO.
2. Instala los drivers virtio de `virtio-win` (disco y red).
3. Monta la compartida: `net use Z: \\.\crisol`.

Crisol no hace ninguno de esos pasos: **no instala Windows, no ejecuta sudo y no
arranca la VM por ti** salvo que tú llames a `crisol vm start`.

## 5. Pasar un instalador

```bash
crisol vm install windows programa.exe
# Arranca la VM si no lo está y ejecuta Z:\INSTALAR.cmd dentro del invitado
```

O directamente desde la instalación normal:

```bash
crisol install programa.exe --vm           # ir directamente a la VM
crisol install programa.exe --vm-fallback  # solo si agotan los reintentos de Wine
```

`vm install` copia el instalador a `share/` y escribe `INSTALAR.cmd` con la
línea silenciosa adecuada (`msiexec /i ... /qn /norestart` para MSI,
`/VERYSILENT ...` para Inno, `/S` en el resto) y el `net use Z:`.

## 6. GPU passthrough (opcional y frágil)

```bash
# 1) Añade al arranque del kernel: amd_iommu=on iommu=pt  (o intel_iommu=on iommu=pt)
# 2) Reinicia y comprueba: crisol vm doctor
crisol vm create windows --iso ~/ISO/Windows.iso --gpu-pci 0000:01:00.0 \
    --ram 8 --disk 60
sudo ~/.local/share/crisol/vms/windows/bind-vfio.sh   # ANTES de arrancar
crisol vm start windows
```

Necesitas grupos IOMMU aislados y un GPU que pueda desligarse del host. Si el
host usa ese GPU para el escritorio, quédate sin pantalla: es configuración de
hardware y arranque, no algo que Crisol pueda garantizar.

## Solución de problemas

| Síntoma | Qué revisar |
| --- | --- |
| `vm doctor` dice que faltan binarios | Instala QEMU con `crisol vm setup` + `sudo ...setup-vm.sh`. |
| `/dev/kvm: ausente` | Virtualización desactivada en BIOS/UEFI o usuario fuera del grupo `kvm`; `newgrp kvm` o reinicia sesión. |
| `OVMF no encontrado` | Instala `ovmf` / `edk2-ovmf`; ajusta la ruta en `start.sh` si tu distro usa otra. |
| `qemu-img no está instalado` | Crisol avisa y no crea el disco: ejecuta el `qemu-img create` de arriba. |
| La VM arranca pero no hay red/disco | Instala los drivers virtio dentro del Windows invitado. |
| `net use Z:` falla | Comprueba la línea `-virtfs ... mount_tag=crisol` en `start.sh` y los permisos de `share/`. |
| `QEMU terminó con error` | Mira la salida de `crisol vm doctor`; el error viene de QEMU, no de Crisol. |
| Pantalla negra con GPU VFIO | Ejecuta `bind-vfio.sh` con sudo antes de arrancar y verifica el grupo IOMMU. |

## Límites honestos

* Crisol no instala QEMU ni Windows, no ejecuta `sudo` y no ha arrancado una VM
  en este entorno.
* El disco, la red, el audio y el passthrough dependen de tu kernel, distro y
  hardware. Los scripts son un punto de partida, no una configuración garantizada.
* Incluso con VM, un Windows con acceso a red puede descargar lo que quiera:
  usa instantáneas y no pongas datos sensibles en la carpeta `share/`.
