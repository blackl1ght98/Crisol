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

`md5sums=('SKIP')` es normal porque la fuente es Git. Opcionales: `qemu`,
`ovmf`, `virt-manager`, `virtio-win` y `pipewire`.

## Fedora / RPM

```bash
sudo dnf install rpm-build rust cargo gcc git
mkdir -p ~/rpmbuild/SPECS
cp packaging/fedora/crisol.spec ~/rpmbuild/SPECS/
rpmbuild -ba ~/rpmbuild/SPECS/crisol.spec
```

El spec usa `git clone` en `%prep` porque `Source0` es la URL de Git, así que
**necesita red durante la compilación**. Para una publicación oficial conviene
sustituirlo por un tarball con etiqueta y checksum. En Fedora el paquete de
firmware UEFI normalmente se llama `edk2-ovmf`: si `qemu-ovmf` no está en tus
repositorios, ajusta la línea `Recommends`.

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

## Limitaciones honestas

* **El binario no lee `/usr/share/crisol/recipes`.** `core.recipe` se incluye
  dentro del ejecutable en tiempo de compilación y las recetas del usuario se
  cargan desde `~/.config/crisol/recipes/*.recipe`. Los archivos instalados en
  `/usr/share/crisol/recipes` son una copia de consulta. además, `install.sh`
  copia las recetas a `~/.local/share/crisol/recipes`, que hoy no es la ruta
  que lee el programa: copia las tuyas a `~/.config/crisol/recipes/` hasta que
  se corrija ese desajuste.
* **No se instala ningún `.desktop`.** `assets/` solo contiene `crisol.svg`; el
  archivo `.desktop` lo genera `crisol setup-desktop` en el directorio de
  aplicaciones del usuario. En Flatpak esa integración queda fuera del sandbox
  y hay que hacerla a mano.
* **La licencia MIT está en el repositorio remoto**, no en el árbol local desde
  el que se prepararon estos archivos. Las recetas la instalan desde la fuente
  clonada; si construyes desde un checkout sin `LICENSE`, añádelo desde upstream.
* **Dependencias externas**: Wine, winetricks y bubblewrap son obligatorios;
  QEMU/OVMF/virtio-win, virt-manager y PipeWire son opcionales y no se empaquetan
  aquí. El fallback a VM también necesita una ISO de Windows, permisos sobre
  `/dev/kvm` y, para GPU, IOMMU/VFIO configurado en el host.
* **Flatpak es el camino más frágil**: ejecutar `bwrap` dentro del sandbox,
  acceder a `/dev/kvm` y usar audio puede fallar; en ese caso Crisol ofrece
  `--no-sandbox`, pero eso reduce el aislamiento. No se promete paridad con la
  instalación nativa.
* Estos archivos se han revisado sintácticamente, pero **no se ha ejecutado**
  `makepkg`, `rpmbuild` ni `flatpak-builder` en este árbol.

Para reportar problemas, usa las plantillas de `.github/ISSUE_TEMPLATE/` y no
incluyas datos personales en los logs.
