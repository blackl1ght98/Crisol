# Cómo contribuir a Crisol

Gracias por ayudar. El proyecto es pequeño y usa solo la biblioteca estándar
de Rust: cualquier mejora en recetas, documentación o empaquetado es bienvenida.

## 1. Abrir un issue antes de crear nada grande

1. Busca issues abiertos y cerrados para no duplicar.
2. Usa la plantilla de **error**, de **petición de funcionalidad** o, si aportas
   una instalación real, la plantilla **`receta-real`**
   (`.github/ISSUE_TEMPLATE/receta-real.md`).
3. Incluye: versión o commit de Crisol, distribución, versiones de Wine y
   winetricks, el comando exacto ejecutado, el log relevante y el
   comportamiento esperado frente al real.
4. **No pegues logs con datos personales**: nombres de usuario, rutas absolutas
   de tu casa, nombres de archivos privados, claves, tokens ni información de
   clientes. Pásalos por `tools/redact-log.sh` y revisa el resultado; la
   herramienta conserva las líneas de Wine pero no detecta nombres propios ni
   hostnames. Los logs de Crisol están en
   `~/.local/share/crisol/logs/<id>-<intentos>.log`.
5. **Se priorizan los logs reales sobre las recetas especulativas.** Consulta
   `community-recipes/REAL-LOGS.md`: 6 de los 8 fixtures actuales son ejemplos
   anonimizados y no cuentan como evidencia; 2 recetas sí usan logs reales
   redactados. El objetivo es llegar a 20 recetas
   con logs reales verificados.

## 2. Añadir una receta de la comunidad

Las recetas compartidas viven en `community-recipes/apps/<id>/`, donde `<id>`
es el mismo valor del `[id]` del DSL, en minúsculas con guiones y **empieza por
`community-`** (por ejemplo `community-recipes/apps/community-mi-app/`). Puedes
crear el esqueleto con `tools/new-recipe.sh community-mi-app`: genera la
receta y un `logs/<id>.log` con **solo comentarios**, nunca líneas de log
inventadas, e imprime la fila que debes añadir al manifest (no lo modifica).
Crea ahí:

* `recipe.recipe`: la receta en el DSL descrito abajo (con `confidence`,
  `priority`, `match_log_any`, `action_winetricks`, `action_message` y
  `action_retry` como mínimo).
* `logs/<fixture>.log`: fragmento **real** de 10-30 líneas del log de Wine que
  disparó la receta, **redactado con `tools/redact-log.sh` y revisado a mano**
  (usa nombres genéricos de aplicación; nada de usuarios, rutas personales, IPs,
  MACs, claves ni datos de terceros). Los 6 fixtures sintéticos actuales son
  anonimizados: no los presentes como logs reales ni los copies como prueba.

Después añade una fila a `community-recipes/manifest.tsv` con cinco columnas
separadas por TAB: `id`, `app`, `match_log_any`, `action_winetricks`, `fixture`.
Cada cadena de `match_log_any` debe aparecer literalmente en el fixture y
`action_winetricks` debe coincidir con el de la receta; el validador comprueba
todo eso. Consulta `community-recipes/README.md` para los detalles.

Antes de abrir el PR, comprueba que el `[id]` no colisiona con los de
`recipes/core.recipe` ni con otra receta comunitaria: las recetas del usuario
sobrescriben a las internas con el mismo identificador. Para probar tu receta
localmente usa `crisol recipes import community-recipes/apps`, que copia cada
`recipe.recipe` a `~/.config/crisol/recipes/` con el nombre de su carpeta.
Respeta las licencias de los runtimes: Crisol no distribuye DLL ni componentes
de Microsoft.

## 3. Formato del DSL de recetas

Archivo de texto, `clave = valor`, `#` o `;` comentan, listas separadas por
comas. Ejemplo:

