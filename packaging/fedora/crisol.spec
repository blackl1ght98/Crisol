# Especificación RPM para Crisol. Source0 es la URL de Git: en %prep se clona
# (no hay tarball que extraer con %autosetup). Para una publicación oficial con
# etiquetas de versión conviene cambiar Source0 por un tarball con checksum.
Name:           crisol
Version:        0.1.0
Release:        1%{?dist}
Summary:        Capa de orquestación de compatibilidad sobre Wine para instalar .exe/.msi
License:        MIT
URL:            https://github.com/blackl1ght98/Crisol
Source0:        https://github.com/blackl1ght98/Crisol.git
BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  git
Requires:       wine
Requires:       winetricks
Requires:       bubblewrap
Recommends:     qemu-ovmf
Recommends:     qemu-kvm
Recommends:     virtio-win

%description
Crisol es una capa de orquestación de compatibilidad que va más allá de Wine:
crea prefijos aislados, aplica argumentos silenciosos a instaladores .exe/.msi,
lee los logs, aplica recetas con winetricks y reintenta. Cuando Wine no basta,
prepara un fallback a una máquina virtual con Windows real. Este paquete es
experimental y no incluye Windows ni runtimes de Microsoft.

%prep
rm -rf %{name}
git clone --depth 1 %{SOURCE0} %{name}
# Si la etiqueta v%{version} existe, se puede fijar con:
# git -C %{name} checkout v%{version}

%build
cd %{name}
cargo build --release

%install
cd %{name}
install -Dm755 target/release/crisol %{buildroot}%{_bindir}/crisol
install -d %{buildroot}%{_datadir}/crisol/recipes
install -m644 recipes/*.recipe %{buildroot}%{_datadir}/crisol/recipes/
install -Dm644 README.md %{buildroot}%{_datadir}/doc/%{name}/README.md
install -Dm644 LICENSE %{buildroot}%{_datadir}/licenses/%{name}/LICENSE
install -Dm644 assets/crisol.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/crisol.svg
# No hay assets/*.desktop en el repositorio; `crisol setup-desktop` lo genera
# en ~/.local/share/applications cuando el usuario lo ejecuta.

%files
%{_bindir}/crisol
%dir %{_datadir}/crisol
%dir %{_datadir}/crisol/recipes
%{_datadir}/crisol/recipes/*.recipe
%{_datadir}/icons/hicolor/scalable/apps/crisol.svg
%license %{_datadir}/licenses/%{name}/LICENSE
%doc %{_datadir}/doc/%{name}/README.md

%changelog
* Thu Nov 20 2026 Crisol contributors - 0.1.0-1
- Primer paquete experimental.
