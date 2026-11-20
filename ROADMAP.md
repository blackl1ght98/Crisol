# Hoja de ruta

Orden de prioridades. «Hecho» significa que existe código o documentación en
este repositorio; no implica que esté publicado o validado en producción.

| # | Prioridad | Estado | Siguiente paso |
| --- | --- | --- | --- |
| 1 | Logs reales y recetas validadas | **En curso** | Conseguir 20 recetas con logs reales verificados; hoy hay 0 y 6 fixtures anonimizados. |
| 2 | CI activo | **Hecho en el repo / pendiente de confirmar en GitHub** | Verificar que los jobs de `.github/workflows/ci.yml` están verdes en `main`. |
| 3 | Release con tag y checksums | **Pendiente** | Publicar `v0.1.0` con tarball y `SHA256SUMS`. |
| 4 | Verificación de paquetes | **Pendiente** | Sustituir `md5sums=('SKIP')` y `Source0` Git por tarball con checksum; probar Arch/Fedora/Flatpak. |
| 5 | Arranque real de VM | **Pendiente** | Instalar QEMU, arrancar Windows y probar `--vm` / `--vm-fallback` de extremo a extremo. |
| 6 | PRs aguas arriba | **Pendiente** | Enviar los borradores de `docs/UPSTREAM.md` a winetricks, Bottles y Lutris. |

## 1. Logs reales y recetas validadas — EN CURSO

Hecho:

* DSL de recetas, motor de emparejamiento y feedback por confianza.
* `tools/validate-recipes.sh` y `community-recipes/manifest.tsv`.
* 6 recetas comunitarias con fixtures **sintéticos/anonimizados**: sirven para
  validar el formato, **no son logs reales**.
* Herramientas de apoyo: `tools/redact-log.sh`, `tools/new-recipe.sh`,
  `community-recipes/REAL-LOGS.md` y la plantilla de issue `receta-real`.

Pendiente:

* **Objetivo: 20 recetas con logs reales verificados.** Cada una con log
  redactado, consentimiento, entorno declarado y validación en verde.
* Priorizar logs sobre recetas especulativas: sin log real no se añade receta
  nueva al directorio comunitario.

## 2. CI activo — HECHO EN EL REPO, PENDIENTE DE CONFIRMACIÓN

Hecho: workflow con `cargo fmt --check`, `cargo clippy -D warnings`,
`cargo test --all-targets`, build release y `bash tests/integration.sh`.

Pendiente: comprobar que el badge de CI está en verde tras el primer push a
`main` y que no hay jobs omitidos por permisos del repositorio.

## 3. Release con tag y checksums — PENDIENTE

No hay tag ni fecha de release. Cuando se publique: etiqueta `v0.1.0`, notas
desde `CHANGELOG.md`, tarball fuente y `SHA256SUMS` firmado o al menos
verificable. El badge de release de `README.en.md` apuntará a esa publicación.

## 4. Verificación de paquetes — PENDIENTE

`packaging/` es experimental: el `PKGBUILD` usa `md5sums=('SKIP')` porque clona
Git, y el spec de Fedora también necesita red. Flatpak usa `branch: main`.
Sustituir todo por el tarball con checksum del punto 3 y ejecutar `makepkg`,
`rpmbuild` y `flatpak-builder` en entornos limpios.

## 5. Arranque real de VM — PENDIENTE

Hecho: `vm doctor`, generación de scripts qcow2/OVMF/9p/VFIO y la prueba de
integración sin QEMU. Pendiente: QEMU no está instalado en este entorno y la VM
nunca ha arrancado; hay que probar Windows, virtio, compartida 9p, `--vm` y
`--vm-fallback`. Detalles y límites en `docs/VM.md`.

## 6. PRs aguas arriba — PENDIENTE

`docs/UPSTREAM.md` contiene borradores para winetricks, Bottles y Lutris.
**Ninguno ha sido enviado.** Solo abrir una issue primero, esperar feedback y
después un PR pequeño, con logs anonimizados.
