# Campaña: logs reales para recetas verificadas

**Objetivo concreto: 20 recetas con logs reales verificados.**

Hoy no hemos alcanzado ese objetivo. Esta campaña existe porque el mayor hueco
de Crisol no es código: son **evidencias reales de instalación**.

## Estado actual: los 6 fixtures NO son logs reales

En `community-recipes/apps/` hay 6 recetas con sus correspondientes fixtures
(ids `community-*`). **Son ejemplos anonimizados y sintéticos, no evidencia de
instalaciones reales.** Sirven para probar el DSL y el validador, nada más.
No digas en un issue, PR, documento ni commit que esos logs proceden de una
instalación real: no es cierto.

Una receta cuenta para el objetivo de las 20 solo cuando cumple **todas** estas
condiciones:

1. El log viene de una instalación real ejecutada por una persona.
2. El log está redactado y esa persona ha dado consentimiento explícito para
   publicarlo.
3. La receta coincide de verdad con el fragmento publicado (lo comprueba
   `tools/validate-recipes.sh`).
4. Se conocen instalador + versión, distribución y versiones de Wine/winetricks.
5. El resultado (éxito tras aplicar la receta) ha sido confirmado por quien lo
   probó.

## Qué log pedir

Crisol guarda la salida del proceso en
`~/.local/share/crisol/logs/<id>-<intentos>.log`; también se muestran las últimas
líneas en pantalla y el mensaje final incluye el código de salida (`exit ...`).
Pide **salida de wine/winetricks del instalador**, no solo un resumen:

* Las 20-80 líneas alrededor del **primer error útil**, no únicamente el final.
* Líneas de `err:module:import_dll` (DLL que falta y quién la necesita).
* Líneas `err:seh`, `err:module:load_builtin_dll`, `err:module:process_exit`.
* Líneas `fixme:msi`, mensajes `MSI (s) ...` y códigos `0x800...`/`0xc000...`.
* Mensajes del propio instalador (NSIS, Inno Setup, InstallShield) y de
  `winetricks -q <verb>` si se aplicó una receta.
* El **código de salida** del instalador (p. ej. `status 5`, `exit Some(5)`) y si
  hubo timeout.
* El comando exacto ejecutado y las opciones usadas (`--engine`, `--attempts`,
  `--no-sandbox`, `--vm`, ...).

Formas de línea que nos interesan (plantilla ilustrativa con marcadores, **no un
log real y no un fragmento para copiar**):

```text
<tid>:err:module:import_dll Library <DLL> (which is needed by ...) not found
<tid>:err:seh:setup_exception ...
<tid>:fixme:msi:ACTION_CallMsiDoAction ...
<tid>:err:module:process_exit process ... exited with status <N>
```

Conserva los prefijos `err:`, `fixme:`, los ids de hilo y los nombres de DLL:
son precisamente lo que el motor de recetas usa para emparejar. Lo que se
redacta son datos personales, no las señales de Wine.

## Cómo redactar antes de publicar

Primero pasa el log por la herramienta del repositorio:

```sh
tools/redact-log.sh log-original.log > log-redactado.log
less log-redactado.log    # revísalo siempre: sed no detecta todo
```

`tools/redact-log.sh --help` explica sus límites. Después revisa manualmente y
sustituye, como mínimo:

* `/home/<usuario>` → `/home/<usuario>` (se conserva `home`; el marcador es
  literal, p. ej. `.../home/juan/.wine/...` → `.../home/<usuario>/.wine/...`).
* `C:\Users\<usuario>` y `C:/Users/<usuario>` → el mismo marcador con la ruta
  conservada.
* `UserProfile`, `%USERPROFILE%` o variables que contengan el usuario.
* Nombres de usuario, hostnames y cualquier ruta que identifique a una persona.
* Direcciones IPv4 y MAC.
* Claves, seriales y tokens con formas tipo `XXXXX-XXXXX-XXXXX-XXXXX-XXXXX`.
* Nombres de archivo descargados si son sensibles (facturas, contratos, datos de
  clientes, nombres de proyecto privados). Sustitúyelos por `installer.exe`,
  `setup.msi` o `<ARCHIVO>`.
* URLs con parámetros de descarga personales y cualquier dato de terceros.

La herramienta es conservadora: no borra líneas de Wine. Aun así, **la
responsabilidad final es de quien publica**: si algo se le escapó, no publiques
ese log.

## Consentimiento

Antes de abrir el issue o el PR, la persona que aportó el log debe confirmar
por escrito:

* que revisó el fragmento redactado y que no contiene datos personales ni de
  terceros;
* que autoriza a publicarlo en el repositorio bajo la licencia MIT del proyecto
  (o CC0 si se acuerda para ese caso);
* que puede retirar el permiso avisando, aunque no podremos borrar copias ya
  publicadas ni forks.

La plantilla `.github/ISSUE_TEMPLATE/receta-real.md` incluye ambas confirmaciones.

## Checklist de calidad

- [ ] Instalador real, con nombre y versión concretos (o `installer.exe` si el
      nombre original es sensible).
- [ ] Distribución, versión de Wine/motor y versión de winetricks indicadas.
- [ ] Comando exacto ejecutado y opciones usadas.
- [ ] Síntoma descrito y fragmento de log pegado en bloque de código.
- [ ] Log redactado y revisado línea a línea (rutas, usuarios, IP, MAC, claves y
      nombres de archivo sensibles).
- [ ] Consentimiento de publicación confirmado por quien ejecutó la instalación.
- [ ] `match_log_any` extraído literalmente del log publicado.
- [ ] `tools/validate-recipes.sh` termina con `OK <id>`.
- [ ] Receta probada con `crisol repair <id>` y resultado confirmado.

## Cómo enviarlo

**Opción A — issue `receta-real` (recomendada si aún no quieres abrir un PR).**
Abre una issue con la plantilla `receta-real` y pega allí el log redactado, el
síntoma y tu receta propuesta. Si hace falta, mantenemos el fragmento en el
issue hasta convertirlo en receta.

**Opción B — Pull request directo.**

```sh
tools/new-recipe.sh community-mi-app
# Sustituye los placeholders del recipe.recipe y pega el log REAL redactado
# (solo comentarios) en community-recipes/apps/community-mi-app/logs/community-mi-app.log
tools/redact-log.sh /ruta/al/log/original > community-recipes/apps/community-mi-app/logs/community-mi-app.log
$EDITOR community-recipes/manifest.tsv     # añade la fila: id / app / match / winetricks / fixture
tools/validate-recipes.sh
crisol recipes import community-recipes/apps
```

`tools/new-recipe.sh` **no** modifica `manifest.tsv`: añade la fila a mano. El
fixture debe contener las cadenas declaradas en `match_log_any` y el valor
exacto de `action_winetricks`.

Si el log es demasiado largo, publica solo el fragmento mínimo que dispare la
receta; no adjuntes el log completo de una sesión de trabajo.
