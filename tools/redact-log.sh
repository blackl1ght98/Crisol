#!/usr/bin/env bash
# Redacta datos personales de un log de Wine/winetricks.
# Conserva las líneas y las señales de Wine (err:, fixme:, ids de hilo, DLLs):
# solo sustituye tokens concretos, nunca borra ni reordena líneas.
set -euo pipefail

usage() {
    cat <<'EOF'
Uso: tools/redact-log.sh [--help] [archivo]

Redacta un log de Wine/winetricks y escribe el resultado en stdout.
Si no se indica archivo, lee de stdin.

Sustituciones (los marcadores son literales):
  /home/<usuario>            se conserva '/home/' y se oculta el usuario
  C:\Users\<usuario>         también C:/Users/<usuario> y variantes con \\
  UserProfile=<usuario>      nombre de usuario en variables UserProfile
  direcciones IPv4           -> <IP>
  direcciones MAC            -> <MAC>
  claves/seriales XXXXX-...  -> <LICENCIA>

Límites: es conservador (no elimina líneas de Wine), pero no reconoce nombres
propios, hostnames ni nombres de archivo sensibles, y puede marcar tokens con
forma de IPv4 que sean versiones de 4 números. Revisa el resultado a mano antes
de publicarlo. Consulta community-recipes/REAL-LOGS.md.
EOF
}

case "${1:-}" in
    --help | -h)
        usage
        exit 0
        ;;
esac

if (( $# > 1 )); then
    echo "ERROR: demasiados argumentos; usa --help" >&2
    usage >&2
    exit 2
fi

if (( $# == 1 )) && [[ ! -f "$1" && "$1" != "-" ]]; then
    echo "ERROR: no existe el archivo: $1" >&2
    exit 1
fi

redact() {
    sed -E \
        -e 's@/home/[A-Za-z0-9._-]+@/home/<usuario>@g' \
        -e 's@C:\\+[Uu]sers\\+[A-Za-z0-9._-]+@C:\\Users\\<usuario>@g' \
        -e 's@C:/[Uu]sers/[A-Za-z0-9._-]+@C:/Users/<usuario>@g' \
        -e 's@(UserProfile=)([A-Za-z0-9._-]+)([[:space:];,]|$)@\1<usuario>\3@gI' \
        -e 's@\b(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])(\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])){3}\b@<IP>@g' \
        -e 's@\b([0-9A-Fa-f]{2}:){5}[0-9A-Fa-f]{2}\b@<MAC>@g' \
        -e 's@\b([0-9A-Fa-f]{2}-){5}[0-9A-Fa-f]{2}\b@<MAC>@g' \
        -e 's@\b[A-Z0-9]{5}(-[A-Z0-9]{5}){4}\b@<LICENCIA>@gI' \
        -- "$@"
}

if (( $# == 0 )) || [[ "$1" == "-" ]]; then
    redact
else
    redact "$1"
fi
