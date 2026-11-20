# Changelog

Todos los cambios notables de Crisol. El formato sigue en líneas generales
[Keep a Changelog](https://keepachangelog.com/es/1.1.0/). No se registran fechas
de release porque aún no hay una publicación con tag.

## [Unreleased]

### Próximamente

* Logs reales verificados para recetas comunitarias (objetivo: 20). Campaña en
  `community-recipes/REAL-LOGS.md`.
* Panel: token local, comprobación de `Origin` / `Sec-Fetch-Site` y cabeceras
  `nosniff` / `referrer-policy` (pendiente; ver `docs/SECURITY.md`).
* Arranque real de una VM y PRs aguas arriba (ver `ROADMAP.md`).

## [0.1.0]

Primera versión del prototipo funcional. Sin fecha de publicación: es la versión
declarada en `Cargo.toml` y del código presente en el repositorio.

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
  catálogo local, `launch`, `repair` y `remove`.
* **Panel de feedback local** en `http://127.0.0.1:<puerto>` con catálogo,
  confianza de recetas, votos **Funcionó / No funcionó**, historial
  (`feedback.log`), estadísticas y export/import (`/export`,
  `crisol feedback stats|export|import`).
* **Importación y exportación de recetas compartidas**, incluida la búsqueda
  recursiva con `crisol recipes import <dir>`.
* **Recetas comunitarias** en `community-recipes/` con 6 recetas y 6 fixtures
  **anonimizados/sintéticos** (no logs reales) validados por
  `tools/validate-recipes.sh` y `community-recipes/manifest.tsv`.
* **Fallback a VM**: `crisol vm doctor|setup|list|create|start|install` genera
  QEMU/KVM con disco qcow2, virtio, OVMF, 9p y scripts para VFIO; opciones
  `--vm` y `--vm-fallback`. No instala QEMU ni Windows y no ejecuta `sudo`.
* **Bubblewrap** como aislamiento práctico del instalador, desactivable con
  `--no-sandbox`.
* **CI** en `.github/workflows/ci.yml`: fmt, clippy, `cargo test`, build de
  release y `tests/integration.sh`.
* **Empaquetado experimental** para Arch (`PKGBUILD`), Fedora (spec) y Flatpak
  en `packaging/`, además de `install.sh` para `~/.local/bin`.
* Documentación en español: README, `docs/CONTRIBUTING.md`,
  `docs/UPSTREAM.md`, plantillas de issue/PR y código de conducta.

### Limitaciones conocidas

* Los 6 fixtures de `community-recipes/apps/` no son evidencia de instalaciones
  reales; no hay todavía 20 recetas con logs reales verificados.
* El panel no dispone aún de token, filtrado de `Origin` ni cabeceras de
  seguridad; ver `docs/SECURITY.md`.
* La VM no ha sido arrancada en el entorno de desarrollo y QEMU no está
  instalado en él; ver `docs/VM.md`.
* Los clics no se eliminan al 100 %: algunos instaladores ignoran los flags
  silenciosos; anti-cheat/DRM/drivers requieren Windows real.
* Crisol no distribuye DLL ni runtimes de Microsoft; `winetricks` los descarga
  de fuentes oficiales y aplica sus licencias.
