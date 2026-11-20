## Qué cambia

<!-- Resumen corto. Si aportas una receta, indica el id y el instalador/versión probado. -->

## Comprobaciones

- [ ] `cargo test`
- [ ] `bash tests/integration.sh`
- [ ] `tools/validate-recipes.sh`
- [ ] Los ids nuevos no colisionan con `recipes/core.recipe` ni con otras
      recetas de `community-recipes/apps/` (y empiezan por `community-`)
- [ ] Los logs están redactados y revisados línea a línea; si son reales, hay
      consentimiento de publicación (`community-recipes/REAL-LOGS.md`)
- [ ] No se incluyen DLL, runtimes ni componentes propietarios
- [ ] La documentación afectada está actualizada (README, `docs/`,
      `community-recipes/README.md`, `CHANGELOG.md` si procede)

## Probado / no probado

<!-- Distribución, Wine/winetricks, comandos ejecutados y límites de la prueba. -->

## Relacionado con

<!-- Issue `receta-real`, si la hay: Closes #... -->
