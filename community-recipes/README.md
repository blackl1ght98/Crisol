# Recetas comunitarias

Este directorio es el repositorio comunitario de recetas compartibles de Crisol.
Aquí se publican soluciones para que otras personas puedan reutilizarlas con
`crisol recipes import`.

**Importante:** 6 de las 8 recetas y sus fixtures actuales son **ejemplos
anonimizados y sintéticos**, no evidencia de instalaciones reales. Sirven para
probar el DSL y el validador; las otras dos usan logs reales redactados (GPU/Vulkan y Rufus/VDS). La campaña para conseguir **20 recetas con logs
reales verificados**, las reglas de redacción y el consentimiento están en
[REAL-LOGS.md](REAL-LOGS.md). Los logs reales tienen prioridad sobre las recetas
especulativas.

Las recetas de aquí **no sustituyen** a las internas (`recipes/core.recipe`): se
importan como recetas de usuario. Por eso los `[id]` deben empezar con
`community-` y no pueden colisionar con los ids internos (`missing-*`,
`try-*`, etc.). Si dos recetas comparten id, la de usuario gana.

## Añadir una receta

Cada receta vive en su propia carpeta:

```
community-recipes/
├── manifest.tsv
└── apps/
    └── <id>/
        ├── recipe.recipe
        └── logs/
            └── <fixture>.log
```

* `<id>` es el mismo valor que el `[id]` del DSL, p. ej.
  `community-vcrun2022-app1`.
* `recipe.recipe` usa el DSL habitual de Crisol (ver `../README.md`, sección
  «Recetas»). Incluye al menos `confidence`, `priority`, `match_log_any`,
  `action_winetricks`, `action_message` y `action_retry`.
* `logs/<fixture>.log` debería ser un fragmento **real y redactado** del log de
  Wine que disparó la receta: 10-30 líneas con `err:module:import_dll`,
  `err:seh`, `fixme:msi`, mensajes MSI/NSIS y el código de salida. Los 6 fixtures sintéticos
  actuales son sintéticos/anonimizados y **no** cumplen esa condición; consulta
  [REAL-LOGS.md](REAL-LOGS.md). **No incluyas rutas personales, nombres de
  usuario, IPs, MACs, claves ni nombres de archivo sensibles**: pásalos antes por
  `tools/redact-log.sh` y usa nombres genéricos (`app-vcrun2022`,
  `app-dotnet48`, ...). Para empezar un id nuevo:
  `tools/new-recipe.sh community-mi-app` (no toca `manifest.tsv`).

Después añade una fila a `manifest.tsv` con columnas separadas por TAB:

```
id<TAB>app<TAB>match_log_any<TAB>action_winetricks<TAB>fixture
```

* `app`: nombre genérico de la aplicación de prueba.
* `match_log_any`: lista de cadenas separadas por coma. Cada cadena debe
  aparecer literalmente en el fixture (la comparación es insensible a
  mayúsculas, igual que en el motor de recetas).
* `action_winetricks`: el valor exacto de `action_winetricks` de la receta.
* `fixture`: ruta del log relativa a `apps/<id>/`, p. ej.
  `logs/vcrun2022-install.log`.

## Validar

Desde la raíz del repositorio:

```sh
tools/validate-recipes.sh
```

El script comprueba por cada fila del manifest que existen la receta y su
fixture, que el log contiene todas las cadenas de `match_log_any` y que la
receta declara el `action_winetricks` anunciado. Imprime `OK <id>` o
`FALLO <id>: motivo` y termina con código 1 si algo falla.

## Importar

El comando **ya funciona de forma recursiva** (no hace falta aplanar ni copiar
los archivos a mano: `collect_recipes` en `src/recipe.rs` recorre los
subdirectorios):

```sh
crisol recipes import community-recipes/apps
```

Eso copia los `recipe.recipe` de cada subdirectorio al directorio de recetas
del usuario (`~/.config/crisol/recipes/`), desde donde `crisol repair <id>`
los selecciona normalmente. Para comprobar qué recetas tienes activas:

```sh
crisol recipes | grep community-
```
