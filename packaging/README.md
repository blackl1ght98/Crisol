# Empaquetado de Crisol

Recetas de empaquetado **experimentales** para Arch Linux, Fedora y Flatpak.
No sustituyen a `install.sh` (que instala en `~/.local/bin`) y no han sido
probadas en todos los escenarios: revisa los avisos del final.

## Arch Linux (makepkg)

```bash
sudo pacman -S --needed base-devel rust wine winetricks bubblewrap
cd ~/crisol/packaging/arch
makepkg -si
```

`PKGBUILD` clona `https://github.com/blackl1ght98/Crisol.git` en `$srcdir`,
compila con `cargo build --release` e instala:

* `/usr/bin/crisol`
* `/usr/share/doc/crisol/README.md`
* `/usr/share/licenses/crisol/LICENSE`
* `/usr/share/crisol/recipes/*.recipe` (copia de referencia)
* `/usr/share/icons/hicolor/scalable/apps/crisol.svg`

`md5sums=('SKIP')` es normal porque la fuente es Git (`git+` en `source` y
`makedepends=('git' 'cargo')`). Opcionales: `qemu`, `ovmf`, `virt-manager`,
`virtio-win` y `pipewire`.

## Fedora / RPM

```bash
sudo dnf install rpm-build rust cargo gcc git
mkdir -p ~/rpmbuild/SPECS
cp packaging/fedora/crisol.spec ~/rpmbuild/SPECS/
rpmbuild -ba ~/rpmbuild/SPECS/crisol.spec
```

El spec **no declara `Source0`**: `rpmbuild` no descarga URLs y `%{SOURCE0}`
sería una ruta local bajo `SOURCES/`, no una URL clonable. `%prep` clona la URL
de `%{upstream_url}` directamente, así que **necesita red durante la compilación**
y el SRPM resultante no es autocontenido. Para una publicación oficial conviene
sustituir esto por un tarball con etiqueta y checksum como `Source0`. `wine` no
está en los repositorios base de Fedora: habilita RPM Fusion antes de instalar.
El firmware UEFI correcto en Fedora es `edk2-ovmf` (no `qemu-ovmf`).

## Flatpak (flatpak-builder)

```bash
flatpak install flathub org.freedesktop.Platform//24.08 \
  org.freedesktop.Sdk//24.08 org.freedesktop.Sdk.Extension.rust-stable
flatpak-builder --user --install --force-clean build-dir \
  packaging/flatpak/org.blackl1ght98.Crisol.yaml
```

El manifiesto compila con `cargo build --release --offline` (el proyecto no
tiene crates externos, así que no hace falta red para compilar) y concede red
en tiempo de ejecución para que `winetricks` descargue los runtimes. El source
Git usa `branch: main`; cámbialo a `tag: v0.1.0` cuando exista esa etiqueta.
También instala `LICENSE` en `/app/share/licenses/org.blackl1ght98.Crisol/`.
Dentro del sandbox `$HOME` es `~/.var/app/org.blackl1ght98.Crisol`, así que las
recetas del usuario deben copiarse a
`~/.var/app/org.blackl1ght98.Crisol/config/crisol/recipes/`.

## Limitaciones honestas

* **El binario no lee `/usr/share/crisol/recipes`.** `core.recipe` se incluye
  dentro del ejecutable en tiempo de compilación y las recetas del usuario se
  cargan desde `~/.config/crisol/recipes/*.recipe`. Los archivos instalados en
  `/usr/share/crisol/recipes` son una copia de consulta. Además, `install.sh`
  copia las recetas a `~/.local/share/crisol/recipes`, que hoy no es la ruta
  que lee el programa: copia las tuyas a `~/.config/crisol/recipes/` hasta que
  se corrija ese desajuste.
* **No se instala ningún `.desktop`.** `assets/` solo contiene `crisol.svg`; el
  archivo `.desktop` lo genera `crisol setup-desktop` en el directorio de
  aplicaciones del usuario. En Flatpak esa integración queda fuera del sandbox
  y hay que hacerla a mano.
* **Licencia**: `LICENSE` (MIT) ya existe en la raíz del repositorio y los tres
  manifiestos la instalan desde la fuente. Si construyes desde un checkout que
  no la contenga, añádela desde upstream antes de empaquetar.
* **Dependencias externas**: Wine, winetricks y bubblewrap son obligatorios;
  QEMU/OVMF/virtio-win, virt-manager y PipeWire son opcionales y no se empaquetan
  aquí. El fallback a VM también necesita una ISO de Windows, permisos sobre
  `/dev/kvm` y, para GPU, IOMMU/VFIO configurado en el host.
* **Flatpak es el camino más frágil**: ejecutar `bwrap` dentro del sandbox,
  acceder a `/dev/kvm` y usar audio puede fallar; en ese caso Crisol ofrece
  `--no-sandbox`, pero eso reduce el aislamiento. No se promete paridad con la
  instalación nativa.
* Estos archivos solo han pasado una **verificación estática**; **no se ha
  ejecutado** `makepkg`, `rpmbuild` ni `flatpak-builder` en este árbol.

## Verificación estática

Desde la raíz del repositorio:

```bash
bash -n tools/check-packaging.sh
bash tools/check-packaging.sh
```

`tools/check-packaging.sh` comprueba que existan los cuatro archivos de
`packaging/`, `LICENSE` y `README.md`; aplica `bash -n` al `PKGBUILD`; verifica
que el spec tenga `Name`, `Version`, `License`, `URL`, `%install` y `%files`;
parsea el YAML con `python3` y PyYAML cuando están disponibles (si no, hace una
comprobación mínima de claves y tabuladores); y confirma que existan los archivos
del repo que los manifiestos instalan (`README.md`, `LICENSE`,
`assets/crisol.svg` y `recipes/core.recipe`). Imprime `OK`/`FALLO` por cada
comprobación y termina con código 1 si hay algún fallo. No construye paquetes,
no instala nada y no usa `sudo`.

**Los paquetes NO han sido construidos en este entorno.** No se ha validado la
instalación real, la ejecución de los binarios empaquetados ni la integración con
Wine/QEMU/Flatpak: en este entorno solo estaban disponibles `makepkg` y
`python3` (con PyYAML); no había `rpmbuild`, `rpmspec`, `flatpak-builder` ni
`yamllint`. La comprobación estática no sustituye una construcción de prueba.

Para reportar problemas, usa las plantillas de `.github/ISSUE_TEMPLATE/` y no
incluyas datos personales en los logs.
