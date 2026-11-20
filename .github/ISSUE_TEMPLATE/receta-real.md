---
name: Receta con log real
about: Aporta una instalación real redactada para convertirla en receta comunitaria
title: "[Receta real] "
labels: receta-real
assignees: ''
---

> Antes de pegar nada: los 6 fixtures de `community-recipes/apps/` son ejemplos
> anonimizados, **no logs reales**. Esta plantilla es solo para instalaciones
> ejecutadas de verdad. Consulta `community-recipes/REAL-LOGS.md`.

## Instalador y versión

* **Aplicación / instalador:** (nombre y versión; si el nombre del archivo es
  sensible, usa `installer.exe` / `setup.msi`)
* **Tipo detectado:** MSI / NSIS / Inno Setup / InstallShield / otro
* **Arquitectura:** 32 bits / 64 bits

## Entorno

* **Distribución y versión:**
* **Wine (versión y motor: wine, wine-staging, wine-ge...):**
* **winetricks (versión o "no instalado"):**
* **bubblewrap / bwrap (versión o "no instalado"):**

## Comando exacto usado

```bash
crisol install ...
```

Incluye todas las opciones (`--engine`, `--attempts`, `--no-sandbox`, `--vm`, ...).

## Síntoma

¿Qué fallaba? ¿Qué señal aparecía en el log? ¿Se llegó a instalar la app?

## Log redactado

Pega aquí el fragmento real (20-80 líneas alrededor del primer error). Debe
incluir, si existen, líneas `err:module:import_dll`, `err:seh`, `fixme:msi`,
mensajes del MSI/instalador y el **código de salida**. Pásalo antes por
`tools/redact-log.sh` y revisa que no queden rutas `/home/<usuario>` reales,
`C:\Users\<usuario>`, IPs, MACs, claves/seriales ni nombres de archivo sensibles.

```text
```

## Receta propuesta

```ini
[community-mi-app]
confidence = 0.80
priority = 10
match_log_any = CADENA_LITERAL_DEL_LOG_ARRIBA
action_winetricks = VERB_DE_WINETRICKS
action_message = Aplicando receta comunitaria...
action_retry = true
```

¿Se probó con `crisol repair <id>`? ¿Funcionó? ¿Cuántos reintentos hicieron falta?

## Confirmación

- [ ] He revisado el fragmento y **no contiene datos personales ni de terceros**
      (usuarios, rutas, IPs, MACs, claves, nombres de archivo sensibles).
- [ ] Autorizo a publicar este log redactado en el repositorio bajo la licencia
      MIT del proyecto (o CC0 si se acuerda para este caso).
- [ ] Confirmo que el log procede de una instalación real que ejecuté yo.
