# Modelo de seguridad

Crisol 0.1.0 es un **prototipo**. Ejecuta instaladores `.exe`/`.msi` que no son
de confianza y mantiene un servidor HTTP local para recibir feedback. Este
documento describe las medidas activas en la versión publicada y las
limitaciones que siguen existiendo.

## 1. Panel de feedback

El panel se inicia con `crisol panel [--port 8765]`.

### Implementado en 0.1.0

* **Escucha solo en `127.0.0.1`.** El código enlaza explícitamente
  `TcpListener::bind(("127.0.0.1", port))`; no escucha en `0.0.0.0`. Es un
  servidor HTTP mínimo con la biblioteca estándar de Rust, sin GTK4.
* Las rutas `/ok` y `/bad` cambian el estado de la app y ajustan la confianza de
  las recetas; `/export` devuelve `feedback.tsv`. El HTML escapa los datos del
  catálogo al renderizarlos.

### Endurecimiento activo

| Medida | Estado |
| --- | --- |
| Token en `~/.config/crisol/panel.token`, exigido como `?t=...` en `/ok`, `/bad` y `/export`. | **Activo**. El enlace de descarga y los botones del HTML ya lo incluyen. |
| Rechazo de peticiones con `Origin` ajeno a `http://127.0.0.1:<puerto>`/`localhost` o con `Sec-Fetch-Site: cross-site`. | **Activo** (HTTP 403). |
| Cabeceras `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer` y `Cache-Control: no-store`. | **Activo** en todas las respuestas. |

`crisol app` arranca el panel si hace falta y abre el navegador; el token se
genera la primera vez y se reutiliza. El test de integración comprueba que un
voto sin token devuelve 403.

Recomendaciones que siguen siendo válidas:

* Deja el panel abierto solo mientras lo usas y ciérralo con `Ctrl+C`.
* No expongas el puerto mediante túneles, proxies inversos ni redirecciones: la
  protección depende de que el servicio siga siendo local.
* No compartas el contenido de `panel.token`; equivale a una sesión local.
* Revisa `~/.config/crisol/feedback.tsv` y `feedback.log` si notas votos que no
  hiciste.

## 2. Ejecución de instaladores

* Crisol crea un **prefijo aislado** por aplicación
  (`~/.local/share/crisol/prefixes/<id>`) y guarda un snapshot antes de aplicar
  recetas. Eso evita que una app rompa el prefijo de otra; **no es un sandbox**.
* Cuando `bubblewrap` está disponible, el instalador se ejecuta con `bwrap`:
  sistema de archivos raíz de solo lectura, `/dev` y `/proc`, y `/tmp`, el
  prefijo y la caché escribibles. `wineboot` se ejecuta sin `bwrap` porque los
  servicios de Wine no arrancan bien dentro.
* **bubblewrap reduce la superficie, pero NO es una frontera de seguridad
  fuerte.** El proceso comparte kernel, red y dispositivos con tu sesión. Además,
  si `bwrap` falla al lanzarse, Crisol avisa y **reintenta sin aislamiento**.
* `--no-sandbox` desactiva `bwrap` deliberadamente. Úsalo solo bajo tu
  responsabilidad, cuando el instalador no funcione de otra forma, y solo con
  instaladores en los que confíes relativamente.
* Crisol **ejecuta `.exe` no confiables**: unpackers, servicios y scripts de
  esos instaladores pueden hacer lo que tu usuario puede hacer. No lo uses para
  software que no hayas verificado, y no lo ejecutes desde una cuenta con
  privilegios.

## 3. Aislamiento más fuerte: VM con VFIO

Cuando Wine no basta (anti-cheat de kernel, DRM, controladores), la opción
robusta es una **VM con Windows real**. Con GPU passthrough (VFIO) y un grupo
IOMMU aislado, el aislamiento frente al huesped es el más fuerte que ofrece
Crisol: Linux no confía en un Windows que puede ejecutar código de kernel.

Crisol solo **prepara** discos `qcow2`, scripts de arranque y el traspaso del
instalador por 9p; no instala QEMU ni Windows, no ejecuta `sudo` por ti y no
puede garantizar IOMMU ni grupos VFIO limpios. Consulta `docs/VM.md`. Una VM con
red virtual y sin VFIO es mejor que `bwrap` para drivers, pero sigue siendo
software; no la trates como una frontera perfecta.

## 4. Reportar vulnerabilidades

**No abras una issue pública con un fallo de seguridad ni con datos personales.**

1. Usa la pestaña **Security** del repositorio
   (`https://github.com/blackl1ght98/Crisol/security`) y la opción
   **Report a vulnerability** (security advisory privada).
2. Si eso no está disponible, contacta a los mantenedores por un canal privado
   y pide que abran una issue privada; no pegues el detalle en una issue normal.
3. Incluye versión/commit, distribución, pasos mínimos para reproducir y un
   impacto concreto. Redacta rutas, usuarios, tokens y logs según
   `community-recipes/REAL-LOGS.md`.

No hay un plazo de respuesta prometido: el proyecto es pequeño y voluntario.
