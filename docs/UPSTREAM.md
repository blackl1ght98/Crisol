# Colaboración aguas arriba (planes y borradores)

**Estado: ninguno de estos borradores ha sido enviado.** No se ha abierto
ninguna issue ni ningún PR en winetricks, Bottles ni Lutris. Todo lo que sigue
es un plan local para iniciar la conversación cuando haya tiempo y consenso en
Crisol. No presentes estos textos como si ya existieran aguas arriba.

Objetivo: reducir el trabajo duplicado. Crisol resuelve hoy con un DSL propio
lo que otros proyectos resuelven con recetas/scripts de instalación; compartir
formato y banderas silenciosas beneficiaría a todos.

---

## 1. winetricks — borrador de issue

Repositorio previsto: `https://github.com/Winetricks/winetricks` (comprobar
antes el nombre y las normas del proyecto).

**Título:** `[Feature] Documentar y unificar opciones silenciosas y argumentos por verb para integradores`

**Cuerpo propuesto:**

> Hola, soy colaborador de Crisol (una capa sobre Wine que instala .exe/.msi en
> modo no interactivo). Hoy usamos `winetricks -q ...` y las variables
> existentes de "unattended", pero seguimos encontrando verbs que muestran
> diálogos de EULA o instaladores que ignoran los flags silenciosos, lo que
> rompe la automatización.
>
> Propongo, sin romper la interfaz actual:
>
> 1. Una opción global `--noninteractive` (y su equivalente
>    `WINETRICKS_NONINTERACTIVE=1`) que centralice el comportamiento ya
>    existente de `-q`/unattended: aceptar los valores por defecto documentados
>    siempre que la licencia lo permita y no lanzar diálogos.
> 2. Una forma documentada de pasar argumentos al instalador de un verb, por
>    ejemplo `WINETRICKS_<VERB>_ARGS` (o un fichero de configuración por verb),
>    para que un integrador pueda usar `/qn` en MSI o `/S` en NSIS.
> 3. Una opción `--print-cmd` / `--dry-run` que muestre los comandos y URLs que
>    se ejecutarían, útil para validar recetas sin descargar nada.
> 4. Documentar en el man page qué verbs siguen necesitando interfaz y cuáles
>    aceptan estas opciones.
>
> No propongo cambiar el comportamiento por defecto ni incluir runtimes de
> Microsoft. Si la opción 2 ya es posible con otra variable, una aclaración en
> la documentación también sería de mucha ayuda. ¿Tiene sentido? ¿Qué nombre
> de variable encajaría mejor con el estilo del proyecto?

### Borrador de PR (solo cuando la issue tenga feedback positivo)

**Rama sugerida:** `crisol-noninteractive-options`.

**Título:** `docs: documentar opciones no interactivas y argumentos por verb`

**Cuerpo propuesto:**

> Relacionado con #<número de la issue>.
>
> Cambios propuestos:
>
> - Documentar `-q`/unattended y el nuevo `--noninteractive` en `winetricks.1`
>   y en la ayuda `--help`, indicando que no cambia el comportamiento por
>   defecto.
> - Añadir `WINETRICKS_<VERB>_ARGS` como forma de pasar argumentos al
>   instalador del verb, con un ejemplo (`vcrun2022` con MSI `/qn`).
> - Añadir `--print-cmd` para imprimir comandos/URLs sin ejecutarlos.
> - Añadir un test pequeño que compruebe que la opción no interactiva no
>   modifica los verbs que ya son silenciosos.
>
> Checklist: `[ ]` comprobado con al menos dos verbs, `[ ]` man page
> actualizada, `[ ]` sin cambios en licencias ni en fuentes de descarga,
> `[ ]` PR abierta solo tras el visto bueno en la issue.

No incluyas en el PR logs de instalaciones reales con rutas personales.

---

## 2. Bottles y Lutris — borradores de issue para intercambio de recetas