```ini
[mi-arreglo]
confidence = 0.9
priority = 20
match_log_any = msvcp140.dll, vcruntime140.dll
match_detector = nsis
match_exit_code = 1
match_file_missing = drive_c/windows/system32/foo.dll
action_winetricks = vcrun2022
action_override = oleaut32=native
action_env = WINEARCH=win64
action_engine = wine-ge
action_reset_prefix = true
action_message = Instalando Visual C++ 2022...
action_retry = true
```

Claves disponibles:

| Clave | Significado |
| --- | --- |
| `[id]` | Identificador único de la receta. |
| `confidence` | Confianza de 0.0 a 1.0; el panel la ajusta con el feedback. |
| `priority` | Desempate numérico. `score = confidence * 100 + priority`. |
| `match_log_any` | Coincide si aparece **cualquiera** de los valores (sin distinguir mayúsculas). |
| `match_log_contains` | Coincide solo si aparece ese valor exacto (todos los enumerados). |
| `match_log_not` | Coincide si **no** aparece ninguno de los valores. |
| `match_detector` | Pistas del tipo de instalador detectado. |
| `match_exit_code` | Código de salida del proceso (se pueden listar varios). |
| `match_file_missing` | Ruta ausente en el prefijo; admite `*` y `?`. |
| `action_winetricks` | Verbs de winetricks a ejecutar. |
| `action_override` | Pares `dll=valor`, por ejemplo `oleaut32=native`. |
| `action_env` | Variable de entorno `CLAVE=valor`. |
| `action_engine` | Motor a usar en el reintento. |
| `action_reset_prefix` | `true` para volver al snapshot antes de reintentar. |
| `action_message` | Mensaje mostrado al usuario. |
| `action_retry` | `true` para volver a intentar la instalación. |
| `action_set_exe` | Ejecutable a registrar, relativo al prefijo (acepta `/`). |

Todas las condiciones `match_*` se combinan con **AND** y se aplican como
máximo 3 recetas por intento, ordenadas por `score`. Las recetas del usuario se
leen desde `~/.config/crisol/recipes/*.recipe` y sobrescriben a las internas si
comparten `[id]`.

## 4. Comprobaciones antes de abrir el PR

```bash
cd ~/crisol
cargo test
tools/validate-recipes.sh
```

`tools/validate-recipes.sh` valida `community-recipes/manifest.tsv`: comprueba
que existen la receta y su fixture, que el log contiene las cadenas de
`match_log_any` y que la receta declara el `action_winetricks` anunciado.
Además, `cargo test` cubre el parser y el emparejamiento de recetas. Para
aportaciones con logs, comprueba también que el fragmento pasó por
`tools/redact-log.sh`, que no quedan datos personales y que hay consentimiento
de publicación (lo recoge la plantilla `receta-real` y el
`.github/PULL_REQUEST_TEMPLATE.md`).

Si tocas código, el CI (`.github/workflows/ci.yml`) también ejecuta
`cargo fmt --all -- --check`, `cargo clippy -- -D warnings` y
`bash tests/integration.sh`; puedes replicarlos localmente antes de abrir el
PR. Para una comprobación manual rápida: `cargo run -- doctor` y
`cargo run -- recipes`.

## 5. Estilo de commits y PR

* Un commit, una idea. Título corto con prefijo: `recetas: añadir vcrun2022
  para Example`, `docs: aclarar match_log_not`, `packaging: corregir ruta del
  icono`, `tests: cubrir parser de recetas`.
* Usa el imperativo y explica el **porqué** en el cuerpo si no es obvio.
* No incluyas logs con datos personales, ni tokens, ni archivos generados
  (`target/`, `*.log`).
* En el PR indica distro, versión de Wine/winetricks, qué probaste y qué no.
  Si es un cambio de empaquetado, menciona qué comandos de validación ejecutaste.
* No hagas force-push sobre un PR ya en revisión sin avisar.

## 6. Código de conducta

Participar implica aceptar el [Código de Conducta](../CODE_OF_CONDUCT.md).
Si detectas un problema, repórtalo por los canales privados del repositorio sin
publicar datos personales.
