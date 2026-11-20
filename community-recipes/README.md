# Recetas comunitarias

Este directorio es el repositorio comunitario de recetas compartibles de Crisol.
Aquí se publican soluciones probadas contra logs reales de instalación para que
otras personas puedan reutilizarlas con `crisol recipes import`.

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
* `logs/<fixture>.log` es un fragmento realista y anonimizado del log de Wine
  que disparó la receta: 10-30 líneas con `err:module:import_dll`,
  `fixme:msi`, mensajes MSI/nsis, etc. **No incluyas rutas personales, nombres
  de usuario, claves ni marcas comerciales**: usa nombres genéricos
  (`app-vcrun2022`, `app-dotnet48`, ...).

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

El comando ya está implementado y busca recursivamente:

```sh
crisol recipes import community-recipes/apps
```

Eso copia los `recipe.recipe` de cada subdirectorio al directorio de recetas
del usuario (`~/.config/crisol/recipes/`), desde donde `crisol repair <id>`
los selecciona normalmente. Para comprobar qué recetas tienes activas:

```sh
crisol recipes | grep community-
```