Repositorios previstos: `https://github.com/bottlesdevs/Bottles` y
`https://github.com/lutris/lutris`.

### Borrador de issue para Bottles

**Título:** `Propuesta: formato común de recetas entre Bottles y Crisol`

**Cuerpo propuesto:**

> Crisol (https://github.com/blackl1ght98/Crisol) mantiene recetas sencillas en un
> DSL de texto con condiciones (`match_log_any`, `match_detector`,
> `match_file_missing`...) y acciones (`action_winetricks`, `action_override`,
> `action_env`...). Bottles ya tiene un sistema de recetas más completo.
>
> Propongo explorar un subconjunto común intercambiable en ambos sentidos:
>
> - Un esquema mínimo con `id`, `match` (log/detector/código de salida) y
>   `actions` (winetricks, overrides, env).
> - Un conversor de ese esquema al formato JSON de Bottles y viceversa,
>   mantenido en Crisol bajo MIT, sin exigir cambios en Bottles al principio.
> - Un directorio de recetas de comunidad con metadatos de versión probada y
>   licencia explícita (MIT o CC0 para las recetas; nunca redistribuyendo
>   runtimes de Microsoft).
>
> ¿Os interesaría definir juntos ese subconjunto? ¿Qué campos de vuestras
> recetas consideráis imprescindibles?

### Borrador de issue para Lutris

**Título:** `Propuesta: compartir soluciones de instalación con Crisol mediante scripts YAML`

**Cuerpo propuesto:**

> Los scripts de instalación YAML de Lutris y las recetas de Crisol resuelven
> problemas parecidos (dependencias, overrides, flags silenciosos). Sugiero un
> mapa de correspondencias documentado y un script de conversión:
>
> - `action_winetricks` → pasos `winetricks` del instalador.
> - `action_override` → configuración de DLL overrides del prefijo.
> - `action_env` → variables de entorno del juego.
> - condiciones `match_log_*` → comprobaciones previas o notas de troubleshooting.
>
> La conversión no tiene por qué ser perfecta: basta con que un mantenedor
> pueda partir de una receta y adaptarla. ¿Dónde sería mejor documentar el
> mapa: en la wiki de Lutris o en un repo neutral de recetas?

### Borrador de PR (en Crisol, no en Bottles ni Lutris)

**Rama sugerida:** `recetas-export-comun`.

**Título:** `tools: añadir exportador de recetas a formato de Bottles/Lutris`

**Cuerpo propuesto:**

> Añade un script sin dependencias externas que lea `recipes/*.recipe` y
> `community-recipes/apps/*/*.recipe` y genere un borrador de JSON (Bottles) o
> YAML (Lutris) para revisión manual. No modifica el runtime de Crisol.
>
> Incluye tests con dos recetas de ejemplo y un README que indique que la
> salida es orientativa y debe revisarse. Licencia MIT.
>
> Checklist: `[ ]` sin datos personales en ejemplos, `[ ]` sin DLLs ni
> runtimes, `[ ]` documentado que el formato de destino puede cambiar.

---

## 3. Pasos para enviarlos (cuando se decida)

1. Comprobar el nombre correcto del repositorio y buscar issues/PR similares.
2. Leer las normas de contribución de cada proyecto; adaptar el tono y el
   formato si piden plantillas propias.
3. Abrir **solo la issue** primero y esperar feedback; no abrir el PR en
   paralelo.
4. Si hay interés, hacer fork, crear la rama indicada y enviar un PR pequeño
   y verificable, con los comandos de prueba ejecutados.
5. Relacionar el PR con la issue (`Relacionado con #...`) y responder a las
   revisiones sin force-push silencioso.
6. Una vez fusionado o rechazado, actualizar este documento con el enlace y el
   resultado. **Hoy no hay ningún enlace porque nada ha sido enviado.**

Recordatorio: los ejemplos de log deben estar anonimizados y no deben incluir
rutas personales, nombres de usuario, tokens ni información de terceros.
