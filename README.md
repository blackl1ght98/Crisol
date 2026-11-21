# Crisol

[![CI](https://github.com/blackl1ght98/Crisol/actions/workflows/ci.yml/badge.svg)](https://github.com/blackl1ght98/Crisol/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/blackl1ght98/Crisol?include_prereleases)](https://github.com/blackl1ght98/Crisol/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Español](https://img.shields.io/badge/docs-español-green.svg)](README.md) [![English](https://img.shields.io/badge/docs-english-blue.svg)](README.en.md)

Capa de orquestación de compatibilidad que va **más allá de Wine** sin reescribirlo.
Wine es el motor; Crisol es el mecánico: decide qué versión usar, crea un prefijo
aislado, instala dependencias, detecta fallos en el log, aplica recetas, reintenta
y deja la aplicación integrada en el escritorio. Cuando Wine no basta, puede
pasar la instalación a una VM con Windows real.

> Estado: prototipo funcional. Cubre instaladores de escritorio y juegos sobre
> Wine + winetricks, con modo silencioso, iconos extraídos del PE, panel de
> feedback y fallback opcional a QEMU/KVM.

## Instalación

```bash
cd ~/crisol
./install.sh
```

Si no tienes Rust, el instalador te lo indicará. También puedes instalarlo antes:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
```

`install.sh` compila, copia el binario a `~/.local/bin/crisol`, instala las
recetas y el icono, y registra el doble clic para `.exe` y `.msi` con `xdg-mime`.

## Uso

```bash
crisol install ~/Descargas/Programa.msi
crisol install juego.exe --engine wine-ge --attempts 7
crisol install setup.exe --no-sandbox        # si bwrap da problemas
crisol install app.exe --exe "drive_c/Program Files/App/app.exe"   # elegir ejecutable
crisol install programa.exe --timeout 1800   # instaladores largos (por defecto 900s)

crisol install app.exe --interactive         # mostrar la UI del instalador
crisol install portable.exe                  # .exe sin instalador: solo se registra
crisol install portable.exe --execute        # forzar ejecución como prueba
crisol install app.exe --vm                  # pasar directamente a la VM de Windows
crisol install app.exe --vm-fallback         # si Wine falla, pasar a la VM

crisol list                 # apps instaladas
crisol launch <id>          # ejecutar (apps VDS: exige --force, ver abajo)
crisol launch <id> --vm     # preparar su instalador en la VM de Windows
crisol repair <id>          # volver a intentar con recetas
crisol remove <id>          # borrar app y prefijo
crisol recipes              # recetas cargadas (con la confianza actual)
crisol doctor               # comprobar entorno Wine

crisol app                  # abre Crisol: arranca el panel y lo abre en el navegador
crisol panel [--port 8765]  # solo el servidor del panel web local
crisol feedback stats       # votos y % de acierto por receta
crisol feedback export [archivo]   # guardar feedback.tsv
crisol feedback import <archivo>   # importar feedback compartido
crisol recipes import <dir> | crisol recipes export <dir>   # recetas compartidas

crisol --version            # versión
crisol --json list          # salida JSON para scripts (también doctor/recipes/feedback stats)
crisol completions bash     # scripts de completions: bash, zsh, fish

crisol vm doctor            # comprueba QEMU, /dev/kvm, OVMF, IOMMU y PCI
crisol vm setup             # genera el script para instalar QEMU (requiere sudo)
crisol vm create windows --iso /ruta/Windows.iso --ram 8 --disk 60
# Si aún no tienes la ISO: crisol vm create windows --ram 8 --disk 60
# El start.sh usará ~/ISO/Windows.iso si existe, o ISO=/otra/ruta ./start.sh
crisol vm list
crisol vm start windows
crisol vm install windows programa.exe
```

Doble clic en un `.exe`/`.msi` → se abre una terminal → Crisol instala,
repara y crea el acceso directo con `crisol launch <id>`.

`crisol setup-desktop` instala **dos** entradas: `Crisol` (lanzador visible que
ejecuta `crisol app`) y `Crisol (instalar con doble clic)` (manejador oculto de
`.exe`/`.msi`). Antes había una sola entrada con `%f`: al abrirla desde el menú
no recibía archivo y terminaba enseguida, que era el fallo que hacía pensar que
"la app no abría". Si ya tenías el menú abierto, vuelve a ejecutar
`crisol setup-desktop` para actualizarlo.

## Instalación silenciosa e iconos

Por defecto Crisol pide al instalador que no muestre diálogos, según el tipo
detectado:

| Tipo | Argumentos |
| --- | --- |
| MSI | `msiexec /i app.msi /qn /norestart` |
| NSIS | `app.exe /S` |
| Inno Setup | `app.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /NOCANCEL` |
| InstallShield | `app.exe /s /v/qn` |
| WiX | `app.exe /quiet /norestart` |

Con `--interactive` se muestra la interfaz original. No todos los instaladores
respetan estos flags: esos sí pueden seguir pidiendo clics.

Tras instalar, Crisol lee los recursos del PE (`RT_GROUP_ICON` / `RT_ICON`) y
guarda un `.ico` para el acceso directo. Si no hay iconos o no se pueden leer,
usa el icono propio de Crisol.

## Fallback a Windows real (VM)

Wine no puede arreglar anti-cheat de kernel, DRM ni controladores de hardware.
Para esos casos Crisol prepara una VM, pero **no instala el sistema por ti**:

```bash
crisol vm doctor                     # qué falta
crisol vm setup                      # escribe setup-vm.sh
sudo ~/.local/share/crisol/vms/setup-vm.sh   # tú ejecutas el script con contraseña
# NO escribas 'sudo crisol vm setup': sudo no ve ~/.local/bin
crisol vm create windows --iso ~/ISO/Windows.iso --ram 8 --disk 60
crisol vm start windows
```

En Arch/CachyOS, `setup-vm.sh` instala `qemu-desktop`, `ovmf` y `virt-manager`.
`virtio-win` no está en los repos oficiales: el script lo intenta, avisa si
falla y sugiere `yay -S virtio-win` o descargar el ISO de
https://github.com/virtio-win/virtio-win-pkg-scripts#readme

La VM se crea con disco `qcow2`, virtio, red, audio, PCIe y una carpeta
compartida 9p (`share`, montable en el guest como `Z: \\.\crisol`). `crisol
install app.exe --vm` (o `--vm-fallback` tras agotar los reintentos de Wine)
copia el instalador a esa carpeta y escribe `Z:\INSTALAR.cmd` con los argumentos
silenciosos adecuados.

Para juegos que exigen GPU, añade `--gpu-pci 0000:01:00.0` al crear la VM y
ejecuta con `sudo` el `bind-vfio.sh` generado. Necesitas IOMMU activado en el
kernel (`amd_iommu=on iommu=pt` o `intel_iommu=on iommu=pt`) y grupos VFIO
aislados; es hardware y configuración de arranque, no algo que Crisol pueda
garantizar.

## Ejecutables portables (Rufus y similares)

Un `.exe` sin firma de instalador (tipo `PlainExe`) **no es un instalador**.
Crisol no lo ejecuta durante `install`: lo copia a un prefijo aislado, crea el
acceso directo y termina. Arráncalo cuando quieras con `crisol launch <id>`.

```bash
cd ~/Descargas
crisol install rufus-4.16.exe     # registra; NO ejecuta
crisol launch rufus-4-16-<hash>   # launch usando el ID que imprime Crisol
```

Esto evita un falso "instalación fallida" con utilidades portables. Aun así,
**Rufus no funciona bajo Wine**: necesita Virtual Disk Service (VDS), un servicio
de Windows que Wine no implementa. No existe un `winetricks` que lo corrija.
El volcado típico termina en `Unhandled exception: page fault ... movzwl (%rcx)`

Desde `v0.1.3`, `crisol launch` **bloquea** estas utilidades antes de ejecutar
Wine: detecta el nombre (`rufus`, `etcher`, `balena`, `usb`, `disk`...), el
ejecutable o la receta `incompatible-vds`, muestra la explicación y avisa en el
escritorio. No se ha lanzado nada y puedes elegir:

```bash
crisol install ~/Descargas/rufus-4.16.exe --vm   # Windows real (recomendado)
crisol launch rufus-4-16-<hash> --force          # forzar; confirma el fallo
```

`--force` solo ignora el aviso: no arregla VDS. Para grabar una ISO/USB de
verdad necesitas la VM con Windows:

```bash
crisol vm create windows --iso /ruta/Windows.iso
crisol install ~/Descargas/rufus-4.16.exe --vm
# equivalente: crisol launch rufus-4-16-<hash> --vm
```

Después de `install --vm` (o `launch --vm`) Crisol imprime los pasos restantes y
dos bloques **listos para copiar y pegar** (uno para tu terminal, otro para el
Símbolo del sistema de Windows), porque no puede arrancar Windows ni hacer clic
dentro del guest:

```bash
crisol vm doctor                      # QEMU, /dev/kvm, OVMF
crisol vm setup                       # si falta QEMU: genera un script; ejecútalo con sudo
crisol vm start windows               # arranca la VM
```

Dentro de Windows, abre un Símbolo del sistema y ejecuta:

```bat
net use Z: \\.\crisol
Z:
cd \
INSTALAR.cmd
```

El archivo ya está en `Z:\` y `INSTALAR.cmd` muestra el resultado y deja la
ventana abierta. Si es la primera vez, instala Windows desde la ISO y los
drivers virtio antes de ejecutar el script.

Cuando se detecta el CLSID de VDS (o un `page fault`) en un log, la receta
interna `incompatible-vds` muestra este diagnóstico y detiene los reintentos en
vez de gastar los 5 intentos. `action_retry = false` ahora se respeta.

## Panel de feedback

```bash
crisol app              # arranca el panel y abre el navegador (lo que hace el menú)
crisol panel --port 8765
# o abre http://127.0.0.1:8765
```

Muestra el catálogo, las recetas aplicadas a cada app y la confianza de todas
las recetas. Los botones **Funcionó** / **No funcionó** cambian el estado de la
app y suman `+0.05` / `-0.10` a la confianza de esas recetas en
`~/.config/crisol/feedback.tsv`. La siguiente ejecución de Crisol ya usa esa
confianza para ordenar las recetas.

Además guarda un historial en `~/.config/crisol/feedback.log` y muestra dos
secciones nuevas: **estadísticas** (votos OK/no y % de acierto por receta) y
**historial reciente**. Puedes descargar el TSV desde `/export` o desde la
terminal:

```bash
crisol feedback stats                 # votos por receta
crisol feedback export compartido.tsv
crisol feedback import compartido.tsv
```

Escucha solo en `127.0.0.1`: es un servidor HTTP mínimo con Rust estándar, en
lugar de GTK4 (que no está instalado). La primera vez genera un token en
`~/.config/crisol/panel.token`; `/ok`, `/bad` y `/export` lo exigen y las peticiones
con `Origin` ajeno se rechazan con 403, para que una web no pueda votar por ti.
Detalles en `docs/SECURITY.md`.

## Cómo funciona

```text
handler MIME
    → detecta MSI / NSIS / Inno Setup / InstallShield y arquitectura (PE)
    → crea prefijo WINEPREFIX nuevo y guarda snapshot
    → ejecuta msiexec /i o wine setup.exe con argumentos silenciosos dentro de bubblewrap
    → lee el log y busca recetas coincidentes
    → aplica winetricks / overrides / cambio de motor / rollback
    → reintenta (hasta 5 veces por defecto)
    → extrae el icono del .exe, genera .desktop y guarda en el catálogo
    → si falla y se pidió --vm-fallback, copia el instalador a la VM
```

Rutas (XDG):

| Qué | Dónde |
| --- | --- |
| Prefijos | `~/.local/share/crisol/prefixes/<id>` |
| Snapshots | `~/.local/share/crisol/backups/` |
| Logs | `~/.local/share/crisol/logs/` |
| Catálogo | `~/.local/share/crisol/catalog.tsv` |
| Recetas del usuario | `~/.config/crisol/recipes/*.recipe` |
| Feedback | `~/.config/crisol/feedback.tsv` |
| VMs | `~/.local/share/crisol/vms/<nombre>/` |
| Iconos extraídos | `~/.local/share/icons/hicolor/scalable/apps/<id>.ico` |

## Recetas

Las recetas son archivos de texto muy simples. Se cargan las internas
(`recipes/core.recipe`) y las del usuario, que **sobrescriben** a las internas
si tienen el mismo `[id]`.

```ini
[mi-arreglo]
confidence = 0.9
priority = 20
match_log_any = msvcp140.dll, vcruntime140.dll
match_detector = nsis              # opcional: pistas del instalador
match_exit_code = 1                # opcional
match_file_missing = drive_c/windows/system32/foo.dll   # opcional, admite *
action_winetricks = vcrun2022
action_override = oleaut32=native
action_env = WINEARCH=win64
action_engine = wine-ge
action_reset_prefix = true         # volver al snapshot antes de reintentar
action_message = Instalando Visual C++ 2022...
action_retry = true
```

* `match_log_any`: se cumple si aparece **cualquiera** de los valores (sin distinción de mayúsculas).
* `match_log_contains`: se cumple solo si aparece ese valor exacto.
* `match_log_not`: se cumple si **no** aparece ninguno.
* Todas las condiciones se combinan con AND.
* `score = confidence * 100 + priority`. Se aplican hasta 3 recetas por intento.
* El panel ajusta la confianza guardando deltas en `feedback.tsv`.
* Cada intento guarda su log en `~/.local/share/crisol/logs/<id>-<intentos>.log`.
* `wineboot` se ejecuta sin `bwrap` (los servicios de Wine no arrancan bien dentro);
  el instalador sí se ejecuta aislado salvo que uses `--no-sandbox`.

Ejemplo: crear `~/.config/crisol/recipes/mi-app.recipe` con una solución que
descubriste, y `crisol repair <id>` la aplicará.

## Recetas compartidas y calidad

El directorio `community-recipes/` es el repositorio comunitario de recetas:
cada app tiene su carpeta en `apps/<id>/` con un `recipe.recipe` y uno o más
logs de ejemplo en `logs/`. **De las 8 recetas actuales, 6 usan fixtures
anonimizados y 2 se basan en logs reales redactados**: `community-recipes/REAL-LOGS.md` explica
cómo aportar más logs reales redactados (objetivo: 20 recetas verificadas). El
`manifest.tsv` relaciona cada receta con la cadena que debe aparecer en su log y
con la acción winetricks esperada.

```bash
tools/validate-recipes.sh                  # comprueba receta, log y match
crisol recipes import community-recipes/apps   # las copia a ~/.config/crisol/recipes
crisol recipes export /tmp/mis-recetas     # exporta las internas + las tuyas
```

Esto permite que cualquiera aporte una receta con su log real y que la
validación automática detecte recetas huérfanas o fixtures que ya no coinciden.
Las recetas importadas se tratan igual que las del usuario: sobrescriben a las
internas si repetimos `[id]`, y el panel puede ajustar su confianza.

## Comunidad y contribuciones

- `community-recipes/REAL-LOGS.md`: campaña de logs reales y redacción de datos personales.
- `docs/CONTRIBUTING.md`: cómo abrir issues, añadir recetas y enviar PRs.
- `docs/SECURITY.md`: modelo de seguridad del panel, `bwrap` y VMs.
- `docs/VM.md`: requisitos y solución de problemas de QEMU/KVM.
- `docs/UPSTREAM.md`: borradores de colaboración con winetricks, Bottles y Lutris (aún no enviados).
- `CHANGELOG.md` y `ROADMAP.md`: historial y prioridades.
- `.github/workflows/ci.yml` + `tests/integration.sh`: `cargo test`, build y prueba de extremo a extremo en cada PR.
- `packaging/`: `PKGBUILD` de Arch, spec de Fedora y manifiesto Flatpak (verificación estática; paquetes no construidos aquí).
- `README.en.md`: resumen en inglés.
- `LICENSE`: MIT.

## Límites honestos

* Los clics se reducen mucho con los argumentos silenciosos, pero **no se eliminan
  al 100 %**: hay instaladores que ignoran esos flags o fuerzan diálogos propios.
  `--interactive` los muestra a propósito.
* Anti-cheat de kernel, DRM y drivers de hardware **no se arreglan en userspace**:
  requieren un Windows real. Crisol solo prepara la VM; necesitas QEMU/KVM, una
  ISO de Windows, reiniciar sesión tras unirte al grupo `kvm`, y (para GPU)
  IOMMU/VFIO. No instala QEMU ni Windows sin tu contraseña.
* El panel es una web local en `127.0.0.1`, no una app de escritorio nativa.
* Crisol no incluye DLL ni runtimes de Microsoft: `winetricks` los descarga de
  fuentes oficiales y aplica sus licencias. Respeta los EULAs.
* `bwrap` se usa con `/tmp` y el prefijo escribibles; es aislamiento práctico,
  no una frontera de seguridad estricta. `--no-sandbox` lo desactiva.

## Desarrollo

```bash
cd ~/crisol
cargo test
cargo run -- doctor
cargo run -- vm doctor
cargo build --release
bash tests/integration.sh
tools/validate-recipes.sh
```

El proyecto usa **solo la biblioteca estándar** de Rust: no hay crates que
descargar, así que compila incluso sin acceso a crates.io.
