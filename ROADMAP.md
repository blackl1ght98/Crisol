# Hoja de ruta

Orden de prioridades. «Hecho» significa que existe código o documentación en
este repositorio; no implica que esté publicado o validado en producción.

| # | Prioridad | Estado | Siguiente paso |
| --- | --- | --- | --- |
| 1 | Logs reales y recetas validadas | **En curso** | Conseguir 20 recetas con logs reales verificados; hoy hay 2 y 6 fixtures anonimizados. |
| 2 | CI activo | **Bloqueado por permisos** | El workflow está en el repo local; subirlo con un PAT que tenga scope `workflow` y verificar los jobs. |
| 3 | Release con tag y checksums | **Hecho** | `v0.1.0` a `v0.1.5` publicados con tarball, binario y `SHA256SUMS`. |
| 4 | Verificación de paquetes | **Parcial** | `tools/check-packaging.sh` pasa; construir paquetes reales con `makepkg`, `rpmbuild` y `flatpak-builder`. |
| 5 | Arranque real de VM | **Pendiente** | Instalar QEMU, arrancar Windows y probar `--vm` / `--vm-fallback` de extremo a extremo. |
| 6 | PRs aguas arriba | **Pendiente** | Enviar los borradores de `docs/UPSTREAM.md` a winetricks, Bottles y Lutris. |

## 1. Logs reales y recetas validadas — EN CURSO

Hecho:

* DSL de recetas, motor de emparejamiento y feedback por confianza.
* `tools/validate-recipes.sh` y `community-recipes/manifest.tsv`.
* 6 recetas comunitarias con fixtures **sintéticos/anonimizados**: sirven para
  validar el formato, **no son logs reales**.
* 2 recetas con logs **reales redactados** de instalaciones locales:
  `community-real-vulkan-gpu-app1` (GPU/Vulkan) y
  `community-real-vds-portable-app1` (Rufus/VDS). Ambas son diagnósticos sin
  winetricks validado.
* Herramientas de apoyo: `tools/redact-log.sh`, `tools/new-recipe.sh`,
  `community-recipes/REAL-LOGS.md` y la plantilla de issue `receta-real`.

Pendiente:

* **Objetivo: 20 recetas con logs reales verificados.** Cada una con log
  redactado, consentimiento, entorno declarado y validación en verde.
* Priorizar logs sobre recetas especulativas: sin log real no se añade receta
  nueva al directorio comunitario.

## 2. CI activo — BLOQUEADO POR PERMISOS

Hecho: workflow con `cargo fmt --check`, `cargo clippy -D warnings`,
`cargo test --all-targets`, build release y `bash tests/integration.sh`.

Pendiente: GitHub rechaza `.github/workflows/ci.yml` con el PAT disponible porque
no tiene el scope `workflow`. Hay un commit local listo; al subirlo hay que
comprobar que el badge está en verde y que no hay jobs omitidos.

## 3. Release con tag y checksums — HECHO

Tags `v0.1.0` a `v0.1.5` publicados con notas desde `CHANGELOG.md`,
tarball fuente, binario x86_64 y `SHA256SUMS` verificable.

## 4. Verificación de paquetes — PARCIAL

`packaging/` sigue siendo experimental. `tools/check-packaging.sh` valida la
estructura, sintaxis del PKGBUILD, campos del spec y YAML, y pasa correctamente.
Queda sustituir las fuentes Git por el tarball con checksum del punto 3 y
ejecutar `makepkg`, `rpmbuild` y `flatpak-builder` en entornos limpios.

## 5. Arranque real de VM — PENDIENTE

Hecho: `vm doctor`, generación de scripts qcow2/OVMF/9p/VFIO y la prueba de
integración sin QEMU. Pendiente: QEMU no está instalado en este entorno y la VM
nunca ha arrancado; hay que probar Windows, virtio, compartida 9p, `--vm` y
`--vm-fallback`. Detalles y límites en `docs/VM.md`.

## 6. PRs aguas arriba — PENDIENTE

`docs/UPSTREAM.md` contiene borradores para winetricks, Bottles y Lutris.
**Ninguno ha sido enviado.** Solo abrir una issue primero, esperar feedback y
después un PR pequeño, con logs anonimizados.
