---
name: Informe de error
about: Reporta un fallo de Crisol con datos suficientes para reproducirlo
title: "[Error] "
labels: bug
assignees: ''
---

## Entorno

* **Versión o commit de Crisol:** (p. ej. 0.1.0, hash del commit o fecha de la compilación)
* **Distribución y versión:** (p. ej. Arch Linux, Fedora 40, Ubuntu 24.04)
* **Wine:** (versión y motor: wine, wine-staging, wine-ge...)
* **winetricks:** (versión o "no instalado")
* **bubblewrap / bwrap:** (versión o "no instalado"; indica si usaste `--no-sandbox`)
* **Escritorio / sesión:** (X11, Wayland, GNOME, KDE...)

## Comando ejecutado

```bash
crisol install ...
```

Indica también las opciones usadas (`--engine`, `--attempts`, `--vm`, etc.) y si
el instalador es `.exe` o `.msi`.

## Comportamiento esperado

Describe lo que debería haber ocurrido.

## Comportamiento actual

Describe lo que ocurrió, incluidos mensajes de error breves y el código de salida
si lo conoces.

## Log relevante

Pega solo las líneas necesarias de
`~/.local/share/crisol/logs/<id>-<intentos>.log` entre comillas triples.
**Anonimiza antes**: sustituye tu usuario, rutas absolutas de tu carpeta personal,
nombres de archivos privados, tokens y cualquier dato de terceros por
`<USUARIO>`, `<RUTA>`, etc. No adjuntos logs completos con información personal.

```text
```

## Pasos para reproducir

1.
2.
3.

## Información adicional

¿Ocurre siempre? ¿Funcionó en otra versión de Wine o en otra distribución?
¿Probaste `crisol doctor` o `crisol vm doctor`?
