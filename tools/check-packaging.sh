#!/usr/bin/env bash
# Verificación ESTÁTICA de las recetas de empaquetado de Crisol.
# No construye paquetes, no instala nada y no usa sudo.
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fallos=0

ok() {
    printf 'OK %s\n' "$1"
}

fallo() {
    printf 'FALLO %s\n' "$1"
    fallos=$((fallos + 1))
}

comprobar_archivo() {
    if [[ -f "$1" ]]; then
        ok "existe $1"
    else
        fallo "falta $1"
    fi
}

# 1. Manifiestos y archivos del repositorio que instalan los manifiestos.
for archivo in \
    packaging/arch/PKGBUILD \
    packaging/fedora/crisol.spec \
    packaging/flatpak/org.blackl1ght98.Crisol.yaml \
    packaging/README.md; do
    comprobar_archivo "$archivo"
done

for archivo in README.md LICENSE assets/crisol.svg recipes/core.recipe; do
    comprobar_archivo "$archivo"
done

# 2. Sintaxis del PKGBUILD (solo bash -n; no se ejecuta makepkg).
if bash -n packaging/arch/PKGBUILD; then
    ok 'bash -n packaging/arch/PKGBUILD'
else
    fallo 'bash -n packaging/arch/PKGBUILD'
fi

# 3. Campos obligatorios del spec (no se ejecuta rpmbuild/rpmspec).
spec=packaging/fedora/crisol.spec
comprobar_spec() {
    local nombre="$1" regex="$2"
    if grep -Eq "$regex" "$spec"; then
        ok "spec: $nombre"
    else
        fallo "spec: falta $nombre"
    fi
}
comprobar_spec 'Name'   '^Name:[[:space:]]*[^[:space:]]'
comprobar_spec 'Version' '^Version:[[:space:]]*[^[:space:]]'
comprobar_spec 'License' '^License:[[:space:]]*[^[:space:]]'
comprobar_spec 'URL'     '^URL:[[:space:]]*[^[:space:]]'
comprobar_spec '%install' '^%install([[:space:]]|$)'
comprobar_spec '%files'   '^%files([[:space:]]|$)'

# 4. YAML: parseo real con PyYAML si está disponible; si no, comprobación mínima.
yaml=packaging/flatpak/org.blackl1ght98.Crisol.yaml
if command -v python3 >/dev/null 2>&1 && python3 -c 'import yaml' >/dev/null 2>&1; then
    if python3 - "$yaml" <<'PY'
import sys
import yaml

with open(sys.argv[1], encoding="utf-8") as fh:
    doc = yaml.safe_load(fh)

required = {"id", "runtime", "command", "modules", "finish-args"}
if not isinstance(doc, dict) or not required.issubset(doc):
    sys.exit(1)
PY
    then
        ok 'YAML parseado con python3/PyYAML y claves mínimas presentes'
    else
        fallo 'YAML no válido o faltan claves mínimas'
    fi
else
    printf 'INFO python3/PyYAML no disponible: comprobación mínima del YAML\n'
    comprobar_yaml_minimo() {
        local clave
        for clave in id runtime command modules finish-args; do
            grep -Eq "^${clave}:" "$yaml" || return 1
        done
        # Los tabuladores no son válidos como indentación YAML.
        ! grep -q $'\t' "$yaml"
    }
    if comprobar_yaml_minimo; then
        ok 'YAML: claves mínimas y sin tabuladores (comprobación mínima)'
    else
        fallo 'YAML: faltan claves mínimas o hay tabuladores'
    fi
fi

printf 'INFO comprobación estática: no se construyen paquetes ni se usa sudo\n'

if ((fallos == 0)); then
    printf 'OK verificación estática completada sin fallos\n'
else
    printf 'FALLO verificación estática: %d fallo(s)\n' "$fallos"
    exit 1
fi
