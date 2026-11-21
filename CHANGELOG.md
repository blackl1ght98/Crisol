# Changelog

Todos los cambios notables de Crisol. El formato sigue en líneas generales
[Keep a Changelog](https://keepachangelog.com/es/1.1.0/). Hay dos versiones
publicadas: `v0.1.0`, `v0.1.1`, `v0.1.2`, `v0.1.3`, `v0.1.4`, `v0.1.5`,
`v0.1.6`, `v0.1.7`, `v0.1.8`, `v0.1.9` y `v0.1.10`, con tarball fuente,
binario y `SHA256SUMS`.

## [Unreleased]

### En curso

* Logs reales verificados para recetas comunitarias (objetivo: 20). Campaña en
  `community-recipes/REAL-LOGS.md`.
* Arranque real de una VM y envío de los borradores de `docs/UPSTREAM.md`.

## [0.1.10]

### Añadido

* El aviso para utilidades de disco ofrece ahora una tercera vía además de la
  VM y `--force`: alternativas nativas de Linux (balenaEtcher, Ventoy, WoeUSB)
  para cuando solo se quiere grabar una ISO. La documentación explica por qué
  Rufus en concreto no puede funcionar bajo Wine (sin VDS ni acceso a
  `\\.\PhysicalDrive`; WineHQ 38309 y 37166).

### Pruebas

* `tests/integration.sh` comprueba que el aviso menciona las alternativas
  nativas.

## [0.1.9]

### Corregido

* `setup-vm.sh` ahora ejecuta `pacman -Syy` antes de instalar (evita 404s por
  base de datos desactualizada) y, si la descarga falla, imprime los comandos
  exactos para descartar un mirror roto: comentar `mirror5.krfoss.org` en
  `cachyos-mirrorlist`, ejecutar `reflector` y volver a intentar.

### Pruebas

* `tests/integration.sh` comprueba que el script refresca la DB y da la guía de
  mirrors.

## [0.1.8]

### Corregido

* `setup-vm.sh` instalaba `virtio-win`, que no existe en los repositorios de
  Arch/CachyOS, y `set -e` abortaba toda la instalación. Ahora instala
  `qemu-desktop` (incluye `qemu-img` y GTK, sin preguntar por proveedores) y
  trata `virtio-win` como opcional, con instrucciones para AUR o el ISO oficial.
* La guía VM explica de dónde sacar los drivers virtio cuando no están en los
  repos.

### Pruebas

* `tests/integration.sh` comprueba que el script usa `qemu-desktop` y que
  `virtio-win` es opcional.

## [0.1.7]

### Corregido

* El comando `sudo` de `vm setup` y de la guía VM se muestra con `~` en vez de
  una ruta absoluta larga: `sudo ~/.local/share/crisol/vms/setup-vm.sh`. El
  shell expande `~` antes de `sudo`, así que se puede copiar y pegar sin saber
  dónde están los archivos. `sudo crisol ...` seguirá sin funcionar porque
  `sudo` no ve `~/.local/bin`.

## [0.1.6]

### Corregido

* `crisol vm setup` mostraba `sudo <script>` de forma ambigua y llevaba a
  escribir `sudo crisol vm setup`, que falla porque `sudo` no ve `~/.local/bin`
  y Crisol no debe ejecutarse como root. Ahora imprime la línea exacta con la
  ruta absoluta: `sudo ~/.local/share/crisol/vms/setup-vm.sh`.
* `crisol vm doctor` aclara que IOMMU solo es necesario para GPU passthrough,
  no para arrancar una VM normal.

### Pruebas

* `tests/integration.sh` comprueba que `vm setup` muestra la ruta absoluta y no
  sugiere `sudo crisol`.

## [0.1.5]

### Añadido

* El aviso de incompatibilidad VDS muestra `crisol launch <id> --vm` como
  primera opción, antes que `--force`, para que el usuario no tenga que
  interpretar el mensaje.
* La guía de la VM termina con dos **bloques listos para copiar y pegar**: los
  comandos de la terminal host (`vm doctor`, `vm setup`, `sudo ...setup-vm.sh`,
  `newgrp kvm`, `vm create`, `vm start`) y las líneas del Símbolo del sistema de
  Windows (`net use Z: \\.\crisol`, `Z:`, `cd \`, `INSTALAR.cmd`).

### Pruebas

* `tests/integration.sh` verifica ambos bloques y que el aviso VDS ofrece `--vm`.

## [0.1.4]

### Añadido

* `crisol launch <id> --vm`: prepara el instalador original de la app en la VM
  `windows` en vez de ignorar la opción. Es la forma corta de
  `crisol install <instalador> --vm`.
* Tras preparar una VM, la salida muestra **cómo continuar** paso a paso:
  `crisol vm doctor`, `crisol vm setup` si falta QEMU o `/dev/kvm`,
  `crisol vm start windows` y las líneas exactas para montar `Z:\` y ejecutar
  `INSTALAR.cmd` dentro de Windows. El `.cmd` generado también muestra el
  resultado y deja la ventana abierta.

### Corregido

* `crisol install --vm` ya no termina con una sola línea ambigua; avisa cuando
  la máquina aún no puede arrancar y dice qué falta.

### Pruebas

* `tests/integration.sh` comprueba que `launch <id> --vm` copia el instalador al
  `share/` de la VM y muestra los pasos, sin llamar a Wine.

## [0.1.3]

### Añadido

* `crisol launch <id> --force`: permiso explícito para intentar arrancar una app
  marcada como incompatible. Sin esa opción, `launch` detecta utilidades de disco
  (por el nombre, el ejecutable o la receta `incompatible-vds`), explica que Wine
  no implementa Virtual Disk Service y **no llama a Wine**. También avisa en el
  escritorio con `notify-send`.
* Los logs con `Unhandled exception` / `page fault` se reconocen como crash de
  una API nativa no implementada y muestran la explicación accionable.

### Corregido

* Tras registrar un portable de disco, se indicaba `crisol launch <id>` como si
  fuese a funcionar; ahora se indica `--vm` y se reserva `--force` para ver el fallo.

### Pruebas

* `tests/integration.sh` comprueba con un `wine` falso y un marcador que `launch`
  no ejecuta la app VDS sin `--force` y que sí lo hace con `--force`.

## [0.1.2]

### Añadido

* **Ejecutables portables**: un `PlainExe` ya no se ejecuta como instalador. Se
  copia al prefijo aislado, se registra en el catálogo con estado `portable` y
  se crea su lanzador; se arranca con `crisol launch <id>`. La nueva opción
  `--execute` (alias `--smoke-test`) fuerza el comportamiento anterior.
* **Diagnóstico VDS**: receta interna `incompatible-vds` y receta comunitaria
  `community-real-vds-portable-app1` basada en el log real redactado del fallo
  de Rufus. Explica que Wine no implementa Virtual Disk Service y apunta a `--vm`.
* `crisol install` muestra una advertencia específica para utilidades de disco
  (Rufus, Etcher, balenaEtcher...) antes de intentar lanzarlas.

### Corregido

* `action_retry = false` se respeta: un fallo determinista ya no agota los 5
  intentos. Antes `Action::Retry` se ignoraba por completo.
* El genérico "No hay recetas que coincidan" se sustituye por una explicación
  accionable cuando el log contiene un CLSID de incompatibilidad conocida.
* El detector añade la pista `portable` y expone `Detection::is_portable()`.

### Pruebas

* 13 tests unitarios (nuevos: detección de portable y mensaje VDS).
* `tests/integration.sh` cubre el registro portable con `wineboot` simulado,
  la copia al prefijo, el `.desktop`, el catálogo y la receta VDS activa.

## [0.1.1]

Correcciones y evidencia añadida sobre `v0.1.0`.

### Añadido

* 1 receta comunitaria (`community-real-vulkan-gpu-app1`) basada en un log real
  redactado de una instalación local, con diagnóstico sin winetricks validado.
  El validador acepta `action_winetricks = -` para este tipo de receta.

### Corregido

* El token del panel se crea con permisos `0600` en sistemas Unix.

## [0.1.0]

Primera versión del prototipo funcional, etiquetada como `v0.1.0`.

### Añadido

* **Instalación silenciosa por tipo de instalador**: MSI (`msiexec /i ... /qn
  /norestart`), NSIS (`/S`), Inno Setup (`/VERYSILENT ...`), InstallShield
  (`/s /v/qn`) y WiX (`/quiet /norestart`), con `--interactive` para mostrar la
  UI original.
* **Detección** del tipo de instalador y de la arquitectura a partir del PE.
* **Prefijos aislados** por aplicación, snapshot previo, rollback y hasta 5
  reintentos configurables con `--attempts`.
* **Recetas DSL** internas (`recipes/core.recipe`) y de usuario, con
  condiciones `match_log_*`, `match_detector`, `match_exit_code`,
  `match_file_missing`, y acciones winetricks/override/env/engine/mensaje/reintento;
  `score = confianza × 100 + prioridad`.
* **Extracción de iconos del PE** (`RT_GROUP_ICON` / `RT_ICON`) para el acceso
  directo `.desktop`, con icono propio de Crisol como reserva.
* **Integración de escritorio**: doble clic en `.exe`/`.msi` vía `xdg-mime`,
  catálogo local, `launch`, `repair` y `remove`. `crisol setup-desktop` instala
  un lanzador visible (`crisol.desktop`, ejecuta `crisol app`) y un manejador
  oculto (`crisol-handler.desktop`) para el doble clic.
* **`crisol app`**: abre el panel en el navegador y lo arranca si no está activo.
  Corrige el fallo por el que la entrada del menú parecía "no abrir".
* **Panel de feedback local** en `http://127.0.0.1:<puerto>` con catálogo,
  confianza de recetas, votos **Funcionó / No funcionó**, historial
  (`feedback.log`), estadísticas y export/import (`/export`,
  `crisol feedback stats|export|import`).
* **Seguridad del panel**: token local en `~/.config/crisol/panel.token`
  exigido en `/ok`, `/bad` y `/export`; rechazo de `Origin` ajeno y de
  `Sec-Fetch-Site: cross-site`; cabeceras `nosniff`, `referrer-policy` y
  `no-store`. Ver `docs/SECURITY.md`.
* **CLI para scripts**: `--version`, `--json` (`list`, `doctor`, `recipes`,
  `feedback stats`) y `crisol completions bash|zsh|fish`.
* **Importación y exportación de recetas compartidas**, incluida la búsqueda
  recursiva con `crisol recipes import <dir>`.
* **Recetas comunitarias** en `community-recipes/` con 6 recetas y 6 fixtures
  **anonimizados/sintéticos**, más 1 receta con un log real redactado, validadas por
  `tools/validate-recipes.sh` y `community-recipes/manifest.tsv`.
* **Herramientas de comunidad**: `tools/redact-log.sh`, `tools/new-recipe.sh`,
  plantilla de issue `receta-real`, PR template y campaña `REAL-LOGS.md`.
* **Fallback a VM**: `crisol vm doctor|setup|list|create|start|install` genera
  QEMU/KVM con disco qcow2, virtio, OVMF, 9p y scripts para VFIO; opciones
  `--vm` y `--vm-fallback`. No instala QEMU ni Windows y no ejecuta `sudo`.
* **Bubblewrap** como aislamiento práctico del instalador, desactivable con
  `--no-sandbox`.
* **CI** en `.github/workflows/ci.yml`: fmt, clippy, `cargo test`, build de
  release y `tests/integration.sh` (incluye comprobación de CSRF y CLI JSON).
* **Empaquetado experimental** para Arch (`PKGBUILD`), Fedora (spec) y Flatpak
  en `packaging/`, con verificación estática en `tools/check-packaging.sh`.
* **Documentación**: README (es/en), `docs/CONTRIBUTING.md`, `docs/SECURITY.md`,
  `docs/VM.md`, `docs/UPSTREAM.md`, `CHANGELOG.md`, `ROADMAP.md`, plantillas de
  issue/PR y código de conducta.

### Limitaciones conocidas

* Solo 2 de las 8 recetas comunitarias se basan en logs reales; faltan 18 para
  alcanzar el objetivo de 20 recetas con logs reales verificados.
* El workflow de CI está en el repositorio pero no pudo subirse con el token
  disponible (GitHub exige el scope `workflow`).
* La VM no ha sido arrancada en el entorno de desarrollo y QEMU no está
  instalado en él; ver `docs/VM.md`.
* Los paquetes no se han construido: `packaging/` solo tiene verificación
  estática; faltan `rpmbuild` y `flatpak-builder` en este entorno.
* Los clics no se eliminan al 100 %: algunos instaladores ignoran los flags
  silenciosos; anti-cheat/DRM/drivers requieren Windows real.
* Crisol no distribuye DLL ni runtimes de Microsoft; `winetricks` los descarga
  de fuentes oficiales y aplica sus licencias.
